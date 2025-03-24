use crate::alpha_move::token_score::TokenMetric;
use crate::alpha_move::{token_score, transaction};
use crate::app::AppState;
use crate::response::HttpPaginationResponse;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: i64,
    #[validate(range(min = 0))]
    pub offset: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoverTransactionResponse {
    pub signature: String,
    pub token_address: String,
    pub wallet_address: String,
    #[serde(rename = "actionType")]
    pub transaction_type: Option<String>,
    pub amount: BigDecimal,
    #[serde(rename = "time")]
    pub block_time: i64,
    pub slot: i64,
    // New fields from tokens table
    #[serde(rename = "coinName")]
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub token_logo: Option<String>,
    // New fields from market_mover table
    #[serde(rename = "alphaGroup")]
    pub mover_role: String,
    #[serde(rename = "name")]
    pub mover_name: String,
    pub token: Option<TokenScoreResponse>,
    pub decimal: i32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenScoreResponse {
    pub token_address: String,
    pub mun_score: f64,
    pub risk_score: f64,
    pub top_fresh_wallet_holders: i64,
    pub top_smart_wallets_holders: i64,
    pub smart_followers: i64,
}

impl From<&TokenMetric> for TokenScoreResponse {
    fn from(
        TokenMetric {
            token_address,
            mun_score,
            top_fresh_wallet_holders,
            top_smart_wallets_holders,
            smart_followers,
            ..
        }: &TokenMetric,
    ) -> Self {
        Self {
            token_address: token_address.clone(),
            mun_score: mun_score.to_f64().unwrap_or_default(),
            risk_score: 0.0,
            top_fresh_wallet_holders: *top_fresh_wallet_holders,
            top_smart_wallets_holders: *top_smart_wallets_holders,
            smart_followers: *smart_followers,
        }
    }
}
pub async fn get_mover_transaction(
    State(app): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<HttpPaginationResponse<Vec<MoverTransactionResponse>>>, (StatusCode, String)> {
    if let Err(validation_errors) = query.validate() {
        return Err((StatusCode::BAD_REQUEST, validation_errors.to_string()));
    }

    let transactions = transaction::fetch_mover_transactions(&app.pool, query.limit, query.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let address = transactions
        .iter()
        .map(|a| a.token_address.clone())
        .collect();
    let token_metrics: HashMap<String, TokenScoreResponse> =
        token_score::fetch_token_metrics_by_addresses(&app.pool, address)
            .await
            .map(|a| {
                a.iter()
                    .map(TokenScoreResponse::from)
                    .map(|a| (a.token_address.clone(), a))
                    .collect()
            })
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Execute query to count total rows in market_movers_transaction.
    let total = transaction::count_mover_transaction(&app)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let transactions = transactions
        .iter()
        .map(|a| MoverTransactionResponse {
            signature: a.signature.clone(),
            token_address: a.token_address.clone(),
            wallet_address: a.wallet_address.clone(),
            transaction_type: a.transaction_type.clone(),
            amount: a.amount.clone(),
            block_time: a.block_time,
            slot: a.slot,
            token_name: a.token_name.clone(),
            token_symbol: a.token_symbol.clone(),
            mover_role: a.mover_role.clone(),
            mover_name: a.mover_name.clone(),
            token: token_metrics.get(&a.token_address).cloned(),
            decimal: a.decimals,
            token_logo: a.token_logo.clone(),
        })
        .collect();

    Ok(Json(HttpPaginationResponse {
        code: 200,
        response: transactions,
        last_updated: Utc::now().timestamp(),
        total,
    }))
}
