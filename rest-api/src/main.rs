use app::{print_request_response, AppState};
use axum::{middleware, Router};
use sqlx::{Pool, Postgres};

pub mod app;
pub mod config;
mod price;
pub mod thirdparty;
pub mod token;
pub mod wallet;

#[tokio::main]
async fn main() {
    // let database_url = "postgres://postgres:postgres@localhost/example_db";
    //
    // // 2) Create a connection pool
    // let pool = Pool::<Postgres>::connect(database_url).await.unwrap();
    //
    // // 3) Optionally, create the TimescaleDB extension (if you're actually using Timescale)
    // // (This might require SUPERUSER privileges depending on your setup)
    // sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
    //     .execute(&pool)
    //     .await.unwrap();
    //

    let app_state = AppState::new();

    let router = Router::new()
        .with_state(app_state);
    let router = router.nest("/api/v1", token::router().merge(price::route()));
    // .layer(middleware::from_fn(print_request_response));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("should create listener");
    axum::serve(listener, router).await.unwrap();
}
