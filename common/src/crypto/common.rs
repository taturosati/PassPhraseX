use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct EncryptedValue {
    pub cipher: String,
    pub nonce: String,
}

impl From<String> for EncryptedValue {
    fn from(text: String) -> Self {
        let mut parts = text.split(';');
        let cipher = parts.next().expect("Missing cipher");
        let nonce = parts.next().expect("Missing nonce");
        EncryptedValue {
            cipher: cipher.to_owned(),
            nonce: nonce.to_owned(),
        }
    }
}


impl Into<String> for EncryptedValue {
    fn into(self) -> String {
        format!("{};{}", self.cipher, self.nonce)
    }
}

impl Display for EncryptedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{};{}", self.cipher, self.nonce)
    }
}