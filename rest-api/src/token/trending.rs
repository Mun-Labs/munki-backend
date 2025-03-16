use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, QueryBuilder};
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
    pub rank: u32,
    pub price: f64,
}

pub trait TrendingSdk {
    async fn get_trending(&self, offset: i32, limit: i32) -> Result<Vec<Trending>, anyhow::Error>;
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
    trending_list: &Vec<Trending>,
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
}

pub async fn query_top_token_volume_history(
    pool: &Pool<Postgres>,
    limit: i64,
) -> anyhow::Result<Vec<TokenVolumeHistory>> {
    let records = sqlx::query_as::<_, TokenVolumeHistory>(
        r#"SELECT tvh.token_address,
                  tvh.volume24h,
                  tvh.record_date,
                  t.image_url AS logo_uri,
                  t.name as name,
                  t.symbol as symbol
           FROM token_volume_history tvh
           INNER JOIN tokens t ON t.token_address = tvh.token_address
           ORDER BY tvh.volume24h DESC
           LIMIT $1"#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(records)
}
