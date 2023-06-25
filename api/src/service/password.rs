use crate::error::common::ApiError;
use crate::model::common::GetCollection;
use crate::service::user::UserService;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::error::ErrorKind;
use mongodb::error::WriteFailure::WriteError;
use mongodb::{Client, Collection};
use passphrasex_common::model::password::Password;

#[derive(Clone)]
pub struct PasswordService {
    user_service: UserService,
    password_collection: Collection<Password>,
}

impl PasswordService {
    pub fn new(client: &Client, user_service: UserService) -> Self {
        Self {
            user_service,
            password_collection: client.get_collection("passwords"),
        }
    }

    pub async fn list_passwords(&self, user_id: String) -> Result<Vec<Password>, ApiError> {
        let filter = doc! {"user_id": user_id.clone()};

        match self.password_collection.find(filter, None).await {
            Ok(result) => result.try_collect().await.map_err(|err| {
                ApiError::InternalServerError(format!("Error while collecting passwords: {}", err))
            }),
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }

    pub async fn add_password(&self, password: Password) -> Result<Password, ApiError> {
        self.user_service.get_user(password.user_id.clone()).await?;
        let result = self.password_collection.insert_one(&password, None).await;
        match result {
            Ok(_) => Ok(password),
            Err(err) => match err.kind.as_ref() {
                ErrorKind::Write(WriteError(error)) => match error.code {
                    11000 => Err(ApiError::PasswordAlreadyExists(password._id)),
                    _ => Err(ApiError::InternalServerError(err.to_string())),
                },
                _ => Err(ApiError::InternalServerError(err.to_string())),
            },
        }
    }

    pub async fn delete_password(
        &self,
        user_id: String,
        password_id: String,
    ) -> Result<(), ApiError> {
        let filter = doc! {"user_id": user_id.clone(), "_id": password_id.clone()};

        match self.password_collection.delete_one(filter, None).await {
            Ok(result) => {
                if result.deleted_count == 0 {
                    Err(ApiError::PasswordNotFound(password_id))
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(ApiError::InternalServerError(err.to_string())),
        }
    }

    pub async fn modify_password(
        &self,
        user_id: String,
        password_id: String,
        password: String,
    ) -> Result<(), ApiError> {
        let filter = doc! {"user_id": user_id.clone(), "_id": password_id.clone()};

        let update = doc! {
            "$set": {
                "password": password.clone()
            }
        };

        let result = self
            .password_collection
            .update_one(filter, update, None)
            .await;

        match result {
            Ok(result) => {
                if result.modified_count == 0 || result.matched_count == 0 {
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
                    },
                    None,
                )
                .await
                .expect("Failed to insert test user");

            let collection: Collection<Password> = client.get_collection("passwords");

            collection
                .delete_one(doc! {"_id": PASSWORD_ID.to_string()}, None)
                .await
                .expect("Failed to delete test password");

            collection
                .insert_one(
                    Password {
                        _id: PASSWORD_ID.to_string(),
                        user_id: USER_ID.to_string(),
                        site: "site".to_string(),
                        username: "username".to_string(),
                        password: "password".to_string(),
                    },
                    None,
                )
                .await
                .expect("Failed to insert test password");
        }

        *initialized = true;
        client
    }

    mod add_password {
        use super::{PASSWORD_ID, USER_ID};
        use std::sync::Mutex;

        use mongodb::bson::doc;
        use mongodb::{Client, Collection};

        use passphrasex_common::model::password::Password;

        use crate::error::common::ApiError;
        use crate::model::common::GetCollection;
        use crate::service::password::PasswordService;
        use crate::service::user::UserService;

        const NEW_PASSWORD_ID: &str = "new_password_id";

        static INIT_MUTEX: Mutex<bool> = Mutex::new(false);

        async fn setup() -> Client {
            let client = super::setup().await;

            let mut initialized = INIT_MUTEX.lock().expect("Failed to get mutex");
            if !*initialized {
                let collection: Collection<Password> = client.get_collection("passwords");
                let filter = doc! {"_id": NEW_PASSWORD_ID.to_string()};
                collection
                    .delete_one(filter, None)
                    .await
                    .expect("Failed to delete passwords");

                *initialized = true;
            }

            client
        }

        #[tokio::test]
        async fn add_password() {
            let client = setup().await;

            let result =
                add_password_internal(&client, USER_ID.to_string(), NEW_PASSWORD_ID.to_string())
                    .await;

            assert!(result.is_ok());

            let collection: Collection<Password> = client.get_collection("passwords");

            let password: Option<Password> = collection
                .find_one(
                    doc! {"user_id": USER_ID.to_string(), "_id": NEW_PASSWORD_ID.to_string() },
                    None,
                )
                .await
                .expect("Failed to get password");

            assert!(password.is_some());
        }

        #[tokio::test]
        async fn add_password_missing_user() {
            let client = setup().await;

            let result =
                add_password_internal(&client, "no_user".to_string(), NEW_PASSWORD_ID.to_string())
                    .await;

            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::UserNotFound(_))));
        }

        #[tokio::test]
        async fn add_password_duplicate() {
            let client = setup().await;

            let result =
                add_password_internal(&client, USER_ID.to_string(), PASSWORD_ID.to_string()).await;

            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::PasswordAlreadyExists(_))));
        }

        async fn add_password_internal(
            client: &Client,
            user_id: String,
            password_id: String,
        ) -> Result<Password, ApiError> {
            let service = PasswordService::new(&client, UserService::new(&client));

            let password = Password {
                _id: password_id,
                user_id,
                site: "site".to_string(),
                username: "username".to_string(),
                password: "password".to_string(),
            };

            service.add_password(password).await
        }
    }

    mod modify_password {
        use super::setup;
        use super::{PASSWORD_ID, USER_ID};
        use crate::error::common::ApiError;
        use crate::model::common::GetCollection;
        use crate::service::password::PasswordService;
        use crate::service::user::UserService;
        use mongodb::bson::doc;
        use mongodb::Collection;
        use passphrasex_common::model::password::Password;

        #[tokio::test]
        async fn modify_password() -> anyhow::Result<()> {
            let client = setup().await;
            let service = PasswordService::new(&client, UserService::new(&client));

            let result = service
                .modify_password(
                    USER_ID.to_string(),
                    PASSWORD_ID.to_string(),
                    "new_password".to_string(),
                )
                .await;

            assert!(result.is_ok());
            let collection: Collection<Password> = client.get_collection("passwords");
            let filter = doc! {"_id": PASSWORD_ID.to_string(), "user_id": USER_ID.to_string()};
            let password: Password = collection
                .find_one(filter, None)
                .await?
                .ok_or(ApiError::PasswordNotFound(PASSWORD_ID.to_string()))?;

            assert!(password.password == "new_password");
            Ok(())
        }

        #[tokio::test]
        async fn modify_password_missing_user() -> anyhow::Result<()> {
            let client = setup().await;
            let service = PasswordService::new(&client, UserService::new(&client));

            let result = service
                .modify_password(
                    "wrong_id".to_string(),
                    PASSWORD_ID.to_string(),
                    "new_password".to_string(),
                )
                .await;

            assert!(result.is_err());
            assert!(matches!(result, Err(ApiError::PasswordNotFound(_))));
            Ok(())
        }

        #[tokio::test]
        async fn modify_password_missing_password() -> anyhow::Result<()> {
            let client = setup().await;
            let service = PasswordService::new(&client, UserService::new(&client));

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
