use crate::app::AppState;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Pool, Postgres};

use super::{TokenOverview, TokenSdk};
// Main struct for the entire data structure
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalytics {
    marketcap: f64,
    v_24h: f64,
    v_24h_change_7d: f64,
    liquidity: f64,
    liquidity_change: f64,
    holders: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenAnalyticsResponse {
    marketcap: f64,
    v_24h: f64,
    v_24h_change_7d: f64,
    liquidity: f64,
    liquidity_change: f64,
    holders: i64,
    market_cap_change_7d: f64, // need to create table to hold
    holders_change_7d: i64,    // need to create table to hold
    market_cap_7d_historical_values: Json<Vec<HistoricalValue>>, // need to create table to hold
    liquidity_historical: Json<Vec<HistoricalValue>>, // need to create table to hold
    volume_historical: Json<Vec<HistoricalValue>>, // need to create table to hold
    holders_historical: Json<Vec<HistoricalValue>>, // need to create table to hold
    top_followers: Json<Vec<FollowerProfile>>, // need to create table to hold
    followers: Json<FollowerMetrics>, // need to create table to hold
    mentions: Json<MentionMetrics>, // need to create table to hold
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalValue {
    value: f64,
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowerProfile {
    profile_url: String,
    tag: String,
    name: String,
    followers: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowerMetrics {
    follower_number: i64,
    follower_number_change_7d: i64,
    smarts: i64,
    smarts_change: i64,
    #[serde(default)]
    follower_numbers_historical: Vec<HistoricalValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentionMetrics {
    mention_number: i64,
    mention_number_change_7d: i64,
    smarts: i64,
    smarts_change: i64,
    mention_numbers_historical: Vec<HistoricalValue>,
}

pub async fn query_token_analytics(
    pool: &Pool<Postgres>,
    address: &str,
) -> Result<Option<TokenAnalytics>> {
    let token = sqlx::query_as::<_, TokenAnalytics>(
        r#"
        SELECT * FROM token_analytics WHERE address = $1
        "#,
    )
    .bind(address)
    .fetch_optional(pool)
    .await?;
    Ok(token)
}

pub async fn save_token_analytics(
    pool: &Pool<Postgres>,
    token: &TokenAnalytics,
    address: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO token_analytics (
            address, market_cap,
            volume_24h, volume_24h_change_7d,
            liquidity, liquidity_change,
            holders
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(address)
    .bind(token.marketcap)
    .bind(token.v_24h)
    .bind(token.v_24h_change_7d)
    .bind(token.liquidity)
    .bind(token.liquidity_change)
    .bind(token.holders as i64)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_token_detail_overview(
    app: &AppState,
    token_address: &str,
) -> Result<TokenAnalytics> {
    let result = app.bird_eye_client.overview(token_address).await;

    match &result {
        Ok(token_overview) => {
            eprintln!(
                "Successfully fetched token overview for address {}: {:?}",
                token_address, token_overview
            );
            Ok((token_overview).into())
        }
        Err(e) => {
            eprintln!(
                "Failed to fetch token overview for address {}: {}",
                token_address, e
            );
            return Err(anyhow::anyhow!("Custom error message: {}", e));
        }
    }
}

pub fn map_to_token_analytics(overview: &TokenOverview) -> TokenAnalyticsResponse {
    TokenAnalyticsResponse {
        marketcap: overview.marketcap.expect("REASON"),
        v_24h: overview.v_24h,
        v_24h_change_7d: overview.v_24h_change_7d,
        liquidity: overview.liquidity.expect("REASON"),
        liquidity_change: overview.liquidity_change,
        holders: overview.holder.unwrap_or(0.0) as i64,
        market_cap_change_7d: 0.00,
        market_cap_7d_historical_values: vec![
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
        ]
        .into(),
        volume_historical: vec![
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
        ]
        .into(),
        liquidity_historical: vec![
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
        ]
        .into(),

        holders_change_7d: 0,
        holders_historical: vec![
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
            HistoricalValue {
                value: overview.v_24h,
                time: Some(Utc::now().timestamp()),
                label: Some("24h".to_string()),
            },
        ]
        .into(),
        top_followers: vec![
            FollowerProfile {
                profile_url: "name".to_string(),
                tag: "name".to_string(),
                name: "name".to_string(),
                followers: 44,
            },
            FollowerProfile {
                profile_url: "name".to_string(),
                tag: "name".to_string(),
                name: "name".to_string(),
                followers: 44,
            },
            FollowerProfile {
                profile_url: "name".to_string(),
                tag: "name".to_string(),
                name: "name".to_string(),
                followers: 44,
            },
            FollowerProfile {
                profile_url: "name".to_string(),
                tag: "name".to_string(),
                name: "name".to_string(),
                followers: 44,
            },
            FollowerProfile {
                profile_url: "name".to_string(),
                tag: "name".to_string(),
                name: "name".to_string(),
                followers: 44,
            },
        ]
        .into(),
        followers: Json(FollowerMetrics {
            follower_number: 0,
            follower_number_change_7d: 0,
            smarts: 0,
            smarts_change: 0,
            follower_numbers_historical: vec![],
        }),
        mentions: Json(MentionMetrics {
            mention_number: 0,
            mention_number_change_7d: 0,
            smarts: 0,
            smarts_change: 0,
            mention_numbers_historical: vec![],
        }),
    }
}

impl From<&TokenOverview> for TokenAnalytics {
    fn from(overview: &TokenOverview) -> Self {
        TokenAnalytics {
            marketcap: overview.marketcap.unwrap_or(0.0),
            v_24h: overview.v_24h,
            v_24h_change_7d: overview.v_24h_change_7d,
            liquidity: overview.liquidity.unwrap_or(0.0),
            liquidity_change: overview.liquidity_change,
            holders: overview.holder.unwrap_or(0.0) as i64,
        }
    }
}

impl From<TokenAnalytics> for TokenAnalyticsResponse {
    fn from(overview: TokenAnalytics) -> Self {
        TokenAnalyticsResponse {
            marketcap: overview.marketcap,
            v_24h: overview.v_24h,
            v_24h_change_7d: overview.v_24h_change_7d,
            liquidity: overview.liquidity,
            liquidity_change: overview.liquidity_change,
            holders: overview.holders,
            market_cap_change_7d: 0.0,
            holders_change_7d: 0,
            market_cap_7d_historical_values: vec![].into(),
            liquidity_historical: vec![].into(),
            volume_historical: (vec![]).into(),
            holders_historical: (vec![]).into(),
            top_followers: (vec![]).into(),
            followers: (FollowerMetrics {
                follower_number: 0,
                follower_number_change_7d: 0,
                smarts: 0,
                smarts_change: 0,
                follower_numbers_historical: vec![].into(),
            })
            .into(),
            mentions: (MentionMetrics {
                mention_number: 0,
                mention_number_change_7d: 0,
                smarts: 0,
                smarts_change: 0,
                mention_numbers_historical: vec![],
            })
            .into(),
        }
    }
}
