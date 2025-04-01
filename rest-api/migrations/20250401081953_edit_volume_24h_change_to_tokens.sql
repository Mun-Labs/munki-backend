-- Add migration script here
ALTER TABLE tokens
ALTER COLUMN volume_24h_change TYPE NUMERIC(30);
