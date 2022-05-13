use desub_current::decoder::Extrinsic;
use desub_current::value::{self, Composite, Primitive, Value};
use desub_current::Metadata;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use sqlx::postgres::{PgPoolOptions, Postgres};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Decimal;
use sqlx::types::Json;
use sqlx::{Pool, Row};
use std::{error::Error, fmt};

mod balance_decoder;
mod common;
mod credit_decoder;
mod delegation_decoder;
mod event_decoder;

const BATCH_SIZE: i32 = 1000;

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentExtrinsic<'a> {
    #[serde(borrow)]
    #[serde(rename = "Current")]
    pub current: Extrinsic<'a>,
}
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:123@localhost:6432/deeper_local")
        .await?;

    let start_block = get_last_synced_block(&pool).await?;
    let to_decode_blocks = get_to_decode_blocks(&pool, start_block).await?;
    let storage_rows = get_block_storage_rows(&pool, &to_decode_blocks).await;

    // TODO: consider using join
    decode_balance(&pool, &to_decode_blocks, &storage_rows).await?;
    decode_credit(&pool, &to_decode_blocks, &storage_rows).await?;
    decode_event(&pool, &to_decode_blocks, &storage_rows).await?;
    decode_delegation(&pool, &to_decode_blocks, &storage_rows).await?;
    decode_timestamp(&pool, &to_decode_blocks).await?; // make sure all the other storages were inserted successfully

    Ok(())
}

// desub-current only support metadata v14 or newer
// desub-legacy support metadata older than v14
// deeper-chain upgraded to polkadot-v0.9.12 and metadata v14 in 2021-12-17
// for simplicity we only consider spec >= 7 blocks, ignore older blocks here
async fn get_to_decode_blocks(
    pool: &Pool<Postgres>,
    start_block: i32,
) -> Result<Vec<(i32, String, Metadata)>, Box<dyn std::error::Error>> {
    let rows: Vec<(i32, String, Vec<u8>)> = sqlx::query_as("select b.block_num, ext.extrinsics::text, mt.meta from extrinsics as ext left join blocks as b on ext.number=b.block_num left join metadata as mt on b.spec=mt.version where b.spec >= 7 and ext.number > $1 and ext.number=1899916 order by ext.number asc limit $2;")
        .bind(start_block)
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await?;
    let mut res: Vec<(i32, String, Metadata)> = vec![];
    for row in rows {
        let meta = Metadata::from_bytes(&row.2).expect("valid metadata");
        res.push((row.0, row.1, meta));
    }
    Ok(res)
}

async fn get_block_storage_rows(
    pool: &Pool<Postgres>,
    block_rows: &[(i32, String, Metadata)],
) -> Vec<(i32, String, String)> {
    let mut block_num_vec = vec![];
    for row in block_rows {
        block_num_vec.push(row.0);
    }
    // storage can't be null because sqlx will report DecodeError
    let rows_result: Result<Vec<(i32, String, Option<String>)>, sqlx::Error> = sqlx::query_as("select block_num, encode(key, 'hex') as key_hex, encode(storage, 'hex') as storage_hex from storage where block_num = Any($1) order by block_num asc;")
    .bind(&block_num_vec[..])
    .fetch_all(pool)
    .await;
    match rows_result {
        Ok(rows) => {
            let mut res = vec![];
            for row in &rows {
                let storage_val = match row.2.clone() {
                    Some(val) => val,
                    None => String::from(""),
                };
                res.push((row.0, row.1.clone(), storage_val));
            }
            res
        }
        Err(err) => {
            println!("get_bloock_storage_rows error {:?}", err);
            vec![]
        }
    }
}

async fn get_last_synced_block(pool: &Pool<Postgres>) -> Result<i32, Box<dyn std::error::Error>> {
    let row_result = sqlx::query("select block_num from block_timestamp order by id desc limit 1;")
        .fetch_one(pool)
        .await;

    let block_num = match row_result {
        Ok(row) => row.try_get("block_num")?,
        Err(_) => 0,
    };

    Ok(block_num)
}

async fn decode_timestamp(
    pool: &Pool<Postgres>,
    rows: &[(i32, String, Metadata)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_insert_data: Vec<(i32, u64)> = vec![];
    for row in rows {
        match serde_json::from_str::<Vec<CurrentExtrinsic>>(&row.1) {
            Ok(extrinsics) => {
                for extrinsic in &extrinsics {
                    if extrinsic.current.call_data.pallet_name == "Timestamp"
                        && extrinsic.current.call_data.ty.name() == "set"
                    {
                        match extrinsic.current.call_data.arguments[0] {
                            Value::Primitive(Primitive::U64(ts_ms)) => {
                                to_insert_data.push((row.0, ts_ms));
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                continue;
            }
        }
    }
    if to_insert_data.is_empty() {
        return Ok(());
    }
    let mut to_insert_block_nums = vec![];
    let mut to_insert_ts = vec![];
    to_insert_data.into_iter().for_each(|value| {
        to_insert_block_nums.push(value.0);
        to_insert_ts.push(OffsetDateTime::from_unix_timestamp((value.1 / 1000) as i64))
    });
    sqlx::query(
        "insert into block_timestamp(block_num, block_time) select * from unnest ($1, $2);",
    )
    .bind(to_insert_block_nums)
    .bind(to_insert_ts)
    .execute(pool)
    .await?;

    Ok(())
}

async fn decode_balance(
    pool: &Pool<Postgres>,
    block_rows: &[(i32, String, Metadata)],
    storage_rows: &[(i32, String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_insert_data: Vec<(i32, String, u32, u128, u128, u128, u128)> = vec![];
    for row in block_rows {
        // loop block
        let block_addr_hs = crate::balance_decoder::get_balance_changed_account_ids(&row.1);
        for addr in block_addr_hs {
            // loop user
            let key = crate::common::system_account_key(addr.clone());
            for storage_row in storage_rows {
                if storage_row.0 == row.0 && storage_row.1 == hex::encode(key.clone()) {
                    let storage_key: String = storage_row.1.clone();
                    let storage_str: String = storage_row.2.clone();
                    let storage_val =
                        crate::common::decode_storage(&storage_key, &storage_str, &row.2);

                    let (nonce, free, reserved, misc_frozen, fee_frozen) = match storage_val {
                        Value::Composite(Composite::Named(cn)) => {
                            let nonce = match cn[0].1.clone() {
                                Value::Primitive(value::Primitive::U32(inner)) => inner,
                                _ => 0,
                            };
                            let (free, reserved, misc_frozen, fee_frozen) = match cn[4].1.clone() {
                                Value::Composite(Composite::Named(cnd)) => {
                                    let free = match cnd[0].1 {
                                        Value::Primitive(value::Primitive::U128(inner)) => inner,
                                        _ => 0,
                                    };
                                    let reserved = match cnd[1].1 {
                                        Value::Primitive(value::Primitive::U128(inner)) => inner,
                                        _ => 0,
                                    };
                                    let misc_frozen = match cnd[2].1 {
                                        Value::Primitive(value::Primitive::U128(inner)) => inner,
                                        _ => 0,
                                    };
                                    let fee_frozen = match cnd[3].1 {
                                        Value::Primitive(value::Primitive::U128(inner)) => inner,
                                        _ => 0,
                                    };
                                    (free, reserved, misc_frozen, fee_frozen)
                                }
                                _ => (0, 0, 0, 0),
                            };
                            (nonce, free, reserved, misc_frozen, fee_frozen)
                        }
                        _ => (0, 0, 0, 0, 0),
                    };

                    to_insert_data.push((
                        row.0,
                        addr.to_ss58check(),
                        nonce,
                        free,
                        reserved,
                        misc_frozen,
                        fee_frozen,
                    ));
                    break;
                }
            }
        }
    }

    if to_insert_data.is_empty() {
        // no data need to insert
        return Ok(());
    }
    let mut to_insert_block_nums: Vec<i32> = vec![];
    let mut to_insert_addrs: Vec<String> = vec![];
    let mut to_insert_nonces: Vec<u32> = vec![];
    let mut to_insert_fee = vec![];
    let mut to_insert_reserved = vec![];
    let mut to_insert_misc_frozen = vec![];
    let mut to_insert_fee_frozen = vec![];
    to_insert_data.into_iter().for_each(|value| {
        to_insert_block_nums.push(value.0);
        to_insert_addrs.push(value.1);
        to_insert_nonces.push(value.2);
        to_insert_fee.push(Decimal::from_i128_with_scale(value.3 as i128, 0));
        to_insert_reserved.push(Decimal::from_i128_with_scale(value.4 as i128, 0));
        to_insert_misc_frozen.push(Decimal::from_i128_with_scale(value.5 as i128, 0));
        to_insert_fee_frozen.push(Decimal::from_i128_with_scale(value.6 as i128, 0));
    });
    sqlx::query(
        "insert into block_balance(block_num, address, nonce, free, reserved, misc_frozen, fee_frozen) select * from unnest ($1, $2, $3, $4, $5, $6, $7);",
    )
    .bind(&to_insert_block_nums)
    .bind(&to_insert_addrs)
    .bind(&to_insert_nonces)
    .bind(&to_insert_fee)
    .bind(&to_insert_reserved)
    .bind(&to_insert_misc_frozen)
    .bind(&to_insert_fee_frozen)
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug)]
struct DecodeAccountIdFailed;

impl Error for DecodeAccountIdFailed {}

impl fmt::Display for DecodeAccountIdFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "decode failed")
    }
}

async fn decode_credit(
    pool: &Pool<Postgres>,
    block_rows: &[(i32, String, Metadata)],
    storage_rows: &[(i32, String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_insert_data = vec![];
    for row in block_rows {
        let sign_addrs = crate::credit_decoder::get_credit_changed_account_ids(&row.1);
        for addr in sign_addrs {
            let key = crate::common::user_credit_key(addr.clone());
            for storage_row in storage_rows {
                if storage_row.0 == row.0 && storage_row.1 == hex::encode(key.clone()) {
                    let storage_key: String = storage_row.1.clone();
                    let storage_str: String = storage_row.2.clone();
                    let storage_val =
                        crate::common::decode_storage(&storage_key, &storage_str, &row.2);

                    match storage_val {
                        Value::Composite(Composite::Named(data)) => {
                            match data[1].1.clone() {
                                Value::Primitive(Primitive::U64(credit)) => {
                                    // insert into database
                                    to_insert_data.push((
                                        row.0,
                                        addr.to_ss58check(),
                                        credit as i32,
                                    ));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    if to_insert_data.is_empty() {
        // no data need to insert
        return Ok(());
    }
    let mut to_insert_block_nums = vec![];
    let mut to_insert_addrs = vec![];
    let mut to_insert_credits = vec![];
    to_insert_data.into_iter().for_each(|value| {
        to_insert_block_nums.push(value.0);
        to_insert_addrs.push(value.1);
        to_insert_credits.push(value.2);
    });
    sqlx::query(
        "insert into block_credit(block_num, address, credit) select * from unnest ($1, $2, $3)",
    )
    .bind(&to_insert_block_nums)
    .bind(&to_insert_addrs)
    .bind(&to_insert_credits) // credit field integer
    .execute(pool)
    .await?;

    Ok(())
}

async fn decode_event(
    pool: &Pool<Postgres>,
    block_rows: &[(i32, String, Metadata)],
    storage_rows: &[(i32, String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut to_insert_data: Vec<(i32, Value)> = vec![];
    let event_key = hex::encode(crate::common::event_key());
    for row in block_rows {
        for storage_row in storage_rows {
            if storage_row.0 == row.0 && storage_row.1 == event_key {
                let events = event_decoder::decode_event(&storage_row.1, &storage_row.2, &row.2);
                for event in &events {
                    to_insert_data.push((row.0, event.to_owned()));
                }
            }
        }
    }

    if to_insert_data.is_empty() {
        return Ok(());
    }
    // insert many rows
    // https://github.com/launchbadge/sqlx/issues/294#issuecomment-830409187
    let mut to_insert_block_nums: Vec<i32> = vec![];
    let mut to_insert_infos: Vec<Json<Value>> = vec![];
    to_insert_data.into_iter().for_each(|value| {
        to_insert_block_nums.push(value.0);
        to_insert_infos.push(Json(value.1));
    });
    sqlx::query(
        r#"INSERT INTO block_event (block_num, info)
        SELECT * FROM UNNEST($1, $2);"#,
    )
    .bind(&to_insert_block_nums)
    .bind(&to_insert_infos)
    .execute(pool)
    .await?;

    Ok(())
}

async fn decode_delegation(
    pool: &Pool<Postgres>,
    block_rows: &[(i32, String, Metadata)],
    storage_rows: &[(i32, String, String)],
) -> Result<(), Box<dyn std::error::Error>> {
    for row in block_rows {
        let block_addr_hs = crate::delegation_decoder::get_delegation_changed_account_ids(&row.1);
        for addr in block_addr_hs {
            for storage_row in storage_rows {
                if storage_row.0 == row.0
                    && storage_row.1
                        == hex::encode(crate::common::staking_delegators_key(addr.clone()))
                {
                    // TODO: handle null storage
                    let validators =
                        delegation_decoder::get_validators(&storage_row.1, &storage_row.2, &row.2);
                    sqlx::query(
                        "insert into block_delegation(block_num, delegator, validators) values ($1, $2, $3)",
                    )
                    .bind(row.0)
                    .bind(addr.to_ss58check())
                    .bind(Json(validators))
                    .execute(pool)
                    .await?;
                }
            }
        }
    }

    Ok(())
}
