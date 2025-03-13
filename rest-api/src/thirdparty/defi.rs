use reqwest::Client;
use serde::{Deserialize, Serialize};

const DEFILLAMA_URL: &str = "https://api.llama.fi";
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct MetricsResponse {
    #[serde(rename = "total24h")]
    pub total_24h: i64,

    #[serde(rename = "total48hto24h")]
    pub total_48h_to_24h: i64,

    #[serde(rename = "total7d")]
    pub total_7d: i64,

    #[serde(rename = "total14dto7d")]
    pub total_14d_to_7d: i64,

    #[serde(rename = "total60dto30d")]
    pub total_60d_to_30d: i64,

    #[serde(rename = "total30d")]
    pub total_30d: i64,

    #[serde(rename = "total1y")]
    pub total_1y: i64,

    #[serde(rename = "change_1d")]
    pub change_1d: f64,

    #[serde(rename = "change_7d")]
    pub change_7d: f64,

    #[serde(rename = "change_1m")]
    pub change_1m: f64,

    #[serde(rename = "change_7dover7d")]
    pub change_7d_over_7d: f64,

    #[serde(rename = "change_30dover30d")]
    pub change_30d_over_30d: f64,

    #[serde(rename = "total7DaysAgo")]
    pub total_7_days_ago: i64,

    #[serde(rename = "total30DaysAgo")]
    pub total_30_days_ago: i64,
}

pub async fn get_blockchain_volume(
    client: &Client,
    chain: &str,
) -> Result<MetricsResponse, anyhow::Error> {
    let resp = client
        .get(format!("{DEFILLAMA_URL}/overview/dexs/{chain}"))
        .query(&[
            ("excludeTotalDataChart", "true"),
            ("excludeTotalDataChartBreakdown", "true"),
            ("dataType", "dailyVolume"),
        ])
        .send()
        .await?;

    if !resp.status().is_success() {
        return Err(anyhow::anyhow!(
            "Request failed with status: {}",
            resp.status()
        ));
    }
    Ok(resp.json::<MetricsResponse>().await?)
}

#[cfg(test)]
mod test {
    use reqwest::Client;

    use crate::thirdparty::get_blockchain_volume;

    #[tokio::test]
    async fn test_defi_api() {
        let client = Client::new();
        let chain = "solana";
        let result = get_blockchain_volume(&client, chain).await.unwrap();
        println!("{result:?}");
    }
}
