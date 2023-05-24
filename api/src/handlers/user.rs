use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use crate::AppData;
use crate::handlers::common::HandlerResponse;
use crate::model::user::User;

pub async fn create_user(State(state): State<AppData>, Json(payload): Json<User>) -> HandlerResponse {
    // TODO: Multiple error types
    match state.user_service.create_user(payload).await {
        Ok(user) => HandlerResponse::new(StatusCode::CREATED, user),
        Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err)
    }
}