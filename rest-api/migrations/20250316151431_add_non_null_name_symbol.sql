-- Add migration script here
ALTER TABLE tokens
    ALTER COLUMN name SET NOT NULL;

ALTER TABLE tokens
    ALTER COLUMN symbol SET NOT NULL;

CREATE EXTENSION IF NOT EXISTS pg_trgm;