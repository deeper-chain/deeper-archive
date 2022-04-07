drop table if exists block_balance;
drop table if exists block_credit;
drop table if exists block_timestamp;

CREATE TABLE IF NOT EXISTS block_balance (
  id bigserial NOT NULL,
  block_num integer NOT NULL,
  nonce integer not null,
  free numeric(30, 0) not null,
  reserved numeric(30, 0) not null,
  misc_frozen numeric(30, 0) not null,
  fee_frozen numeric(30, 0) not null,
  address varchar(48) not null
);

CREATE TABLE IF NOT EXISTS block_credit (
  id bigserial NOT NULL,
  block_num integer NOT NULL,
  credit integer not null,
  address varchar(48) not null
);

CREATE TABLE IF NOT EXISTS block_timestamp (
  id bigserial NOT NULL,
  block_num integer NOT NULL,
  block_time timestamp with time zone
);