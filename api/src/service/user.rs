use crate::error::common::ApiError;
use crate::model::common::GetCollection;
use mongodb::bson::doc;
use mongodb::error::ErrorKind;
use mongodb::error::WriteFailure::WriteError;
use mongodb::{Client, Collection};
use passphrasex_common::model::user::User;

#[derive(Clone)]
pub struct UserService {
    user_collection: Collection<User>,
}

impl UserService {
    pub fn new(client: &Client) -> Self {
        Self {
            user_collection: client.get_collection("users"),
        }
    }

    pub async fn create_user(&self, user: User) -> Result<User, ApiError> {
        match self.user_collection.insert_one(&user, None).await {
            Ok(_) => Ok(user),
            Err(err) => match err.kind.as_ref() {
                ErrorKind::Write(WriteError(error)) => match error.code {
                    11000 => Err(ApiError::UserAlreadyExists(user._id)),
                    _ => Err(ApiError::InternalServerError(err.to_string())),
                },
                _ => Err(ApiError::InternalServerError(err.to_string())),
            },
        }
    }

    pub async fn get_user(&self, user_id: String) -> Result<User, ApiError> {
        let collection = &self.user_collection;
        let filter = doc! {"_id": user_id.clone()};
        match collection.find_one(filter, None).await {
            Ok(Some(user)) => Ok(user),
            Ok(None) => Err(ApiError::UserNotFound(user_id)),
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }
}
