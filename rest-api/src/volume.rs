use chrono::{DateTime, NaiveDate, Timelike, Utc};
use sqlx::PgPool;

use crate::thirdparty::defi::DefiLlamaVolumeResponse;

// Database Model Struct
#[derive(Debug, sqlx::FromRow)]
pub struct BlockchainVolume {
   pub id: i64,
   pub chain: String,
   pub total24h: i64,
   pub total48hto24h: i64,
   pub total7d: i64,
   pub total14dto7d: i64,
   pub total60dto30d: i64,
   pub total30d: i64,
   pub total1y: i64,
   pub change_1d: f64,
   pub change_7d: f64,
   pub change_1m: f64,
   pub change_7dover7d: f64,
   pub change_30dover30d: f64,
   pub total7_days_ago: i64,
   pub total30_days_ago: i64,
   pub recorded_date: NaiveDate,
   pub recorded_at: DateTime<Utc>, // Set to start of day
}

// Conversion: Set recorded_at to start of day
impl From<DefiLlamaVolumeResponse> for BlockchainVolume {
    fn from(api: DefiLlamaVolumeResponse) -> Self {
        let now = Utc::now();
        let start_of_day = now
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now); // Fallback to now if adjustment fails
        BlockchainVolume {
            id: 0,
            chain: "".to_string(),
            total24h: api.total24h as i64,
            total48hto24h: api.total48hto24h as i64,
            total7d: api.total7d as i64,
            total14dto7d: api.total14dto7d as i64,
            total60dto30d: api.total60dto30d as i64,
            total30d: api.total30d as i64,
            total1y: api.total1y as i64,
            change_1d: api.change_1d,
            change_7d: api.change_7d,
            change_1m: api.change_1m,
            change_7dover7d: api.change_7dover7d,
            change_30dover30d: api.change_30dover30d,
            total7_days_ago: api.total7_days_ago as i64,
            total30_days_ago: api.total30_days_ago as i64,
            recorded_date: now.date_naive(),
            recorded_at: start_of_day,
        }
    }
}

pub async fn upsert_metrics(
    pool: &PgPool,
    data: DefiLlamaVolumeResponse,
    chain: &str,
) -> Result<(), sqlx::Error> {
    let metrics: BlockchainVolume = data.into();
    sqlx::query(
        r#"
        INSERT INTO block_chain_volume (
            total24h, total48hto24h, total7d, total14dto7d, total60dto30d,
            total30d, total1y, change_1d, change_7d, change_1m,
            change_7dover7d, change_30dover30d, total7_days_ago, total30_days_ago,
            recorded_date, recorded_at, chain
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
        ON CONFLICT (recorded_at)
        DO UPDATE SET
            total24h = EXCLUDED.total24h,
            total48hto24h = EXCLUDED.total48hto24h,
            total7d = EXCLUDED.total7d,
            total14dto7d = EXCLUDED.total14dto7d,
            total60dto30d = EXCLUDED.total60dto30d,
            total30d = EXCLUDED.total30d,
            total1y = EXCLUDED.total1y,
            change_1d = EXCLUDED.change_1d,
            change_7d = EXCLUDED.change_7d,
            change_1m = EXCLUDED.change_1m,
            change_7dover7d = EXCLUDED.change_7dover7d,
            change_30dover30d = EXCLUDED.change_30dover30d,
            total7_days_ago = EXCLUDED.total7_days_ago,
            total30_days_ago = EXCLUDED.total30_days_ago,
            recorded_date = EXCLUDED.recorded_date
        "#,
    )
    .bind(metrics.total24h)
    .bind(metrics.total48hto24h)
    .bind(metrics.total7d)
    .bind(metrics.total14dto7d)
    .bind(metrics.total60dto30d)
    .bind(metrics.total30d)
    .bind(metrics.total1y)
    .bind(metrics.change_1d)
    .bind(metrics.change_7d)
    .bind(metrics.change_1m)
    .bind(metrics.change_7dover7d)
    .bind(metrics.change_30dover30d)
    .bind(metrics.total7_days_ago)
    .bind(metrics.total30_days_ago)
    .bind(metrics.recorded_date)
    .bind(metrics.recorded_at)
    .bind(chain)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_volume_by_date(
    pool: &PgPool,
    date: DateTime<Utc>,
    chain: &str,
) -> Result<Option<BlockchainVolume>, sqlx::Error> {
    let rows = sqlx::query_as::<_, BlockchainVolume>(
        r#"
        SELECT * FROM block_chain_volume
        WHERE recorded_at = $1 AND chain = $2
        "#
    )
        .bind(date)
        .bind(chain)
        .fetch_optional(pool)
        .await?;

    Ok(rows)
}
