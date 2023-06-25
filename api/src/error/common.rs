use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug, Serialize)]
pub enum ApiError {
    #[error("User with id {0} not found")]
    UserNotFound(String),
    #[error("Password with hash {0} not found")]
    PasswordNotFound(String),
    #[error("User already exists: {0}")]
    UserAlreadyExists(String),
    #[error("Password already exists: {0}")]
    PasswordAlreadyExists(String),
    #[error("Internal server error: {0}")]
    InternalServerError(String),
}
