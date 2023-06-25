use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Password {
    pub _id: String,
    pub user_id: String,
    pub site: String,
    pub username: String,
    pub password: String,
}
