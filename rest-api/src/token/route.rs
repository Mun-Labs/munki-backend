// use std::path::Path;
use crate::app::AppState;
use crate::response::HttpResponse;
use crate::time_util;
use crate::token::{
    background_job, create_dummy_token_analysis, create_dummy_token_distribution,
    fetch_token_details, token_bio, token_by_address, TokenAnalytics, TokenDistributions,
    TokenOverviewResponse, TokenSdk, TokenVolumeHistory,
};
use axum::extract::{Path, Query, State};
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
    let vol = query_top_token_volume_history_by_date(
        &app.pool,
        100,
        time_util::get_start_of_day(Utc::now()).timestamp(),
    )
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
use tracing::{error, info};
use validator::Validate;

use super::{query_top_token_volume_history_by_date, TokenOverview};

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
    pub volume24h: Option<BigDecimal>,
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
    pub volume24h: f64,
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
            volume24h: value
                .volume24h
                .clone()
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
        }
    }
}

impl From<&TokenOverview> for TokenResponse {
    fn from(value: &TokenOverview) -> Self {
        Self {
            token_address: value.address.clone(),
            name: value.name.clone(),
            symbol: value.symbol.clone(),
            logo_uri: value.logo_uri.clone(),
            marketcap: value
                .marketcap
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
            price24hchange: value
                .price_change24h_percent
                .unwrap_or_default()
                .to_f64()
                .unwrap_or_default(),
            price: value.price.unwrap_or_default().to_f64().unwrap_or_default(),
            volume24h: value
                .volume24h
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
    pub volume24h_percent: Option<BigDecimal>,
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
            volume24h_percent,
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
            volume24h_percent: volume24h_percent.clone(),
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

    let mut search_result = app.bird_eye_client.search(&query.q).await.map_err(|e| {
        error!("Failed to search tokens: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    info!("inserting token {search_result:?}");
    for token in search_result.iter_mut() {
        let TokenOverview {
            address,
            decimals,
            symbol,
            name,
            logo_uri,
            price,
            history24h_price,
            price_change24h_percent,
            total_supply,
            marketcap,
            volume24h,
            ..
        } = token;
        let mut logo_uri = logo_uri.clone();
        if logo_uri.is_none() {
            info!("token {address} missing logo {:?}", logo_uri);
            if let Ok(a) = app.bird_eye_client.overview(address).await {
                logo_uri = a.logo_uri.clone();
                token.logo_uri = a.logo_uri;
            }
        }
        if let Err(e) = background_job::insert_token_with_params(
            &app.pool,
            address,
            name,
            symbol,
            logo_uri.unwrap_or_default().as_str(),
            total_supply.unwrap_or_default(),
            marketcap.unwrap_or_default(),
            history24h_price.unwrap_or_default(),
            price_change24h_percent.unwrap_or_default(),
            price.unwrap_or_default(),
            *decimals,
            None,
            "",
            volume24h.unwrap_or_default(),
        )
        .await
        {
            error!("insert token {} error: {}", token.address, e);
        };
    }

    let tokens = search_result.iter().map(TokenResponse::from).collect();

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
        SELECT t.token_address, t.name, t.symbol, t.image_url as logo_uri, t.marketcap, t.price_change24h_percent as price24hchange, t.current_price, tvh.volume24h
        FROM tokens t
                 INNER JOIN (
            SELECT token_address, volume24h, record_date
            FROM token_volume_history
            WHERE record_date = (SELECT MAX(record_date) FROM token_volume_history)
        ) tvh ON t.token_address = tvh.token_address
        WHERE t.token_address % $1 OR name % $1 OR symbol % $1
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

pub async fn get_token_bio(
    State(app): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<HttpResponse<TokenOverviewResponse>>, (StatusCode, String)> {
    let missing = token_by_address(&app.pool, vec![address.clone()])
        .await
        .map_err(|e| {
            error!("Failed to fetch token by address: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    let resp = if !missing.is_empty() {
        let token = fetch_token_details(&app, &address).await.map_err(|e| {
            error!("Failed to fetch token details: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

        background_job::insert_token(&app.pool, &token)
            .await
            .map_err(|e| {
                error!("Failed to insert token: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?
    } else {
        token_bio(&app.pool, &address).await.map_err(|e| {
            error!("Failed to fetch token bio: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
    };

    Ok(Json(HttpResponse {
        code: 200,
        response: resp,
        last_updated: Utc::now().timestamp(),
    }))
}

pub async fn get_token_analytics(
    State(_app): State<AppState>,
    Path(_address): Path<String>,
) -> Result<Json<HttpResponse<TokenAnalytics>>, (StatusCode, String)> {
    let resp: TokenAnalytics = create_dummy_token_analysis();
    Ok(Json(HttpResponse {
        code: 200,
        response: resp,
        last_updated: Utc::now().timestamp(),
    }))
}

pub async fn get_token_distributions(
    State(_app): State<AppState>,
    Path(_address): Path<String>,
) -> Result<Json<HttpResponse<Vec<TokenDistributions>>>, (StatusCode, String)> {
    let resp: Vec<TokenDistributions> = create_dummy_token_distribution();

    Ok(Json(HttpResponse {
        code: 200,
        response: resp,
        last_updated: Utc::now().timestamp(),
    }))
}
