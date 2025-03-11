use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TokenMeta {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub total_supply: u64,
    pub current_price: u64,
}
