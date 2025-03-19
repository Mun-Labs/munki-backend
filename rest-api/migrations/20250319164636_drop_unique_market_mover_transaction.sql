-- Add migration script here
ALTER TABLE market_movers_transaction
DROP CONSTRAINT IF EXISTS market_movers_transaction_token_address_key;

ALTER TABLE market_movers_transaction
DROP CONSTRAINT IF EXISTS market_movers_transaction_wallet_address_key;
