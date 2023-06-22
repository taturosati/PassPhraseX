use axum::{extract::{Path, State}, http::StatusCode, Json, middleware, Router};
use axum::body::Body;
use axum::routing::{delete, get, post, put};
use passphrasex_common::model::password::Password;
use crate::AppData;
use crate::handlers::common::HandlerResponse;
use crate::handlers::middleware::auth::only_user;

pub struct PasswordController {
    pub router: Router<AppData, Body>
}


impl PasswordController {
    pub fn new() -> Self {
        let router = Router::new()
            .route("/users/:user_id/passwords", post(Self::add_password))
            .route("/users/:user_id/passwords", get(Self::list_passwords))
            .route("/users/:user_id/passwords/:password_id", delete(Self::delete_password))
            .route("/users/:user_id/passwords/:password_id/password", put(Self::modify_password))
            .route_layer(middleware::from_fn(only_user));

        Self {
            router
        }
    }

    pub async fn add_password(
        State(state): State<AppData>,
        Path(user_id): Path<String>,
        Json(payload): Json<Password>
    ) -> HandlerResponse {
        match state.password_service.add_password(user_id, payload).await {
            Ok(password) => HandlerResponse::new(StatusCode::CREATED, password),
            Err(err) => HandlerResponse::from(err)
        }
    }

    pub async fn list_passwords(
        State(state): State<AppData>,
        Path(user_id): Path<String>
    ) -> HandlerResponse {
        match state.password_service.list_passwords(user_id).await {
            Ok(passwords) => HandlerResponse::new(StatusCode::OK, passwords),
            Err(err) => HandlerResponse::from(err)
        }
    }

    pub async fn delete_password(
        State(state): State<AppData>,
        Path((user_id, password_id)): Path<(String, String)>
    ) -> HandlerResponse {
        match state.password_service.delete_password(user_id, password_id).await {
            Ok(_) => HandlerResponse::new(StatusCode::NO_CONTENT, ()),
            Err(err) => HandlerResponse::from(err)
        }
    }

    pub async fn modify_password(
        State(state): State<AppData>,
        Path((user_id, password_id)): Path<(String, String)>,
        payload: String
    ) -> HandlerResponse {
        match state.password_service.modify_password(user_id, password_id, payload).await {
            Ok(_) => HandlerResponse::new(StatusCode::NO_CONTENT, ()),
            Err(err) => HandlerResponse::from(err)
        }
    }
}