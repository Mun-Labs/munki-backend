pub mod route;

use chrono::{DateTime, Timelike, Utc};
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
#[derive(Serialize, FromRow, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FearAndGreed {
    pub value: i64,
    pub value_classification: String, // Match the API field name
    pub timestamp: i64,
    pub chain: String,
}

impl From<&FearAndGreedApiResponse> for FearAndGreed {
    fn from(fear_and_greed: &FearAndGreedApiResponse) -> Self {
        Self {
            value: fear_and_greed.value.parse::<i64>().unwrap_or_default(),
            value_classification: fear_and_greed.status.clone(),
            timestamp: fear_and_greed.timestamp.parse::<i64>().unwrap_or_default(),
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
        .bind(record.value)
        .bind(&record.value_classification)
        .bind(record.timestamp)
        .bind(&record.chain)
        .execute(pool)
        .await?;
    }
    Ok(())
}
#[allow(dead_code)]
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

pub async fn get_fear_and_greed_by_timestamp(
    pool: &PgPool,
    timestamp: i64,
) -> Result<Option<FearAndGreed>, sqlx::Error> {
    let naive_datetime = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap();
    let start_of_today = naive_datetime
        .with_hour(0)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(naive_datetime);

    sqlx::query_as::<_, FearAndGreed>(
        r#"
        SELECT value,
               status as value_classification,
               timestamp,
               chain,
               NULL as time_until_update
        FROM fear_and_greed
        WHERE timestamp = $1
        "#,
    )
    .bind(start_of_today.timestamp())
    .fetch_optional(pool)
    .await
}


#[derive(Serialize, FromRow, Deserialize, Debug)]
pub struct FearAndGreedHistory {
    pub value: i32,
    pub unix_timestamp: i64,
    pub value_classification: String,
}

pub async fn gear_and_fear_history_by_unixtime(
    pool: &PgPool,
    unixtimes: Vec<i64>,
) -> Result<Vec<FearAndGreedHistory>, sqlx::Error> {
    sqlx::query_as::<_, FearAndGreedHistory>(
        r#"
select value, unix_timestamp, value_classification
from greed_and_fear_history
where unix_timestamp = any ($1)
order by recorded_at desc;
        "#,
    )
    .bind(unixtimes)
    .fetch_all(pool)
    .await
}

pub async fn current_gear_and_fear_history(
    pool: &PgPool,
    unixtimes: i64,
) -> Result<Option<FearAndGreedHistory>, sqlx::Error> {
    sqlx::query_as::<_, FearAndGreedHistory>(
        r#"
select value, unix_timestamp, value_classification
from greed_and_fear_history
where unix_timestamp = $1
  and recorded_at + interval '1 hours' > now()
order by recorded_at desc;
        "#,
    )
    .bind(unixtimes)
    .fetch_optional(pool)
    .await
}

pub async fn upsert_fear_and_greed(pool: &PgPool, record: &FearAndGreed) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
insert into greed_and_fear_history(value, recorded_at, unix_timestamp, value_classification)
values ($1, now(), $2, $3)
on conflict (unix_timestamp)
    do update set value       = excluded.value,
                  recorded_at = excluded.recorded_at;
        "#,
    )
    .bind(record.value)
    .bind(record.timestamp)
    .bind(&record.value_classification)
    .execute(pool)
    .await?;
    Ok(())
}
