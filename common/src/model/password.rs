use crate::crypto::asymmetric::KeyPair;
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
        let username_dec = key_pair.decrypt(&self.username);
        let password_dec = key_pair.decrypt(&self.password);

        let mut password = self.clone();
        password.username = username_dec;
        password.password = password_dec;
        password
    }
}
