use crate::price::{PriceSdk, TimeFilters};
use chrono::{Duration, Timelike, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct BirdEyeClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl BirdEyeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BirdEyeResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Items {
    pub value: f64,
    pub unix_time: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistory {
    pub items: Vec<Items>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    pub price: f64,
    pub update_unix_time: i64,
    pub update_human_time: String,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: f64,
    pub volume_change_percent: f64,
    pub price_change_percent: f64,
}

impl PriceSdk for BirdEyeClient {
    async fn get_price(&self, token: &str) -> Result<TokenData, anyhow::Error> {
        let url = format!("{}/defi/price_volume/single", self.base_url);
        info!("price endpoint: {url}");
        let resp = self
            .client
            .get(url)
            .query(&[("address", token)])
            .header("X-API-KEY", &self.api_key)
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
        let resp = resp.json::<BirdEyeResponse<TokenData>>().await?.data;
        Ok(resp)
    }

    async fn get_price_by_time_filter(
        &self,
        token: &str,
        filter: TimeFilters,
    ) -> Result<PriceHistory, anyhow::Error> {
        let now = Utc::now();

        // Beginning of today (midnight UTC)
        let start_of_today = now
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now);
        let today_unix = start_of_today.timestamp().to_string();

        // Beginning of last week (7 days ago at midnight UTC)
        let last_week = start_of_today - Duration::days(7);
        let last_week_unix = last_week.timestamp().to_string();
        let url = format!("{}/defi/history_price", self.base_url);
        let resp = self
            .client
            .get(url)
            .query(&[
                ("address", token),
                ("address_type", "token"),
                ("type", filter.as_query_param()),
                ("time_from", &last_week_unix),
                ("time_to", &today_unix),
            ])
            .header("X-API-KEY", &self.api_key)
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

        let resp = resp.json::<BirdEyeResponse<PriceHistory>>().await?.data;
        info!("fetch price history: {resp:?}");
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    use crate::price::PriceSdk;
    use crate::thirdparty::{BirdEyeResponse, TokenData};

    #[tokio::test]
    async fn test_get_price_should_return_token_price() {
        let client = super::BirdEyeClient::new("https://api.birdeye.com", "api_key");
        let token_address = "0x1234567890abcdef";
        let token_price = client.get_price(token_address).await.unwrap();
        assert_eq!(token_price.price, 0.0);
    }

    #[tokio::test]
    async fn test_get_price_history() {
        use std::collections::HashMap;
        let resp = reqwest::get("https://httpbin.org/ip")
            .await
            .unwrap()
            .json::<HashMap<String, String>>()
            .await
            .unwrap();
        println!("{resp:#?}");
    }

    #[test]
    fn deserialize() {
        let resp_string = r#"{"success":true,"data":{"price":134.8625396815726,"updateUnixTime":1741526897,"updateHumanTime":"2025-03-09T13:28:17","volumeUSD":1726429451.421973,"volumeChangePercent":-37.9876049836133,"priceChangePercent":-2.2107631867842215}}"#;
        let body = serde_json::from_str::<BirdEyeResponse<TokenData>>(resp_string).unwrap();
        println!("Decoded response:\n{:#?}", body);
    }
}
