use crate::app::AppState;
use crate::response::HttpResponse;
use crate::time_util;
use crate::token::{query_top_token_volume_history, TokenVolumeHistory};
use axum::extract::{Query, State};
use axum::{http::StatusCode, Json};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use serde::Serialize;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthyResponse {
    pub message: String,
}
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenMindshare {
    pub token_address: String,
    pub change_percentage: f64,
    pub logo_url: String,
    pub name: String,
    pub symbol: String,
    pub volume: BigDecimal,
}
pub async fn mindshare(
    State(app): State<AppState>,
) -> Result<Json<HttpResponse<Vec<TokenMindshare>>>, (StatusCode, String)> {
    let vol = query_top_token_volume_history(&app.pool, 100)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let total_volume: f64 = vol
        .iter()
        .map(|v| v.volume24h.to_f64().unwrap_or_default())
        .sum();
    let percent = vol
        .iter()
        .map(|v| TokenMindshare {
            token_address: v.token_address.clone(),
            change_percentage: (v.volume24h.to_f64().unwrap_or_default() / total_volume) * 100.0,
            logo_url: v.logo_uri.clone().unwrap_or_default(),
            name: v.name.clone(),
            symbol: v.symbol.clone(),
            volume: v.volume24h.clone(),
        })
        .collect::<Vec<_>>();
    Ok(Json(HttpResponse {
        code: 200,
        response: percent,
        last_updated: Utc::now().timestamp(),
    }))
}

// rust
use anyhow::Result;
use serde::Deserialize;
use sqlx::{Pool, Postgres};
use validator::Validate;

use super::query_top_token_volume_history_by_date;

#[derive(Deserialize, Validate)]
pub struct SearchQuery {
    pub q: String,
    #[validate(range(min = 1, max = 100))]
    pub limit: i64,
    #[validate(range(min = 0))]
    pub offset: i64,
}

#[derive(serde::Serialize, sqlx::FromRow)]
pub struct Token {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
    #[serde(rename = "mc")]
    pub marketcap: Option<BigDecimal>,
    #[sqlx(rename = "price24hchange")]
    pub price24hchange: Option<BigDecimal>,
    #[sqlx(rename = "current_price")]
    pub price: Option<BigDecimal>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
    #[serde(rename = "mc")]
    pub marketcap: f64,
    #[serde(rename = "price24hchange")]
    pub price24hchange: f64,
    #[serde(rename = "current_price")]
    pub price: f64,
}

impl From<&Token> for TokenResponse {
    fn from(value: &Token) -> Self {
        Self {
            token_address: value.token_address.clone(),
            name: value.name.clone(),
            symbol: value.symbol.clone(),
            logo_uri: value.logo_uri.clone(),
            marketcap: value
                .marketcap
                .clone()
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
            price24hchange: value
                .price24hchange
                .clone()
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
            price: value
                .price
                .clone()
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingTokenResponse {
    pub token_address: String,
    pub volume24h: BigDecimal,
    pub record_date: i64,
    pub logo_uri: Option<String>,
    pub symbol: String,
    pub name: String,
    pub holder_count: i32,
}

impl From<&TokenVolumeHistory> for TrendingTokenResponse {
    fn from(
        TokenVolumeHistory {
            token_address,
            volume24h,
            record_date,
            name,
            symbol,
            logo_uri,
        }: &TokenVolumeHistory,
    ) -> Self {
        Self {
            token_address: token_address.clone(),
            volume24h: volume24h.clone(),
            record_date: *record_date,
            name: name.clone(),
            symbol: symbol.clone(),
            logo_uri: logo_uri.clone(),
            holder_count: 0,
        }
    }
}
pub async fn trending_token(
    State(app): State<AppState>,
) -> Result<Json<HttpResponse<Vec<TrendingTokenResponse>>>, (StatusCode, String)> {
    let tokens = query_top_token_volume_history_by_date(
        &app.pool,
        20,
        time_util::get_start_of_day(Utc::now()).timestamp(),
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    .iter()
    .map(TrendingTokenResponse::from)
    .collect();
    Ok(Json(HttpResponse {
        code: 200,
        response: tokens,
        last_updated: Utc::now().timestamp(),
    }))
}

pub async fn search_token(
    State(app): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<HttpResponse<Vec<TokenResponse>>>, (StatusCode, String)> {
    if let Err(validation_errors) = query.validate() {
        return Err((StatusCode::BAD_REQUEST, validation_errors.to_string()));
    }
    let tokens = search_tokens(&app.pool, &query.q, query.limit, query.offset)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .iter()
        .map(TokenResponse::from)
        .collect();
    Ok(Json(HttpResponse {
        code: 200,
        response: tokens,
        last_updated: Utc::now().timestamp(),
    }))
}
pub async fn search_tokens(
    pool: &Pool<Postgres>,
    search: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Token>> {
    // Use full-text search by concatenating name and symbol into a tsvector and comparing against a tsquery.

    let tokens = sqlx::query_as::<_, Token>(
        r#"
        SELECT token_address, name, symbol, image_url as logo_uri, marketcap, price_change24h_percent as price24hchange, current_price
        FROM tokens t
        WHERE (t.token_address % $1 OR name % $1 OR symbol % $1)
          and EXISTS(SELECT 1 FROM token_watch WHERE token_watch.token_address = t.token_address)
        ORDER BY marketcap DESC
        LIMIT $2 OFFSET $3
        "#,
    )
        .bind(search)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;
    Ok(tokens)
}
