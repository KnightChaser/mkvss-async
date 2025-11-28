// src/db.rs

use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn init_pool(db_name: &str) -> DbPool {
    // Create the SQLite connection manager
    let manager = SqliteConnectionManager::file(db_name).with_init(|c| {
        // Enable WAL mode (Readers don't block writers)
        c.execute_batch("PRAGMA journal_mode = WAL;")?;

        // Set busy timeout to 5000 ms
        c.execute_batch("PRAGMA busy_timeout=5000;")?;
        Ok(())
    });

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

    // Set synchronous to NORMAL for better performance
    conn.execute("PRAGMA synchronous = NORMAL;", []).ok();

    println!("Database initialized and table ensured at {}", db_name);

    pool
}
