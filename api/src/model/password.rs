use mongodb::bson::{Bson, doc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub site: String,
    pub username: String,
    pub password: String
}

impl From<Password> for Bson {
    fn from(password: Password) -> Bson {
        Bson::Document(doc! {
            "site": password.site,
            "username": password.username,
            "password": password.password
        })
    }
}
