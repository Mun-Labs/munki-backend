use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct DefiLlamaVolumeResponse {
    pub total24h: u64,          // 24-hour total
    pub total48hto24h: u64,     // 48-to-24-hour total
    pub total7d: u64,           // 7-day total
    pub total14dto7d: u64,      // 14-to-7-day total
    pub total60dto30d: u64,     // 60-to-30-day total
    pub total30d: u64,          // 30-day total
    pub total1y: u64,           // 1-year total
    pub change_1d: f64,         // 1-day percentage change
    pub change_7d: f64,         // 7-day percentage change
    pub change_1m: f64,         // 1-month percentage change
    pub change_7dover7d: f64,   // 7-day-over-7-day percentage change
    pub change_30dover30d: f64, // 30-day-over-30-day percentage change
    #[serde(rename = "total7DaysAgo")]
    pub total7_days_ago: u64, // Total from 7 days ago
    #[serde(rename = "total30DaysAgo")]
    pub total30_days_ago: u64, // Total from 30 days ago
}

const BASE_URL: &str = "https://api.llama.fi/overview/dexs";
pub struct DefiClient {
    pub client: Client,
}

impl DefiClient {
    pub async fn get_blockchain_volum(
        &self,
        chain: &str,
    ) -> Result<DefiLlamaVolumeResponse, anyhow::Error> {
        let resp = self
            .client
            .get(format!("{BASE_URL}/{chain}"))
            .query(&[
                ("excludeTotalDataChart", "true"),
                ("excludeTotalDataChartBreakdown", "true"),
                ("dataType", "dailyVolume"),
            ])
            .header("accept", "application/json")
            .header("x-chain", "solana")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                resp.status()
            ));
        }

        let resp = resp.json::<DefiLlamaVolumeResponse>().await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    #[tokio::test]
    async fn should_call_api_successfully() {
        use crate::thirdparty::defi::*;
        let client = DefiClient {
            client: Client::new(),
        };
        let resp = client.get_blockchain_volum("solana").await.unwrap();
        println!("{resp:?}")
    }

    #[test]
    fn should_deserialize_correctly() {
        use crate::thirdparty::defi::*;

        let json_data = r#"{
            "total24h": 1446817296,
            "total48hto24h": 1725371113,
            "total7d": 12822457325,
            "total14dto7d": 18401385020,
            "total60dto30d": 267278272724,
            "total30d": 76952019302,
            "total1y": 960584692474,
            "change_1d": -16.14,
            "change_7d": -34.86,
            "change_1m": -64.22,
            "change_7dover7d": -30.32,
            "change_30dover30d": -71.21,
            "total7DaysAgo": 2220929417,
            "total30DaysAgo": 4043810076
        }"#;

        let response: DefiLlamaVolumeResponse = serde_json::from_str(json_data).unwrap();
        println!("{:?}", response);
    }
}
