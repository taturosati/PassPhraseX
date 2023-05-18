use std::fmt::Debug;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use mongodb::{Client, Collection};
use mongodb::error::ErrorKind;
use serde::Serialize;
use crate::handlers::common::HandlerResponse;
use crate::model::user::{CreateUser, User};

pub async fn create_user(State(state): State<Client>, Json(payload): Json<CreateUser>) -> HandlerResponse {
    println!("Create user: {}", payload.public_key);
    let db = state.database("passphrasex");
    let collection: Collection<CreateUser> = db.collection("users");
    match collection.insert_one(&payload, None).await {
        Ok(_) => HandlerResponse::new(StatusCode::CREATED, User::from_pk(payload.public_key)),
        Err(err) =>
            HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}