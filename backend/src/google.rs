use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use rocket::State;
use rocket::request::FromRequest;
use rocket::{Request, http::Status, request::Outcome};
use serde::{Deserialize, Serialize};
use std::error::Error;

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
    pub fn get(&self, kid: &str) -> Result<DecodingKey, Box<dyn Error>> {
        for key in &self.keys {
            if key.kid == *kid {
                return Ok(DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Err("missing".into())
    }
}

pub fn parse(json: &str) -> Result<Certs, Box<dyn Error>> {
    let certs: Certs = serde_json::from_str(json)?;
    Ok(certs)
}

pub fn fetch() -> Result<Certs, Box<dyn Error>> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://www.googleapis.com/oauth2/v3/certs")
        .send()?
        .text()?;
    let certs: Certs = parse(&resp)?;
    Ok(certs)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub email: String,
    pub name: String,
    pub exp: usize,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let Outcome::Success(certs) = request.guard::<&State<Certs>>().await else {
            return Outcome::Error((Status::InternalServerError, ()));
        };
        let Some(auth_header) = request.headers().get_one("Authorization") else {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        let parts: Vec<&str> = auth_header.split(' ').collect();
        if parts.len() != 2 || parts[0] != "Bearer" {
            return Outcome::Error((Status::Unauthorized, ()));
        }
        let bearer = parts[1];
        let Ok(header) = jsonwebtoken::decode_header(bearer) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        let Some(kid) = header.kid else {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        let Ok(key) = certs.get(&kid) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        let mut validation = Validation::new(jsonwebtoken::Algorithm::RS256);
        validation.set_audience(&[
            "981002175662-g8jr2n89bptsn8n9ds1fn5edfheojr7i.apps.googleusercontent.com",
        ]);
        validation.set_issuer(&["https://accounts.google.com"]);
        let Ok(token) = jsonwebtoken::decode::<Claims>(bearer, &key, &validation) else {
            return Outcome::Error((Status::Unauthorized, ()));
        };
        Outcome::Success(Claims {
            email: token.claims.email,
            name: token.claims.name,
            exp: token.claims.exp,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::google::Certs;
    use crate::google::{fetch, parse};

    fn certs() -> Certs {
        parse(include_str!("testdata/certs.json")).unwrap()
    }

    #[test]
    fn fetch_succeeds() {
        let result = fetch();
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
