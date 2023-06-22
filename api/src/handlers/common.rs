use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use serde_json;
use crate::error::common::ApiError;

pub struct HandlerResponse {
    pub status: StatusCode,
    pub body: Option<String>,
}

impl HandlerResponse {
    pub fn new<T: Serialize>(status: StatusCode, body: T) -> Self {
        match serde_json::to_string(&body) {
            Ok(body) => Self {
                status,
                body: Some(body),
            },
            Err(err) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: Some(err.to_string()),
            }
        }
    }
}

impl From<ApiError> for HandlerResponse {
    fn from(value: ApiError) -> Self {
        match value {
            ApiError::UserNotFound(_) => Self::new(StatusCode::NOT_FOUND, value),
            ApiError::PasswordNotFound(_) => Self::new(StatusCode::NOT_FOUND, value),
            ApiError::UserAlreadyExists(_) => Self::new(StatusCode::BAD_REQUEST, value),
            _ => Self::new(StatusCode::INTERNAL_SERVER_ERROR, value)
        }
    }
}

impl IntoResponse for HandlerResponse {
    fn into_response(self) -> Response {
        match self.body {
            Some(body) => (self.status, body).into_response(),
            None => self.status.into_response()
        }
    }
}

impl Default for HandlerResponse {
    fn default() -> Self {
        Self {
            status: StatusCode::OK,
            body: None,
        }
    }
}