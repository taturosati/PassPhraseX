use axum::http::StatusCode;
use axum::response::{Response, IntoResponse};
use serde::Serialize;
use serde_json;

pub struct HandlerResponse {
    pub status: StatusCode,
    pub body: String,
}


impl HandlerResponse {
    pub fn new<T: Serialize>(status: StatusCode, body: T) -> Self {
        match serde_json::to_string(&body) {
            Ok(body) => Self {
                status,
                body,
            },
            Err(err) => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: err.to_string(),
            }
        }
    }
}

impl IntoResponse for HandlerResponse {
    fn into_response(self) -> Response {
        (self.status, self.body).into_response()
    }
}