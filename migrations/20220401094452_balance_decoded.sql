CREATE TABLE IF NOT EXISTS balance_decoded (
  id bigserial NOT NULL,
  block_num integer NOT NULL,
  balance numeric(18) not null,
  address varchar(48) not null,
  block_time timestamp with time zone
);