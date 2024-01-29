// #![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
use futures::executor::block_on;
use serde_json::json;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions},
    Column, Executor, Row, TypeInfo,
};
use std::{collections::HashMap, str::FromStr};

async fn init_db() -> SqlitePool {
    SqlitePoolOptions::new()
        .min_connections(1)
        .max_connections(1)
        .max_lifetime(None)
        .after_connect(|conn, _meta| Box::pin(async move {
            conn.execute(
                "create table dataset (id int primary key, unit int, active bool, name text, value float);
                insert into dataset values (1,42,False,'A unit',12.34);
                commit;").await.ok();
            Ok(())
        }))
        .connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .create_if_missing(true),)
        .await.expect("Unable to connect to SQLite")
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
