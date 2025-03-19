use axum::{extract::State, Json};
use helius::types::TokenTransfer;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use tracing::info;

use crate::app::{AppState, SOL_ADDRESS};
use crate::token;
use crate::token::TokenSdk;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnhancedTransaction {
    pub native_transfers: Option<Vec<helius::types::NativeTransfer>>,
    pub token_transfers: Option<Vec<TokenTransfer>>,
    pub slot: i64,
    pub signature: String,
    pub timestamp: i64,
}

#[derive(sqlx::FromRow, Debug)]
pub struct MarketMover {
    pub wallet_address: String,
    // additional fields if needed
}

// Helper function to extract unique wallet addresses from the payload
fn extract_wallets(transactions: &[EnhancedTransaction]) -> Vec<String> {
    let mut wallets = HashSet::new();
    for trans in transactions {
        if let Some(token_transfers) = trans.token_transfers.as_ref() {
            for transfer in token_transfers {
                if let Some(from) = transfer.user_accounts.from_user_account.as_ref() {
                    wallets.insert(from.clone());
                }

                if let Some(from) = transfer.user_accounts.to_user_account.as_ref() {
                    wallets.insert(from.clone());
                }
            }
        }
    }
    wallets.into_iter().collect()
}

// Helper function to load market movers whose wallet addresses appear in the payload
async fn load_wallets_by_list(
    pool: &PgPool,
    wallet_addresses: &[String],
) -> Result<Vec<MarketMover>, sqlx::Error> {
    // Using ANY($1) requires passing a slice
    sqlx::query_as::<_, MarketMover>(
        "SELECT wallet_address FROM market_mover WHERE wallet_address = ANY($1)",
    )
    .bind(wallet_addresses)
    .fetch_all(pool)
    .await
}

pub async fn webhook_handler(
    State(app): State<AppState>,
    Json(payload): Json<Vec<EnhancedTransaction>>,
) -> &'static str {
    // Extract distinct wallet addresses from payload token transfers.
    let wallet_list = extract_wallets(&payload);
    info!("Wallets from payload: {:?}", wallet_list);

    // Load only wallets that are present in the payload.
    let wallets = match load_wallets_by_list(&app.pool, &wallet_list).await {
        Ok(wallets) => wallets,
        Err(e) => {
            tracing::error!("Failed to load wallets: {:?}", e);
            return "Error loading wallets";
        }
    };
    info!("Wallets from payload: {:?}", wallets);

    // Process each transaction against the wallets retrieved from the database.
    let mut token_addresses = vec![];
    for transaction in payload {
        if let Some(transfers) = transaction.token_transfers {
            for transfer in transfers {
                let mut action = ("".to_string(), "".into());
                let from_user = transfer
                    .user_accounts
                    .from_user_account
                    .clone()
                    .unwrap_or_default();
                if wallets
                    .iter()
                    .find(|a| a.wallet_address == from_user)
                    .is_some()
                {
                    action = ("sell".to_string(), from_user);
                }

                let to_user = transfer
                    .user_accounts
                    .to_user_account
                    .clone()
                    .unwrap_or_default();
                if wallets
                    .iter()
                    .find(|a| a.wallet_address == to_user)
                    .is_some()
                {
                    action = ("buy".to_string(), to_user);
                }
                if action.0 == "".to_string() {
                    continue;
                }

                // Adjust field extraction as needed.
                let token_address = transfer.mint.clone();
                let amount = transfer.token_amount.as_f64().unwrap_or_default();
                let block_time = transaction.timestamp;
                let slot = transaction.slot;

                let (action, wallet) = action;
                if let Err(e) = upsert_transaction(
                    &app.pool,
                    &transaction.signature,
                    &token_address,
                    &wallet,
                    &action,
                    amount,
                    block_time,
                    slot,
                )
                .await
                {
                    tracing::error!("Failed to upsert transaction: {:?}", e);
                }

                if token_address != SOL_ADDRESS {
                    token_addresses.push(token_address);
                }
            }
        }
    }
    match token::token_by_address(&app.pool, token_addresses).await {
        Err(e) => {
            tracing::error!("Failed to fetch token data: {:?}", e);
        }
        Ok(missing) if missing.is_empty() => {
            info!("no token to fetch");
        }
        Ok(missing) => {
            // Token doesn't exist or data is stale, fetch from BirdEye
            match app.bird_eye_client.overview(missing).await {
                Ok(token_meta_list) if token_meta_list.is_empty() => {
                    info!("birdeye no tokens");
                }
                Ok(token_meta_list) => {
                    info!("Fetched token metadata: {:?}", token_meta_list);
                    // Map the token metadata list to a list of Trending records.
                    let trending: Vec<token::Trending> = token_meta_list
                        .into_iter()
                        .map(|tm| token::Trending {
                            address: tm.address,
                            decimals: tm.decimals,
                            logo_uri: Some(tm.logo_uri),
                            name: tm.name,
                            symbol: tm.symbol,
                            volume24h_usd: 0.0, // No volume data in overview
                            rank: 0,            // No rank data in overview
                            price: 0.0,
                        })
                        .collect();

                    if let Err(e) = crate::token::upsert_token_meta(&app.pool, &trending).await {
                        tracing::error!("Failed to upsert token metadata: {:?}", e);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch token metadata from BirdEye: {:?}", e);
                }
            }
        }
    }
    "Webhook received"
}

async fn upsert_transaction(
    pool: &PgPool,
    signature: &str,
    token_address: &str,
    wallet_address: &str,
    transaction_type: &str,
    amount: f64,
    block_time: i64,
    slot: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO market_movers_transaction
         (signature, token_address, wallet_address, transaction_type, amount, block_time, slot)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (signature) DO UPDATE
         SET token_address = EXCLUDED.token_address,
             wallet_address = EXCLUDED.wallet_address,
             transaction_type = EXCLUDED.transaction_type,
             amount = EXCLUDED.amount,
             block_time = EXCLUDED.block_time,
             slot = EXCLUDED.slot,
             additional = EXCLUDED.additional",
    )
    .bind(signature)
    .bind(token_address)
    .bind(wallet_address)
    .bind(transaction_type)
    .bind(amount)
    .bind(block_time)
    .bind(slot)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::webhook::EnhancedTransaction;

    #[test]
    fn should_deser_enhanced_trans() {
        let enhanced_trans = r#"[{"accountData":[{"account":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","nativeBalanceChange":-72938049280,"tokenBalanceChanges":[]},{"account":"NTYeYJ1wr4bpM5xo6zx5En44SvJFAd35zTxxNoERYqd","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"AAaTGaA3uVqikfVEwoSG7EwkCb4bBDsMEyueiVUS5CaU","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"autMW8SgBkVYeBgqYiTuJZnkvDZMVU2MHJh9Jh7CSQ2","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"D8TxfGwdu9MiNMoJmUoC9wQfNfNT7Lnm6DzifQHRTy6B","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"5DxD5ViWjvRZEkxQEaJHZw2sBsso6xoXx3wGFNKgXUzE","nativeBalanceChange":71860273440,"tokenBalanceChanges":[]},{"account":"25DTUAd1roBFoUQaxJQByL6Qy2cKQCBp4bK9sgfy9UiM","nativeBalanceChange":-2039280,"tokenBalanceChanges":[{"mint":"FdsNQE5EeCe57tbEYCRV1JwW5dzNCof7MUTaGWhmzYqu","rawTokenAmount":{"decimals":0,"tokenAmount":"-1"},"tokenAccount":"25DTUAd1roBFoUQaxJQByL6Qy2cKQCBp4bK9sgfy9UiM","userAccount":"1BWutmTvYPwDtmw9abTkS4Ssr8no61spGAvW1X6NDix"}]},{"account":"DTYuh7gAGGZg2okM7hdFfU1yMY9LUemCiPyD5Z5GCs6Z","nativeBalanceChange":2039280,"tokenBalanceChanges":[{"mint":"FdsNQE5EeCe57tbEYCRV1JwW5dzNCof7MUTaGWhmzYqu","rawTokenAmount":{"decimals":0,"tokenAmount":"1"},"tokenAccount":"DTYuh7gAGGZg2okM7hdFfU1yMY9LUemCiPyD5Z5GCs6Z","userAccount":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX"}]},{"account":"rFqFJ9g7TGBD8Ed7TPDnvGKZ5pWLPDyxLcvcH2eRCtt","nativeBalanceChange":1080000000,"tokenBalanceChanges":[]},{"account":"CgXS5xC3qAGSg9txD9bS7BUgugZwshivGXpCJcGmdwrd","nativeBalanceChange":-2234160,"tokenBalanceChanges":[]},{"account":"M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"E8cU1WiRWjanGxmn96ewBgk9vPTcL6AEZ1t6F6fkgUWe","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"11111111111111111111111111111111","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"FdsNQE5EeCe57tbEYCRV1JwW5dzNCof7MUTaGWhmzYqu","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"AYZsWahcrSnkwqbA1ji7wEzgAnGjLNJhVUMDPfACECZf","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"SysvarRent111111111111111111111111111111111","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL","nativeBalanceChange":0,"tokenBalanceChanges":[]},{"account":"1BWutmTvYPwDtmw9abTkS4Ssr8no61spGAvW1X6NDix","nativeBalanceChange":0,"tokenBalanceChanges":[]}],"description":"5DxD5ViWjvRZEkxQEaJHZw2sBsso6xoXx3wGFNKgXUzE sold Fox #7637 to CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX for 72 SOL on MAGIC_EDEN.","events":{"nft":{"amount":72000000000,"buyer":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","description":"5DxD5ViWjvRZEkxQEaJHZw2sBsso6xoXx3wGFNKgXUzE sold Fox #7637 to CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX for 72 SOL on MAGIC_EDEN.","fee":10000,"feePayer":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","nfts":[{"mint":"FdsNQE5EeCe57tbEYCRV1JwW5dzNCof7MUTaGWhmzYqu","tokenStandard":"NonFungible"}],"saleType":"INSTANT_SALE","seller":"5DxD5ViWjvRZEkxQEaJHZw2sBsso6xoXx3wGFNKgXUzE","signature":"5nNtjezQMYBHvgSQmoRmJPiXGsPAWmJPoGSa64xanqrauogiVzFyGQhKeFataHGXq51jR2hjbzNTkPUpP787HAmL","slot":171942732,"source":"MAGIC_EDEN","staker":"","timestamp":1673445241,"type":"NFT_SALE"}},"fee":10000,"feePayer":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","nativeTransfers":[{"amount":72936000000,"fromUserAccount":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","toUserAccount":"AAaTGaA3uVqikfVEwoSG7EwkCb4bBDsMEyueiVUS5CaU"},{"amount":2011440,"fromUserAccount":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","toUserAccount":"D8TxfGwdu9MiNMoJmUoC9wQfNfNT7Lnm6DzifQHRTy6B"},{"amount":71856000000,"fromUserAccount":"AAaTGaA3uVqikfVEwoSG7EwkCb4bBDsMEyueiVUS5CaU","toUserAccount":"5DxD5ViWjvRZEkxQEaJHZw2sBsso6xoXx3wGFNKgXUzE"},{"amount":1080000000,"fromUserAccount":"AAaTGaA3uVqikfVEwoSG7EwkCb4bBDsMEyueiVUS5CaU","toUserAccount":"rFqFJ9g7TGBD8Ed7TPDnvGKZ5pWLPDyxLcvcH2eRCtt"},{"amount":2039280,"fromUserAccount":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","toUserAccount":"DTYuh7gAGGZg2okM7hdFfU1yMY9LUemCiPyD5Z5GCs6Z"}],"signature":"5nNtjezQMYBHvgSQmoRmJPiXGsPAWmJPoGSa64xanqrauogiVzFyGQhKeFataHGXq51jR2hjbzNTkPUpP787HAmL","slot":171942732,"source":"MAGIC_EDEN","timestamp":1673445241,"tokenTransfers":[{"fromTokenAccount":"25DTUAd1roBFoUQaxJQByL6Qy2cKQCBp4bK9sgfy9UiM","fromUserAccount":"1BWutmTvYPwDtmw9abTkS4Ssr8no61spGAvW1X6NDix","mint":"FdsNQE5EeCe57tbEYCRV1JwW5dzNCof7MUTaGWhmzYqu","toTokenAccount":"DTYuh7gAGGZg2okM7hdFfU1yMY9LUemCiPyD5Z5GCs6Z","toUserAccount":"CKs1E69a2e9TmH4mKKLrXFF8kD3ZnwKjoEuXa6sz9WqX","tokenAmount":1,"tokenStandard":"NonFungible"}],"type":"NFT_SALE"}]"#;
        let transactions: Vec<EnhancedTransaction> = serde_json::from_str(enhanced_trans).unwrap();
        println!("aaaa {transactions:?}");
    }
}
