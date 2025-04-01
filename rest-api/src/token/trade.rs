use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketData {
    pub address: String,
    pub liquidity: f64,
    pub price: f64,
    pub total_supply: f64,
    pub circulating_supply: f64,
    pub fdv: f64,
    pub market_cap: f64,
}
//{
//    "address": "So11111111111111111111111111111111111111112",
//    "liquidity": 7889932801.542948,
//    "price": 128.00523659221298,
//    "total_supply": 583981435.5103763,
//    "circulating_supply": 583981433.2307372,
//    "fdv": 74790067044.55531,
//    "market_cap": 74790067336.507
//  },
