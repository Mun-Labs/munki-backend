#[allow(dead_code)]
pub struct BirdEyeClient {
    base_url: String,
    api_key: String,
}

pub struct TrendingTokenResponse {}

pub trait TokenSdk {
    fn trending_token(&self) -> TrendingTokenResponse;
}

#[cfg(test)]
mod test {

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
