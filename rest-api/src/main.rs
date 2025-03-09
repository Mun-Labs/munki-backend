use app::{print_request_response, AppState};
use axum::routing::get;
use axum::{middleware, Router};

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
    let r = Router::new();

    let router = Router::new()
        .route("/price/{address}", get(price::route::get_price))
        .with_state(app_state)
        .layer(middleware::from_fn(print_request_response));

    let app = r.nest("/api/v1", router);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("should create listener");

    axum::serve(listener, app).await.unwrap();
}
