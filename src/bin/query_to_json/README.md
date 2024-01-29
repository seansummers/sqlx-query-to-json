# Query to JSON

## Problem
From [StackOverflow Question 77029413](https://stackoverflow.com/questions/77029413/getting-column-data-using-sqlx-with-rust):

> I am using ... sqlx to ... [take] a table name and returns all the data in the table in the form of json.
The number of columns and their types are not known beforehand.

## Solution

Use `serde_json` to wrap SQL datatypes in a `sqlx::types::JsonValue` enum.

To run:

```bash
$ # From the repo directory:
$ cargo run --bin query_to_json
```

### Notes

* Using `futures::executor::block_on` to keep [main.rs](main.rs) synchronous (SQLx supports async only)
* Using `sqlx::sqlite::SqlitePoolOptions` to ensure we use the same `:memory:` SQLite db
* Using `anyhow::Result` to _do it right_
* Overriding clippy's opinion on `.into_iter`, because we don't want to keep the original `Vec` from the result set around anyway.
