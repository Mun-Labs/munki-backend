-- Add migration script here
ALTER TABLE market_movers_transaction DROP CONSTRAINT market_movers_transaction_pkey;
ALTER TABLE market_movers_transaction ADD PRIMARY KEY (signature, wallet_address, token_address);
