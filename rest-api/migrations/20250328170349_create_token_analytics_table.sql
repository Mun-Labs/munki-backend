-- Add migration script here
CREATE TABLE token_analytics (
    address TEXT PRIMARY KEY,
    market_cap DOUBLE PRECISION,
    market_cap_change_7d DOUBLE PRECISION,
    market_cap_7d_historical_values JSONB,
    volume_24h DOUBLE PRECISION,
    volume_24h_change_7d DOUBLE PRECISION,
    volume_historical JSONB,
    liquidity DOUBLE PRECISION,
    liquidity_change DOUBLE PRECISION,
    liquidity_historical JSONB,
    holders BIGINT,
    holders_change_7d BIGINT,
    holders_historical JSONB,
    top_followers JSONB,
    followers JSONB,
    mentions JSONB
);