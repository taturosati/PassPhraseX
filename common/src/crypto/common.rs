use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct EncryptedValue {
    pub cipher: String,
    pub nonce: String,
}

impl From<String> for EncryptedValue {
    fn from(value: String) -> Self {
        let mut parts = value.split(';');
        let cipher = parts.next().expect("Missing cipher");
        let nonce = parts.next().expect("Missing nonce");
        Self {
            cipher: cipher.to_owned(),
            nonce: nonce.to_owned(),
        }
    }
}

impl From<EncryptedValue> for String {
    fn from(value: EncryptedValue) -> Self {
        format!("{};{}", value.cipher, value.nonce)
    }
}

impl Display for EncryptedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.cipher, self.nonce)
    }
}
