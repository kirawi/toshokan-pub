use axum::{http::StatusCode, routing, Json, Router};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", routing::get(root))
        .route("/users", routing::post(create_user));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> &'static str {
    "Hello, world!"
}

async fn create_user(Json(payload): Json<CreateUser>) -> (StatusCode, Json<User>) {
    let user = User {
        id: 1337,
        username: payload.username,
    };
    (StatusCode::CREATED, Json(user))
}

#[derive(Deserialize, Serialize)]
struct CreateUser {
    username: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    id: usize,
    username: String,
}
