use anyhow::Error;
use crate::price::{PriceSdk, TimeFilters};
use crate::token::{Trending, TokenSdk, TokenMetadata};
use chrono::{Duration, Timelike, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::info;

#[allow(dead_code)]
#[derive(Debug, Default, Clone)]
pub struct BirdEyeClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl BirdEyeClient {
    pub fn new(base_url: &str, api_key: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BirdEyeResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Items {
    pub value: f64,
    pub unix_time: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceHistory {
    pub items: Vec<Items>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenData {
    pub price: f64,
    pub update_unix_time: i64,
    pub update_human_time: String,
    #[serde(rename = "volumeUSD")]
    pub volume_usd: f64,
    pub volume_change_percent: f64,
    pub price_change_percent: f64,
}
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendingResponse {
    pub update_unix_time: i64,
    pub update_time: String,
    pub tokens: Vec<Trending>,
}
impl TokenSdk for BirdEyeClient {
     async fn get_trending(&self, offset: i32, limit: i32) -> Result<Vec<Trending>, anyhow::Error> {
        let url = format!("{}/defi/token_trending", self.base_url);
        let resp = self
            .client
            .get(url)
            .query(&[
                ("sort_by", "volume24hUSD"),
                ("sort_type", "desc"),
                ("offset", offset.to_string().as_str()),
                ("limit", limit.to_string().as_str()),
            ])
            .header("X-API-KEY", &self.api_key)
            .header("accept", "application/json")
            .header("x-chain", "solana")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                resp.status()
            ));
        }

         let resp = resp.text().await?;
         println!("{}", resp);

         let body = serde_json::from_str::<BirdEyeResponse<TrendingResponse>>(&resp)?;
         info!("fetch price history: {body:?}");
         Ok(body.data.tokens)
    }

    async fn overview(&self, address: &str) -> Result<TokenMetadata, Error> {
        let url = format!("{}/defi/token_overview", self.base_url);
        let resp = self
            .client
            .get(url)
            .query(&[("address", address)])
            .header("X-API-KEY", &self.api_key)
            .header("accept", "application/json")
            .header("x-chain", "solana")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
            "Request failed with status: {}",
            resp.status()
        ));
        }

        let resp = resp.json::<BirdEyeResponse<TokenMetadata>>().await?.data;
        Ok(resp)
    }
}

impl PriceSdk for BirdEyeClient {
    async fn get_price(&self, token: &str) -> Result<TokenData, anyhow::Error> {
        let url = format!("{}/defi/price_volume/single", self.base_url);
        info!("price endpoint: {url}");
        let resp = self
            .client
            .get(url)
            .query(&[("address", token)])
            .header("X-API-KEY", &self.api_key)
            .header("accept", "application/json")
            .header("x-chain", "solana")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                resp.status()
            ));
        }
        let resp = resp.json::<BirdEyeResponse<TokenData>>().await?.data;
        Ok(resp)
    }

    async fn get_price_by_time_filter(
        &self,
        token: &str,
        filter: TimeFilters,
    ) -> Result<PriceHistory, anyhow::Error> {
        let now = Utc::now();

        // Beginning of today (midnight UTC)
        let start_of_today = now
            .with_hour(0)
            .and_then(|t| t.with_minute(0))
            .and_then(|t| t.with_second(0))
            .and_then(|t| t.with_nanosecond(0))
            .unwrap_or(now);
        let today_unix = start_of_today.timestamp().to_string();

        // Beginning of last week (7 days ago at midnight UTC)
        let last_week = start_of_today - Duration::days(7);
        let last_week_unix = last_week.timestamp().to_string();
        let url = format!("{}/defi/history_price", self.base_url);
        let resp = self
            .client
            .get(url)
            .query(&[
                ("address", token),
                ("address_type", "token"),
                ("type", filter.as_query_param()),
                ("time_from", &last_week_unix),
                ("time_to", &today_unix),
            ])
            .header("X-API-KEY", &self.api_key)
            .header("accept", "application/json")
            .header("x-chain", "solana")
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                resp.status()
            ));
        }

        let resp = resp.json::<BirdEyeResponse<PriceHistory>>().await?.data;
        info!("fetch price history: {resp:?}");
        Ok(resp)
    }
}

#[cfg(test)]
mod test {
    use crate::price::PriceSdk;
    use crate::thirdparty::{BirdEyeResponse, TokenData, TrendingResponse};

    #[tokio::test]
    async fn test_get_price_should_return_token_price() {
        let client = super::BirdEyeClient::new("https://api.birdeye.com", "api_key");
        let token_address = "0x1234567890abcdef";
        let token_price = client.get_price(token_address).await.unwrap();
        assert_eq!(token_price.price, 0.0);
    }

    #[tokio::test]
    async fn test_get_price_history() {
        use std::collections::HashMap;
        let resp = reqwest::get("https://httpbin.org/ip")
            .await
            .unwrap()
            .json::<HashMap<String, String>>()
            .await
            .unwrap();
        println!("{resp:#?}");
    }
    #[test]
    fn deserialize_trending() {
        let resp_string = r#"{
  "data": {
    "updateUnixTime": 1742128301,
    "updateTime": "2025-03-16T12:31:41",
    "tokens": [
      {
        "address": "Gh6cBL11RRwVYHUyoGFXdYJXhWW1HETnPriNZN71pump",
        "decimals": 6,
        "liquidity": 23374.936617022333,
        "logoURI": "https://ipfs.io/ipfs/QmYm5zetTLHMKpLKTjiEyLHRMWBZbA1ZYLEmWKTqPbogQE",
        "name": "Finna",
        "symbol": "Finna",
        "volume24hUSD": 86.09042125467224,
        "volume24hChangePercent": -79.4667735403863,
        "fdv": 62794.24442184402,
        "marketcap": 62794.24442184402,
        "rank": 931,
        "price": 0.00006322636037191058,
        "price24hChangePercent": -3.059299175164433
      },
      {
        "address": "SLAMG93vQPmhfDCbT5NpDg6f8E7VNDs9FxBxhvdUwcX",
        "decimals": 9,
        "liquidity": 29875.692211517307,
        "logoURI": "https://slotana.io/token/coin.png",
        "name": "Slotana",
        "symbol": "SLA",
        "volume24hUSD": 255.72584687083875,
        "volume24hChangePercent": -67.70354498176744,
        "fdv": 386746.28036267095,
        "marketcap": 386746.28036267095,
        "rank": 981,
        "price": 0.07161594366731956,
        "price24hChangePercent": -4.178002850143017
      },
      {
        "address": "FmQ7v2QUqXVVtAXkngBh3Mwx7s3mKT55nQ5Z673dURYS",
        "decimals": 9,
        "liquidity": 33425.56733340546,
        "logoURI": "https://www.arweave.net/3VPYgJz-wlRAm1H5_4zrsAckyz55qa5ILyk3Uq6l4Ms?ext=png",
        "name": "Dark Protocol",
        "symbol": "DARK",
        "volume24hUSD": 317.0664209655937,
        "volume24hChangePercent": -71.77256273101001,
        "fdv": 884201.7224876794,
        "marketcap": 884201.7224876794,
        "rank": 876,
        "price": 0.04212489172114791,
        "price24hChangePercent": 0.6141745946522192
      },
      {
        "address": "4AG81mGbaiMJRfCPgz1z5RCiN5JTH5TDHNWNbD4Tpump",
        "decimals": 6,
        "liquidity": 19780.368510307897,
        "logoURI": "https://ipfs.io/ipfs/Qmd2areGaVFg4Fhozat73YqsSB1pUUetDWLGFZGdkCFEE4",
        "name": "Nietzsche AGI",
        "symbol": "POWER",
        "volume24hUSD": 349.2315228399822,
        "volume24hChangePercent": -6.675169501155478,
        "fdv": 15895.892106931058,
        "marketcap": 15895.892106931058,
        "rank": 584,
        "price": 0.00001591643212100099,
        "price24hChangePercent": -2.2062436241376226
      },
      {
        "address": "39Mzpdw7NDGiXmZZGWiCdR6Nzoc7muWuYkPsVDV4pump",
        "decimals": 6,
        "liquidity": 35379.117329468565,
        "logoURI": "https://ipfs.io/ipfs/QmdQaV6E16YUmKzBMnPfa7woVyA9585oE4CKmP5ZQJuH9c",
        "name": "MEME",
        "symbol": "MPX6900",
        "volume24hUSD": 378.1963109346867,
        "volume24hChangePercent": -76.91215463661868,
        "fdv": 53290.505012784735,
        "marketcap": 53290.505012784735,
        "rank": 787,
        "price": 0.00005330212252473534,
        "price24hChangePercent": -3.423318640469346
      },
      {
        "address": "CotWkXoBD3edLb6opEGHV9tb3pyKmeoWBLwdMJ8ZDimW",
        "decimals": 8,
        "liquidity": 72025.11151184855,
        "logoURI": "https://i.ibb.co/tJX6pWd/wb-PG1-Ny7-400x400.jpg",
        "name": "AlphaKEK.AI (Wormhole)",
        "symbol": "AIKEK",
        "volume24hUSD": 383.15277693657833,
        "volume24hChangePercent": -85.26939038948844,
        "fdv": 278703.23150564777,
        "marketcap": 278703.23150564777,
        "rank": 820,
        "price": 0.014729524994832365,
        "price24hChangePercent": -3.4856705676547994
      },
      {
        "address": "Eb1w71iiSiud9qkEKbKL2UMfmyR5wRMWqpthco3wPpC5",
        "decimals": 9,
        "liquidity": 45768.38004068219,
        "logoURI": "https://ipfs.io/ipfs/QmQUHTbyEzm2F2arZTRGNufhEuezq8ZXaef76fuiqTEeTn",
        "name": "LIBERTYCOIN",
        "symbol": "LBTC",
        "volume24hUSD": 392.0088196703995,
        "volume24hChangePercent": -40.35484270414164,
        "fdv": 149803.31664550502,
        "marketcap": 149803.31664550502,
        "rank": 817,
        "price": 0.00014981831121561547,
        "price24hChangePercent": -1.494795960060036
      },
      {
        "address": "6jEp4iQeLYJuxF4vUeiESsVJH1L16HhjAnuyQrz7pump",
        "decimals": 6,
        "liquidity": 45260.68023635108,
        "logoURI": "https://ipfs.io/ipfs/QmR8QKZ94qTPpXvKZkFkC3ivQxAhzhELP74YSF6A8wDBbH",
        "name": "Fuck Around = Find Out",
        "symbol": "FA=FO",
        "volume24hUSD": 454.24126964974107,
        "volume24hChangePercent": -5.339191243220265,
        "fdv": 70229.15963737122,
        "marketcap": 70229.15963737122,
        "rank": 563,
        "price": 0.00007026531319552128,
        "price24hChangePercent": -2.0200668741863423
      },
      {
        "address": "H1G6sZ1WDoMmMCFqBKAbg9gkQPCo1sKQtaJWz9dHmqZr",
        "decimals": 9,
        "liquidity": 46500.26691194457,
        "logoURI": "https://img.fotofolio.xyz/?url=https%3A%2F%2Fraw.githubusercontent.com%2FSperlo64%2FSHIBONK%2Fmain%2Fbonklogo2.png",
        "name": "SHIBONK",
        "symbol": "SBONK",
        "volume24hUSD": 537.8689615995117,
        "volume24hChangePercent": -41.104661056704465,
        "fdv": 109969.86987087216,
        "marketcap": 109969.86987087216,
        "rank": 797,
        "price": 0.629113621192403,
        "price24hChangePercent": -4.0746319709796825
      },
      {
        "address": "DvyxUPDrDSLLziu5YsTkyirRDTncqkrWTsUdNCuipump",
        "decimals": 6,
        "liquidity": 71802.45486958034,
        "logoURI": "https://ipfs.io/ipfs/QmdvmVqvfEh9GeU1swPR86UThvCKHj4Y6yFHXGH1wMEtdB",
        "name": "ai69x",
        "symbol": "ai69x",
        "volume24hUSD": 691.9225530053523,
        "volume24hChangePercent": 182.89551213374048,
        "fdv": 77787.02593193129,
        "marketcap": 77787.02593193129,
        "rank": 782,
        "price": 0.00007778899999725797,
        "price24hChangePercent": -4.479798085568931
      },
      {
        "address": "6gx6Ph2ek73kF6EWDrG4GQ54pcLJB6CYpATuRyxKXumo",
        "decimals": 9,
        "liquidity": 30428.80273582539,
        "logoURI": "https://gateway.irys.xyz/lUbE3hfwk7lOAeHT5ct_u2TfhBQMe7q9wFG4G9baElI",
        "name": "FillmorePHX",
        "symbol": "fPHX",
        "volume24hUSD": 698.9612207783457,
        "volume24hChangePercent": -16.62297916523932,
        "fdv": 16864.255641494685,
        "marketcap": 16864.255641494685,
        "rank": 519,
        "price": 0.00001688301741507314,
        "price24hChangePercent": -1.569005692064072
      },
      {
        "address": "FAdDY5y9LWFBTY3pBHJwRkKGoeVNMjDjzrBJpiX8gfzE",
        "decimals": 6,
        "liquidity": 29221.687041891266,
        "logoURI": "https://imgur.fotofolio.xyz/?w=128&h=128&default=1&url=https%3A%2F%2Fimgur.fotofolio.xyz%2F%3Fw%3D128%26h%3D128%26default%3D1%26url%3Dhttps%253A%252F%252Fimgur.fotofolio.xyz%252F%253Fw%253D128%2526h%253D128%2526default%253D1%2526url%253Dhttps%25253A%25252F%25252Fi.imgur.com%25252FusYbxdx.jpeg",
        "name": "Faddy",
        "symbol": "Faddy",
        "volume24hUSD": 745.9197884682408,
        "volume24hChangePercent": -72.62167494567161,
        "fdv": 22703.135901143578,
        "marketcap": 22703.135901143578,
        "rank": 598,
        "price": 0.00002594183376884227,
        "price24hChangePercent": -4.042008056006601
      },
      {
        "address": "G9tt98aYSznRk7jWsfuz9FnTdokxS6Brohdo9hSmjTRB",
        "decimals": 9,
        "liquidity": 37602.013419766256,
        "logoURI": "https://i.ibb.co/qrS60ks/puff-logo.png",
        "name": "PUFF",
        "symbol": "PUFF",
        "volume24hUSD": 758.2001302704019,
        "volume24hChangePercent": -47.45026524327446,
        "fdv": 149425.35673381857,
        "marketcap": 149425.35673381857,
        "rank": 574,
        "price": 0.0010841741937362376,
        "price24hChangePercent": -2.8723888529477284
      },
      {
        "address": "CH74tuRLTYcxG7qNJCsV9rghfLXJCQJbsu7i52a8F1Gn",
        "decimals": 9,
        "liquidity": 1929.7460912778765,
        "logoURI": "https://raw.githubusercontent.com/solana-labs/token-list/main/assets/mainnet/CH74tuRLTYcxG7qNJCsV9rghfLXJCQJbsu7i52a8F1Gn/logo.png",
        "name": "Soldex",
        "symbol": "SOLX",
        "volume24hUSD": 808.7363181870184,
        "volume24hChangePercent": -68.99767588743926,
        "fdv": 409598.9581036476,
        "marketcap": 409598.9581036476,
        "rank": 831,
        "price": 0.00014886444245157016,
        "price24hChangePercent": 7.948824756559122
      },
      {
        "address": "LMFzmYL6y1FX8HsEmZ6yNKNzercBmtmpg2ZoLwuUboU",
        "decimals": 9,
        "liquidity": 25612.946204472337,
        "logoURI": "https://www.lamas.co/resource/lmf_token.png",
        "name": "Lamas Finance",
        "symbol": "LMF",
        "volume24hUSD": 811.1757301037115,
        "volume24hChangePercent": -34.363763268722344,
        "fdv": 81301.710816835,
        "marketcap": 81301.710816835,
        "rank": 671,
        "price": 0.011034716664792476,
        "price24hChangePercent": 2.112425613273158
      },
      {
        "address": "3F4aTvvQBdtWj672kNYRenMtMG4jwQE63redg2cN4tCm",
        "decimals": 8,
        "liquidity": 57136.633191963236,
        "logoURI": null,
        "name": "Skol! (Wormhole)",
        "symbol": "SKOL",
        "volume24hUSD": 826.1130169782382,
        "volume24hChangePercent": -79.04727148215012,
        "fdv": 37816.51040720516,
        "marketcap": 37816.51040720516,
        "rank": 1000,
        "price": 0.0971818163896763,
        "price24hChangePercent": -0.9254412274530692
      },
      {
        "address": "bebsu58wGeMew3S5c2m2ZB5zvj8tcZ1itwSxcJJYHnA",
        "decimals": 6,
        "liquidity": 9543.20057483496,
        "logoURI": "https://v2.shdwdrive.com/EkDtEc2RhfkA6ViDtVt11z7uKqPAi9xwYuxUaKj6bFXj/Untitled_design_21.png",
        "name": "bebs",
        "symbol": "bebs",
        "volume24hUSD": 841.8986063511734,
        "volume24hChangePercent": -18.192645786815355,
        "fdv": 6249548168920.177,
        "marketcap": 6249548168920.177,
        "rank": 977,
        "price": 1.2499096337840354,
        "price24hChangePercent": -2.0199441213623155
      },
      {
        "address": "4MPD7cGs9746SkZnqhfRigNmpNq17EgTt76YHKC6GEbN",
        "decimals": 9,
        "liquidity": 12473.999765085684,
        "logoURI": "https://bafkreid2enfyrfdewdhkdm5gcelpwlzqbknbkgvxhntkst6zmxp77yvy64.ipfs.nftstorage.link",
        "name": "Brolana",
        "symbol": "BROS",
        "volume24hUSD": 892.8322099057389,
        "volume24hChangePercent": 252.698111419899,
        "fdv": 12266.30179210358,
        "marketcap": 12266.30179210358,
        "rank": 811,
        "price": 0.000029234716694118398,
        "price24hChangePercent": 17.606508237863313
      },
      {
        "address": "82XZhairZh5jF1rBQsxGYXcri99cSS4xKpf7fV7Dpump",
        "decimals": 6,
        "liquidity": 13765.240452246484,
        "logoURI": "https://ipfs.io/ipfs/QmcTQTDYP5Lrzduaecfr6VawdxtfDij2qusBkcAb5YYMqg",
        "name": "Bugs Bunny",
        "symbol": "BugsBunny",
        "volume24hUSD": 897.5595434851738,
        "volume24hChangePercent": 8.106292818909804,
        "fdv": 8200.539512409778,
        "marketcap": 8200.539512409778,
        "rank": 856,
        "price": 0.000008203194481509079,
        "price24hChangePercent": -7.395061610950156
      },
      {
        "address": "CPcf58MNikQw2G23kTVWQevRDeFDpdxMH7KkR7Lhpump",
        "decimals": 6,
        "liquidity": 68496.15368927967,
        "logoURI": "https://ipfs.io/ipfs/QmamiH2SwsGjKXyim4wU7MokBLkBycSPKh1fiwYmULtTM2",
        "name": "DOBBY",
        "symbol": "DOBBY",
        "volume24hUSD": 933.2939222244133,
        "volume24hChangePercent": -68.12090110496985,
        "fdv": 67098.79938070885,
        "marketcap": 67098.79938070885,
        "rank": 937,
        "price": 0.00006711585725099871,
        "price24hChangePercent": -1.7411434177497898
      }
    ],
    "total": 1000
  },
  "success": true
}"#;
        let body = serde_json::from_str::<BirdEyeResponse<TrendingResponse>>(resp_string).unwrap();
        println!("Decoded response:\n{:#?}", body);
    }

    #[test]
    fn deserialize() {
        let resp_string = r#"{"success":true,"data":{"price":134.8625396815726,"updateUnixTime":1741526897,"updateHumanTime":"2025-03-09T13:28:17","volumeUSD":1726429451.421973,"volumeChangePercent":-37.9876049836133,"priceChangePercent":-2.2107631867842215}}"#;
        let body = serde_json::from_str::<BirdEyeResponse<TokenData>>(resp_string).unwrap();
        println!("Decoded response:\n{:#?}", body);
    }
}
