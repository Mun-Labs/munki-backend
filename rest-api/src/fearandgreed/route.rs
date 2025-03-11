use crate::fearandgreed::FearAndGreedSdk;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use tracing::error;
use crate::app::AppState;
use crate::fearandgreed::FearAndGreed;

#[derive(Debug, Deserialize)]
pub struct FearAndGreedQuery {
    #[serde(default = "default_limit")]
    limit: i8,
}

// Default values for query parameters
fn default_limit() -> i8 {
    31
}

pub async fn get_fear_and_greed(
    State(app): State<AppState>,
    Query(params): Query<FearAndGreedQuery>,
) -> Result<Json<Vec<FearAndGreed>>, (StatusCode, String)> {
    let alternative_client = &app.alternative_client;
    match alternative_client
        .get_fear_and_greed(&params.limit)
        .await
    {
        Ok(result) => Ok(Json(result)),
        Err(e) => {
            error!("get_fear_and_greed error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Alternative get_fear_and_greed failed: {}", e),
            ))
        }
    }
}