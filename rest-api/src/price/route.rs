use axum::routing::get;
use axum::Router;

pub fn route() -> Router {
    Router::new().route("/price", get(get_price))
}

async fn get_price() -> &'static str {
    "Hello, world!"
}
