// src/db.rs

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_name: &str) -> DbPool {
    // Create the SQLite connection manager
    let manager = SqliteConnectionManager::file(db_name);

    let pool = Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("Failed to create DB pool");

    let conn = pool.get().expect("Failed to get connection for init");

    // Create the key-value table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS kv_store (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .expect("Failed to create kv_store table");

    println!("Database initialized and table ensured at {}", db_name);

    pool
}
