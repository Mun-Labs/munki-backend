use app::{print_request_response, AppState};
use axum::middleware;

pub mod app;
pub mod config;
pub mod token;
pub mod thirdparty;
pub mod wallet;

#[tokio::main]
async fn main() {
    let _ = AppState::new();
    let router = token::router().layer(middleware::from_fn(print_request_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("should create listener");
    axum::serve(listener, router).await.unwrap();
}
