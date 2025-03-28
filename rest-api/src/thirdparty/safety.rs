use reqwest::Client;
use serde::Deserialize;

const SAFE_API_URL: &str =
    "https://l7db1lpgkb.execute-api.us-east-2.amazonaws.com/prod/orchestration";

#[derive(Deserialize)]
pub struct SafeScore {
    pub orchestration: Orchestration,
}

#[derive(Deserialize)]
pub struct Orchestration {
    pub safety_score: f64,
}

pub async fn get_safe_score(client: &Client, token: &str) -> Result<f64, anyhow::Error> {
    let response = client
        .get(SAFE_API_URL)
        .query(&[("ca", token)])
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to get safe score: {}",
            response.text().await?
        ));
    }

    let safe_score: SafeScore = response.json().await?;
    Ok(safe_score.orchestration.safety_score)
}

#[cfg(test)]
mod test {
    use crate::thirdparty::safety::get_safe_score;

    #[tokio::test]
    async fn test_get_safe_score() {
        use reqwest::Client;
        let client = Client::new();
        let token = "34HDZNbUkTyTrgYKy2ox43yp2f8PJ5hoM7xsrfNApump";
        let score = get_safe_score(&client, token).await.unwrap();
        println!("Safe score: {score}");
    }

    #[test]
    fn deserialize_safe_score() {
        let json = r#"
        {
            "orchestration": {
                "safety_score": 62.2,
                "token_info": {
                    "ca": "34HDZNbUkTyTrgYKy2ox43yp2f8PJ5hoM7xsrfNApump",
                    "symbol": "Routine",
                    "description": "",
                    "mint_auth_disabled": true,
                    "lp_burnt_per": 100,
                    "age": "6d 17hr 39m",
                    "mc": "1.2m",
                    "top_ten_per": 19.96,
                    "top_indv_per": 8.20998673662497,
                    "total_wallets": 11374
                }
            }
        }
        "#;
        let safe_score: super::SafeScore = serde_json::from_str(json).unwrap();
        assert_eq!(safe_score.orchestration.safety_score, 62.2);
    }
}
