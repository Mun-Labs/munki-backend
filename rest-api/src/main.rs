use app::{print_request_response, AppState};
use axum::routing::get;
use axum::{middleware, Router};
use std::env;
use std::sync::Arc;
use tracing::info;

pub mod app;
pub mod config;
mod fearandgreed;
pub mod jobs;
mod price;
mod response;
pub mod thirdparty;
mod time_util;
pub mod token;
pub mod volume;
pub mod wallet;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;
    let shared_state = Arc::new(app_state.clone());
    let r = Router::new();
    let router = Router::new()
        .route("/price/{address}", get(price::route::get_price))
        .route("/health", get(token::search::search_token))
        .route(
            "/fearandgreed",
            get(fearandgreed::route::get_fear_and_greed),
        )
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response));
    AppState::start_worker(shared_state.clone()).await;

    let app = r.nest("/api/v1", router);
    let port = env::var("PORT").expect("PORT environment variable not set");
    info!("starting http server on port {port}");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("should create listener");

    axum::serve(listener, app).await.unwrap();
}
