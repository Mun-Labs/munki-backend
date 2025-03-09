pub mod route;
pub use route::route;

pub struct TokenPrice {
    pub token: String,
    pub price: f64,
}

pub trait PriceSdk {
    async fn get_price(&self, token: &str) -> Result<TokenPrice, anyhow::Error>;
}
