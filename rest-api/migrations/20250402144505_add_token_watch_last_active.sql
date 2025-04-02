-- Add migration script here
ALTER TABLE token_watch ADD COLUMN last_active TIMESTAMP WITH TIME ZONE;
