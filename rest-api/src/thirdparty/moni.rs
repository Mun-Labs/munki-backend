use anyhow::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct MoniClient {
    pub client: Client,
    pub moni_key: String,
}

impl MoniClient {
    pub fn new(moni_key: String, client: Client) -> Self {
        Self { moni_key, client }
    }
}

const BASE_URL: &str = "https://api.discover.getmoni.io/api/v2";

// curl --request GET \
// --url https://api.discover.getmoni.io/api/v2/twitters/aaaaaa/info/full/ \
// --header 'Api-Key: aaaaaaa' \
// --header 'accept: application/json'

pub trait MunScoreSdk {
    async fn get_mun_score(&self, username: &str) -> Result<MunScoreData, Error>;
}

impl MunScoreSdk for MoniClient {
    async fn get_mun_score(&self, username: &str) -> Result<MunScoreData, Error> {
        let url = format!("{BASE_URL}/twitters/{username}/info/full");
        let resp = self
            .client
            .get(&url)
            .header("Api-Key", self.moni_key.as_str())
            .send()
            .await?;
        if !resp.status().is_success() {
            return Err(Error::msg("Failed to get mun score with status code"));
        }
        Ok(MunScoreData {})
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MunScoreData {}
