-- Add migration script here
CREATE TABLE IF NOT EXISTS token_watch
(
    token_address VARCHAR PRIMARY KEY,
    updated_at    TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Optional index for performance on the updated_at column.
CREATE INDEX idx_token_watch_updated_at ON token_watch (updated_at);