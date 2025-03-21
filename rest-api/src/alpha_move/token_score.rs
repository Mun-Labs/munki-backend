use bigdecimal::BigDecimal;

#[derive(serde::Serialize, Debug, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetric {
    pub token_address: String,
    pub mun_score: BigDecimal,
    pub top_fresh_wallet_holders: i64,
    pub top_smart_wallets_holders: i64,
    pub smart_followers: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_token_metrics_by_addresses(
    pool: &sqlx::Pool<sqlx::Postgres>,
    addresses: Vec<String>,
) -> Result<Vec<TokenMetric>, sqlx::Error> {
    sqlx::query_as::<_, TokenMetric>(
        "SELECT
             token_address,
             mun_score,
             top_fresh_wallet_holders,
             top_smart_wallets_holders,
             smart_followers,
             created_at,
             updated_at
         FROM alpha_move_token_metric
         WHERE token_address = ANY($1)",
    )
    .bind(addresses)
    .fetch_all(pool)
    .await
}
