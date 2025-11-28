// src/db.rs

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use std::time::Duration;

pub type DbPool = SqlitePool;

pub async fn init_pool(db_url: &str) -> DbPool {
    // Create the database if it's missing, enable WAL mode, and set a busy timeout
    let connection_options = SqliteConnectOptions::from_str(db_url)
        .expect("Invalid connection string")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    // Create the pool (asynchronous)
    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(connection_options)
        .await
        .expect("Failed to create database pool");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS kv_store (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .execute(&pool)
    .await
    .expect("Failed to create kv_store table");

    println!("Database initialized and connected.");

    pool
}
