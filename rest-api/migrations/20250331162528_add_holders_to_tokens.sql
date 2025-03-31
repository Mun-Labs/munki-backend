-- Add migration script here
ALTER TABLE tokens ADD COLUMN holders INT DEFAULT 0;
