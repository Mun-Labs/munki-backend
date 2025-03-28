use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDistributions {
    category: Category,
    percentage: f64,
    amount: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Bubbles,
    Shrimp,
    Fish,
    Dolphin,
    Shark,
    Whale,
}

pub fn create_dummy_token_distribution() -> Vec<TokenDistributions> {
    vec![
        TokenDistributions {
            category: Category::Bubbles,
            percentage: 0.18,
            amount: 4212,
        },
        TokenDistributions {
            category: Category::Shrimp,
            percentage: 0.18,
            amount: 4212,
        },
        TokenDistributions {
            category: Category::Fish,
            percentage: 5.0,
            amount: 800,
        },
        TokenDistributions {
            category: Category::Dolphin,
            percentage: 12.75,
            amount: 300,
        },
        TokenDistributions {
            category: Category::Shark,
            percentage: 20.0,
            amount: 50,
        },
        TokenDistributions {
            category: Category::Whale,
            percentage: 25.5,
            amount: 15,
        },
    ]
}
