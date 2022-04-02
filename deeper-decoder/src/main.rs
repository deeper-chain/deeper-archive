use desub_current::decoder::Extrinsic;
use desub_current::value::{Primitive, Value};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::time::OffsetDateTime;

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

    let rows: Vec<(i32, String)> = sqlx::query_as("select number, extrinsics::text from extrinsics where not exists (select block_num from block_timestamp where block_timestamp.block_num = extrinsics.number) order by number asc limit $1;")
        .bind(10_i32)
        .fetch_all(&pool)
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
                        .execute(&pool)
                        .await?;
                    }
                    _ => {}
                }
            }

            // filter system.account
        }
    }

    Ok(())
}
