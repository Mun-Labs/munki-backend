pub mod route;
pub mod formula;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Serialize)]
pub struct FearAndGreedApiResponse {
    pub value: String,
    pub status: String,
    pub timestamp: String,
    pub chain: String,
}

pub trait FearAndGreedSdk {
    async fn get_fear_and_greed(
        &self,
        limit: i8,
    ) -> Result<Vec<FearAndGreedApiResponse>, anyhow::Error>;
}
#[derive(Serialize, FromRow, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FearAndGreed {
    pub value: i64,
    pub value_classification: String, // Match the API field name
    pub timestamp: i64,
    pub time_until_update: Option<String>,
    pub chain: String,
}

impl From<&FearAndGreedApiResponse> for FearAndGreed {
    fn from(fear_and_greed: &FearAndGreedApiResponse) -> Self {
        Self {
            value: fear_and_greed.value.parse::<i64>().unwrap_or_default(),
            value_classification: fear_and_greed.status.clone(),
            timestamp: fear_and_greed.timestamp.parse::<i64>().unwrap_or_default(),
            time_until_update: None, // Assuming this field is not available in FearAndGreedApiRespnse
            chain: "BTC".to_string(),
        }
    }
}

pub async fn batch_insert_fear_and_greed(
    pool: &PgPool,
    records: &Vec<FearAndGreed>,
) -> Result<(), sqlx::Error> {
    for record in records {
        sqlx::query(
            r#"
                INSERT INTO fear_and_greed (value, status, timestamp, chain)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (timestamp, chain) DO NOTHING
                "#,
        )
        .bind(&record.value)
        .bind(&record.value_classification)
        .bind(record.timestamp)
        .bind(&record.chain)
        .execute(pool)
        .await?;
    }
    Ok(())
}

pub async fn get_fear_and_greed_last_31_days(
    pool: &PgPool,
) -> Result<Vec<FearAndGreed>, sqlx::Error> {
    sqlx::query_as::<_, FearAndGreed>(
        r#"
        SELECT value,
               status as value_classification,
               timestamp,
               chain,
               NULL as time_until_update
        FROM fear_and_greed
        WHERE timestamp BETWEEN
            EXTRACT(EPOCH FROM date_trunc('day', CURRENT_TIMESTAMP - INTERVAL '31 days'))::bigint
            AND
            EXTRACT(EPOCH FROM date_trunc('day', CURRENT_TIMESTAMP))::bigint
        ORDER BY timestamp DESC
        "#,
    )
    .fetch_all(pool)
    .await
}
