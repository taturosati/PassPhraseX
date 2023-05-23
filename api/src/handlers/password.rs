use axum::{extract::{Path, State}, http::StatusCode, Json};
use mongodb::{Client, Collection, bson::doc};
use mongodb::options::{FindOneAndUpdateOptions, ReturnDocument};
use crate::handlers::common::HandlerResponse;
use crate::model::{common::GetCollection, password::Password, user::User};

pub async fn add_password(State(state): State<Client>, Path(user_id): Path<String>, Json(payload): Json<Password>) -> HandlerResponse {
    let collection: Collection<User> = state.get_collection("users");

    let filter = doc!{"_id": user_id.clone()};
    let update = doc!{"$addToSet": {"passwords": payload}};
    let options = FindOneAndUpdateOptions::builder()
        .return_document(ReturnDocument::After).build();

    match collection.find_one_and_update(filter, update, options).await {
        Ok(result) => match result {
            Some(user) => HandlerResponse::new(StatusCode::OK, user.passwords),
            None => HandlerResponse::new(StatusCode::NOT_FOUND, format!("User with id {} not found", user_id))
        },
        Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}