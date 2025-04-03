// use std::path::Path;
use crate::app::AppState;
use crate::response::HttpResponse;
use crate::time_util;
use crate::token::{
    background_job, create_dummy_token_analysis, create_dummy_token_distribution,
    fetch_token_details, last_active, token_bio, token_by_address, TokenAnalytics,
    TokenDistributions, TokenOverviewResponse, TokenSdk, TokenVolumeHistory,
};
use axum::extract::{Path, Query, State};
use axum::{http::StatusCode, Json};
use bigdecimal::{BigDecimal, FromPrimitive, ToPrimitive};
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
    pub price24h_percent: Option<BigDecimal>,
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
            price24h_percent,
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
            price24h_percent: price24h_percent.clone(),
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

    let a = search_tokens(&app.pool, &query.q, query.limit, query.offset)
        .await
        .map_err(|e| {
            error!("Failed to search tokens: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    if !a.is_empty() {
        let tokens = a.iter().map(TokenResponse::from).collect();

        if let Err(e) = last_active(&app.pool, &[query.q]).await {
            error!("Failed to update last active: {e}");
        }

        return Ok(Json(HttpResponse {
            code: 200,
            response: tokens,
            last_updated: Utc::now().timestamp(),
        }));
    }

    let mut search_result = app.bird_eye_client.search(&query.q).await.map_err(|e| {
        error!("Failed to search tokens: {e}");
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    if let Err(e) = last_active(&app.pool, &[query.q]).await {
        error!("Failed to update last active: {e}");
    }

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
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDetailResponse {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: String,
    pub website_url: String,
    pub metadata: serde_json::Value,
    pub decimals: i32,
    pub marketcap: BigDecimal,
    pub history24h_price: BigDecimal,
    pub price_change24h_percent: BigDecimal,
    pub holders: i32,
    pub liquidity: BigDecimal,
    pub volume_24h: BigDecimal,
    pub volume_24h_change: BigDecimal,
    pub mun_score: f64,
    pub total_supply: BigDecimal,
    pub current_price: BigDecimal,
    pub risk_score: f64,
}

impl From<TokenOverviewResponse> for TokenDetailResponse {
        fn from(value: TokenOverviewResponse) -> Self {
            Self {
                token_address: value.token_address,
                name: value.name,
                symbol: value.symbol,
                logo_uri: value.logo_uri.unwrap_or_default(),
                website_url: value.website_url.unwrap_or_default(),
                metadata: value.metadata.unwrap_or_default(),
                decimals: value.decimals.unwrap(),
                marketcap: value.marketcap.unwrap(),
                history24h_price: value.history24h_price.unwrap_or_default(),
                price_change24h_percent: value.price_change24h_percent.unwrap_or_default(),
                holders: value.holders.unwrap_or(0),
                liquidity: value.liquidity.unwrap_or_default(),
                volume_24h: value.volume_24h.unwrap_or_default(),
                volume_24h_change: value.volume_24h_change.unwrap_or_default(),
                mun_score: value.mun_score.unwrap().to_f64().unwrap(),
                total_supply: value.total_supply.unwrap(),
                current_price: value.current_price.unwrap(),
                risk_score: value.risk_score.unwrap().to_f64().unwrap(),
            }
        }
    }

pub async fn get_token_bio(
    State(app): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<HttpResponse<TokenDetailResponse>>, (StatusCode, String)> {
    let missing = token_by_address(&app.pool, vec![address.clone()])
        .await
        .map_err(|e| {
            error!("Failed to fetch token by address: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    let resp: TokenDetailResponse = if !missing.is_empty() {
        let token = fetch_token_details(&app, &address).await.map_err(|e| {
            error!("Failed to fetch token details: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

        background_job::insert_token(&app.pool, &token)
            .await
            .map_err(|e| {
                error!("Failed to insert token: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })
            .map(|_| {
                TokenDetailResponse {
                    token_address: token.address,
                    name: token.name,
                    symbol: token.symbol,
                    logo_uri: token.logo_uri.unwrap().to_string(),
                    website_url: token.website_url.unwrap_or_default().to_string(),
                    metadata: serde_json::to_value(token.extensions.unwrap_or_default()).unwrap_or_default(),
                    decimals: token.decimals.to_i32().unwrap(),
                    marketcap: BigDecimal::from_f64(token.marketcap.unwrap()).unwrap(),
                    history24h_price: BigDecimal::from_f64(token.history24h_price.unwrap_or_default()).unwrap(),
                    price_change24h_percent: BigDecimal::from_f64(token.price_change24h_percent.unwrap_or_default()).unwrap(),
                    holders: token.holder.unwrap_or_default(),
                    liquidity: BigDecimal::from_f64(token.liquidity.unwrap_or_default()).unwrap(),
                    volume_24h: BigDecimal::from_f64(token.volume24h.unwrap_or_default()).unwrap(),
                    volume_24h_change: BigDecimal::from_f64(token.volume_24h_change.unwrap_or_default()).unwrap(),
                    mun_score: 0.0,
                    total_supply: BigDecimal::from_f64(token.total_supply.unwrap_or_default()).unwrap(),
                    current_price: BigDecimal::from_f64(token.price.unwrap()).unwrap(),
                    risk_score: 0.0,
                }
            })?
    } else {
        let token_bio_response = token_bio(&app.pool, &address)
            .await
            .map_err(|e| {
                error!("Failed to fetch token bio: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?;
        TokenDetailResponse::from(token_bio_response)
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
