use aes_gcm::{
    AeadCore, Aes256Gcm,
    aead::{Aead, OsRng},
};
use base64::prelude::*;
use google::Certs;
use rmp_serde::{from_slice, to_vec};
use rocket::{Build, Rocket, State, fs::FileServer, launch, post, routes};
use rocket::{http::Status, serde::json::Json};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

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

#[post("/seal", data = "<envelope>")]
fn seal(
    envelope: Json<Envelope>,
    keks: &State<HashMap<String, Aes256Gcm>>,
) -> Result<Json<SealedEnvelope>, Status> {
    let buf = to_vec::<Envelope>(&envelope).map_err(|_| Status::InternalServerError)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let kek = keks.get("v1").ok_or(Status::InternalServerError)?;
    let ciphertext = kek
        .encrypt(&nonce, buf.as_slice())
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(SealedEnvelope {
        kid: "v1".into(),
        nonce: BASE64_URL_SAFE_NO_PAD.encode(nonce.as_slice()),
        data: BASE64_URL_SAFE_NO_PAD.encode(ciphertext.as_slice()),
    }))
}

#[post("/unseal", data = "<sealed_envelope>")]
fn unseal(
    sealed_envelope: Json<SealedEnvelope>,
    keks: &State<HashMap<String, Aes256Gcm>>,
    claims: google::Claims,
) -> Result<Json<Envelope>, Status> {
    let nonce = BASE64_URL_SAFE_NO_PAD
        .decode(&sealed_envelope.nonce)
        .map_err(|_| Status::Unauthorized)?;
    let ciphertext = BASE64_URL_SAFE_NO_PAD
        .decode(&sealed_envelope.data)
        .map_err(|_| Status::Unauthorized)?;
    let kek = keks.get(&sealed_envelope.kid).ok_or(Status::Unauthorized)?;
    let plaintext = kek
        .decrypt(nonce.as_slice().into(), ciphertext.as_slice())
        .map_err(|_| Status::Unauthorized)?;
    let envelope: Envelope = from_slice(&plaintext).map_err(|_| Status::Unauthorized)?;
    if !envelope.emails.contains(&claims.email) {
        return Err(Status::Unauthorized);
    }
    Ok(Json(envelope))
}

fn cipherly(json_keks: &str, certs: Certs) -> Rocket<Build> {
    let keks = kek::parse(json_keks).expect("Failed to parse KEKs");

    rocket::build()
        .manage(keks)
        .manage(certs)
        .mount("/api", routes![seal, unseal])
        .mount("/", FileServer::from("./static"))
}

#[launch]
fn rocket() -> Rocket<Build> {
    // TODO: Audit that the environment access only happens in single-threaded code.
    unsafe { env::set_var("ROCKET_PORT", env::var("PORT").unwrap_or("8000".into())) };
    let kek = env::var("KEKS").expect("KEKS environment variable is not set");
    let certs = google::fetch().expect("Failed to fetch Google certs");
    cipherly(&kek, certs)
}

#[cfg(test)]
mod tests {
    use super::cipherly;
    use crate::google::{Claims, parse};
    use jsonwebtoken::{EncodingKey, encode};
    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;

    const TEST_KEK: &str = r#"{"v1":"jRg36ErQ6FLcc7nZgngOpjJnJLGwA3xaMy0yx1pxJrI"}"#;
    const ALICE_ENVELOPE: &str =
        r#"{"dek":"gVwG8pMMMtdq6mS0OW19Kn7XwvdUcFJpkYN8cEnwnvs","emails":["alice@email.com"]}"#;

    fn bearer<'h>(email: &str, name: &str) -> Header<'h> {
        let encoding_key =
            EncodingKey::from_rsa_pem(include_str!("testdata/pk.pem").as_bytes()).unwrap();
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some("1".into());
        let claims = Claims {
            email: email.into(),
            name: name.into(),
            exp: 2524636800,
        };
        let token = encode(&header, &claims, &encoding_key).unwrap();
        Header::new("Authorization", format!("Bearer {token}"))
    }

    fn client() -> Client {
        let certs = parse(include_str!("testdata/certs.json")).unwrap();
        Client::tracked(cipherly(TEST_KEK, certs)).expect("valid rocket instance")
    }

    #[test]
    fn post_seal_succeeds() {
        let client = client();
        let resp = client.post("/api/seal").body(ALICE_ENVELOPE).dispatch();
        assert!(resp.status() == Status::Ok);
        println!("{:?}", resp.into_string());
    }

    #[test]
    fn post_unseal_alice_succeeds() {
        let client = client();
        let resp = client
            .post("/api/unseal")
            .header(bearer("alice@email.com", "Alice"))
            .body(include_str!("testdata/alice.sealed"))
            .dispatch();
        assert!(resp.status() == Status::Ok);
    }

    #[test]
    fn post_unseal_eve_fails() {
        let client = client();
        let resp = client
            .post("/api/unseal")
            .header(bearer("eve@email.com", "Eve"))
            .body(include_str!("testdata/alice.sealed"))
            .dispatch();
        assert!(resp.status() == Status::Unauthorized);
    }

    #[test]
    fn post_unseal_no_auth_fails() {
        let client = client();
        let resp = client
            .post("/api/unseal")
            .body(include_str!("testdata/alice.sealed"))
            .dispatch();
        assert!(resp.status() == Status::Unauthorized);
    }

    #[test]
    fn seal_and_unseal_succeeds() {
        let client = client();
        let seal_resp = client.post("/api/seal").body(ALICE_ENVELOPE).dispatch();
        assert!(seal_resp.status() == Status::Ok);
        let body = seal_resp.into_string().unwrap();

        let unseal_resp = client
            .post("/api/unseal")
            .header(bearer("alice@email.com", "Alice"))
            .body(body)
            .dispatch();
        assert!(unseal_resp.status() == Status::Ok);
        assert_eq!(unseal_resp.into_string().unwrap(), ALICE_ENVELOPE);
    }
}
