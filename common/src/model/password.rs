use crate::crypto::asymmetric::KeyPair;
use crate::crypto::common::EncryptedValue;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub _id: String,
    pub user_id: String,
    pub site: String,
    pub username: String,
    pub password: String,
}

impl Password {
    pub fn encrypt(&self, key_pair: &KeyPair) -> Self {
        let username_enc = key_pair.encrypt(&self.username);
        let password_enc = key_pair.encrypt(&self.password);

        let mut password = self.clone();
        password.username = username_enc.to_string();
        password.password = password_enc.to_string();
        password
    }

    pub fn decrypt(&self, key_pair: &KeyPair) -> Self {
        let username_enc = EncryptedValue::from(self.username.clone());
        let username_dec = key_pair.decrypt(&username_enc);

        let password_enc = EncryptedValue::from(self.password.clone());
        let password_dec = key_pair.decrypt(&password_enc);

        let mut password = self.clone();
        password.username = username_dec;
        password.password = password_dec;
        password
    }
}
