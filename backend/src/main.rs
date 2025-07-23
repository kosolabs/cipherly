use crate::{google::Certs, kek::Keks};
use aes_gcm::{
    AeadCore, Aes256Gcm,
    aead::{Aead, OsRng},
};
use anyhow::{Context as _, Result};
use axum::{
    Extension, Json, Router,
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::post,
};
use base64::prelude::*;
use rmp_serde::{from_slice, to_vec};
use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{net::TcpListener, signal, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tower::ServiceBuilder;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod google;
mod kek;

#[derive(Debug, Serialize, Deserialize)]
struct Envelope {
    dek: String,
    emails: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SealedEnvelope {
    kid: String,
    nonce: String,
    data: String,
}

#[tracing::instrument(skip_all)]
async fn seal(
    Extension(keks): Extension<Arc<Keks>>,
    Json(envelope): Json<Envelope>,
) -> Result<Json<SealedEnvelope>, StatusCode> {
    let buf = to_vec::<Envelope>(&envelope).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let kek = keks.get("v1").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let ciphertext = kek
        .encrypt(&nonce, buf.as_slice())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(SealedEnvelope {
        kid: "v1".into(),
        nonce: BASE64_URL_SAFE_NO_PAD.encode(nonce.as_slice()),
        data: BASE64_URL_SAFE_NO_PAD.encode(ciphertext.as_slice()),
    }))
}

#[tracing::instrument(skip_all)]
async fn unseal(
    Extension(keks): Extension<Arc<Keks>>,
    Extension(claims): Extension<google::Claims>,
    Json(sealed_envelope): Json<SealedEnvelope>,
) -> Result<Json<Envelope>, StatusCode> {
    let nonce = BASE64_URL_SAFE_NO_PAD
        .decode(&sealed_envelope.nonce)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let ciphertext = BASE64_URL_SAFE_NO_PAD
        .decode(&sealed_envelope.data)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let kek = keks
        .get(&sealed_envelope.kid)
        .ok_or(StatusCode::UNAUTHORIZED)?;
    let plaintext = kek
        .decrypt(nonce.as_slice().into(), ciphertext.as_slice())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    let envelope: Envelope = from_slice(&plaintext).map_err(|_| StatusCode::UNAUTHORIZED)?;
    if !envelope.emails.contains(&claims.email) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(Json(envelope))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let shutdown_signal = CancellationToken::new();
    tokio::join!(
        async {
            run_server(Config {
                shutdown_signal: shutdown_signal.clone(),
                ..Default::default()
            })
            .await
            .unwrap()
        },
        async { signal_shutdown(shutdown_signal.clone()).await.unwrap() },
    );
}

#[derive(Default)]
pub struct Config {
    pub port: Option<u16>,
    pub certs: Option<Certs>,
    pub keks: Option<Keks>,
    pub shutdown_signal: CancellationToken,
}

async fn run_server(config: Config) -> Result<(SocketAddr, JoinHandle<Result<()>>)> {
    let certs = match config.certs {
        Some(certs) => certs,
        None => google::fetch()
            .await
            .context("Failed to fetch Google certs")?,
    };
    let keks = match config.keks {
        Some(keks) => keks,
        None => {
            let keks = env::var("KEKS").context("KEKS environment variable is not set")?;
            kek::parse(&keks).context("Failed to parse KEKs")?
        }
    };
    let shutdown_signal = config.shutdown_signal;

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route(
                    "/unseal",
                    post(unseal).layer(middleware::from_fn(google::authenticate)),
                )
                .route("/seal", post(seal)),
        )
        .layer(
            ServiceBuilder::new()
                .layer((Extension(Arc::new(certs)), Extension(Arc::new(keks))))
                .layer(SetRequestIdLayer::new(
                    HeaderName::from_static("x-request-id"),
                    MakeRequestUuid,
                ))
                .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
                    "x-request-id",
                )))
                // Enable request tracing. Must enable `tower_http=debug)
                .layer(TraceLayer::new_for_http())
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                .layer(TimeoutLayer::new(Duration::from_secs(10))),
        )
        .fallback_service(
            ServiceBuilder::new()
                .layer((
                    TimeoutLayer::new(Duration::from_secs(20)),
                    middleware::from_fn(set_static_cache_control),
                ))
                .service(
                    ServeDir::new("static")
                        .precompressed_gzip()
                        .precompressed_br()
                        .fallback(ServeFile::new("static/index.html")),
                ),
        );

    let port = config
        .port
        .map(|port| port.to_string())
        .unwrap_or_else(|| env::var("PORT").unwrap_or("8000".into()));
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    let addr = listener.local_addr()?;

    let serve = tokio::spawn(async move {
        tracing::info!("server listening on {}", addr);
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async move { shutdown_signal.cancelled().await })
        .await
        .context("serve failed")?;
        Ok(())
    });
    Ok((addr, serve))
}

// This function waits for a shutdown signal (e.g. ctrl-c, SIGTERM)
// and then cancels the provided CancellationToken in order
// to enable graceful shutdown.
async fn signal_shutdown(shutdown_signal: CancellationToken) -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .context("failed to install Ctrl+C handler")?;
        tracing::info!("Terminating with ctrl-c...");
        Ok::<(), anyhow::Error>(())
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .context("failed to install signal handler")?
            .recv()
            .await;
        tracing::info!("Terminating...");
        Ok(())
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<Result<()>>();

    // Wait for one of the signals to fire.
    tokio::select! {
        res = ctrl_c => {res},
        res = terminate => {res},
    }?;

    // Initiate shutdown.
    shutdown_signal.cancel();

    Ok(())
}

// Built frontend files in /_app/immutable/ are immutable and never change.
// Allow them to be cached as such.
async fn set_static_cache_control(request: Request, next: Next) -> Response {
    let header = if request.uri().path().starts_with("/_app/immutable/") {
        "public, immutable, max-age=31536000"
    } else if request.uri().path() == "/robots.txt" || request.uri().path() == "/favicon.svg" {
        "public, max-age=345600, stale-while-revalidate=345600"
    } else {
        "public, max-age=3600, stale-while-revalidate=3600"
    };

    let mut response = next.run(request).await;
    if response.status().is_success() {
        response.headers_mut().insert(
            reqwest::header::CACHE_CONTROL,
            HeaderValue::from_static(header),
        );
    }
    response
}

#[cfg(test)]
mod tests {
    use crate::{
        google::{Claims, parse},
        kek, run_server,
    };
    use anyhow::{Result, anyhow};
    use jsonwebtoken::{EncodingKey, encode};
    use reqwest::{Client, StatusCode};
    use std::{net::SocketAddr, time::Duration};
    use tokio::task::JoinHandle;
    use tokio_util::sync::CancellationToken;

    const TEST_KEK: &str = r#"{"v1":"jRg36ErQ6FLcc7nZgngOpjJnJLGwA3xaMy0yx1pxJrI"}"#;
    const ALICE_ENVELOPE: &str =
        r#"{"dek":"gVwG8pMMMtdq6mS0OW19Kn7XwvdUcFJpkYN8cEnwnvs","emails":["alice@email.com"]}"#;

    fn bearer(email: &str, name: &str) -> String {
        let encoding_key =
            EncodingKey::from_rsa_pem(include_str!("testdata/pk.pem").as_bytes()).unwrap();
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("1".into());
        let claims = Claims {
            email: email.into(),
            name: name.into(),
            exp: 2524636800,
            iss: "https://accounts.google.com".to_string(),
            aud: "981002175662-g8jr2n89bptsn8n9ds1fn5edfheojr7i.apps.googleusercontent.com"
                .to_string(),
        };
        let token = encode(&header, &claims, &encoding_key).unwrap();
        format!("Bearer {token}")
    }

    #[test_log::test(tokio::test)]
    async fn post_seal_succeeds() {
        let (server, addr) = start_server().await;
        let client = Client::default();

        let resp = client
            .post(format!("http://{addr}/api/seal"))
            .header("Content-Type", "application/json")
            .body(ALICE_ENVELOPE)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        println!("{:?}", resp.text().await.unwrap());

        server.shutdown_and_wait().await.unwrap();
    }

    #[test_log::test(tokio::test)]
    async fn post_unseal_alice_succeeds() {
        let (server, addr) = start_server().await;
        let client = Client::default();

        let resp = client
            .post(format!("http://{addr}/api/unseal"))
            .header("Content-Type", "application/json")
            .header("Authorization", bearer("alice@email.com", "Alice"))
            .body(include_str!("testdata/alice.sealed"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        server.shutdown_and_wait().await.unwrap();
    }

    #[test_log::test(tokio::test)]

    async fn post_unseal_eve_fails() {
        let (server, addr) = start_server().await;
        let client = Client::default();

        let resp = client
            .post(format!("http://{addr}/api/unseal"))
            .header("Content-Type", "application/json")
            .header("Authorization", bearer("eve@email.com", "Eve"))
            .body(include_str!("testdata/alice.sealed"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        server.shutdown_and_wait().await.unwrap();
    }

    #[test_log::test(tokio::test)]
    async fn post_unseal_no_auth_fails() {
        let (server, addr) = start_server().await;
        let client = Client::default();

        let resp = client
            .post(format!("http://{addr}/api/unseal"))
            .header("Content-Type", "application/json")
            .body(include_str!("testdata/alice.sealed"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        server.shutdown_and_wait().await.unwrap();
    }

    #[test_log::test(tokio::test)]
    async fn seal_and_unseal_succeeds() {
        let (server, addr) = start_server().await;
        let client = Client::default();

        let seal_resp = client
            .post(format!("http://{addr}/api/seal"))
            .header("Content-Type", "application/json")
            .body(ALICE_ENVELOPE)
            .send()
            .await
            .expect("Failed to send seal request.");

        assert_eq!(seal_resp.status(), StatusCode::OK);
        let body = seal_resp.text().await.expect("Failed to read response");

        let unseal_resp = client
            .post(format!("http://{addr}/api/unseal"))
            .header("Content-Type", "application/json")
            .header("Authorization", bearer("alice@email.com", "Alice"))
            .body(body)
            .send()
            .await
            .expect("Failed to send unseal request.");
        assert_eq!(unseal_resp.status(), StatusCode::OK);
        assert_eq!(unseal_resp.text().await.unwrap(), ALICE_ENVELOPE);

        server.shutdown_and_wait().await.unwrap();
    }

    struct ServerHandle {
        closer: CancellationToken,
        serve: JoinHandle<Result<()>>,
    }

    impl ServerHandle {
        async fn shutdown_and_wait(mut self) -> Result<()> {
            self.start_shutdown().await;
            self.wait_for_shutdown().await
        }

        async fn start_shutdown(&mut self) {
            tracing::info!("Sending server shutdown signal...");
            self.closer.cancel();
        }

        async fn wait_for_shutdown(self) -> Result<()> {
            match tokio::time::timeout(Duration::from_secs(20), self.serve).await {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(
                    "Timed out waiting for shutdown after 20 seconds: {e}"
                )),
            }
        }
    }

    async fn start_server() -> (ServerHandle, SocketAddr) {
        let certs = parse(include_str!("testdata/certs.json")).unwrap();
        let keks = kek::parse(TEST_KEK).unwrap();

        let cancel = CancellationToken::new();
        let (addr, serve) = run_server(crate::Config {
            port: Some(0),
            certs: Some(certs),
            keks: Some(keks),
            shutdown_signal: cancel.clone(),
        })
        .await
        .unwrap();

        (
            ServerHandle {
                closer: cancel,
                serve,
            },
            addr,
        )
    }
}
