use serde::{Deserialize, Serialize};

// Main struct for the entire data structure
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalytics {
    market_cap: f64,
    market_cap_change_7d: f64,
    market_cap_7d_historical_values: Vec<HistoricalValue>,
    volume_24h: f64,
    volume_24h_change_7d: f64,
    volume_historical: Vec<HistoricalValue>,
    liquidity: f64,
    liquidity_change: f64,
    liquidity_historical: Vec<HistoricalValue>,
    holders: u64,
    holders_change_7d: i64,
    holders_historical: Vec<HistoricalValue>,

    moon_score: u32,
    level: Level,
    risk_score: f64,
    top_followers: Vec<FollowerProfile>,
    followers: FollowerMetrics,
    mentions: MentionMetrics,
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
#[serde(rename_all = "UPPERCASE")]
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
    followers: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FollowerMetrics {
    follower_number: u64,
    follower_number_change_7d: i64,
    smarts: u64,
    smarts_change: i64,
    follower_numbers_historical: Vec<HistoricalValue>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MentionMetrics {
    mention_number: u64,
    mention_number_change_7d: i64,
    smarts: u64,
    smarts_change: i64,
    mention_numbers_historical: Vec<HistoricalValue>,
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
        ],
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
        ],
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
        ],
        holders: 5000,
        holders_change_7d: -200,
        holders_historical: vec![HistoricalValue {
            value: 5200.0,
            time: Some(1711500000),
            label: Some("Day 1".to_string()),
        }],
        moon_score: 960,
        level: Level::Alpha,
        risk_score: 10.96,
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
        ],
        followers: FollowerMetrics {
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
            ],
        },
        mentions: MentionMetrics {
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
            ],
        },
    }
}
