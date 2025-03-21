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

        let status = resp.status();
        if !status.is_success() {
            return Err(Error::msg(format!(
                "Failed to get mun score for {username}: {status}",
            )));
        }

        let data = resp.json::<MunScoreData>().await?;
        Ok(data)
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MunScoreData {
    //pub meta: Meta,
    pub smart_engagement: SmartEngagement,
    //pub smart_profile: SmartProfile,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Meta {
    pub twitter_user_id: String,
    pub name: String,
    pub username: String,
    pub description: String,
    pub twitter_created_at: u64,
    pub tweet_count: u64,
    pub followers_count: u64,
    pub profile_image_url: String,
    pub profile_banner_url: String,
    pub links: Vec<Link>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub url: String,
    pub logo_url: String,
    pub r#type: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartEngagement {
    pub smart_followers_count: u64,
    pub followers_score: u64,
    pub mentions_count: u64,
    pub smart_mentions_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartProfile {
    pub smart_tier: SmartTier,
    pub smart_tags: Vec<SmartTag>,
    pub smart_tag_categories: Vec<SmartTagCategory>,
    pub project_tags: Vec<ProjectTag>,
    pub chains: Vec<Chain>,
    pub bio_changes_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartTier {
    pub tier: u8,
    pub logo_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartTag {
    pub slug: String,
    pub name: String,
    pub total_count: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SmartTagCategory {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTag {
    pub slug: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    pub slug: String,
    pub name: String,
}

#[cfg(test)]
mod test {
    use crate::thirdparty::{MoniClient, MunScoreSdk};

    #[tokio::test]
    async fn test_get_mun_score() {
        let key = "5e1738d2-79a1-40d4-9977-6a8425f2a721";
        let username = "elonmusk";
        let client = reqwest::Client::new();
        let moni_client = MoniClient::new(key.into(), client);
        let data = moni_client.get_mun_score(username).await.unwrap();
        println!("{:?}", data);
    }
}
