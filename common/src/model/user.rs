use crate::model::password::Password;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(alias = "public_key")]
    pub _id: String,
    #[serde(default)]
    pub passwords: Vec<Password>,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let is_human_readable = serializer.is_human_readable();
        let mut state = serializer.serialize_struct("User", 2)?;

        // If serializer is json then rename _id to public_key
        if is_human_readable {
            state.serialize_field("public_key", &self._id)?;
        } else {
            state.serialize_field("_id", &self._id)?;
        }

        // state.serialize_field("_id", &self._id)?;
        state.serialize_field("passwords", &self.passwords)?;
        state.end()
    }
}
