use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq, Clone)]
pub struct EncryptedValue {
    pub cipher: String,
}

impl From<String> for EncryptedValue {
    fn from(value: String) -> Self {
        Self { cipher: value }
    }
}

impl From<EncryptedValue> for String {
    fn from(value: EncryptedValue) -> Self {
        format!("{}", value.cipher)
    }
}

impl Display for EncryptedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cipher)
    }
}
