# deeper-archive

substrate-archive for deeper-chain

# build

```bash
cargo build
```

# run

run postgres

```bash
docker-compose up -d
```

migrate database and tables, change to `substrate-archive/substrate-archive/src`

```bash
DATABASE_URL=postgres://postgres:123@localhost:6432/deeper_local sqlx database create 
DATABASE_URL=postgres://postgres:123@localhost:6432/deeper_local sqlx migrate run
```

run local 2 node test net

```bash
./deeper-chain --chain local --base-path /tmp/alice --alice --validator --rpc-cors all

./deeper-chain --chain local --base-path /tmp/bob --bob --validator --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWCKo66QARowx7hqD5epuaXJUrrG6jEHFyACP2CiT1BLe3 --pruning archive
```

start index chain

```bash
cargo build
./target/debug/deeper-archive -c archive.toml --chain local
```

create decoded tables

```bash
DATABASE_URL=postgres://postgres:123@localhost:6432/deeper_local sqlx migrate add balance_decoded
```

how to get metadata for test

```bash
subxt metadata -f bytes > metadata.scale
```

## architecture

### deeper-archive

read from rocksdb and write block info, extrinsic info into postgres

### deeper-decoder

for custom storage, because we don't know the key, so first step we need to know all storage keys in the block.
after that we can decode the correspond storage value.

for events, the storage key is fixed, so the only thing is to decode the value.

## useful queries

### query events

query events by pallet name and event name

```SQL
select * from block_event where info->'event'->>'name'='Credit' and info->'event'->'values'->0->>'name' = 'CreditDataUpdated' limit 10;
```

query events by account id

```SQL
select * from block_event where info->'event'->>'name'='Credit' and info->'event'->'values'->0->'values'->0 = '[[168, 139, 89, 175, 231, 63, 14, 118, 158, 79, 157, 133, 205, 64, 253, 19, 240, 135, 68, 70, 242, 45, 42, 182, 120, 15, 156, 184, 144, 89, 48, 126]]' limit 1;
```

the account id is a 32 bytes big number, but its scale expression is a Vector of Value, so a little tricky to query by account id.currently I didn't find any simple ways to convert `[u8; 32]` into hex string or ss58 format.