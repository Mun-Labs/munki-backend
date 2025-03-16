-- Add migration script here
CREATE TABLE IF NOT EXISTS token_volume_history
(
    token_address TEXT    NOT NULL,
    volume24h     NUMERIC NOT NULL,
    record_date   BIGINT  NOT NULL,
    PRIMARY KEY (token_address, record_date)
);
