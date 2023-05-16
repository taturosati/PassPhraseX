use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(hello_world));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().expect("Failed to parse address"))
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

async fn hello_world() -> &'static str {
    "Hello, World!\n"
}