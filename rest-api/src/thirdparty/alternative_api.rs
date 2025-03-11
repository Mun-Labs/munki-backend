use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::fearandgreed::{FearAndGreed, FearAndGreedSdk};

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct AlternativeClient {
    base_url: String,
    limit: i8,
}

impl AlternativeClient {
    pub fn new(base_url: String, limit: i8) -> Self {
        Self {
            base_url: base_url.to_string(),
            limit,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AlternativeResponse {
    pub name: String,
    pub data: Vec<AlternativeResponseData>, // Use Vec for the array
    pub metadata: Metadata,
}

#[derive(Debug, serde::Deserialize)]
pub struct AlternativeResponseData {
    pub value: String,
    pub value_classification: String, // Match the API field name
    pub timestamp: String,
    pub time_until_update: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    pub error: Option<String>,
}

#[derive(Serialize)]
#[derive(Debug)]
struct QueryParams {
    limit: i8,
}

// Make sure this is inside the impl FearAndGreedSdk for AlternativeClient block
impl FearAndGreedSdk for AlternativeClient {
    async fn get_fear_and_greed(
        &self,
        limit: &i8,
    ) -> Result<Vec<FearAndGreed>, anyhow::Error> {
        let client = Client::new();
        let url = &self.base_url;

        let effective_limit = if *limit == 0 { &self.limit } else { limit };

        let query_params = QueryParams {
            limit: *effective_limit,
        };

        let resp = client
            .get(url)
            .query(&query_params)
            .header("accept", "application/json")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                resp.status()
            ));
        }

        let response_text = resp.text().await?;
        let parsed_resp: AlternativeResponse = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => {
                return Err(anyhow::anyhow!("Failed to parse response: {}\nResponse body: {}", e, response_text));
            }
        };

        let fear_and_greed_list: Vec<FearAndGreed> = parsed_resp.data.into_iter().map(|data| FearAndGreed {
            value: data.value.parse::<i64>().unwrap(),
            status: data.value_classification,
            timestamp: data.timestamp.to_string(),
            chain: "BTC".to_string(),
        }).collect();

        if fear_and_greed_list.is_empty() {
            return Err(anyhow::anyhow!("No data in response"));
        }

        Ok(fear_and_greed_list)
    }
}