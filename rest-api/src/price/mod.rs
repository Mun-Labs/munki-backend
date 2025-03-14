pub mod route;

use bigdecimal::ToPrimitive;
use serde::Serialize;
use sqlx::types::BigDecimal;
use sqlx::PgPool;
use sqlx::Row;

use crate::thirdparty::TokenData;

#[derive(Serialize)]
pub struct TokenPrice {
    pub token: String,
    pub price: f64,
}

pub trait PriceSdk {
    async fn get_price(&self, token: &str) -> Result<TokenData, anyhow::Error>;
}

pub async fn get_price<T: PriceSdk>(
    pool: &PgPool,
    client: &T,
    token_address: &str,
) -> Result<TokenData, Box<dyn std::error::Error>> {
    // ✅ 1. Try to get metric from the database
    if let Some(metric) = get_metric_from_db(pool, token_address).await? {
        return Ok(metric);
    }

    // ✅ 2. If not found, call 3rd-party API
    let metric = client.get_price(token_address).await?;

    // ✅ 3. Store the metric in the database
    store_metric_in_db(pool, &metric, token_address).await?;

    Ok(metric)
}

/// ✅ Query the database for an existing metric
async fn get_metric_from_db(
    pool: &PgPool,
    token_address: &str,
) -> Result<Option<TokenData>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT token_address, update_unix_time, update_human_time, volume_usd, volume_change_percent, price_change_percent, price
        FROM token_metrics WHERE token_address = $1 ORDER BY update_unix_time DESC LIMIT 1"
    )
    .bind(token_address)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        Ok(Some(TokenData {
            update_unix_time: row.get("update_unix_time"),
            update_human_time: row.get("update_human_time"),
            volume_usd: row
                .get::<BigDecimal, _>("volume_usd")
                .to_f64()
                .unwrap_or_default(),
            volume_change_percent: row
                .get::<BigDecimal, _>("volume_change_percent")
                .to_f64()
                .unwrap_or(0.0),
            price_change_percent: row
                .get::<BigDecimal, _>("price_change_percent")
                .to_f64()
                .unwrap_or(0.0),
            price: row
                .get::<BigDecimal, _>("price")
                .to_f64()
                .unwrap_or_default(),
        }))
    } else {
        Ok(None)
    }
}

/// ✅ Store the fetched metric in the database
async fn store_metric_in_db(
    pool: &PgPool,
    metric: &TokenData,
    token_address: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO token_metrics (token_address, update_unix_time, update_human_time, volume_usd, volume_change_percent, price_change_percent, price, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"
    )
    .bind(token_address)
    .bind(metric.update_unix_time)
    .bind(&metric.update_human_time)
    .bind(metric.volume_usd)
    .bind(metric.volume_change_percent)
    .bind(metric.price_change_percent)
    .bind(metric.price)
    .execute(pool)
    .await?;

    Ok(())
}
