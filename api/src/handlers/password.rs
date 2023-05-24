use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::AppData;
use crate::handlers::common::HandlerResponse;
use crate::model::password::Password;

pub async fn add_password(State(state): State<AppData>, Path(user_id): Path<String>, Json(payload): Json<Password>) -> HandlerResponse {

    // TODO: Multiple error types
    match state.password_service.add_password(user_id, payload).await {
        Ok(passwords) => HandlerResponse::new(StatusCode::OK, passwords),
        Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err)
    }

}