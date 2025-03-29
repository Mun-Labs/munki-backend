use crate::app::AppState;
use crate::token::TokenDetailOverview;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::{FromRow, Pool, Postgres};

use super::TokenSdk;
// Main struct for the entire data structure
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalytics {
    market_cap: f64,
    market_cap_change_7d: f64,
    market_cap_7d_historical_values: Json<Vec<HistoricalValue>>,
    volume_24h: f64,
    volume_24h_change_7d: f64,
    volume_historical: Json<Vec<HistoricalValue>>,
    liquidity: f64,
    liquidity_change: f64,
    liquidity_historical: Json<Vec<HistoricalValue>>,
    holders: i64,
    holders_change_7d: i64,
    holders_historical: Json<Vec<HistoricalValue>>,
    top_followers: Json<Vec<FollowerProfile>>,
    followers: Json<FollowerMetrics>,
    mentions: Json<MentionMetrics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoricalValue {
    value: f64,
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "UPPERCASE")]
pub enum Level {
    Alpha,
    Beta,
    Gamma,
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
            address, market_cap, market_cap_change_7d, market_cap_7d_historical_values,
            volume_24h, volume_24h_change_7d, volume_historical,
            liquidity, liquidity_change, liquidity_historical,
            holders, holders_change_7d, holders_historical,
            top_followers, followers, mentions
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
        "#,
    )
    .bind(address)
    .bind(token.market_cap)
    .bind(token.market_cap_change_7d)
    .bind(&token.market_cap_7d_historical_values)
    .bind(token.volume_24h)
    .bind(token.volume_24h_change_7d)
    .bind(&token.volume_historical)
    .bind(token.liquidity)
    .bind(token.liquidity_change)
    .bind(&token.liquidity_historical)
    .bind(token.holders as i64)
    .bind(token.holders_change_7d)
    .bind(&token.holders_historical)
    .bind(&token.top_followers)
    .bind(&token.followers)
    .bind(&token.mentions)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_token_detail_overview(
    app: &AppState,
    token_address: &str,
) -> Result<TokenDetailOverview> {
    let result = app
        .bird_eye_client
        .token_detail_overview(token_address)
        .await;

    match &result {
        Ok(token_overview) => {
            eprintln!(
                "Successfully fetched token overview for address {}: {:?}",
                token_address, token_overview
            );
            return result;
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

pub fn map_to_token_analytics(overview: &TokenDetailOverview) -> TokenAnalytics {
    TokenAnalytics {
        market_cap: overview.market_cap,
        market_cap_change_7d: 0.0,
        market_cap_7d_historical_values: vec![].into(),
        volume_24h: overview.v_24h,
        volume_24h_change_7d: overview.price_change_24h_percent,
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
        liquidity: overview.liquidity,
        liquidity_change: 0.0,
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
        holders: overview.holder,
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

pub fn create_dummy_token_analysis() -> TokenAnalytics {
    TokenAnalytics {
        market_cap: 1_234_567.89,
        market_cap_change_7d: -5.43,
        market_cap_7d_historical_values: vec![
            HistoricalValue {
                value: 1_230_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 1_234_567.89,
                time: Some(1711586400),
                label: None,
            },
            HistoricalValue {
                value: 1_434_567.89,
                time: Some(1711586400),
                label: None,
            },
            HistoricalValue {
                value: 1_134_567.89,
                time: Some(1711586400),
                label: None,
            },
            HistoricalValue {
                value: 1_734_567.89,
                time: Some(1711586400),
                label: None,
            },
        ]
        .into(),
        volume_24h: 98_765.43,
        volume_24h_change_7d: 2.1,
        volume_historical: vec![
            HistoricalValue {
                value: 95_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 95_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 98_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 100_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 195_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
            HistoricalValue {
                value: 185_000.0,
                time: Some(1711500000),
                label: Some("Day 1".to_string()),
            },
        ]
        .into(),
        liquidity: 45_678.9,
        liquidity_change: 1.5,
        liquidity_historical: vec![
            HistoricalValue {
                value: 45_000.0,
                time: Some(1711500000),
                label: None,
            },
            HistoricalValue {
                value: 45_000.0,
                time: Some(1711500000),
                label: None,
            },
            HistoricalValue {
                value: 45_000.0,
                time: Some(1711500000),
                label: None,
            },
            HistoricalValue {
                value: 45_000.0,
                time: Some(1711500000),
                label: None,
            },
            HistoricalValue {
                value: 45_000.0,
                time: Some(1711500000),
                label: None,
            },
        ]
        .into(),
        holders: 5000,
        holders_change_7d: -200,
        holders_historical: vec![HistoricalValue {
            value: 5200.0,
            time: Some(1711500000),
            label: Some("Day 1".to_string()),
        }]
        .into(),
        top_followers: vec![
            FollowerProfile {
                profile_url: "https://x.com/user1".to_string(),
                tag: "@user1".to_string(),
                name: "User One".to_string(),
                followers: 10_000,
            },
            FollowerProfile {
                profile_url: "https://x.com/user2".to_string(),
                tag: "@user2".to_string(),
                name: "User Two".to_string(),
                followers: 5_000,
            },
            FollowerProfile {
                profile_url: "https://x.com/user2".to_string(),
                tag: "@user2".to_string(),
                name: "User Two".to_string(),
                followers: 5_000,
            },
            FollowerProfile {
                profile_url: "https://x.com/user2".to_string(),
                tag: "@user2".to_string(),
                name: "User Two".to_string(),
                followers: 5_000,
            },
            FollowerProfile {
                profile_url: "https://x.com/user2".to_string(),
                tag: "@user2".to_string(),
                name: "User Two".to_string(),
                followers: 5_000,
            },
        ]
        .into(),
        followers: Json(FollowerMetrics {
            follower_number: 25_000,
            follower_number_change_7d: 500,
            smarts: 3_000,
            smarts_change: -50,
            follower_numbers_historical: vec![
                HistoricalValue {
                    value: 24_500.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 24_500.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 24_500.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 24_500.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 24_500.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
            ]
            .into(),
        }),
        mentions: Json(MentionMetrics {
            mention_number: 1_200,
            mention_number_change_7d: 100,
            smarts: 150,
            smarts_change: 10,
            mention_numbers_historical: vec![
                HistoricalValue {
                    value: 1_100.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 1_100.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 1_100.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 1_100.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
                HistoricalValue {
                    value: 1_100.0,
                    time: Some(1711500000),
                    label: Some("Day 1".to_string()),
                },
            ]
            .into(),
        }),
    }
}
