-- Add migration script here
ALTER TABLE alpha_move_token_metric ADD COLUMN risk_score NUMERIC(20, 10) DEFAULT 0;
