use axum::{routing::get, Router};
use raphaeldarley::{blog, keys};

use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route_service("/", ServeFile::new("content/index.html"))
        .route("/keys", get(keys::handler_all))
        .route("/keys/:name", get(keys::handler))
        .route("/blog", get(blog::handler_root))
        .route("/blog/:name", get(blog::handler))
        .route_service("/content", ServeDir::new("content"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
