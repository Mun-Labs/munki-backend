CREATE TABLE token_analytics (
    address TEXT PRIMARY KEY,
    market_cap NUMERIC,
    volume_24h NUMERIC,
    volume_24h_change_7d NUMERIC,
    liquidity NUMERIC,
    liquidity_change NUMERIC,
    holders NUMERIC,
    holders_change_7d NUMERIC,
);