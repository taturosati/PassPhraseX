use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum ApiError {
    #[error("User with id {0} not found")]
    UserNotFound(String),
    #[error("Password with hash {0} not found")]
    PasswordNotFound(String),
    #[error("Error creating user: {0}")]
    UserAlreadyExists(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}
