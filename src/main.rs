// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use futures::executor::block_on;
use serde_json::json;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    Column, Row, TypeInfo,
};
use std::{collections::HashMap, str::FromStr};

async fn init_db() -> SqlitePool {
    let db_options = SqliteConnectOptions::from_str("sqlite:testing.sqlite")
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(db_options).await.unwrap();
    let mut db = pool.acquire().await.unwrap();
    let _ = sqlx::query(
        "create table dataset (id int primary key, unit int, active bool, name text, value float);
         insert into dataset values (1,42,False,'A unit',12.34);
         commit;",
    )
    .execute(&mut *db)
    .await;
    db.close().await.ok();
    pool
}

fn main() -> anyhow::Result<()> {
    let db = block_on(init_db());
    #[allow(clippy::into_iter_on_ref)] // We *want* to release early, so we're going .into_
    let result: Vec<_> = block_on(sqlx::query(r#"select * from dataset"#).fetch_all(&db))?
        .into_iter()
        .map(|row| {
            json!(row
                .columns()
                .into_iter()
                .map(|column| {
                    let ordinal = column.ordinal();
                    let type_name = column.type_info().name();
                    (
                        column.name(),
                        match type_name {
                            "TEXT" => json!(row.get::<String, _>(ordinal)),
                            "INTEGER" => json!(row.get::<i64, _>(ordinal)),
                            "BOOLEAN" => json!(row.get::<bool, _>(ordinal)),
                            "REAL" => json!(row.get::<f64, _>(ordinal)),
                            // probably missed a few other types?
                            _ => {
                                json!(format!("UNPROCESSED TYPE '{}'", type_name))
                            }
                        },
                    )
                })
                .collect::<HashMap<_, _>>())
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&result).unwrap());
    Ok(())
}
