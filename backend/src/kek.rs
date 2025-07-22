use aes_gcm::{Aes256Gcm, Key, aead::KeyInit};
use base64::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

pub fn parse(json: &str) -> Result<HashMap<String, Aes256Gcm>, Box<dyn Error>> {
    serde_json::from_str::<HashMap<String, Value>>(json)?
        .into_iter()
        .map(|(key, value)| {
            let base64_kek = value
                .as_str()
                .ok_or("KEK should be a Base64 encoded string")?;
            let bytes_kek = BASE64_URL_SAFE_NO_PAD.decode(base64_kek)?;
            let kek = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&bytes_kek));
            Ok::<(String, Aes256Gcm), Box<dyn Error>>((key, kek))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse;

    const TEST_KEKS: &str = r#"{"t1":"jRg36ErQ6FLcc7nZgngOpjJnJLGwA3xaMy0yx1pxJrI","t2":"5wasFWpc1thRkR8Wkghn5hZwWF-vimSxIYYZuALL3i8"}"#;

    #[test]
    fn parse_succeeds() {
        let keks = parse(TEST_KEKS).unwrap();
        assert_eq!(keks.len(), 2);
        assert!(keks.contains_key("t1"));
        assert!(keks.contains_key("t2"));
    }
}
