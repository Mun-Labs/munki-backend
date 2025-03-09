use crate::price::{PriceSdk, TokenPrice};

#[allow(dead_code)]
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
impl PriceSdk for BirdEyeClient {
    async fn get_price(&self, token: &str) -> Result<TokenPrice, anyhow::Error> {
        Ok(TokenPrice {
            token: token.to_string(),
            price: 0.0,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::price::PriceSdk;

    #[tokio::test]
    async fn test_get_price_should_return_token_price() {
        let client = super::BirdEyeClient::new("https://api.birdeye.com", "api_key");
        let token_address = "0x1234567890abcdef";
        let token_price = client.get_price(token_address).await.unwrap();
        assert_eq!(token_price.token, token_address);
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
}
