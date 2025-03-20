use crate::app::AppState;
use crate::token::{TokenOverview, TokenSdk};
use anyhow::Result;
use log::{error, info};
use sqlx::{Pool, Postgres};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug)]
pub struct TokenData {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
struct TokenRow {
    token_address: String,
}

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
        let token_data = fetch_token_details(&app, &token_address).await?;
        println!("Token details: {:?}", token_data);
        insert_token(pool, &token_data).await?;
        info!("Token address {} is refreshed", token_address);
        // Renew the token watch interval instead of deleting.
        renew_token_in_watch(pool, &token_address).await?;
        sleep(Duration::from_millis(1000)).await;
    }
    Ok(())
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
    tokens (token_address, name, symbol, image_url, total_supply, marketcap, history24h_price, price_change24h_percent, current_price, decimals)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    ON CONFLICT (token_address) do UPDATE SET
    total_supply = EXCLUDED.total_supply,
    marketcap = EXCLUDED.marketcap,
    price_change24h_percent = EXCLUDED.price_change24h_percent,
    history24h_price = EXCLUDED.history24h_price,
        decimals = EXCLUDED.decimals,
    current_price = EXCLUDED.current_price
     RETURNING token_address"#,
    )
    .bind(&token.address)
    .bind(&token.name)
    .bind(&token.symbol)
    .bind(&token.logo_uri)
    .bind(&token.total_supply)
    .bind(&token.marketcap)
    .bind(&token.history24h_price)
    .bind(&token.price_change24h_percent)
    .bind(&token.price)
        .bind(token.decimals as i64)
    .execute(pool)
    .await?;
    Ok(())
}
