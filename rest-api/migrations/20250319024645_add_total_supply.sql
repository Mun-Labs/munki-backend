-- Add migration script here
alter table if exists tokens add column if not exists total_supply numeric(30, 10) default 0;
