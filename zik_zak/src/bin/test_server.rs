use axum::{response::Json, routing::get, Router};
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Starting test server...");

    let app = Router::new().route("/", get(hello));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Test server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await.unwrap();
}

async fn hello() -> Json<serde_json::Value> {
    Json(json!({"message": "Hello from test server!"}))
}
