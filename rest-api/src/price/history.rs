use bigdecimal::{BigDecimal, FromPrimitive};
use crate::thirdparty::birdeye;
use crate::thirdparty::PriceHistory;
use chrono::DateTime;
use chrono::Timelike;
use chrono::Utc;
use sqlx::PgPool;

pub async fn insert_token_prices(
    pool: &PgPool,
    response: PriceHistory,
    address: &str,
) -> Result<(), sqlx::Error> {
    // Set recorded_at to start of today
    let now = Utc::now();
    let start_of_day = now
        .with_hour(0)
        .and_then(|t| t.with_minute(0))
        .and_then(|t| t.with_second(0))
        .and_then(|t| t.with_nanosecond(0))
        .unwrap_or(now);

    for item in response.items.iter() {
        let token_price = TokenPrice::from_item(item, address, start_of_day);
        sqlx::query(
            r#"
            INSERT INTO token_prices (address, price, unixtime, recorded_at)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (address, unixtime)
            DO UPDATE SET
                price = EXCLUDED.price,
                recorded_at = EXCLUDED.recorded_at
            "#,
        )
        .bind(token_price.address)
        .bind(token_price.price)
        .bind(token_price.unixtime)
        .bind(token_price.recorded_at)
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_token_prices_between(
    pool: &PgPool,
    address: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<TokenPrice>, sqlx::Error> {
    let prices = sqlx::query_as::<_, TokenPrice>(
        r#"
        SELECT address, price, unixtime, recorded_at
        FROM token_prices
        WHERE address = $1 AND recorded_at BETWEEN $2 AND $3
        "#,
    )
    .bind(address)
    .bind(start)
    .bind(end)
    .fetch_all(pool)
    .await?;

    Ok(prices)
}

// Database model for token_prices
#[derive(Debug, sqlx::FromRow)]
pub struct TokenPrice {
    pub address: String,
    pub price: BigDecimal,
    pub unixtime: i64,
    pub recorded_at: DateTime<Utc>,
}

// Convert Item to TokenPrice with a given address and recorded_at
impl TokenPrice {
    fn from_item(item: &birdeye::Items, address: &str, recorded_at: DateTime<Utc>) -> Self {
        TokenPrice {
            address: address.into(),
            price: BigDecimal::from_f64(item.value).unwrap_or_default(),
            unixtime: item.unix_time,
            recorded_at,
        }
    }
}
