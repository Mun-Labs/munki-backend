use crate::app;
use crate::token::{background_job, fetch_token_details};
use axum::http::StatusCode;
use bigdecimal::BigDecimal;
use moka::ops::compute::Op;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Trending {
    pub address: String,
    pub decimals: i32,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub name: String,
    pub symbol: String,
    #[serde(rename = "volume24hUSD")]
    pub volume24h_usd: f64,
    pub rank: i32,
    pub price: f64,
}
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenMetadata {
    pub address: String,
    pub decimals: i32,
    pub symbol: String,
    pub name: String,
    pub extensions: Option<HashMap<String, Option<String>>>,
    pub logo_uri: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TokenOverview {
    pub address: String,
    pub decimals: i64,
    pub symbol: String,
    pub name: String,
    pub extensions: Option<HashMap<String, Option<String>>>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub liquidity: Option<f64>,
    pub price: Option<f64>,
    #[serde(rename = "history24hPrice")]
    pub history24h_price: Option<f64>,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change24h_percent: Option<f64>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<f64>,
    #[serde(rename = "mc")]
    pub marketcap: Option<f64>,
    pub holder: Option<f64>,
    #[serde(rename = "websiteURL")]
    pub website_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenHolder {
    pub amount: String,
    pub decimals: i32,
    pub mint: String,
    pub owner: String,
    pub token_account: String,
    pub ui_amount: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Extensions {
    #[serde(rename = "coingeckoId")]
    pub coingecko_id: Option<String>,
    #[serde(rename = "serumV3Usdc")]
    pub serum_v3_usdc: Option<String>,
    #[serde(rename = "serumV3Usdt")]
    pub serum_v3_usdt: Option<String>,
    pub website: Option<String>,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub description: Option<String>,
    pub discord: Option<String>,
    pub medium: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetailOverview {
    pub address: String,
    pub decimals: i32,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "marketCap")]
    pub market_cap: f64,
    pub fdv: f64,
    pub extensions: Extensions,
    #[serde(rename = "logoURI")]
    pub logo_uri: String,
    pub liquidity: f64,
    #[serde(rename = "lastTradeUnixTime")]
    pub last_trade_unix_time: i64,
    #[serde(rename = "lastTradeHumanTime")]
    pub last_trade_human_time: String,
    pub price: f64,
    #[serde(rename = "history30mPrice")]
    pub history_30m_price: f64,
    #[serde(rename = "priceChange30mPercent")]
    pub price_change_30m_percent: f64,
    #[serde(rename = "history1hPrice")]
    pub history_1h_price: f64,
    #[serde(rename = "priceChange1hPercent")]
    pub price_change_1h_percent: f64,
    #[serde(rename = "history2hPrice")]
    pub history_2h_price: f64,
    #[serde(rename = "priceChange2hPercent")]
    pub price_change_2h_percent: f64,
    #[serde(rename = "history4hPrice")]
    pub history_4h_price: f64,
    #[serde(rename = "priceChange4hPercent")]
    pub price_change_4h_percent: f64,
    #[serde(rename = "history6hPrice")]
    pub history_6h_price: f64,
    #[serde(rename = "priceChange6hPercent")]
    pub price_change_6h_percent: f64,
    #[serde(rename = "history8hPrice")]
    pub history_8h_price: f64,
    #[serde(rename = "priceChange8hPercent")]
    pub price_change_8h_percent: f64,
    #[serde(rename = "history12hPrice")]
    pub history_12h_price: f64,
    #[serde(rename = "priceChange12hPercent")]
    pub price_change_12h_percent: f64,
    #[serde(rename = "history24hPrice")]
    pub history_24h_price: f64,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h_percent: f64,
    #[serde(rename = "uniqueWallet30m")]
    pub unique_wallet_30m: i64,
    #[serde(rename = "uniqueWalletHistory30m")]
    pub unique_wallet_history_30m: i64,
    #[serde(rename = "uniqueWallet30mChangePercent")]
    pub unique_wallet_30m_change_percent: f64,
    #[serde(rename = "uniqueWallet1h")]
    pub unique_wallet_1h: i64,
    #[serde(rename = "uniqueWalletHistory1h")]
    pub unique_wallet_history_1h: i64,
    #[serde(rename = "uniqueWallet1hChangePercent")]
    pub unique_wallet_1h_change_percent: f64,
    #[serde(rename = "uniqueWallet2h")]
    pub unique_wallet_2h: i64,
    #[serde(rename = "uniqueWalletHistory2h")]
    pub unique_wallet_history_2h: i64,
    #[serde(rename = "uniqueWallet2hChangePercent")]
    pub unique_wallet_2h_change_percent: f64,
    #[serde(rename = "uniqueWallet4h")]
    pub unique_wallet_4h: i64,
    #[serde(rename = "uniqueWalletHistory4h")]
    pub unique_wallet_history_4h: i64,
    #[serde(rename = "uniqueWallet4hChangePercent")]
    pub unique_wallet_4h_change_percent: f64,
    #[serde(rename = "uniqueWallet8h")]
    pub unique_wallet_8h: i64,
    #[serde(rename = "uniqueWalletHistory8h")]
    pub unique_wallet_history_8h: i64,
    #[serde(rename = "uniqueWallet8hChangePercent")]
    pub unique_wallet_8h_change_percent: f64,
    #[serde(rename = "uniqueWallet24h")]
    pub unique_wallet_24h: i64,
    #[serde(rename = "uniqueWalletHistory24h")]
    pub unique_wallet_history_24h: i64,
    #[serde(rename = "uniqueWallet24hChangePercent")]
    pub unique_wallet_24h_change_percent: f64,
    #[serde(rename = "totalSupply")]
    pub total_supply: f64,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: f64,
    pub holder: i64,
    #[serde(rename = "trade30m")]
    pub trade_30m: i64,
    #[serde(rename = "tradeHistory30m")]
    pub trade_history_30m: i64,
    #[serde(rename = "trade30mChangePercent")]
    pub trade_30m_change_percent: f64,
    #[serde(rename = "sell30m")]
    pub sell_30m: i64,
    #[serde(rename = "sellHistory30m")]
    pub sell_history_30m: i64,
    #[serde(rename = "sell30mChangePercent")]
    pub sell_30m_change_percent: f64,
    #[serde(rename = "buy30m")]
    pub buy_30m: i64,
    #[serde(rename = "buyHistory30m")]
    pub buy_history_30m: i64,
    #[serde(rename = "buy30mChangePercent")]
    pub buy_30m_change_percent: f64,
    #[serde(rename = "v30m")]
    pub v_30m: f64,
    #[serde(rename = "v30mUSD")]
    pub v_30m_usd: f64,
    #[serde(rename = "vHistory30m")]
    pub v_history_30m: f64,
    #[serde(rename = "vHistory30mUSD")]
    pub v_history_30m_usd: f64,
    #[serde(rename = "v30mChangePercent")]
    pub v_30m_change_percent: f64,
    #[serde(rename = "vBuy30m")]
    pub v_buy_30m: f64,
    #[serde(rename = "vBuy30mUSD")]
    pub v_buy_30m_usd: f64,
    #[serde(rename = "vBuyHistory30m")]
    pub v_buy_history_30m: f64,
    #[serde(rename = "vBuyHistory30mUSD")]
    pub v_buy_history_30m_usd: f64,
    #[serde(rename = "vBuy30mChangePercent")]
    pub v_buy_30m_change_percent: f64,
    #[serde(rename = "vSell30m")]
    pub v_sell_30m: f64,
    #[serde(rename = "vSell30mUSD")]
    pub v_sell_30m_usd: f64,
    #[serde(rename = "vSellHistory30m")]
    pub v_sell_history_30m: f64,
    #[serde(rename = "vSellHistory30mUSD")]
    pub v_sell_history_30m_usd: f64,
    #[serde(rename = "vSell30mChangePercent")]
    pub v_sell_30m_change_percent: f64,
    #[serde(rename = "trade1h")]
    pub trade_1h: i64,
    #[serde(rename = "tradeHistory1h")]
    pub trade_history_1h: i64,
    #[serde(rename = "trade1hChangePercent")]
    pub trade_1h_change_percent: f64,
    #[serde(rename = "sell1h")]
    pub sell_1h: i64,
    #[serde(rename = "sellHistory1h")]
    pub sell_history_1h: i64,
    #[serde(rename = "sell1hChangePercent")]
    pub sell_1h_change_percent: f64,
    #[serde(rename = "buy1h")]
    pub buy_1h: i64,
    #[serde(rename = "buyHistory1h")]
    pub buy_history_1h: i64,
    #[serde(rename = "buy1hChangePercent")]
    pub buy_1h_change_percent: f64,
    #[serde(rename = "v1h")]
    pub v_1h: f64,
    #[serde(rename = "v1hUSD")]
    pub v_1h_usd: f64,
    #[serde(rename = "vHistory1h")]
    pub v_history_1h: f64,
    #[serde(rename = "vHistory1hUSD")]
    pub v_history_1h_usd: f64,
    #[serde(rename = "v1hChangePercent")]
    pub v_1h_change_percent: f64,
    #[serde(rename = "vBuy1h")]
    pub v_buy_1h: f64,
    #[serde(rename = "vBuy1hUSD")]
    pub v_buy_1h_usd: f64,
    #[serde(rename = "vBuyHistory1h")]
    pub v_buy_history_1h: f64,
    #[serde(rename = "vBuyHistory1hUSD")]
    pub v_buy_history_1h_usd: f64,
    #[serde(rename = "vBuy1hChangePercent")]
    pub v_buy_1h_change_percent: f64,
    #[serde(rename = "vSell1h")]
    pub v_sell_1h: f64,
    #[serde(rename = "vSell1hUSD")]
    pub v_sell_1h_usd: f64,
    #[serde(rename = "vSellHistory1h")]
    pub v_sell_history_1h: f64,
    #[serde(rename = "vSellHistory1hUSD")]
    pub v_sell_history_1h_usd: f64,
    #[serde(rename = "vSell1hChangePercent")]
    pub v_sell_1h_change_percent: f64,
    #[serde(rename = "trade2h")]
    pub trade_2h: i64,
    #[serde(rename = "tradeHistory2h")]
    pub trade_history_2h: i64,
    #[serde(rename = "trade2hChangePercent")]
    pub trade_2h_change_percent: f64,
    #[serde(rename = "sell2h")]
    pub sell_2h: i64,
    #[serde(rename = "sellHistory2h")]
    pub sell_history_2h: i64,
    #[serde(rename = "sell2hChangePercent")]
    pub sell_2h_change_percent: f64,
    #[serde(rename = "buy2h")]
    pub buy_2h: i64,
    #[serde(rename = "buyHistory2h")]
    pub buy_history_2h: i64,
    #[serde(rename = "buy2hChangePercent")]
    pub buy_2h_change_percent: f64,
    #[serde(rename = "v2h")]
    pub v_2h: f64,
    #[serde(rename = "v2hUSD")]
    pub v_2h_usd: f64,
    #[serde(rename = "vHistory2h")]
    pub v_history_2h: f64,
    #[serde(rename = "vHistory2hUSD")]
    pub v_history_2h_usd: f64,
    #[serde(rename = "v2hChangePercent")]
    pub v_2h_change_percent: f64,
    #[serde(rename = "vBuy2h")]
    pub v_buy_2h: f64,
    #[serde(rename = "vBuy2hUSD")]
    pub v_buy_2h_usd: f64,
    #[serde(rename = "vBuyHistory2h")]
    pub v_buy_history_2h: f64,
    #[serde(rename = "vBuyHistory2hUSD")]
    pub v_buy_history_2h_usd: f64,
    #[serde(rename = "vBuy2hChangePercent")]
    pub v_buy_2h_change_percent: f64,
    #[serde(rename = "vSell2h")]
    pub v_sell_2h: f64,
    #[serde(rename = "vSell2hUSD")]
    pub v_sell_2h_usd: f64,
    #[serde(rename = "vSellHistory2h")]
    pub v_sell_history_2h: f64,
    #[serde(rename = "vSellHistory2hUSD")]
    pub v_sell_history_2h_usd: f64,
    #[serde(rename = "vSell2hChangePercent")]
    pub v_sell_2h_change_percent: f64,
    #[serde(rename = "trade4h")]
    pub trade_4h: i64,
    #[serde(rename = "tradeHistory4h")]
    pub trade_history_4h: i64,
    #[serde(rename = "trade4hChangePercent")]
    pub trade_4h_change_percent: f64,
    #[serde(rename = "sell4h")]
    pub sell_4h: i64,
    #[serde(rename = "sellHistory4h")]
    pub sell_history_4h: i64,
    #[serde(rename = "sell4hChangePercent")]
    pub sell_4h_change_percent: f64,
    #[serde(rename = "buy4h")]
    pub buy_4h: i64,
    #[serde(rename = "buyHistory4h")]
    pub buy_history_4h: i64,
    #[serde(rename = "buy4hChangePercent")]
    pub buy_4h_change_percent: f64,
    #[serde(rename = "v4h")]
    pub v_4h: f64,
    #[serde(rename = "v4hUSD")]
    pub v_4h_usd: f64,
    #[serde(rename = "vHistory4h")]
    pub v_history_4h: f64,
    #[serde(rename = "vHistory4hUSD")]
    pub v_history_4h_usd: f64,
    #[serde(rename = "v4hChangePercent")]
    pub v_4h_change_percent: f64,
    #[serde(rename = "vBuy4h")]
    pub v_buy_4h: f64,
    #[serde(rename = "vBuy4hUSD")]
    pub v_buy_4h_usd: f64,
    #[serde(rename = "vBuyHistory4h")]
    pub v_buy_history_4h: f64,
    #[serde(rename = "vBuyHistory4hUSD")]
    pub v_buy_history_4h_usd: f64,
    #[serde(rename = "vBuy4hChangePercent")]
    pub v_buy_4h_change_percent: f64,
    #[serde(rename = "vSell4h")]
    pub v_sell_4h: f64,
    #[serde(rename = "vSell4hUSD")]
    pub v_sell_4h_usd: f64,
    #[serde(rename = "vSellHistory4h")]
    pub v_sell_history_4h: f64,
    #[serde(rename = "vSellHistory4hUSD")]
    pub v_sell_history_4h_usd: f64,
    #[serde(rename = "vSell4hChangePercent")]
    pub v_sell_4h_change_percent: f64,
    #[serde(rename = "trade8h")]
    pub trade_8h: i64,
    #[serde(rename = "tradeHistory8h")]
    pub trade_history_8h: i64,
    #[serde(rename = "trade8hChangePercent")]
    pub trade_8h_change_percent: f64,
    #[serde(rename = "sell8h")]
    pub sell_8h: i64,
    #[serde(rename = "sellHistory8h")]
    pub sell_history_8h: i64,
    #[serde(rename = "sell8hChangePercent")]
    pub sell_8h_change_percent: f64,
    #[serde(rename = "buy8h")]
    pub buy_8h: i64,
    #[serde(rename = "buyHistory8h")]
    pub buy_history_8h: i64,
    #[serde(rename = "buy8hChangePercent")]
    pub buy_8h_change_percent: f64,
    #[serde(rename = "v8h")]
    pub v_8h: f64,
    #[serde(rename = "v8hUSD")]
    pub v_8h_usd: f64,
    #[serde(rename = "vHistory8h")]
    pub v_history_8h: f64,
    #[serde(rename = "vHistory8hUSD")]
    pub v_history_8h_usd: f64,
    #[serde(rename = "v8hChangePercent")]
    pub v_8h_change_percent: f64,
    #[serde(rename = "vBuy8h")]
    pub v_buy_8h: f64,
    #[serde(rename = "vBuy8hUSD")]
    pub v_buy_8h_usd: f64,
    #[serde(rename = "vBuyHistory8h")]
    pub v_buy_history_8h: f64,
    #[serde(rename = "vBuyHistory8hUSD")]
    pub v_buy_history_8h_usd: f64,
    #[serde(rename = "vBuy8hChangePercent")]
    pub v_buy_8h_change_percent: f64,
    #[serde(rename = "vSell8h")]
    pub v_sell_8h: f64,
    #[serde(rename = "vSell8hUSD")]
    pub v_sell_8h_usd: f64,
    #[serde(rename = "vSellHistory8h")]
    pub v_sell_history_8h: f64,
    #[serde(rename = "vSellHistory8hUSD")]
    pub v_sell_history_8h_usd: f64,
    #[serde(rename = "vSell8hChangePercent")]
    pub v_sell_8h_change_percent: f64,
    #[serde(rename = "trade24h")]
    pub trade_24h: i64,
    #[serde(rename = "tradeHistory24h")]
    pub trade_history_24h: i64,
    #[serde(rename = "trade24hChangePercent")]
    pub trade_24h_change_percent: f64,
    #[serde(rename = "sell24h")]
    pub sell_24h: i64,
    #[serde(rename = "sellHistory24h")]
    pub sell_history_24h: i64,
    #[serde(rename = "sell24hChangePercent")]
    pub sell_24h_change_percent: f64,
    #[serde(rename = "buy24h")]
    pub buy_24h: i64,
    #[serde(rename = "buyHistory24h")]
    pub buy_history_24h: i64,
    #[serde(rename = "buy24hChangePercent")]
    pub buy_24h_change_percent: f64,
    #[serde(rename = "v24h")]
    pub v_24h: f64,
    #[serde(rename = "v24hUSD")]
    pub v_24h_usd: f64,
    #[serde(rename = "vHistory24h")]
    pub v_history_24h: f64,
    #[serde(rename = "vHistory24hUSD")]
    pub v_history_24h_usd: f64,
    #[serde(rename = "v24hChangePercent")]
    pub v_24h_change_percent: f64,
    #[serde(rename = "vBuy24h")]
    pub v_buy_24h: f64,
    #[serde(rename = "vBuy24hUSD")]
    pub v_buy_24h_usd: f64,
    #[serde(rename = "vBuyHistory24h")]
    pub v_buy_history_24h: f64,
    #[serde(rename = "vBuyHistory24hUSD")]
    pub v_buy_history_24h_usd: f64,
    #[serde(rename = "vBuy24hChangePercent")]
    pub v_buy_24h_change_percent: f64,
    #[serde(rename = "vSell24h")]
    pub v_sell_24h: f64,
    #[serde(rename = "vSell24hUSD")]
    pub v_sell_24h_usd: f64,
    #[serde(rename = "vSellHistory24h")]
    pub v_sell_history_24h: f64,
    #[serde(rename = "vSellHistory24hUSD")]
    pub v_sell_history_24h_usd: f64,
    #[serde(rename = "vSell24hChangePercent")]
    pub v_sell_24h_change_percent: f64,
    #[serde(rename = "numberMarkets")]
    pub number_markets: i32,
}

pub trait TokenSdk {
    async fn get_trending(&self, offset: i32, limit: i32) -> Result<Vec<Trending>, anyhow::Error>;
    async fn token_meta_multiple(
        &self,
        addresses: Vec<String>,
    ) -> Result<Vec<TokenMetadata>, anyhow::Error>;
    async fn overview(&self, address: &str) -> Result<TokenOverview, anyhow::Error>;
    async fn token_detail_overview(
        &self,
        address: &str,
    ) -> Result<TokenDetailOverview, anyhow::Error>;
    async fn holders(&self, address: &str) -> Result<Vec<TokenHolder>, anyhow::Error>;
}

pub async fn upsert_token_meta(
    pool: &Pool<Postgres>,
    trending_list: &Vec<Trending>,
) -> Result<(), sqlx::Error> {
    let mut qb = QueryBuilder::new(
        "INSERT INTO tokens (token_address, name, symbol, decimals, image_url, current_price, updated_at) ",
    );

    qb.push_values(trending_list.iter(), |mut b, item| {
        b.push_bind(&item.address)
            .push_bind(&item.name)
            .push_bind(&item.symbol)
            .push_bind(item.decimals)
            .push_bind(&item.logo_uri)
            .push_bind(item.price)
            .push("NOW()");
    });

    qb.push(
        " ON CONFLICT (token_address) DO UPDATE SET \
         name = EXCLUDED.name, \
         symbol = EXCLUDED.symbol, \
         decimals = EXCLUDED.decimals, \
         image_url = EXCLUDED.image_url, \
         current_price = EXCLUDED.current_price, \
         updated_at = NOW()",
    );

    qb.build().execute(pool).await?;
    Ok(())
}
pub async fn upsert_daily_volume(
    pool: &Pool<Postgres>,
    trending_list: &[Trending],
    record_date: i64,
) -> Result<(), sqlx::Error> {
    let mut qb = QueryBuilder::new(
        "INSERT INTO token_volume_history (token_address, volume24h, record_date) ",
    );

    qb.push_values(trending_list.iter(), |mut b, trending| {
        b.push_bind(&trending.address)
            .push_bind(trending.volume24h_usd)
            .push_bind(record_date);
    });

    qb.push(
        " ON CONFLICT (token_address, record_date) DO UPDATE SET \
         volume24h = EXCLUDED.volume24h",
    );

    qb.build().execute(pool).await?;
    Ok(())
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenVolumeHistory {
    pub token_address: String,
    pub volume24h: BigDecimal,
    pub record_date: i64,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
    #[sqlx(default)]
    pub volume24h_percent: Option<BigDecimal>,
}

pub async fn query_top_token_volume_history_by_date(
    pool: &Pool<Postgres>,
    limit: i64,
    timestamp: i64,
) -> anyhow::Result<Vec<TokenVolumeHistory>> {
    let records = sqlx::query_as::<_, TokenVolumeHistory>(
        r#"
        SELECT tvh.token_address,
        tvh.volume24h,
        tvh.record_date,
        t.image_url AS logo_uri,
        t.name as name,
        t.symbol as symbol,
        tm.volume_change_percent as volume24h_percent
        FROM token_volume_history tvh
        INNER JOIN tokens t ON t.token_address = tvh.token_address
        LEFT JOIN token_metrics tm ON tm.token_address = tvh.token_address
        WHERE record_date = $2
        ORDER BY tvh.volume24h DESC LIMIT $1"#,
    )
    .bind(limit)
    .bind(timestamp)
    .fetch_all(pool)
    .await?;
    Ok(records)
}

pub async fn query_top_token_volume_history(
    pool: &Pool<Postgres>,
    limit: i64,
) -> anyhow::Result<Vec<TokenVolumeHistory>> {
    let records = sqlx::query_as::<_, TokenVolumeHistory>(
        r#"
        with tvh as (SELECT token_address, max(record_date) as max_record_date, sum(volume24h) as total_volume24h
            FROM token_volume_history
            WHERE record_date > (EXTRACT(EPOCH FROM NOW()) - 86400 * 2)
            GROUP BY token_address
            LIMIT 100)
        SELECT tvh.token_address,
        tvh.total_volume24h as volume24h,
        tvh.max_record_date as record_date,
        t.image_url         AS logo_uri,
        t.name              as name,
        t.symbol            as symbol
        FROM tvh INNER JOIN tokens t
        ON t.token_address = tvh.token_address LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(records)
}

pub async fn token_by_address(
    pool: &Pool<Postgres>,
    addresses: Vec<String>,
) -> anyhow::Result<Vec<String>> {
    // Query the existing token addresses from the tokens table.
    let existing: Vec<String> =
        sqlx::query_scalar("SELECT token_address FROM tokens WHERE token_address = ANY($1)")
            .bind(&addresses)
            .fetch_all(pool)
            .await?;

    // Retain only addresses that are not present in the existing list.
    let missing: Vec<String> = addresses
        .iter()
        .filter(|addr| !existing.contains(addr))
        .cloned()
        .collect();
    Ok(missing)
}
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Token {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub logo_uri: Option<String>,
}
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenOverviewResponse {
    pub token_address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: Option<i32>,
    pub logo_uri: Option<String>,
    pub website_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub current_price: Option<BigDecimal>,
    pub total_supply: Option<BigDecimal>,
    pub marketcap: Option<BigDecimal>,
    pub history24h_price: Option<BigDecimal>,
    pub price_change24h_percent: Option<BigDecimal>,
}

pub async fn token_bio(
    pool: &Pool<Postgres>,
    address: &str,
) -> anyhow::Result<TokenOverviewResponse> {
    let token = sqlx::query_as::<_, TokenOverviewResponse>(
        "
        SELECT
            token_address,
            name,
            symbol,
            decimals,
            description,
            image_url as logo_uri,
            website_url,
            metadata,
            current_price,
            total_supply,
            marketcap,
            history24h_price,
            price_change24h_percent
        FROM tokens
        WHERE token_address = $1
        ",
    )
    .bind(&address)
    .fetch_one(pool)
    .await?;
    Ok(token)
}

#[cfg(test)]
mod internal_test {
    use crate::thirdparty::BirdEyeResponse;
    use crate::token::TokenMetadata;
    use std::collections::HashMap;

    #[test]
    fn test_dese() {
        let a : BirdEyeResponse<HashMap<String, TokenMetadata>> = serde_json::from_str(r#"
{
  "data": {
    "Kruj63Qx9EQX9QzukLCBgx5g9AGW69gPDsSK25FRZAi": {
      "address": "Kruj63Qx9EQX9QzukLCBgx5g9AGW69gPDsSK25FRZAi",
      "name": "EnKryptedAI",
      "symbol": "KRAI",
      "decimals": 6,
      "extensions": {
        "description": "Your favorite superhero's dog is now your ultimate AI-powered guardian protecting against crypto scams. With superintelligent detection, EnKrypto sniffs out fraud, protects your assets, and provides real-time market insights. Loyal, fast, and unstoppable, EnKryptedAI ensures you stay ahead in the world of blockchain. 🚀🐶💎"
      },
      "logo_uri": "https://ipfs.io/ipfs/QmeR75gX8kuwbFzLzj2GBDLNbZjpS4ezsV1zjyCP4uw7F7"
    },
    "So11111111111111111111111111111111111111112": {
      "address": "So11111111111111111111111111111111111111112",
      "name": "Wrapped SOL",
      "symbol": "SOL",
      "decimals": 9,
      "extensions": {
        "coingecko_id": "solana",
        "serum_v3_usdc": "9wFFyRfZBsuAha4YcuxcXLKwMxJR43S7fPfQLusDBzvT",
        "serum_v3_usdt": "HWHvQhFmJB3NUcu1aihKmrKegfVxBEHzwVX6yZCKEsi1",
        "website": "https://solana.com/",
        "telegram": null,
        "twitter": "https://twitter.com/solana",
        "description": "Wrapped Solana ",
        "discord": "https://discordapp.com/invite/pquxPsq",
        "medium": "https://medium.com/solana-labs"
      },
      "logo_uri": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/So11111111111111111111111111111111111111112/logo.png"
    }
  },
  "success": true
}
        "#).unwrap();
    }
}
