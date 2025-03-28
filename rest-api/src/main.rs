use crate::token::start_token_fetcher;
use app::{print_request_response, AppState};
use axum::routing::{get, post};
use axum::{middleware, Router};
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

mod alpha_move;
pub mod app;
pub mod config;
mod fearandgreed;
pub mod jobs;
mod market_mover;
mod price;
mod response;
mod thirdparty;
mod time_util;
mod token;
pub mod volume;
pub mod wallet;
mod webhook;

#[tokio::main]
async fn main() {
    let app_state = AppState::new().await;
    let shared_state = Arc::new(app_state.clone());
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    let r = Router::new();
    let router = Router::new()
        .route("/price/{address}", get(price::route::get_price))
        .route("/health", get(token::health::health))
        .route("/mindshare", get(token::route::mindshare))
        .route("/token", get(token::route::search_token))
        .route("/token/trending", get(token::route::trending_token))
        .route("/webhook", post(webhook::webhook_handler))
        .route("/vibecheck", get(fearandgreed::route::vibe_check))
        .route("/alphamoves", get(alpha_move::get_mover_transaction))
        .route("/token/{address}/details", get(token::route::get_token_bio))
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response))
        .layer(cors);
    AppState::start_worker(shared_state.clone()).await;
    start_token_fetcher(shared_state.clone()).await;

    let app = r.nest("/api/v1", router);
    let port = env::var("PORT").expect("PORT environment variable not set");
    info!("starting http server on port {port}");

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .expect("should create listener");

    axum::serve(listener, app).await.unwrap();
}
