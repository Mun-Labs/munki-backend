-- Add migration script here
ALTER TABLE tokens
    ADD COLUMN marketcap numeric(30, 10) DEFAULT 0;

ALTER TABLE tokens
    ADD COLUMN history24h_price numeric(20, 10);

ALTER TABLE tokens
    ADD COLUMN price_change24h_percent numeric(20, 10);