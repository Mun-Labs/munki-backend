-- Add migration script here
CREATE TABLE IF NOT EXISTS fear_and_greed
(
    value     BIGINT       NOT NULL,
    status    VARCHAR(255) NOT NULL,
    timestamp BIGINT       NOT NULL,
    chain     VARCHAR(255) NOT NULL,
    UNIQUE (timestamp, chain)
);

CREATE INDEX if not exists idx_fear_and_greed_timestamp ON fear_and_greed (timestamp);
