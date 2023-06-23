use crate::error::common::ApiError;
use crate::model::common::GetCollection;
use mongodb::bson::doc;
use mongodb::options::UpdateOptions;
use mongodb::{Client, Collection};
use passphrasex_common::model::password::Password;
use passphrasex_common::model::user::User;

#[derive(Clone)]
pub struct PasswordService {
    user_collection: Collection<User>,
}

impl PasswordService {
    pub fn new(client: &Client) -> Self {
        Self {
            user_collection: client.get_collection("users"),
        }
    }

    pub async fn list_passwords(&self, user_id: String) -> Result<Vec<Password>, ApiError> {
        let filter = doc! {"_id": user_id.clone()};

        match self.user_collection.find_one(filter, None).await {
            Ok(result) => match result {
                Some(user) => Ok(user.passwords),
                None => Err(ApiError::UserNotFound(user_id)),
            },
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }

    pub async fn add_password(
        &self,
        user_id: String,
        password: Password,
    ) -> Result<Password, ApiError> {
        let filter = doc! {"_id": user_id.clone()};

        let update = doc! {
            "$addToSet": {
                "passwords": {
                    "_id": password._id.clone(),
                    "site": password.site.clone(),
                    "username": password.username.clone(),
                    "password": password.password.clone()
                }
            }
        };

        match self
            .user_collection
            .find_one_and_update(filter, update, None)
            .await
        {
            Ok(result) => match result {
                Some(_) => Ok(password),
                None => Err(ApiError::UserNotFound(user_id)),
            },
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }

    pub async fn delete_password(
        &self,
        user_id: String,
        password_id: String,
    ) -> Result<(), ApiError> {
        let filter = doc! {"_id": user_id.clone()};

        let update = doc! {
            "$pull": {
                "passwords": {
                    "_id": password_id.clone()
                }
            }
        };

        match self
            .user_collection
            .find_one_and_update(filter, update, None)
            .await
        {
            Ok(result) => match result {
                Some(_) => Ok(()),
                None => Err(ApiError::UserNotFound(user_id)),
            },
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }

    pub async fn modify_password(
        &self,
        user_id: String,
        password_id: String,
        password: String,
    ) -> Result<(), ApiError> {
        let filter = doc! {"_id": user_id.clone()};

        let update = doc! {
            "$set": {
                "passwords.$[password].password": password.clone()
            }
        };

        let options = UpdateOptions::builder()
            .array_filters(Some(vec![doc! {"password._id": password_id.clone()}]))
            .build();

        match self
            .user_collection
            .update_one(filter, update, Some(options))
            .await
        {
            Ok(result) => {
                if result.matched_count == 0 {
                    Err(ApiError::UserNotFound(user_id))
                } else if result.modified_count == 0 {
                    Err(ApiError::PasswordNotFound(password_id))
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crate::model::common::{DatabaseConfig, GetCollection};
    use mongodb::bson::doc;
    use mongodb::{Client, Collection};
    use passphrasex_common::model::password::Password;
    use passphrasex_common::model::user::User;

    const USER_ID: &str = "user_id";
    const PASSWORD_ID: &str = "password_id";

    static INIT_MUTEX: Mutex<bool> = Mutex::new(false);

    async fn setup() -> Client {
        let db_config = DatabaseConfig::new();

        let client = db_config
            .into_client()
            .await
            .expect("Failed to connect to database");

        let mut initialized = INIT_MUTEX.lock().expect("Failed to get mutex");
        if !*initialized {
            let collection: Collection<User> = client.get_collection("users");

            collection
                .delete_one(doc! {"_id": USER_ID.to_string()}, None)
                .await
                .expect("Failed to delete test user");

            collection
                .insert_one(
                    User {
                        _id: USER_ID.to_string(),
                        passwords: vec![Password {
                            _id: PASSWORD_ID.to_string(),
                            site: "test.com".to_string(),
                            username: "user".to_string(),
                            password: "password".to_string(),
                        }],
                    },
                    None,
                )
                .await
                .expect("Failed to insert test user");
        }

        *initialized = true;
        client
    }

    mod add_password {
        use super::setup;
        use crate::model::common::GetCollection;
        use crate::service::password::PasswordService;
        use mongodb::bson::doc;
        use mongodb::Collection;
        use passphrasex_common::model::password::Password;
        use passphrasex_common::model::user::User;

        #[tokio::test]
        async fn add_password() {
            let client = setup().await;
            let collection: Collection<User> = client.get_collection("users");
            let service = PasswordService::new(&client);

            const NEW_PASSWORD_ID: &str = "new_password_id";
            let result = service
                .add_password(
                    "user_id".to_string(),
                    Password {
                        _id: NEW_PASSWORD_ID.to_string(),
                        site: "new_site".to_string(),
                        username: "new_username".to_string(),
                        password: "new_password".to_string(),
                    },
                )
                .await;

            assert!(result.is_ok());

            let user: User = collection
                .find_one(doc! {"_id": "user_id"}, None)
                .await
                .expect("Failed to find user")
                .expect("User not found");

            assert_eq!(user.passwords.len(), 2);
            assert_eq!(user.passwords[1]._id, NEW_PASSWORD_ID);
        }
    }

    mod modify_password {
        use super::setup;
        use super::{PASSWORD_ID, USER_ID};
        use crate::error::common::ApiError;
        use crate::model::common::GetCollection;
        use crate::service::password::PasswordService;
        use mongodb::bson::doc;
        use mongodb::Collection;
        use passphrasex_common::model::user::User;

        #[tokio::test]
        async fn modify_password() -> anyhow::Result<()> {
            let client = setup().await;
            let collection: Collection<User> = client.get_collection("users");
            let service = PasswordService::new(&client);

            let result = service
                .modify_password(
                    USER_ID.to_string(),
                    PASSWORD_ID.to_string(),
                    "new_password".to_string(),
                )
                .await;

            assert!(result.is_ok());

            let user: User = collection
                .find_one(doc! {"_id": USER_ID.to_string()}, None)
                .await?
                .ok_or(anyhow::anyhow!("User not found"))?;

            assert_eq!(user.passwords.len(), 1);
            assert_eq!(user.passwords[0].password, "new_password".to_string());
            Ok(())
        }

        #[tokio::test]
        async fn modify_password_missing_user() -> anyhow::Result<()> {
            let client = setup().await;
            let service = PasswordService::new(&client);

            let result = service
                .modify_password(
                    "wrong_id".to_string(),
                    PASSWORD_ID.to_string(),
                    "new_password".to_string(),
                )
                .await;

            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::UserNotFound(_))));
            Ok(())
        }

        #[tokio::test]
        async fn modify_password_missing_password() -> anyhow::Result<()> {
            let client = setup().await;
            let service = PasswordService::new(&client);

            let result = service
                .modify_password(
                    USER_ID.to_string(),
                    "wrong_id".to_string(),
                    "new_password".to_string(),
                )
                .await;

            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::PasswordNotFound(_))));
            Ok(())
        }
    }
}
