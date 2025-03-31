-- Add migration script here
ALTER TABLE tokens ADD COLUMN volume_24h_change DECIMAL(10, 2) DEFAULT 0;
