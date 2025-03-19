-- Add migration script here
CREATE TABLE IF NOT EXISTS market_mover (
  wallet_address VARCHAR(128) NOT NULL PRIMARY KEY,
  role VARCHAR(50) NOT NULL,  -- e.g., 'whale', 'KOL', 'smart_trader'
  name VARCHAR(256),
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);


CREATE TABLE IF NOT EXISTS market_movers_transaction (
  signature VARCHAR(256) PRIMARY KEY,
  token_address VARCHAR(128) UNIQUE NOT NULL,
  wallet_address VARCHAR(128) UNIQUE NOT NULL,
  transaction_type VARCHAR(28),         -- e.g., "buy", "sell", "transfer"
  amount  NUMERIC(30, 10),
  block_time BIGINT,
  slot BIGINT,
  additional JSONB
);
