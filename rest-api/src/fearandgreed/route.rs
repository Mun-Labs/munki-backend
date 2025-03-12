use std::collections::HashMap;

use crate::app::AppState;
use crate::fearandgreed::FearAndGreed;
use crate::fearandgreed::{
    batch_insert_fear_and_greed, get_fear_and_greed_last_31_days, FearAndGreedSdk,
};
use crate::price;
use crate::response::HttpResponse;
use crate::thirdparty::TokenData;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct FearAndGreedQuery {
    #[serde(default = "default_limit")]
    limit: i8,
}

// Default values for query parameters
fn default_limit() -> i8 {
    31
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FearAndGreedResponse {
    pub fear_and_greed: Vec<FearAndGreed>,
    pub token_prices: HashMap<String, TokenData>,
}

pub async fn get_fear_and_greed(
    State(app): State<AppState>,
    Query(params): Query<FearAndGreedQuery>,
) -> Result<Json<HttpResponse<FearAndGreedResponse>>, (StatusCode, String)> {
    let resp = get_fear_and_greed_last_31_days(&app.pool)
        .await
        .map_err(|e| {
            error!("get_fear_and_greed error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Alternative get_fear_and_greed failed: {e}"),
            )
        })?;

    let gaf = if resp.len() == default_limit() as usize {
        info!("Returning cached data");
        resp
    } else {
        let alternative_client = &app.alternative_client;
        let result = alternative_client
            .get_fear_and_greed(params.limit)
            .await
            .map_err(|e| {
                error!("get_fear_and_greed error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Alternative get_fear_and_greed failed: {e}"),
                )
            })?;

        let result = result.iter().map(FearAndGreed::from).collect();
        batch_insert_fear_and_greed(&app.pool, &result)
            .await
            .map_err(|e| {
                error!("get_fear_and_greed error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Alternative get_fear_and_greed failed: {e}"),
                )
            })?;
        result
    };

    let sol_address = "So11111111111111111111111111111111111111112".to_string();
    let sol_price = price::get_price(&app.pool, &app.bird_eye_client, &sol_address)
        .await
        .map_err(|e| {
            error!("BirdEye get_price failed error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "get price failed".to_string(),
            )
        })?;
    Ok(Json(HttpResponse {
        status_code: 200,
        data: FearAndGreedResponse {
            fear_and_greed: gaf,
            token_prices: HashMap::from([(sol_address, sol_price)]),
        },
    }))
}
