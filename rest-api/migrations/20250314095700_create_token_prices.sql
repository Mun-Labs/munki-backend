-- Add migration script here
CREATE TABLE IF NOT EXISTS token_prices
(
    address     varchar(128)    NOT NULL,
    price       NUMERIC(20, 10) NOT NULL, -- Price precision may be adjusted as needed
    unixtime    BIGINT          NOT NULL, -- Price precision may be adjusted as needed
    recorded_at TIMESTAMPTZ     NOT NULL DEFAULT NOW(),
    primary key (address, unixtime)
);
