use aes_gcm::{
    Aes256Gcm,
    aead::{KeyInit, OsRng},
};
use base64::prelude::*;

fn main() {
    let key = Aes256Gcm::generate_key(OsRng);
    println!("New KEK: {}", BASE64_URL_SAFE_NO_PAD.encode(key));
}
