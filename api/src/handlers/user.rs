use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use mongodb::Client;
use mongodb::error::ErrorKind;
use crate::model::user::User;

pub async fn create_user(State(state): State<Client>, Json(payload): Json<User>) -> (StatusCode, String){
    println!("Create user: {}", payload.public_key);
    let db = state.database("passphrasex");
    let collection = db.collection("users");
    match collection.insert_one(payload, None).await {
        Ok(_) => (StatusCode::CREATED, "".to_string()),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
    }
}