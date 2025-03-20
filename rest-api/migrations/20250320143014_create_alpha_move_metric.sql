-- Add migration script here
CREATE TABLE IF NOT EXISTS alpha_move_token_metric
(
    token_address             varchar(128) PRIMARY KEY,
    mun_score                 numeric(20, 10) NOT NULL default 0,
    top_fresh_wallet_holders  BIGINT                   DEFAULT 0,
    top_smart_wallets_holders BIGINT                   DEFAULT 0,
    smart_followers           BIGINT                   DEFAULT 0,
    created_at                TIMESTAMPTZ     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at                TIMESTAMPTZ     NOT NULL DEFAULT CURRENT_TIMESTAMP
);
