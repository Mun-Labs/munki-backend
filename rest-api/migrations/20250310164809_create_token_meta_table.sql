CREATE TABLE if not exists tokens (
  id BIGSERIAL PRIMARY KEY,
  token_address VARCHAR(126) UNIQUE, -- e.g., Ethereum token address
  name TEXT NOT NULL,
  symbol TEXT NOT NULL,
  decimals INT,                     -- Number of decimal places, if applicable
  description TEXT,
  image_url TEXT,
  website_url TEXT,
  metadata JSONB,                   -- Additional token properties
  current_price NUMERIC(20,10),     -- Latest price for quick lookup
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
