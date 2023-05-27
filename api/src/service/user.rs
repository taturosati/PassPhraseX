use mongodb::{Client, Collection};
use common::model::user::User;
use crate::model::common::GetCollection;

#[derive(Clone)]
pub struct UserService {
	user_collection: Collection<User>
}

impl UserService {
	pub fn new(client: &Client) -> Self {
		Self {
			user_collection: client.get_collection("users")
		}
	}

	pub async fn create_user(&self, user: User) -> Result<User, String> {
		match self.user_collection.insert_one(&user, None).await {
			Ok(_) => Ok(user),
			Err(err) => Err(err.to_string())
		}
	}
}