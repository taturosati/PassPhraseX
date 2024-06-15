mod error;
mod handlers;
mod model;
mod service;

use model::common::DatabaseConfig;

use service::password::PasswordService;
use service::user::UserService;

use handlers::user::UserController;

use axum::Router;

#[derive(Clone)]
pub struct AppData {
    user_service: UserService,
    password_service: PasswordService,
}

#[tokio::main]
async fn main() {
    let user_controller = UserController::new();

    let client = DatabaseConfig::new()
        .into_client()
        .await
        .expect("Failed to connect to database");

    println!("Connected to database");

    let user_service = UserService::new(&client);

    let app = Router::new()
        .merge(user_controller.router)
        .with_state(AppData {
            user_service: user_service.clone(),
            password_service: PasswordService::new(&client, user_service),
        });

    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Failed to parse address"))
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}
