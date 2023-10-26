use crate::model::password::Password;
use std::collections::HashMap;

pub type CredentialsMap = HashMap<String, HashMap<String, Password>>;

pub mod password;
pub mod user;
