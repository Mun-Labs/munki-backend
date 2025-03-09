use crate::app::AppState;
use crate::price::{PriceSdk, TokenPrice};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tracing::error;

pub async fn get_price(
    State(app): State<AppState>,
) -> Result<Json<TokenPrice>, (StatusCode, String)> {
    let birdeye_client = &app.bird_eye_client;
    match birdeye_client.get_price("So11111111111111111111111111111111111111112").await {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("get_price {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "BirdEye get_price failed".to_string(),
            ))
        }
    }
}
