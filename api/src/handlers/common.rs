use axum::{http::StatusCode, response::{IntoResponse, Response}};
use serde::Serialize;
use serde_json;

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