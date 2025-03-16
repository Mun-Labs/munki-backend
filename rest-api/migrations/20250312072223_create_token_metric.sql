-- Add migration script here
CREATE TABLE IF NOT EXISTS token_metrics (
    id BIGSERIAL PRIMARY KEY,
    token_address VARCHAR(126) NOT NULL,  -- Direct token address for quick lookups
    update_unix_time BIGINT NOT NULL,  -- Timestamp in Unix format
    update_human_time VARCHAR(126) NOT NULL,  -- Human-readable timestamp
    volume_usd NUMERIC(30,10) NOT NULL,  -- USD volume with high precision
    volume_change_percent NUMERIC(10,6),  -- Percentage change in volume
    price_change_percent NUMERIC(10,6),  -- Percentage change in price
    price NUMERIC(10,6),  -- Percentage change in price
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ✅ Index for fast queries by token address and time
CREATE INDEX if not exists idx_token_metrics_address_time ON token_metrics(token_address, update_unix_time DESC);

-- ✅ Index for fast token lookups using ID
CREATE INDEX if not exists idx_token_metrics_token_address ON token_metrics(token_address);
