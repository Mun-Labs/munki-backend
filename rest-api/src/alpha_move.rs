use crate::app::AppState;
use crate::response::HttpPaginationResponse;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use validator::Validate;

#[derive(serde::Serialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MoverTransaction {
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
    // New fields from market_mover table
    #[serde(rename = "alphaGroup")]
    pub mover_role: String,
    #[serde(rename = "name")]
    pub mover_name: String,
}

#[derive(Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, max = 100))]
    pub limit: i64,
    #[validate(range(min = 0))]
    pub offset: i64,
}
pub async fn get_mover_transaction(
    State(app): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<HttpPaginationResponse<Vec<MoverTransaction>>>, (StatusCode, String)> {
    if let Err(validation_errors) = query.validate() {
        return Err((StatusCode::BAD_REQUEST, validation_errors.to_string()));
    }

    let transactions = fetch_mover_transactions(&app.pool, query.limit, query.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Execute query to count total rows in market_movers_transaction.
    let total: (i64,) = sqlx::query_as(
        "
    SELECT COUNT(*)
    FROM market_movers_transaction mm
    INNER JOIN market_mover m ON mm.wallet_address = m.wallet_address
    WHERE EXISTS(SELECT 1 FROM token_watch WHERE token_watch.token_address = mm.token_address)
    ",
    )
    .fetch_one(&app.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(HttpPaginationResponse {
        code: 200,
        response: transactions,
        last_updated: Utc::now().timestamp(),
        total: total.0,
    }))
}

pub async fn fetch_mover_transactions(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Vec<MoverTransaction>, sqlx::Error> {
    let transactions = sqlx::query_as::<_, MoverTransaction>(
        "
SELECT
            mm.signature,
            mm.token_address,
            mm.wallet_address,
            mm.transaction_type,
            mm.amount,
            mm.block_time,
            mm.slot,
            mm.additional,
            t.name AS token_name,
            t.symbol AS token_symbol,
            m.role AS mover_role,
            m.name AS mover_name
         FROM market_movers_transaction mm
         INNER JOIN market_mover m ON mm.wallet_address = m.wallet_address
         LEFT JOIN tokens t ON mm.token_address = t.token_address
         WHERE EXISTS(SELECT 1 FROM token_watch WHERE token_watch.token_address = mm.token_address)
         ORDER BY mm.slot DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(transactions)
}
