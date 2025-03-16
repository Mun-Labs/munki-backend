-- Add migration script here
CREATE TABLE IF NOT EXISTS block_chain_volume (
    id BIGSERIAL PRIMARY KEY,                  -- Auto-incrementing ID
    chain VARCHAR(126) NOT NULL,                  
    total24h BIGINT NOT NULL CHECK (total24h >= 0),      -- 24-hour total
    total48hto24h BIGINT NOT NULL CHECK (total48hto24h >= 0), -- 48-to-24-hour total
    total7d BIGINT NOT NULL CHECK (total7d >= 0),        -- 7-day total
    total14dto7d BIGINT NOT NULL CHECK (total14dto7d >= 0), -- 14-to-7-day total
    total60dto30d BIGINT NOT NULL CHECK (total60dto30d >= 0), -- 60-to-30-day total
    total30d BIGINT NOT NULL CHECK (total30d >= 0),      -- 30-day total
    total1y BIGINT NOT NULL CHECK (total1y >= 0),        -- 1-year total
    change_1d DOUBLE PRECISION NOT NULL,         -- 1-day percentage change
    change_7d DOUBLE PRECISION NOT NULL,         -- 7-day percentage change
    change_1m DOUBLE PRECISION NOT NULL,         -- 1-month percentage change
    change_7dover7d DOUBLE PRECISION NOT NULL,   -- 7-day-over-7-day change
    change_30dover30d DOUBLE PRECISION NOT NULL, -- 30-day-over-30-day change
    total7_days_ago BIGINT NOT NULL CHECK (total7_days_ago >= 0), -- 7 days ago total
    total30_days_ago BIGINT NOT NULL CHECK (total30_days_ago >= 0), -- 30 days ago total
    recorded_date DATE NOT NULL,                -- Date of the record (no time)
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP, -- Full timestamp for reference
    CONSTRAINT unique_daily_metrics UNIQUE (recorded_at)
);

