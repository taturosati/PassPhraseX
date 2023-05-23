use mongodb::bson::{Bson, doc};
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct Password {
    pub site: String,
    pub credentials: String
}

impl From<Password> for Bson {
    fn from(password: Password) -> Bson {
        Bson::Document(doc! {
            "site": password.site,
            "credentials": password.credentials
        })
    }
}
