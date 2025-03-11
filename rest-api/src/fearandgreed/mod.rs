pub mod route;

use serde::Serialize;

#[derive(Serialize)]
pub struct FearAndGreed{
    pub value: i64,
    pub status: String,
    pub timestamp: String,
    pub chain: String
}

pub trait FearAndGreedSdk {
    async fn get_fear_and_greed(&self, limit: &i8) -> Result<Vec<FearAndGreed>, anyhow::Error>;
}