use codec::Encode;
use desub_current::decoder::{self, Extrinsic};
use desub_current::value::{self, Composite, Primitive, Value};
use desub_current::Metadata;
use serde::{Deserialize, Serialize};
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_core::ByteArray;
use sp_runtime::MultiAddress;
use sqlx::postgres::{PgPoolOptions, Postgres};
use sqlx::types::time::OffsetDateTime;
use sqlx::Pool;
use std::collections::HashSet;
use std::{error::Error, fmt};

static V14_METADATA_DEEPER_SCALE: &[u8] = include_bytes!("../data/v14_metadata_deeper.scale");

// TODO: read metadata from database
fn deeper_metadata() -> Metadata {
    Metadata::from_bytes(V14_METADATA_DEEPER_SCALE).expect("valid metadata")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CurrentExtrinsic<'a> {
    #[serde(borrow)]
    #[serde(rename = "Current")]
    pub current: Extrinsic<'a>,
}
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // use from_value to turn desub_current::Value to serde_json::Value
    // let amount_value = p[2].current.call_data.arguments[1].clone();
    // let amount: serde_json::Value = from_value(amount_value).unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:123@localhost:6432/deeper_local")
        .await?;

    // decode_timestamp(&pool).await?;

    decode_balance(&pool).await?;

    Ok(())
}

async fn decode_timestamp(pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where not exists (select block_num from block_timestamp where block_timestamp.block_num = extrinsics.number) order by number asc limit $1;")
    .bind(10_i32)
    .fetch_all(pool)
    .await?;

    for row in &rows {
        let extrinsics: Vec<CurrentExtrinsic> = serde_json::from_str(&row.1)?;
        // filter timestamp.now
        for extrinsic in &extrinsics {
            if extrinsic.current.call_data.pallet_name == "Timestamp"
                && extrinsic.current.call_data.ty.name() == "set"
            {
                // let a = extrinsic.current.call_data.arguments[0];
                match extrinsic.current.call_data.arguments[0] {
                    Value::Primitive(Primitive::U64(ts_ms)) => {
                        let tss = ts_ms / 1000;
                        println!("block: {}, timestamp: {:?}", row.0, tss);
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

// TODO: set a starting block
async fn decode_balance(pool: &Pool<Postgres>) -> Result<(), Box<dyn std::error::Error>> {
    let meta = deeper_metadata();
    let storage = decoder::decode_storage(&meta);
    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where not exists (select block_num from block_balance where block_balance.block_num = extrinsics.number) order by number asc limit $1;")
    .bind(100_i32)
    .fetch_all(pool)
    .await?;

    // let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where number=1529 order by number asc limit $1;")
    // .bind(100_i32)
    // .fetch_all(pool)
    // .await?;

    for row in &rows {
        let mut sign_addrs: HashSet<AccountId32> = HashSet::new(); // 签名收取手续费的地址
        let extrinsics: Vec<CurrentExtrinsic> = serde_json::from_str(&row.1)?;
        for extrinsic in &extrinsics {
            match extrinsic.current.signature.clone() {
                Some(signature_val) => match signature_val.address {
                    MultiAddress::Id(account_id) => {
                        sign_addrs.insert(account_id);
                    }
                    _ => {}
                },
                _ => {}
            };
            if extrinsic.current.call_data.pallet_name == "Balances"
                && extrinsic.current.call_data.ty.name() == "transfer_keep_alive"
            {
                let dest_account_id =
                    decode_account_id(extrinsic.current.call_data.arguments[0].clone())?;
                sign_addrs.insert(dest_account_id);
            }
        }
        // println!("block = {}, sign_addrs: {:?}", row.0, sign_addrs);

        for addr in sign_addrs {
            let mut key = sp_core::twox_128("System".as_bytes()).to_vec();
            key.extend(sp_core::twox_128("Account".as_bytes()).iter());
            let addr_encode = addr.encode();
            key.extend(sp_core::blake2_128(&addr_encode));
            key.extend(&addr_encode); // blake2_128_concat
            let storage_key = hex::encode(key.clone());

            let storage_rows: Vec<(i32, String, String)> = sqlx::query_as("select block_num, encode(storage, 'hex') as storage_hex, encode(key, 'hex') as key_hex from storage where encode(key, 'hex')=$1;")
                .bind(storage_key)
                .fetch_all(pool)
                .await?;
            // println!("val {:?}", storage_rows);

            for storage_row in &storage_rows {
                let mut kk = key.as_slice();
                let entry = storage
                    .decode_key(&meta, &mut kk)
                    .expect("can decode storage");
                let storage_val = decoder::decode_value_by_id(
                    &meta,
                    &entry.ty,
                    &mut hex::decode(&storage_row.1).unwrap().as_slice(),
                )
                .unwrap();

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

                // println!(
                //     "balance {}, {}, {}, {}, {}",
                //     nonce, free, reserved, misc_frozen, fee_frozen
                // );

                // 准备插入数据库
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

fn decode_account_id(dest_value: Value) -> Result<AccountId32, Box<dyn std::error::Error>> {
    match dest_value {
        Value::Composite(vc) => {
            match vc {
                Composite::Named(nvc) => {
                    if nvc.len() >= 2 {
                        match nvc[1].1.clone() {
                            Value::Composite(nvcc) => {
                                match nvcc {
                                    Composite::Unnamed(nvccu) => {
                                        match nvccu[0].clone() {
                                            Value::Composite(nvccuu) => {
                                                match nvccuu {
                                                    Composite::Unnamed(nvccuuu) => {
                                                        match nvccuuu[0].clone() {
                                                            Value::Composite(uvccuuuu) => {
                                                                match uvccuuuu {
                                                                    Composite::Unnamed(
                                                                        uvccuuuuu,
                                                                    ) => {
                                                                        let mut dest: Vec<u8> =
                                                                            vec![];
                                                                        for c in &uvccuuuuu {
                                                                            match c {
                                                                                Value::Primitive(Primitive::U64(cu)) => {
                                                                                    dest.push(*cu as u8);
                                                                                },
                                                                                _ => {
                                                                                    // careful match
                                                                                    return Err(Box::new(
                                                                                        DecodeAccountIdFailed,
                                                                                    ));
                                                                                },
                                                                            }
                                                                        }
                                                                        let account = AccountId32::from_slice(&dest).unwrap();
                                                                        return Ok(account);
                                                                    }
                                                                    _ => Err(Box::new(
                                                                        DecodeAccountIdFailed,
                                                                    )),
                                                                }
                                                            }
                                                            _ => {
                                                                Err(Box::new(DecodeAccountIdFailed))
                                                            }
                                                        }
                                                    }
                                                    _ => Err(Box::new(DecodeAccountIdFailed)),
                                                }
                                            }
                                            _ => Err(Box::new(DecodeAccountIdFailed)),
                                        }
                                    }
                                    _ => Err(Box::new(DecodeAccountIdFailed)),
                                }
                            }
                            _ => Err(Box::new(DecodeAccountIdFailed)),
                        }
                    } else {
                        Err(Box::new(DecodeAccountIdFailed))
                    }
                }
                _ => Err(Box::new(DecodeAccountIdFailed)),
            }
        }
        _ => Err(Box::new(DecodeAccountIdFailed)),
    }
}
