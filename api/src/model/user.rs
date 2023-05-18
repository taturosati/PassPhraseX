use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUser {
    #[serde(rename(serialize = "_id"))]
    pub public_key: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename(serialize = "public_key"))]
    pub _id: String,
    pub passwords: HashMap<String, HashMap<String, String>>
}

impl User {
    pub fn from_pk(public_key: String) -> Self {
        Self {
            _id: public_key,
            passwords: HashMap::new()
        }
    }
}