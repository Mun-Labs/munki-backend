use serde::{Deserialize, Serialize};

use crate::token::TokenOverview;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TokenSearchResult {
    #[serde(rename = "token")]
    Token { result: Vec<TokenSearchItem> },
    #[serde(rename = "market")]
    Market {},
}

impl From<TokenSearchResult> for Vec<TokenOverview> {
    fn from(value: TokenSearchResult) -> Self {
        match value {
            TokenSearchResult::Token { result } => {
                result.into_iter().map(|item| item.into()).collect()
            }
            TokenSearchResult::Market {} => {
                vec![]
            }
        }
    }
}

impl From<TokenSearchItem> for TokenOverview {
    fn from(value: TokenSearchItem) -> Self {
        Self {
            address: value.address,
            name: value.name,
            symbol: value.symbol,
            logo_uri: Some(value.logo_uri),
            marketcap: Some(value.market_cap),
            price: Some(value.price),
            //volume24h: Some(value.volume_24h_usd),
            price_change24h_percent: Some(value.price_change_24h_percent),
            decimals: value.decimals as u64,
            extensions: None,
            liquidity: Some(value.liquidity),
            history24h_price: Some(value.price_change_24h_percent),
            total_supply: Some(value.supply),
            holder: None,
            website_url: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct TokenSearchItem {
    pub name: String,
    pub symbol: String,
    pub address: String,
    pub decimals: u8,
    pub fdv: f64,
    pub market_cap: f64,
    pub liquidity: f64,
    pub volume_24h_change_percent: f64,
    pub price: f64,
    pub price_change_24h_percent: f64,
    pub buy_24h: u64,
    pub buy_24h_change_percent: f64,
    pub sell_24h: u64,
    pub sell_24h_change_percent: f64,
    pub trade_24h: u64,
    pub trade_24h_change_percent: f64,
    pub unique_wallet_24h: u64,
    pub unique_view_24h_change_percent: f64,
    pub last_trade_human_time: String,
    pub last_trade_unix_time: u64,
    pub creation_time: String,
    pub volume_24h_usd: f64,
    pub logo_uri: String,
    pub supply: f64,
    pub updated_time: i64,
}

#[cfg(test)]
mod test {
    use crate::thirdparty::{token_search::TokenSearchResult, BirdEyeResponse, ItemsResponse};

    #[tokio::test]
    async fn test_search_token_from_birdeye() {
        let a = TokenSearchResult::Token { result: vec![] };
        println!("{}", serde_json::to_string(&a).unwrap());
        let json_string = r#"
{"data":{"items":[{"type":"token","result":[{"name":"AI Rig Complex","symbol":"arc","address":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","network":"solana","decimals":6,"logo_uri":"https://ipfs.io/ipfs/QmPDJuEobBcLZihjFCvkWA8c1FiW7UzM2ctFdiffSLxf1d","verified":true,"fdv":49652814.648762316,"market_cap":49652814.648762316,"liquidity":5349418.8166030925,"price":0.04965305905722615,"price_change_24h_percent":6.498295518641598,"sell_24h":16492,"sell_24h_change_percent":-13.149718257938806,"buy_24h":17776,"buy_24h_change_percent":-7.387725330832552,"unique_wallet_24h":1700,"unique_wallet_24h_change_percent":-15.841584158415841,"trade_24h":34268,"trade_24h_change_percent":-10.253254065945578,"volume_24h_change_percent":-24.46269489171233,"volume_24h_usd":3433793.3958887877,"last_trade_unix_time":1743327461,"last_trade_human_time":"2025-03-30T09:37:41","supply":999995322.111251,"updated_time":1743327471}]},{"type":"market","result":[{"name":"SOL-arc","address":"J3b6dvheS2Y1cbMtVz5TCWXNegSjJDbUKxdUVDPoqmS7","network":"solana","liquidity":3036455.687514504,"unique_wallet_24h":929,"unique_wallet_24h_change_percent":-7.654075546719683,"trade_24h":7049,"trade_24h_change_percent":0.7143877696813831,"volume_24h_usd":2713709.0802143374,"last_trade_unix_time":1741859516,"last_trade_human_time":"2025-03-13T09:51:56.000Z","source":"Raydium","base_mint":"So11111111111111111111111111111111111111112","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":12190,"amout_quote":23618043,"creation_time":"2024-12-10T21:14:57.736Z"},{"name":"SOL-arc","address":"57mP5WoNrg3uiGFUdoeYr2CPUZak1L2ZgFtyFwoT7K6G","network":"solana","liquidity":424224.8825966626,"unique_wallet_24h":1384,"unique_wallet_24h_change_percent":3.052866716306776,"trade_24h":10807,"trade_24h_change_percent":-3.52615604356365,"volume_24h_usd":1346077.8880586333,"last_trade_unix_time":1741859484,"last_trade_human_time":"2025-03-13T09:51:24.000Z","source":"Orca","base_mint":"So11111111111111111111111111111111111111112","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":227.361913427,"amout_quote":6159208.166541,"creation_time":"2025-01-03T04:04:19.834Z"},{"name":"arc-USDC","address":"8fZcY7VhFVEKQrk9nJB1VUKEBxUPSxZiKK6UCz73xAxd","network":"solana","liquidity":86490.63095675928,"unique_wallet_24h":972,"unique_wallet_24h_change_percent":-25.517241379310345,"trade_24h":6319,"trade_24h_change_percent":-26.050321825629023,"volume_24h_usd":1015554.7809731979,"last_trade_unix_time":1741859518,"last_trade_human_time":"2025-03-13T09:51:58.000Z","source":"Phoenix","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount_base":482794.6,"amout_quote":55484.497703,"creation_time":"2025-01-06T11:26:21.067Z"},{"name":"arc-USDC","address":"uXYCkjhppZFxMz44yq1Mz9LPR2iLWEwF5MgXNGDgT9u","network":"solana","liquidity":254443.07843702246,"unique_wallet_24h":240,"unique_wallet_24h_change_percent":-18.367346938775512,"trade_24h":4089,"trade_24h_change_percent":-22.55681818181818,"volume_24h_usd":870920.9065216068,"last_trade_unix_time":1741859500,"last_trade_human_time":"2025-03-13T09:51:40.000Z","source":"Openbook V2","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount_base":1957908,"amout_quote":128604.192307,"creation_time":"2025-02-26T03:22:35.686Z"},{"name":"arc-USDC","address":"8sXVpYTP137uQuqtwkFi1H1bGEm3uJ5P5pYpNFr2nBi6","network":"solana","liquidity":357226.8194912168,"unique_wallet_24h":231,"unique_wallet_24h_change_percent":-13.48314606741573,"trade_24h":995,"trade_24h_change_percent":-35.76500968366688,"volume_24h_usd":386084.8252287802,"last_trade_unix_time":1741859367,"last_trade_human_time":"2025-03-13T09:49:27.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount_base":2583864.082522,"amout_quote":191158.565348,"creation_time":"2024-12-18T19:31:41.474Z"},{"name":"arc-ai16z","address":"A1mq259YfZWW9Q9R9nEXEvE3hj9sHdjcZRbDga18Mfv8","network":"solana","liquidity":135288.50046316502,"unique_wallet_24h":109,"unique_wallet_24h_change_percent":-3.5398230088495577,"trade_24h":576,"trade_24h_change_percent":-6.946688206785137,"volume_24h_usd":133150.44288435785,"last_trade_unix_time":1741859365,"last_trade_human_time":"2025-03-13T09:49:25.000Z","source":"Raydium Clamm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"HeLp6NuQkmYB4pYWo2zYs22mESHXPQYzXbB8n4V98jwC","amount_base":1323899.585841,"amout_quote":313213.175779811,"creation_time":"2025-01-20T21:45:24.288Z"},{"name":"arc-Anon","address":"7UMb4EMDVpvujHA5ZZiZbe4uCUEuAWSu4aq84Sv53sPH","network":"solana","liquidity":51724.05348777996,"unique_wallet_24h":146,"unique_wallet_24h_change_percent":-12.574850299401197,"trade_24h":835,"trade_24h_change_percent":2.4539877300613497,"volume_24h_usd":101649.25567720387,"last_trade_unix_time":1741859377,"last_trade_human_time":"2025-03-13T09:49:37.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"9McvH6w97oewLmPxqQEoHUAv3u5iYMyQ9AeZZhguYf1T","amount_base":725806.17569,"amout_quote":1022.340735428,"creation_time":"2025-02-03T20:52:31.239Z"},{"name":"arc-USDC","address":"FAijmttYim75twHhCKyNCH7sEU5QQkiqD3mcg6jF95Eb","network":"solana","liquidity":470825.49139635,"unique_wallet_24h":84,"unique_wallet_24h_change_percent":-38.23529411764706,"trade_24h":296,"trade_24h_change_percent":-30.8411214953271,"volume_24h_usd":98398.35402459183,"last_trade_unix_time":1741859524,"last_trade_human_time":"2025-03-13T09:52:04.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount_base":947399.483347,"amout_quote":409969.040354,"creation_time":"2024-12-10T22:49:02.267Z"},{"name":"arc-USDC","address":"DW2QC5ychRKmA3YpY6eWetxt1YdMK8hir3vNjNjRW6i9","network":"solana","liquidity":30092.960311959534,"unique_wallet_24h":308,"unique_wallet_24h_change_percent":689.7435897435898,"trade_24h":681,"trade_24h_change_percent":333.7579617834395,"volume_24h_usd":79495.7125476178,"last_trade_unix_time":1741859416,"last_trade_human_time":"2025-03-13T09:50:16.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v","amount_base":467947.341996,"amout_quote":34.438876,"creation_time":"2025-01-17T16:28:16.342Z"},{"name":"arc-SOL","address":"4qLjN2jnhCqS741hkkSP1DnznLNe5wA5buJRSUtbSWk7","network":"solana","liquidity":89702.63414493817,"unique_wallet_24h":192,"unique_wallet_24h_change_percent":-15.789473684210526,"trade_24h":485,"trade_24h_change_percent":-19.834710743801654,"volume_24h_usd":66677.72318626422,"last_trade_unix_time":1741859369,"last_trade_human_time":"2025-03-13T09:49:29.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"So11111111111111111111111111111111111111112","amount_base":901882.871855,"amout_quote":254.516868916,"creation_time":"2024-12-10T21:56:13.235Z"},{"name":"arc-swarms","address":"AxPKExaKyTi6qtshR9VE58BJcM5At5gVtpKU7zM9xLoh","network":"solana","liquidity":87253.38575435757,"unique_wallet_24h":238,"unique_wallet_24h_change_percent":-7.03125,"trade_24h":850,"trade_24h_change_percent":-20.037629350893695,"volume_24h_usd":65350.52408629982,"last_trade_unix_time":1741859372,"last_trade_human_time":"2025-03-13T09:49:32.000Z","source":"Raydium Clamm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"74SBV4zDXxTRgv1pEMoECskKBkZHc2yGPnc7GYVepump","amount_base":805603.204629,"amout_quote":923151.75477,"creation_time":"2024-12-26T02:27:59.284Z"},{"name":"arc-SOL","address":"92J3X46dbnoPs25Vc3fr5m1xt9GVrQddqiPD9uzXQHH","network":"solana","liquidity":102087.40854564948,"unique_wallet_24h":315,"unique_wallet_24h_change_percent":0.9615384615384616,"trade_24h":1019,"trade_24h_change_percent":5.159958720330237,"volume_24h_usd":61714.585982800934,"last_trade_unix_time":1741859367,"last_trade_human_time":"2025-03-13T09:49:27.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"So11111111111111111111111111111111111111112","amount_base":1243265.156339,"amout_quote":178.009974366,"creation_time":"2025-01-12T15:29:08.568Z"},{"name":"listen-arc","address":"D7LThB5wBANDotQcKDSSspaA5CW9Tov6BfNvf5HPVHJS","network":"solana","liquidity":50717.95824378346,"unique_wallet_24h":223,"unique_wallet_24h_change_percent":28.901734104046245,"trade_24h":650,"trade_24h_change_percent":27.20156555772994,"volume_24h_usd":58745.89268412541,"last_trade_unix_time":1741857032,"last_trade_human_time":"2025-03-13T09:10:32.000Z","source":"Meteora Dlmm","base_mint":"Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":6115304.191819,"amout_quote":527429.208169,"creation_time":"2025-01-31T09:14:15.488Z"},{"name":"AskJ-arc","address":"HMn6o5rM2NGgNAuYeH7JxRKfQNoQgsiqyxVUYwVpuZcV","network":"solana","liquidity":749085.6409741433,"unique_wallet_24h":56,"unique_wallet_24h_change_percent":16.666666666666664,"trade_24h":170,"trade_24h_change_percent":-10.99476439790576,"volume_24h_usd":40427.480483770574,"last_trade_unix_time":1741859370,"last_trade_human_time":"2025-03-13T09:49:30.000Z","source":"Meteora Dlmm","base_mint":"DgkKrQ1ErdRNjT2yTcAdEBa92JjFx75yxi4owArQarc","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":94276327.93089019,"amout_quote":6591205.610447,"creation_time":"2025-02-13T13:14:15.352Z"},{"name":"TANK-arc","address":"E7xyDDmS9LoqzRAUFwBDDPspMdr3jL1VhCTnu7Rz3McD","network":"solana","liquidity":32577.411246389187,"unique_wallet_24h":124,"unique_wallet_24h_change_percent":3.3333333333333335,"trade_24h":833,"trade_24h_change_percent":-8.158765159867695,"volume_24h_usd":36210.36145401003,"last_trade_unix_time":1741859202,"last_trade_human_time":"2025-03-13T09:46:42.000Z","source":"Meteora Dlmm","base_mint":"GAMwtMB6onAvBNBQJCJFuxoaqfPH8uCQ2dewNMVVpump","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":8291322.903712,"amout_quote":188160.004285,"creation_time":"2025-03-10T18:02:56.637Z"},{"name":"GRPH-arc","address":"F9Mbi6XcUqGd1cLNUkpVC4LjMcJsnYwyapDHaH4tDYek","network":"solana","liquidity":39385.72344941084,"unique_wallet_24h":80,"unique_wallet_24h_change_percent":-10.112359550561797,"trade_24h":517,"trade_24h_change_percent":-25.289017341040466,"volume_24h_usd":20775.177553920843,"last_trade_unix_time":1741859382,"last_trade_human_time":"2025-03-13T09:49:42.000Z","source":"Meteora Dlmm","base_mint":"9doRRAik5gvhbEwjbZDbZR6GxXSAfdoomyJR57xKpump","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":8863541.321124,"amout_quote":322770.269751,"creation_time":"2025-01-15T15:51:30.306Z"},{"name":"listen-arc","address":"7hRswsmJ9Dz2jRWKbQMqKZmYLTiHNDLkVRZdmvoYyUSw","network":"solana","liquidity":16191.287593565772,"unique_wallet_24h":148,"unique_wallet_24h_change_percent":4.225352112676056,"trade_24h":774,"trade_24h_change_percent":41.49908592321755,"volume_24h_usd":19387.875069173897,"last_trade_unix_time":1741859449,"last_trade_human_time":"2025-03-13T09:50:49.000Z","source":"Meteora Dlmm","base_mint":"Cn5Ne1vmR9ctMGY9z5NC71A3NYFvopjXNyxYtfVYpump","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":1953560.550475,"amout_quote":168321.772114,"creation_time":"2025-01-20T18:30:47.298Z"},{"name":"arc-SNAI","address":"DMxbt8VuWDDi23PEFCP43CE31ot3ARvj3Ft1crfeUf26","network":"solana","liquidity":21974.935483413396,"unique_wallet_24h":252,"unique_wallet_24h_change_percent":-37.62376237623762,"trade_24h":790,"trade_24h_change_percent":-39.74065598779558,"volume_24h_usd":14781.035695714409,"last_trade_unix_time":1741859446,"last_trade_human_time":"2025-03-13T09:50:46.000Z","source":"Orca","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"Hjw6bEcHtbHGpQr8onG3izfJY5DJiWdt7uk2BfdSpump","amount_base":318459.394402,"amout_quote":67860.622629,"creation_time":"2025-02-14T01:16:47.771Z"},{"name":"arc-GRIFT","address":"HLqxBGCYXFdosvedtx3V4g6H4NS8tUzx4aHGYu5q9rSU","network":"solana","liquidity":5252.377502948364,"unique_wallet_24h":130,"unique_wallet_24h_change_percent":-19.25465838509317,"trade_24h":429,"trade_24h_change_percent":-19.056603773584907,"volume_24h_usd":13399.73339048969,"last_trade_unix_time":1741859370,"last_trade_human_time":"2025-03-13T09:49:30.000Z","source":"Meteora Dlmm","base_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","quote_mint":"GekTNfm84QfyP2GdAHZ5AgACBRd69aNmgA5FDhZupump","amount_base":56301.479275,"amout_quote":254166.855643,"creation_time":"2024-12-21T17:06:45.377Z"},{"name":"SNAI-arc","address":"3ARYTW684hDvnWzzXUEX8gcP5JxYy6AuTouyt1bXsnGJ","network":"solana","liquidity":19430.530592045096,"unique_wallet_24h":115,"unique_wallet_24h_change_percent":-37.83783783783784,"trade_24h":297,"trade_24h_change_percent":-39.878542510121456,"volume_24h_usd":12207.674191690016,"last_trade_unix_time":1741859373,"last_trade_human_time":"2025-03-13T09:49:33.000Z","source":"Meteora Dlmm","base_mint":"Hjw6bEcHtbHGpQr8onG3izfJY5DJiWdt7uk2BfdSpump","quote_mint":"61V8vBaqAGMpgDQi4JcAwo1dmBGHsyhzodcPqnEVpump","amount_base":78003.710707,"amout_quote":275376.342285,"creation_time":"2024-12-31T09:20:28.618Z"}]}]},"success":true}
                 "#;
        let result: BirdEyeResponse<ItemsResponse<TokenSearchResult>> =
            serde_json::from_str(json_string).unwrap();
        println!("{:?}", result);
    }
}
