use std::sync::Arc;

use anyhow::{Result, anyhow};
use axum::{
    Extension,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Key {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Certs {
    keys: Vec<Key>,
}

impl Certs {
    pub fn get(&self, kid: &str) -> Result<DecodingKey> {
        for key in &self.keys {
            if key.kid == *kid {
                return Ok(DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Err(anyhow!("missing"))
    }
}

pub fn parse(json: &str) -> Result<Certs> {
    let certs: Certs = serde_json::from_str(json)?;
    Ok(certs)
}

pub async fn fetch() -> Result<Certs> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()
        .await?
        .text()
        .await?;
    let certs: Certs = parse(&resp)?;
    Ok(certs)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub name: String,
    pub exp: usize,
}

#[tracing::instrument(skip(request, next), fields(email))]
pub(crate) async fn authenticate(
    Extension(certs): Extension<Arc<Certs>>,
    mut request: Request,
    next: Next,
) -> Response {
    let Some(auth_header) = request.headers().get("Authorization") else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(auth_header) = auth_header.to_str() else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let parts: Vec<&str> = auth_header.split(' ').collect();
    if parts.len() != 2 || parts[0] != "Bearer" {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let bearer = parts[1];
    let Ok(header) = jsonwebtoken::decode_header(bearer) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(kid) = header.kid else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Ok(key) = certs.get(&kid) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
    validation.set_audience(&[
        "981002175662-g8jr2n89bptsn8n9ds1fn5edfheojr7i.apps.googleusercontent.com",
    ]);
    validation.required_spec_claims.insert("aud".to_string());
    validation.set_issuer(&["https://accounts.google.com"]);
    validation.required_spec_claims.insert("iss".to_string());
    let Ok(token) = jsonwebtoken::decode::<Claims>(bearer, &key, &validation) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let claims = Claims {
        email: token.claims.email,
        name: token.claims.name,
        exp: token.claims.exp,
    };

    tracing::Span::current().record("email", &claims.email);
    assert!(request.extensions_mut().insert(claims).is_none());

    next.run(request).await
}

#[cfg(test)]
mod tests {
    use crate::google::{Certs, fetch, parse};

    fn certs() -> Certs {
        parse(include_str!("testdata/certs.json")).unwrap()
    }

    #[tokio::test]
    async fn fetch_succeeds() {
        let result = fetch().await;
        assert!(result.is_ok());
    }

    #[test]
    fn get_returns_error_if_kid_is_missing() {
        let certs = certs();
        assert!(certs.get("missing").is_err())
    }

    #[test]
    fn get_returns_key_if_kid_exists() {
        let certs = certs();
        assert!(certs.get("1").is_ok())
    }
}
