use std::collections::HashMap;

use crate::app::{AppState, SOLANA, SOL_ADDRESS};
use crate::fearandgreed::{
    batch_insert_fear_and_greed, current_gear_and_fear_history, gear_and_fear_history_by_unixtime,
    get_fear_and_greed_last_31_days, upsert_fear_and_greed, FearAndGreedHistory, FearAndGreedSdk,
};
use crate::fearandgreed::{get_fear_and_greed_by_timestamp, FearAndGreed};
use crate::response::HttpResponse;
use crate::thirdparty::TokenData;
use crate::{app, price, time_util, volume};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use bigdecimal::ToPrimitive;
use chrono::{DateTime, Duration, Timelike, Utc};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct FearAndGreedQuery {
    #[serde(default = "default_limit")]
    limit: i8,
}

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
    let now = Utc::now();
    let start_of_a_day = time_util::get_start_of_day(now);
    let last_week = start_of_a_day - Duration::days(7);
    let yesterday = start_of_a_day - Duration::days(1);
    let last_month = start_of_a_day - Duration::days(31);
    let mut histories: Vec<FearAndGreed> = gear_and_fear_history_by_unixtime(
        &app.pool,
        vec![
            yesterday.timestamp(),
            last_week.timestamp(),
            last_month.timestamp(),
        ],
    )
    .await
    .map(|items| items.into_iter().map(FearAndGreed::from).collect())
    .map_err(|e| {
        error!("gear_and_fear_history_by_unixtime error: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("get_fear_and_greed failed: {e}"),
        )
    })?;

    let Some(solana_price) = price::get_metric_from_db(&app.pool, SOL_ADDRESS)
        .await
        .map_err(|e| {
            error!("BirdEye get_price failed error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "get price failed".to_string(),
            )
        })?
    else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "get price failed".to_string(),
        ));
    };

    let Some(sol_vol) =
        volume::get_volume_by_date(&app.pool, time_util::get_start_of_day(now), SOLANA)
            .await
            .map_err(|e| {
                error!("BirdEye get_price failed error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "get price failed".to_string(),
                )
            })?
    else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "get price failed".to_string(),
        ));
    };

    if let Some(resp) = current_gear_and_fear_history(&app.pool, start_of_a_day.timestamp())
        .await
        .map_err(|e| {
            error!("current_gear_and_fear_history error: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Alternative get_fear_and_greed failed: {e}"),
            )
        })?
    {
        info!("get data from database");
        let current = FearAndGreed::from(resp);
        histories.push(current);

        Ok(Json(HttpResponse {
            status_code: 200,
            data: FearAndGreedResponse {
                fear_and_greed: histories,
                token_prices: HashMap::from([(SOL_ADDRESS.to_string(), solana_price)]),
            },
        }))
    } else {
        info!("refresh data");
        let Some(resp) = get_fear_and_greed_by_timestamp(&app.pool, Utc::now().timestamp())
            .await
            .map_err(|e| {
                error!("get_fear_and_greed error: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Alternative get_fear_and_greed failed: {e}"),
                )
            })?
        else {
            return Err((
                StatusCode::NOT_FOUND,
                "Alternative get_fear_and_greed not found".to_string(),
            ));
        };
        let a = resp.value;
        let c = sol_vol.total24h;
        let billions = c as f64 / 1_000_000_000.0;

        // Map to score based on ranges
        let c = match billions {
            b if b < 1.0 => 10,
            b if b < 2.0 => 20,
            b if b < 3.0 => 30,
            b if b < 4.0 => 40,
            b if b < 5.0 => 50,
            b if b < 6.0 => 60,
            b if b < 7.0 => 70,
            b if b < 8.0 => 80,
            b if b < 9.0 => 90,
            b if b < 10.0 => 90, // $9B to < $10B
            _ => 100,            // ≥ $10B
        } as i64;

        let start_of_today = now
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now);

        let last_week = start_of_today - Duration::days(7);

        let prices =
            price::get_token_prices_between(&app.pool, SOL_ADDRESS, last_week, start_of_today)
                .await
                .map_err(|e| {
                    error!("BirdEye get_price failed error: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "get price failed".to_string(),
                    )
                })?;
        let mut changes = Vec::new();
        for i in 1..prices.len() {
            let prev_price = prices[i - 1].price.to_f64().unwrap_or_default();
            let curr_price = prices[i].price.to_f64().unwrap_or_default();
            let change = ((curr_price - prev_price) / prev_price) * 100.0; // % change
            changes.push(change);
        }

        // Calculate 7-day average change (P)
        let p = if changes.is_empty() {
            0.0 // No change if only one price
        } else {
            changes.iter().sum::<f64>() / changes.len() as f64
        };

        // Normalize to B (0-100 scale)
        let b = (50.0 + p).clamp(0.0, 100.0); // min(max(50 + P, 0), 100)
        let b = b as i64;
        let value = (a as i64 + b + c) / 3;

        let value_classification = if value < 25 {
            "Extreme Fear"
        } else if value < 50 {
            "Fear"
        } else if value < 75 {
            "Greed"
        } else {
            "Extreme Greed"
        };
        let greed = FearAndGreed {
            value,
            value_classification: value_classification.into(),
            timestamp: start_of_today.timestamp(),
            chain: SOLANA.into(),
        };
        if let Err(e) = upsert_fear_and_greed(&app.pool, &greed).await {
            error!("upsert fear and greed error: {e}");
        }
        histories.push(greed);

        Ok(Json(HttpResponse {
            status_code: 200,
            data: FearAndGreedResponse {
                fear_and_greed: histories,
                token_prices: HashMap::from([(SOL_ADDRESS.to_string(), solana_price)]),
            },
        }))
    }
}

impl From<FearAndGreedHistory> for FearAndGreed {
    fn from(value: FearAndGreedHistory) -> Self {
        Self {
            value: value.value as i64,
            value_classification: value.value_classification,
            timestamp: value.unix_timestamp,
            chain: SOLANA.into(),
        }
    }
}
