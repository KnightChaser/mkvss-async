// src/router.rs

use super::db::DbPool;
use super::http::method::Method;
use super::http::request::Request;
use super::http::response::Response;
use super::http::status_code::StatusCode;

use rusqlite::params;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize)]
struct CreateKeyRequest {
    key: String,
    value: String,
}

// Main router function that directs requests to appropriate handlers
pub fn route(req: Request, pool: &DbPool) -> Response {
    // Split the path into parts
    // e.g., "/keys/1" => ["", "keys", "1"]
    let path_parts: Vec<&str> = req.path.split('/').collect();

    // Route based on the "resouce" (the second part of the path)
    match path_parts.get(1) {
        Some(&"keys") => handle_keys_route(&req, &path_parts, pool),
        _ => Response::new(
            StatusCode::NotFound,
            Some("404 Route Not Found (Did you mean /keys or /keys/{id}?)".to_string()),
        ),
    }
}

// Sub-router specifically for "/keys" routes
fn handle_keys_route(req: &Request, path_parts: &Vec<&str>, pool: &DbPool) -> Response {
    // Check if we have an ID (Length 3: ["", "keys", "{id}"])
    let id = path_parts.get(2);

    match (&req.method, id) {
        // GET /keys/{id}
        (Method::GET, Some(id)) if !id.is_empty() => get_key(id, pool),

        // POST /keys (No ID, because the ID is inside the JSON body now)
        (Method::POST, None) | (Method::POST, Some(&"")) => create_key(req.body.clone(), pool),

        // PUT /keys/{id}
        (Method::PUT, Some(id)) if !id.is_empty() => update_key(id, req.body.clone(), pool),

        // DELETE /keys/{id}
        (Method::DELETE, Some(id)) if !id.is_empty() => delete_key(id, pool),

        // Anything else is invalid for this resource
        _ => Response::new(
            StatusCode::BadRequest,
            Some("Invalid Method for Path".to_string()),
        ),
    }
}

fn get_key(id: &str, pool: &DbPool) -> Response {
    // Get a connection from the pool
    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => {
            return Response::new(
                StatusCode::InternalServerError,
                Some("500 Internal Server Error: DB Connection Failed".to_string()),
            );
        }
    };

    // 2. Prepare SQL to fetch the key
    let result: Result<String, _> = conn.query_row(
        "SELECT value FROM kv_store WHERE key = ?1",
        params![id],
        |row| row.get(0),
    );

    match result {
        Ok(value) => Response::new(StatusCode::Ok, Some(value)),
        Err(_) => Response::new(
            StatusCode::NotFound,
            Some(format!("404 Key Not Found: {}", id)),
        ),
    }
}

fn create_key(body: Option<String>, pool: &DbPool) -> Response {
    let body_text = match body {
        Some(b) => b,
        None => {
            return Response::new(
                StatusCode::BadRequest,
                Some("400 Bad Request: Missing Body".to_string()),
            );
        }
    };

    // Parse JSON body using serde
    let request_data: CreateKeyRequest = match serde_json::from_str(&body_text) {
        Ok(data) => data,
        Err(_) => {
            // malformed JSON
            return Response::new(
                StatusCode::BadRequest,
                Some("400 Bad Request: Invalid JSON".to_string()),
            );
        }
    };

    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => {
            return Response::new(
                StatusCode::InternalServerError,
                Some("500 Internal Server Error: DB Connection Failed".to_string()),
            );
        }
    };

    let result = conn.execute(
        "INSERT INTO kv_store (key, value) VALUES (?1, ?2)",
        params![request_data.key, request_data.value],
    );

    match result {
        Ok(_) => Response::new(
            StatusCode::Created,
            Some(format!("Key {} created successfully", request_data.key)),
        ),
        Err(rusqlite::Error::SqliteFailure(err_code, _)) => {
            // NOTE:
            // Detailed check for duplicate keys (SQL constraint violations)
            if err_code.code == rusqlite::ffi::ErrorCode::ConstraintViolation {
                Response::new(
                    StatusCode::BadRequest,
                    Some(format!(
                        "409 Conflict: Key {} already exists",
                        request_data.key
                    )),
                )
            } else {
                Response::new(
                    StatusCode::InternalServerError,
                    Some("500 Internal Server Error: Failed to create key".to_string()),
                )
            }
        }
        Err(_) => Response::new(
            StatusCode::InternalServerError,
            Some("500 Internal Server Error: Failed to create key".to_string()),
        ),
    }
}

fn update_key(id: &str, body: Option<String>, pool: &DbPool) -> Response {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => {
            return Response::new(
                StatusCode::InternalServerError,
                Some("500 Internal Server Error: DB Connection Failed".to_string()),
            );
        }
    };

    let value = body.unwrap_or_default();

    let result = conn.execute(
        "INSERT OR REPLACE INTO kv_store (key, value) VALUES (?1, ?2)",
        params![id, value],
    );

    match result {
        Ok(_) => Response::new(
            StatusCode::Ok,
            Some(format!("Key {} updated successfully", id)),
        ),
        Err(_) => Response::new(
            StatusCode::InternalServerError,
            Some("500 Internal Server Error: Failed to update key".to_string()),
        ),
    }
}

fn delete_key(id: &str, pool: &DbPool) -> Response {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(_) => {
            return Response::new(
                StatusCode::InternalServerError,
                Some("500 Internal Server Error: DB Connection Failed".to_string()),
            );
        }
    };

    let result = conn.execute("DELETE FROM kv_store WHERE key = ?1", params![id]);

    match result {
        Ok(rows_affected) => {
            if rows_affected > 0 {
                Response::new(
                    StatusCode::Ok,
                    Some(format!("Key {} deleted successfully", id)),
                )
            } else {
                Response::new(
                    StatusCode::NotFound,
                    Some(format!("404 Key Not Found: {}", id)),
                )
            }
        }
        Err(_) => Response::new(
            StatusCode::InternalServerError,
            Some("500 Internal Server Error: Failed to delete key".to_string()),
        ),
    }
}
