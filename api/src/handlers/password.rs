use axum::{extract::{Path, State}, http::StatusCode, Json, middleware, Router};
use axum::body::Body;
use axum::routing::{get, post};
use crate::AppData;
use crate::handlers::common::HandlerResponse;
use crate::handlers::middleware::auth::only_user;
use common::model::password::Password;


pub struct PasswordController {
    pub router: Router<AppData, Body>
}

impl PasswordController {
    pub fn new() -> Self {
        let router = Router::new()
            .route("/users/:user_id/passwords", post(Self::add_password))
            .route("/users/:user_id/passwords", get(Self::list_passwords))
            .route_layer(middleware::from_fn(only_user));

        Self {
            router
        }
    }

    pub async fn add_password(State(state): State<AppData>, Path(user_id): Path<String>, Json(payload): Json<Password>) -> HandlerResponse {
        // TODO: Multiple error types
        match state.password_service.add_password(user_id, payload).await {
            Ok(password) => HandlerResponse::new(StatusCode::CREATED, password),
            Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err)
        }
    }

    pub async fn list_passwords(State(state): State<AppData>, Path(user_id): Path<String>) -> HandlerResponse {
        // TODO: Multiple error types
        match state.password_service.list_passwords(user_id).await {
            Ok(passwords) => HandlerResponse::new(StatusCode::OK, passwords),
            Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err)
        }
    }
}