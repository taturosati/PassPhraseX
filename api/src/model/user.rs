use serde::{Deserialize, Serialize};
use crate::model::password::Password;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    #[serde(rename(serialize = "_id"))]
    pub public_key: String,
    #[serde(default)]
    pub passwords: Vec<Password>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename(serialize = "public_key"))]
    pub _id: String,
    pub passwords: Vec<Password>
}

impl User {
    pub fn from_pk(public_key: String) -> Self {
        Self {
            _id: public_key,
            passwords: Vec::new()
        }
    }
}