use crate::app::AppState;
use crate::thirdparty::{self, MunScoreSdk};
use crate::token::{TokenOverview, TokenOverviewResponse, TokenSdk};
use anyhow::Result;
use log::{error, info};
use sqlx::types::Json;
use sqlx::{PgPool, Pool, Postgres};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use super::market::TradeData;
use super::trade::MarketData;

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
        if let Ok(token_data) = fetch_token_details(app, &token_address).await.map_err(|e| {
            error!("Error fetching token details for {}: {}", token_address, e);
            e
        }) {
            info!("Token details: {:?}", token_data);
            insert_token(pool, &token_data).await?;

            match fetch_trade_data(app, &token_address).await {
                Ok(market_data) => {
                    info!("trade data: {:?}", market_data);
                    if let Err(e) = insert_trade_data(pool, &market_data).await {
                        error!("Error inserting trade data for {token_address}: {e}");
                    }
                }
                Err(e) => {
                    error!("Error fetching trade data for {token_address}: {e}");
                }
            }

            match fetch_market_data(app, &token_address).await {
                Ok(market_data) => {
                    info!("market data: {:?}", market_data);
                    if let Err(e) = insert_market_data(pool, &market_data).await {
                        error!("Error inserting market data for {token_address}: {e}");
                    }
                }
                Err(e) => {
                    error!("Error fetching market data for {token_address}: {e}");
                }
            };

            if let Ok(safety_score) = thirdparty::get_safe_score(&app.client, &token_address)
                .await
                .map_err(|e| {
                    error!("Error fetching safe score for {}: {}", token_address, e);
                    e
                })
            {
                let _ = upsert_safe_score(pool, &token_address, safety_score)
                    .await
                    .map_err(|e| {
                        error!("Error fetching safe score for {}: {}", token_address, e);
                        e
                    });
            }

            if let Ok(holders) = app
                .bird_eye_client
                .holders(&token_address)
                .await
                .map_err(|e| {
                    error!("Error fetching holding for {}: {}", token_address, e);
                    e
                })
            {
                let count =
                    query_in_mover(&app.pool, holders.iter().map(|a| a.owner.clone()).collect())
                        .await?;
                match upsert_alpha_metric(&app.pool, &token_address, count).await {
                    Err(err) => error!(
                        "Error upserting alpha metric for {}: {}",
                        token_address, err
                    ),
                    _ => info!("Found {} holders in market_mover", count),
                };
            }

            if check_if_exists_munscore(pool, &token_address)
                .await
                .map_err(|e| {
                    error!(
                        "Error checking if mun score exists for {}: {}",
                        token_address, e
                    );
                    e
                })?
                .is_none()
            {
                if let Some(username) = extract_twitter_username(token_data.extensions) {
                    info!("Fetching mun score for {}", username);
                    if let Ok(mun_score) =
                        app.moni_client.get_mun_score(&username).await.map_err(|e| {
                            error!("Error fetching mun score for {}: {}", token_address, e);
                            e
                        })
                    {
                        info!("Mun score: {mun_score:?}");
                        match upsert_alpha_metric_munscore(
                            pool,
                            &token_address,
                            mun_score.smart_engagement.followers_score as f64,
                            mun_score.smart_engagement.smart_followers_count,
                        )
                        .await
                        {
                            Ok(_) => info!("Mun score {username} of {token_address} is updated"),
                            Err(err) => error!(
                            "Error upserting mun score for {username} of {token_address}: {err}"
                        ),
                        };
                    } else {
                        mark_failed_munscore(pool, &token_address)
                            .await
                            .map_err(|e| {
                                error!(
                                    "Error marking mun score as failed for {}: {}",
                                    token_address, e
                                );
                                e
                            })?;
                    }
                }
            }
        };
        renew_token_in_watch(pool, &token_address).await?;
        info!("Token address {} is refreshed", token_address);
        sleep(Duration::from_millis(5000)).await;
    }
    Ok(())
}

async fn insert_market_data(
    pool: &Pool<Postgres>,
    market_data: &MarketData,
) -> std::result::Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE tokens SET total_supply = $2, marketcap = $3 WHERE token_address = $1"#)
        .bind(&market_data.address)
        .bind(market_data.total_supply)
        .bind(market_data.market_cap)
        .execute(pool)
        .await?;

    Ok(())
}

async fn insert_trade_data(
    pool: &Pool<Postgres>,
    market_data: &TradeData,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE tokens SET volume_24h = $2, volume_24h_change = $3 WHERE token_address = $1"#,
    )
    .bind(&market_data.address)
    .bind(market_data.volume_24h)
    .bind(market_data.volume_24h_change_percent)
    .execute(pool)
    .await?;
    Ok(())
}
async fn mark_failed_munscore(
    pool: &Pool<Postgres>,
    token_address: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE alpha_move_token_metric SET mun_score = -1 WHERE token_address = $1")
        .bind(token_address)
        .execute(pool)
        .await?;
    Ok(())
}

async fn check_if_exists_munscore(
    pool: &Pool<Postgres>,
    token_address: &str,
) -> Result<Option<bigdecimal::BigDecimal>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT mun_score FROM alpha_move_token_metric WHERE token_address = $1 and (mun_score > 0 or mun_score <= -1)",
    )
    .bind(token_address)
    .fetch_optional(pool)
    .await
}

const TWTITTER_URL: &str = "https://twitter.com/";
const X_URL: &str = "https://x.com/";

fn extract_twitter_username(extensions: Option<HashMap<String, Option<String>>>) -> Option<String> {
    extensions
        .and_then(|ext| ext.get("twitter").cloned())
        .and_then(|url| {
            let url = url?;
            url.strip_prefix(TWTITTER_URL)
                .or_else(|| url.strip_prefix(X_URL))
                .map(|stripped| stripped.split('/').next().unwrap_or_default().to_string())
        })
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
    munscore: f64,
    smart_followers_count: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alpha_move_token_metric (token_address, mun_score, smart_followers)
        VALUES ($1, $2, $3)
        ON CONFLICT (token_address) DO UPDATE
        SET mun_score = EXCLUDED.mun_score,
        smart_followers = EXCLUDED.smart_followers
        "#,
    )
    .bind(address)
    .bind(munscore)
    .bind(smart_followers_count as i64)
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
        "SELECT token_address FROM token_watch WHERE updated_at <= NOW() - INTERVAL '3600 seconds' and last_active >= NOW() - INTERVAL '1 hour' LIMIT $1",)
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
pub async fn fetch_token_details(app: &AppState, token_address: &str) -> Result<TokenOverview> {
    //fetch overview -> fetch market data -> fectch trade data
    app.bird_eye_client.overview(token_address).await
}

pub async fn fetch_trade_data(app: &AppState, token_address: &str) -> Result<TradeData> {
    app.bird_eye_client.trade_data(token_address).await
}

pub async fn fetch_market_data(app: &AppState, token_address: &str) -> Result<MarketData> {
    //fetch overview -> fetch market data -> fectch trade data
    app.bird_eye_client.market_data(token_address).await
}

// Insert token data into the tokens table.
pub async fn insert_token_with_params(
    pool: &Pool<Postgres>,
    address: &str,
    name: &str,
    symbol: &str,
    logo_uri: &str,
    total_supply: f64,
    marketcap: f64,
    history24h_price: f64,
    price_change24h_percent: f64,
    price: f64,
    decimals: u64,
    extensions: Option<serde_json::Value>,
    website_url: &str,
    volume_24h: f64,
) -> Result<TokenOverviewResponse> {
    let token = sqlx::query_as::<_, TokenOverviewResponse>(
        "
    INSERT INTO
    tokens (token_address, name, symbol, image_url, total_supply, marketcap, history24h_price, price_change24h_percent, current_price, decimals, metadata, website_url, volume_24h)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
    ON CONFLICT (token_address) do UPDATE SET
    total_supply = EXCLUDED.total_supply,
    marketcap = EXCLUDED.marketcap,
    price_change24h_percent = EXCLUDED.price_change24h_percent,
    history24h_price = EXCLUDED.history24h_price,
        decimals = EXCLUDED.decimals,
    current_price = EXCLUDED.current_price,
    metadata = EXCLUDED.metadata,
    volume_24h = EXCLUDED.volume_24h
     RETURNING token_address, name, symbol, image_url as logo_uri, total_supply, marketcap, history24h_price, price_change24h_percent, current_price, decimals, metadata, website_url, volume_24h, volume_24h_change, holders, liquidity",
    )
    .bind(address)
    .bind(name)
    .bind(symbol)
    .bind(logo_uri)
    .bind(total_supply)
    .bind(marketcap)
    .bind(history24h_price)
    .bind(price_change24h_percent)
    .bind(price)
    .bind(decimals as i64)
    .bind(Json(extensions))
    .bind(website_url)
    .bind(volume_24h)
    .fetch_one(pool)
    .await?;
    Ok(token)
}
// Insert token data into the tokens table.
pub async fn insert_token(
    pool: &Pool<Postgres>,
    token: &TokenOverview,
) -> Result<TokenOverviewResponse> {
    let token = sqlx::query_as::<_, TokenOverviewResponse>(
        "
    INSERT INTO
    tokens (token_address, name, symbol, image_url, total_supply, history24h_price, price_change24h_percent, current_price, decimals, metadata, website_url, holders, marketcap, liquidity)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    ON CONFLICT (token_address) do UPDATE SET
    total_supply = EXCLUDED.total_supply,
    price_change24h_percent = EXCLUDED.price_change24h_percent,
    history24h_price = EXCLUDED.history24h_price,
        decimals = EXCLUDED.decimals,
    current_price = EXCLUDED.current_price,
    metadata = EXCLUDED.metadata,
    marketcap = EXCLUDED.marketcap,
    liquidity = EXCLUDED.liquidity,
    holders = EXCLUDED.holders
    RETURNING token_address, name, symbol, image_url as logo_uri, total_supply, marketcap, history24h_price, price_change24h_percent, current_price, decimals, metadata, website_url, volume_24h, volume_24h_change, holders, liquidity",
    )
    .bind(&token.address)
    .bind(&token.name)
    .bind(&token.symbol)
    .bind(&token.logo_uri)
    .bind(token.total_supply)
    .bind(token.history24h_price)
    .bind(token.price_change24h_percent)
    .bind(token.price)
    .bind(token.decimals as i64)
    .bind(Json(&token.extensions))
    .bind(&token.website_url)
    .bind(token.holder)
    .bind(token.marketcap)
    .bind(token.liquidity)
    .fetch_one(pool)
    .await?;
    Ok(token)
}

async fn upsert_safe_score(
    pool: &PgPool,
    token_address: &str,
    risk_score: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO alpha_move_token_metric (token_address, risk_score)
        VALUES ($1, $2)
        ON CONFLICT (token_address) DO UPDATE
        SET risk_score = EXCLUDED.risk_score
        "#,
    )
    .bind(token_address)
    .bind(risk_score)
    .execute(pool)
    .await?;
    Ok(())
}
