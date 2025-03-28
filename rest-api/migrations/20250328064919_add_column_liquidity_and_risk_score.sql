-- Add migration script here
ALTER TABLE tokens
    ADD COLUMN liquidity numeric(30, 10) DEFAULT 0;
ALTER TABLE tokens
    ADD COLUMN risk_score numeric(10, 6) DEFAULT 0;