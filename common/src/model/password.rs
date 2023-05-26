use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub site: String,
    pub username: String,
    pub password: String
}

