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

pub const TEST_USER_SUFFIX: &str = "@test.koso.app";
const INTEG_TEST_KID: &str = "koso-integration-test";

#[derive(Debug, Serialize, Deserialize)]
struct Key {
    kid: String,
    alg: String,
    n: String,
    e: String,
    kty: String,
    r#use: String,
}

pub struct KeySet {
    keys: Vec<Key>,
    enable_test_creds: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Certs {
    keys: Vec<Key>,
}

impl KeySet {
    pub async fn new(enable_test_creds: bool) -> Result<KeySet> {
        let keys = Self::fetch().await?.keys;
        Ok(KeySet {
            keys,
            enable_test_creds,
        })
    }

    fn get(&self, kid: &str) -> Result<DecodingKey> {
        if kid == INTEG_TEST_KID {
            if !self.enable_test_creds {
                return Err(anyhow!(
                    "Tried to fetch key for test creds ({kid}) but test creds aren't enabled."
                ));
            }
            return Ok(DecodingKey::from_rsa_components("MA", "MA")?);
        }

        for key in &self.keys {
            if key.kid == *kid {
                return Ok(DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Err(anyhow!("missing"))
    }

    async fn fetch() -> Result<Certs> {
        let client = reqwest::Client::new();
        let resp = client
            .get("https://www.googleapis.com/oauth2/v3/certs")
            .send()
            .await?
            .text()
            .await?;
        let certs: Certs = Certs::parse(&resp)?;
        Ok(certs)
    }
}

impl Certs {
    fn parse(json: &str) -> Result<Certs> {
        let certs: Certs = serde_json::from_str(json)?;
        Ok(certs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub name: String,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
}

#[tracing::instrument(skip_all, fields(email))]
pub(crate) async fn authenticate(
    Extension(key_set): Extension<Arc<KeySet>>,
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
    let Ok(key) = key_set.get(&kid) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let claims = if kid == INTEG_TEST_KID {
        // Example Jwt:
        //   eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Imtvc28taW50ZWdyYXRpb24tdGVzdCJ9.eyJlbWFpbCI6InRlc3RAdGVzdC5rb3NvLmFwcCIsIm5hbWUiOiJQb2ludHktSGFpcmVkIEJvc3MiLCJwaWN0dXJlIjoiaHR0cHM6Ly9zdGF0aWMud2lraWEubm9jb29raWUubmV0L2RpbGJlcnQvaW1hZ2VzLzYvNjAvQm9zcy5QTkciLCJleHAiOjIwMjQ3ODgwMTR9.3btheBY5h0nQRpWNODfYWQ_mMc26551178jrSDmpv_c
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.insecure_disable_signature_validation();
        validation.set_audience(&["cipherly-tests"]);
        validation.required_spec_claims.insert("aud".to_string());
        validation.set_issuer(&["cipherly-tests"]);
        validation.required_spec_claims.insert("iss".to_string());
        let claims = match jsonwebtoken::decode::<Claims>(bearer, &key, &validation) {
            Ok(token) => token.claims,
            Err(err) => {
                tracing::debug!("Decoding bearer test token failed: {err}");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        };

        if !claims.email.ends_with(TEST_USER_SUFFIX) {
            tracing::debug!("Invalid test cred email: {}", claims.email);
            return StatusCode::UNAUTHORIZED.into_response();
        }
        claims
    } else {
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[
            "981002175662-g8jr2n89bptsn8n9ds1fn5edfheojr7i.apps.googleusercontent.com",
        ]);
        validation.required_spec_claims.insert("aud".to_string());
        validation.set_issuer(&["https://accounts.google.com"]);
        validation.required_spec_claims.insert("iss".to_string());
        match jsonwebtoken::decode::<Claims>(bearer, &key, &validation) {
            Ok(token) => token.claims,
            Err(err) => {
                tracing::debug!("Decoding bearer token failed: {err}");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
    };

    tracing::Span::current().record("email", &claims.email);
    assert!(request.extensions_mut().insert(claims).is_none());

    next.run(request).await
}

#[cfg(test)]
pub mod testing {
    use crate::google::{Certs, KeySet};
    use anyhow::{Context, Result};

    pub fn new_fake_key_set(enable_test_creds: bool) -> Result<KeySet> {
        let keys = Certs::parse(include_str!("testdata/certs.json"))
            .context("Failed to parse test certs")?
            .keys;
        Ok(KeySet {
            keys,
            enable_test_creds,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::google::{INTEG_TEST_KID, KeySet, testing::new_fake_key_set};

    #[tokio::test]
    async fn new_succeeds() {
        let key_set = KeySet::new(false).await.unwrap();
        assert!(!key_set.keys.is_empty());
    }

    #[test]
    fn get_returns_error_if_kid_is_missing() {
        let key_set = new_fake_key_set(false).unwrap();
        assert!(key_set.get("missing").is_err())
    }

    #[test]
    fn get_returns_key_if_kid_exists() {
        let key_set = new_fake_key_set(false).unwrap();
        assert!(key_set.get("1").is_ok())
    }

    #[test]
    fn get_returns_test_key_when_test_creds_enabled() {
        let key_set = new_fake_key_set(true).unwrap();
        assert!(key_set.get(INTEG_TEST_KID).is_ok())
    }

    #[test]
    fn get_returns_error_when_test_creds_enabled() {
        let key_set = new_fake_key_set(false).unwrap();
        assert!(key_set.get(INTEG_TEST_KID).is_err())
    }
}
