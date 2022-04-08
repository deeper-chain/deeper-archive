use desub_current::decoder::Extrinsic;
use desub_current::value::{self, Composite, Primitive, Value};
use desub_current::Metadata;
use serde::{Deserialize, Serialize};
use sp_core::crypto::Ss58Codec;
use sqlx::postgres::{PgPoolOptions, Postgres};
use sqlx::types::bstr;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Json;
use sqlx::{Pool, Row};
use std::{error::Error, fmt};

mod balance_decoder;
mod common;
mod credit_decoder;
mod event_decoder;

static V14_METADATA_DEEPER_SCALE: &[u8] = include_bytes!("../data/v14_metadata_deeper.scale");

// TODO: read metadata from database
fn deeper_metadata() -> Metadata {
    Metadata::from_bytes(V14_METADATA_DEEPER_SCALE).expect("valid metadata")
}

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

    // TODO: consider using join
    decode_timestamp(&pool, start_block).await?;
    decode_balance(&pool, start_block).await?;
    decode_credit(&pool, start_block).await?;
    decode_event(&pool, start_block).await?;

    Ok(())
}

async fn get_last_synced_block(pool: &Pool<Postgres>) -> Result<i32, Box<dyn std::error::Error>> {
    let row_result = sqlx::query("select block_num from block_timestamp order by id desc limit 1;")
        .fetch_one(pool)
        .await;

    let block_num = match row_result {
        Ok(row) => row.try_get("block_num")?,
        Err(_) => 0,
    };
    // let block_num = row.try_get("block_num")?;

    Ok(block_num)
}

async fn decode_timestamp(
    pool: &Pool<Postgres>,
    start_block: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where number > $1 order by number asc limit $2;")
    .bind(start_block)
    .bind(BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    for row in &rows {
        let extrinsics: Vec<CurrentExtrinsic> = serde_json::from_str(&row.1)?;
        for extrinsic in &extrinsics {
            if extrinsic.current.call_data.pallet_name == "Timestamp"
                && extrinsic.current.call_data.ty.name() == "set"
            {
                match extrinsic.current.call_data.arguments[0] {
                    Value::Primitive(Primitive::U64(ts_ms)) => {
                        let tss = ts_ms / 1000;
                        // println!("block: {}, timestamp: {:?}", row.0, tss);
                        let ts_timestamp = OffsetDateTime::from_unix_timestamp(tss as i64);
                        sqlx::query(
                            "insert into block_timestamp(block_num, block_time) values ($1, $2)",
                        )
                        .bind(row.0)
                        .bind(ts_timestamp)
                        .execute(pool)
                        .await?;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

async fn decode_balance(
    pool: &Pool<Postgres>,
    start_block: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where number > $1 order by number asc limit $2;")
    .bind(start_block)
    .bind(BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    for row in &rows {
        let sign_addrs = crate::balance_decoder::get_balance_changed_account_ids(&row.1);
        for addr in sign_addrs {
            let key = crate::common::system_account_key(addr.clone());
            let storage_row = sqlx::query("select block_num, encode(storage, 'hex') as storage_hex, encode(key, 'hex') as key_hex from storage where key=$1;")
                .bind(bstr::BString::from(key.clone()))
                .fetch_one(pool)
                .await?;

            let storage_key: String = storage_row.try_get("key_hex")?;
            let storage_str: String = storage_row.try_get("storage_hex")?;
            let storage_val =
                crate::common::decode_storage(&storage_key, &storage_str, deeper_metadata());

            // println!("storage_val {:?}", storage_val);
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

            sqlx::query(
                "insert into block_balance(block_num, address, nonce, free, reserved, misc_frozen, fee_frozen) values ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(row.0)
            .bind(addr.to_ss58check())
            .bind(nonce)
            .bind(sqlx::types::Decimal::from_i128_with_scale(free as i128, 0))
            .bind(sqlx::types::Decimal::from_i128_with_scale(reserved as i128, 0))
            .bind(sqlx::types::Decimal::from_i128_with_scale(misc_frozen as i128, 0))
            .bind(sqlx::types::Decimal::from_i128_with_scale(fee_frozen as i128, 0))
            .execute(pool)
            .await?;
        }
    }

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
    start_block: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where number > $1 order by number asc limit $2;")
    .bind(start_block)
    .bind(BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    for row in &rows {
        let sign_addrs = crate::credit_decoder::get_credit_changed_account_ids(&row.1);
        for addr in sign_addrs {
            let key = crate::common::user_credit_key(addr.clone());

            let storage_row = sqlx::query("select block_num, encode(storage, 'hex') as storage_hex, encode(key, 'hex') as key_hex from storage where key=$1;")
                .bind(bstr::BString::from(key.clone()))
                .fetch_one(pool)
                .await?;
            let storage_key: String = storage_row.try_get("key_hex")?;
            let storage_str: String = storage_row.try_get("storage_hex")?;
            let storage_val =
                crate::common::decode_storage(&storage_key, &storage_str, deeper_metadata());

            println!("credit storage {:?}, {}", storage_val, storage_str);

            match storage_val {
                Value::Composite(Composite::Named(data)) => {
                    match data[1].1.clone() {
                        Value::Primitive(Primitive::U64(credit)) => {
                            // insert into database
                            sqlx::query(
                                "insert into block_credit(block_num, address, credit) values ($1, $2, $3)",
                            )
                            .bind(row.0)
                            .bind(addr.to_ss58check())
                            .bind(credit as i32) // credit field integer
                            .execute(pool)
                            .await?;
                        }
                        _ => {}
                    }
                }
                _ => assert!(false),
            }
        }
    }

    Ok(())
}

async fn decode_event(
    pool: &Pool<Postgres>,
    start_block: i32,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_key = hex::encode(crate::common::event_key());
    // this sql won't return null because every block have events
    let rows: Vec<(i32, String, String)> = sqlx::query_as("select block_num, encode(key, 'hex') as key_hex, encode(storage, 'hex') as storage_hex from storage where block_num > $1 and encode(key, 'hex') = $2 order by block_num asc limit $3;")
    .bind(start_block)
    .bind(event_key)
    .bind(BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    let mut values: Vec<(i32, String, String, event_decoder::EventInfo)> = vec![];
    for row in &rows {
        let events = event_decoder::decode_event(&row.1, &row.2, deeper_metadata());
        for event in &events {
            values.push((
                row.0,
                event.pallet.clone(),
                event.name.clone(),
                event.info.clone(),
            ));
        }
    }

    // insert many rows
    // https://github.com/launchbadge/sqlx/issues/294#issuecomment-830409187
    let mut block_nums: Vec<i32> = vec![];
    let mut pallet_names: Vec<String> = vec![];
    let mut event_names: Vec<String> = vec![];
    let mut infos: Vec<Json<event_decoder::EventInfo>> = vec![];
    values.into_iter().for_each(|value| {
        block_nums.push(value.0);
        pallet_names.push(value.1);
        event_names.push(value.2);
        infos.push(Json(value.3));
    });
    sqlx::query(
        r#"INSERT INTO block_event (block_num, pallet_name, event_name, info)
        SELECT * FROM UNNEST($1, $2, $3, $4);"#,
    )
    .bind(&block_nums)
    .bind(&pallet_names)
    .bind(&event_names)
    .bind(&infos)
    .execute(pool)
    .await?;

    Ok(())
}
