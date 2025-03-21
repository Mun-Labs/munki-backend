use crate::app::AppState;
use crate::token::{TokenOverview, TokenSdk};
use anyhow::Result;
use log::{error, info};
use sqlx::types::Json;
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, sqlx::FromRow)]
struct TokenRow {}

pub async fn start_token_fetcher(app_state: Arc<AppState>) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = process_token_watch(&app_state).await {
                error!("Error processing token fetch queue: {}", e);
            }
            // Sleep for 60 seconds between runs.
            sleep(Duration::from_secs(60)).await;
        }
    });
}

async fn process_token_watch(app: &Arc<AppState>) -> Result<()> {
    let pool = &app.pool;
    let batch_size = 50; // adjust as needed
    let token_addresses = get_token_watch_due(pool, batch_size).await?;
    for token_address in token_addresses {
        let token_data = fetch_token_details(app, &token_address)
            .await
            .map_err(|e| {
                error!("Error fetching token details for {}: {}", token_address, e);
                e
            })?;
        info!("Token details: {:?}", token_data);
        insert_token(pool, &token_data).await?;
        info!("Token address {} is refreshed", token_address);
        let holders = app
            .bird_eye_client
            .holders(&token_address)
            .await
            .map_err(|e| {
                error!("Error fetching holding for {}: {}", token_address, e);
                e
            })?;
        let count =
            query_in_mover(&app.pool, holders.iter().map(|a| a.owner.clone()).collect()).await?;
        upsert_alpha_metric(&app.pool, &token_address, count).await?;
        info!("Found {} holders in market_mover", count);
        renew_token_in_watch(pool, &token_address).await?;
        sleep(Duration::from_millis(1000)).await;
    }
    Ok(())
}

async fn upsert_smart_holder_metric(
    pool: &Pool<Postgres>,
    address: &str,
    top_holders: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alpha_move_token_metric (token_address, top_smart_wallets_holders)
        VALUES ($1, $2)
        ON CONFLICT (token_address) DO UPDATE
        SET top_smart_wallets_holders = EXCLUDED.top_smart_wallets_holders
        "#,
    )
    .bind(address)
    .bind(top_holders)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_alpha_metric_munscore(
    pool: &Pool<Postgres>,
    address: &str,
    top_holders: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alpha_move_token_metric (token_address, mun_score)
        VALUES ($1, $2)
        ON CONFLICT (token_address) DO UPDATE
        SET top_smart_wallets_holders = EXCLUDED.top_smart_wallets_holders
        "#,
    )
    .bind(address)
    .bind(top_holders)
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_alpha_metric(
    pool: &Pool<Postgres>,
    address: &String,
    top_holders: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alpha_move_token_metric (token_address, top_smart_wallets_holders)
        VALUES ($1, $2)
        ON CONFLICT (token_address) DO UPDATE
        SET top_smart_wallets_holders = EXCLUDED.top_smart_wallets_holders
        "#,
    )
    .bind(address)
    .bind(top_holders)
    .execute(pool)
    .await?;
    Ok(())
}

async fn query_in_mover(
    pool: &Pool<Postgres>,
    wallet_addresses: Vec<String>,
) -> Result<i64, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM market_mover WHERE wallet_address = ANY($1)")
            .bind(&wallet_addresses)
            .fetch_one(pool)
            .await?;
    Ok(count)
}

/// Fetch tokens from the token_watch table whose updated_at timestamp is older than 60 seconds.
pub async fn get_token_watch_due(pool: &Pool<Postgres>, batch_size: i64) -> Result<Vec<String>> {
    let addresses = sqlx::query_scalar(
        "SELECT token_address FROM token_watch
         WHERE updated_at <= NOW() - INTERVAL '3600 seconds'
         LIMIT $1",
    )
    .bind(batch_size)
    .fetch_all(pool)
    .await?;
    Ok(addresses)
}

// Remove a token from the queue and return the token address.
pub async fn renew_token_in_watch(pool: &Pool<Postgres>, token_address: &str) -> Result<()> {
    let _: Option<TokenRow> = sqlx::query_as::<_, TokenRow>(
        "UPDATE token_watch
         SET updated_at = NOW()
         WHERE token_address = $1
         RETURNING token_address",
    )
    .bind(token_address)
    .fetch_optional(pool)
    .await?;
    Ok(())
}

// Simulate fetching token details; replace with an actual implementation.
async fn fetch_token_details(app: &AppState, token_address: &str) -> Result<TokenOverview> {
    app.bird_eye_client.overview(token_address).await
}

// Insert token data into the tokens table.
async fn insert_token(pool: &Pool<Postgres>, token: &TokenOverview) -> Result<()> {
    sqlx::query(
        r#"
    INSERT INTO
    tokens (token_address, name, symbol, image_url, total_supply, marketcap, history24h_price, price_change24h_percent, current_price, decimals, metadata)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
    ON CONFLICT (token_address) do UPDATE SET
    total_supply = EXCLUDED.total_supply,
    marketcap = EXCLUDED.marketcap,
    price_change24h_percent = EXCLUDED.price_change24h_percent,
    history24h_price = EXCLUDED.history24h_price,
        decimals = EXCLUDED.decimals,
    current_price = EXCLUDED.current_price,
    metadata = EXCLUDED.metadata
     RETURNING token_address"#,
    )
    .bind(&token.address)
    .bind(&token.name)
    .bind(&token.symbol)
    .bind(&token.logo_uri)
    .bind(token.total_supply)
    .bind(token.marketcap)
    .bind(token.history24h_price)
    .bind(token.price_change24h_percent)
    .bind(token.price)
    .bind(token.decimals as i64)
    .bind(Json(&token.extensions))
    .execute(pool)
    .await?;
    Ok(())
}
