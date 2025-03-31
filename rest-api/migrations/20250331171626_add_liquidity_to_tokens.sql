-- Add migration script here
ALTER TABLE tokens ADD COLUMN liquidity DECIMAL(30, 0) DEFAULT 0;
