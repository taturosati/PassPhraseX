use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use mongodb::{Client, Collection};
use mongodb::bson::doc;
use crate::handlers::common::HandlerResponse;
use crate::model::{common::GetCollection, password::Password, user::User};

pub async fn add_password(State(state): State<Client>, Path(user_id): Path<String>, Json(payload): Json<Password>) -> HandlerResponse {
    let collection: Collection<User> = state.get_collection("users");

    let filter = doc!{"_id": user_id.clone()};
    match collection.find_one(filter.clone(), None).await {
        Ok(user) => match user {
            Some(_) => (),
            None => return HandlerResponse::new(StatusCode::NOT_FOUND, "User not found")
        },
        Err(err) => return HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    };

    let update = doc!{"$addToSet": {"passwords": payload}};
    match collection.update_one(filter, update, None).await {
        Ok(_) => HandlerResponse::new(StatusCode::CREATED, ""),
        Err(err) => HandlerResponse::new(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
    }
}