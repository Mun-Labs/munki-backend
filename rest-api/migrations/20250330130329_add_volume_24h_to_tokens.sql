-- Add migration script here
ALTER TABLE tokens ADD COLUMN volume_24h DECIMAL(30, 0) DEFAULT 0;
