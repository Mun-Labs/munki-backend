use app::{print_request_response, AppState};
use axum::middleware;
use sqlx::{Pool, Postgres};

pub mod app;
pub mod config;
pub mod token;
pub mod thirdparty;
pub mod wallet;

#[tokio::main]
async fn main() {
    let _ = AppState::new();

    let database_url = "postgres://postgres:postgres@localhost/example_db";

    // 2) Create a connection pool
    let pool = Pool::<Postgres>::connect(database_url).await.unwrap();

    // 3) Optionally, create the TimescaleDB extension (if you're actually using Timescale)
    // (This might require SUPERUSER privileges depending on your setup)
    sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
        .execute(&pool)
        .await.unwrap();

    let router = token::router().layer(middleware::from_fn(print_request_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("should create listener");
    axum::serve(listener, router).await.unwrap();


}
