use crate::price::PriceSdk;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct BirdEyeClient {
    base_url: String,
    api_key: String,
}

impl BirdEyeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
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
        let client = Client::new();
        let url = format!("{}/defi/price_volume/single", self.base_url);
        info!("price endpoint: {url}");
        let resp = client
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
    async fn test_reqwest() {
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
