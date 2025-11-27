# mkvss

> Multithreaded Key-Value Storage Server, written in Rust

## Specification

- SQL setup

```sql
CREATE TABLE IF NOT EXISTS store (
    key TEXT PRIMARY KEY,
    value TEXT,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

- Mapping REST to SQL
| REST Method | Endpoint       | SQL Query                                      | Expected Code |
|-------------|----------------|------------------------------------------------|---------------|
| GET         | `/keys/{id}`     | `SELECT value FROM store WHERE key = ?;`         | `200`, `404`      |
| POST        | `/keys`          | `INSERT INTO store (key, value) VALUES (?, ?);`  | `201`, `409`      |
| PUT         | `/keys/{id}`     | `UPDATE store SET value = ? WHERE key = ?;`      | `200`, `204`      |
| DELETE      | `/keys/{id}`     | `DELETE FROM store WHERE key = ?;`               | `200`, `204`      |
