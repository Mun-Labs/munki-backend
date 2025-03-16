-- Add migration script here
create table if not exists greed_and_fear_history
(
    unix_timestamp       bigint not null
    constraint greed_and_fear_history_pk
    primary key,
    recorded_at          timestamp with time zone,
    value                integer,
    value_classification varchar(96),
    chain                varchar(96) default 'solana'::character varying
    );