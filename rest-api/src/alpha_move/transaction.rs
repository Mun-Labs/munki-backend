use crate::app::AppState;
use bigdecimal::BigDecimal;
use sqlx::{Pool, Postgres};

#[derive(serde::Serialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MoverTransaction {
    pub signature: String,
    pub token_address: String,
    pub wallet_address: String,
    #[serde(rename = "actionType")]
    pub transaction_type: Option<String>,
    pub amount: BigDecimal,
    #[serde(rename = "time")]
    pub block_time: i64,
    pub slot: i64,
    // New fields from tokens table
    #[serde(rename = "coinName")]
    pub token_name: Option<String>,
    pub token_symbol: Option<String>,
    pub total_supply: Option<BigDecimal>,
    // New fields from market_mover table
    #[serde(rename = "alphaGroup")]
    pub mover_role: String,
    #[serde(rename = "name")]
    pub mover_name: String,
    pub decimals: Option<i32>,
    pub token_logo: Option<String>,
    pub mun_score: BigDecimal,
    pub risk_core: BigDecimal,
    pub top_fresh_wallet_holders: i64,
    pub top_smart_wallets_holders: i64,
    pub smart_followers: i64,

    pub marketcap: Option<BigDecimal>,
    pub history24h_price: Option<BigDecimal>,
    pub price_change24h_percent: Option<BigDecimal>,
    pub holders: Option<i32>,
    pub liquidity: Option<BigDecimal>,
    pub volume_24h: Option<BigDecimal>,
    pub volume_24h_change: Option<BigDecimal>,
}

pub async fn count_mover_transaction(app: &AppState) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*)
    FROM market_movers_transaction mm
    INNER JOIN market_mover m ON mm.wallet_address = m.wallet_address
    WHERE EXISTS(SELECT 1 FROM token_watch WHERE token_watch.token_address = mm.token_address)
    ",
    )
    .fetch_one(&app.pool)
    .await
}

pub async fn fetch_mover_transactions(
    pool: &Pool<Postgres>,
    limit: i64,
    offset: i64,
) -> Result<Vec<MoverTransaction>, sqlx::Error> {
    let transactions = sqlx::query_as::<_, MoverTransaction>(
        "
        SELECT
            mm.signature,
            mm.token_address,
            mm.wallet_address,
            mm.transaction_type,
            mm.amount,
            mm.block_time,
            mm.slot,
            mm.additional,
            t.name AS token_name,
            t.symbol AS token_symbol,
            t.decimals AS decimals,
            t.image_url AS token_logo,
            t.total_supply AS total_supply,
            t.marketcap as marketcap,
            t.history24h_price,
            t.price_change24h_percent,
            t.holders,
            t.liquidity,
            t.volume_24h,
            t.volume_24h_change,
            m.role AS mover_role,
            m.name AS mover_name,
            GREATEST(am.mun_score, 0::numeric) as mun_score,
            GREATEST(am.risk_score, 0::numeric) as risk_core,
            am.top_fresh_wallet_holders as top_fresh_wallet_holders,
            am.top_smart_wallets_holders as top_smart_wallets_holders,
            am.smart_followers as smart_followers
         FROM market_movers_transaction mm
         INNER JOIN market_mover m ON mm.wallet_address = m.wallet_address
         INNER JOIN tokens t ON mm.token_address = t.token_address
         INNER JOIN alpha_move_token_metric am ON am.token_address = mm.token_address
         WHERE EXISTS(SELECT 1 FROM token_watch WHERE token_watch.token_address = mm.token_address)
         ORDER BY mm.slot DESC
         LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(transactions)
}
