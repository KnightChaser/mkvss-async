// src/router.rs

use super::db::DbPool;
use super::http::method::Method;
use super::http::request::Request;
use super::http::response::Response;
use super::http::status_code::StatusCode;

use serde::Deserialize;
use serde_json;
use sqlx::error::ErrorKind;

use std::sync::Arc;

#[derive(Deserialize)]
struct CreateKeyRequest {
    key: String,
    value: String,
}

// Main router function that directs requests to appropriate handlers
pub async fn route(req: Request, pool: &DbPool) -> Response {
    // Split the path into parts
    // e.g., "/keys/1" => ["", "keys", "1"]
    let path_parts: Vec<&str> = req.path.split('/').collect();

    // Route based on the "resouce" (the second part of the path)
    match path_parts.get(1) {
        // Await the sub-router
        Some(&"keys") => handle_keys_route(&req, &path_parts, pool).await,
        _ => Response::new(
            StatusCode::NotFound,
            Some("404 Route Not Found (Did you mean /keys or /keys/{id}?)".to_string()),
        ),
    }
}

// Asynchronous sub-router specifically for "/keys" routes
async fn handle_keys_route(req: &Request, path_parts: &Vec<&str>, pool: &DbPool) -> Response {
    // Check if we have an ID (Length 3: ["", "keys", "{id}"])
    let id = path_parts.get(2);

    match (&req.method, id) {
        // GET /keys/{id}
        (Method::GET, Some(id)) if !id.is_empty() => get_key(id, pool).await,

        // POST /keys (No ID, because the ID is inside the JSON body now)
        (Method::POST, None) | (Method::POST, Some(&"")) => {
            create_key(req.body.clone(), pool).await
        }

        // PUT /keys/{id}
        (Method::PUT, Some(id)) if !id.is_empty() => update_key(id, req.body.clone(), pool).await,

        // DELETE /keys/{id}
        (Method::DELETE, Some(id)) if !id.is_empty() => delete_key(id, pool).await,

        // Anything else is invalid for this resource
        _ => Response::new(
            StatusCode::BadRequest,
            Some("Invalid Method for Path".to_string()),
        ),
    }
}

async fn get_key(id: &str, pool: &DbPool) -> Response {
    // Prepare SQL to fetch the key
    let result: Result<Option<String>, sqlx::Error> =
        sqlx::query_scalar("SELECT value FROM kv_store WHERE key = ?")
            .bind(id)
            .fetch_optional(pool)
            .await;

    match result {
        Ok(Some(value)) => Response::new(StatusCode::Ok, Some(value)),
        Ok(None) => Response::new(StatusCode::NotFound, Some("Key not found".to_string())),
        Err(_) => Response::new(
            StatusCode::InternalServerError,
            Some("Database error".to_string()),
        ),
    }
}

async fn create_key(body: Option<Arc<str>>, pool: &DbPool) -> Response {
    let body_text = match body {
        Some(text) => text,
        None => {
            return Response::new(
                StatusCode::BadRequest,
                Some("Missing request body".to_string()),
            );
        }
    };

    let request_data: CreateKeyRequest = match serde_json::from_str(&body_text) {
        Ok(data) => data,
        Err(_) => {
            return Response::new(
                StatusCode::BadRequest,
                Some("Invalid JSON body".to_string()),
            );
        }
    };

    let result = sqlx::query("INSERT INTO kv_store (key, value) VALUES (?, ?)")
        .bind(&request_data.key)
        .bind(&request_data.value)
        .execute(pool)
        .await;

    match result {
        Ok(_) => Response::new(
            StatusCode::Created,
            Some(format!("Key '{}' created", request_data.key)),
        ),

        // Handle unique constraint violation
        Err(sqlx::Error::Database(db_err)) if db_err.kind() == ErrorKind::UniqueViolation => {
            Response::new(
                StatusCode::BadRequest,
                Some(format!("Key '{}' already exists", request_data.key)),
            )
        }
        Err(e) => Response::new(
            StatusCode::InternalServerError,
            Some(format!("Failed to create key: {}", e)),
        ),
    }
}

async fn update_key(id: &str, body: Option<Arc<str>>, pool: &DbPool) -> Response {
    // Dereference the body(Option<Arc<str>>) to Option<&str>,
    // defaulting to an empty string if None
    let value = body.as_deref().unwrap_or("");

    let result = sqlx::query("INSERT OR REPLACE INTO kv_store (key, value) VALUES (?, ?)")
        .bind(id)
        .bind(value)
        .execute(pool)
        .await;

    match result {
        Ok(_) => Response::new(StatusCode::Ok, Some(format!("Key '{}' updated", id))),
        Err(e) => Response::new(
            StatusCode::InternalServerError,
            Some(format!("Failed to update key: {}", e)),
        ),
    }
}

async fn delete_key(id: &str, pool: &DbPool) -> Response {
    let result = sqlx::query("DELETE FROM kv_store WHERE key = ?")
        .bind(id)
        .execute(pool)
        .await;

    match result {
        Ok(r) => {
            if r.rows_affected() > 0 {
                Response::new(StatusCode::Ok, Some(format!("Key '{}' deleted", id)))
            } else {
                Response::new(StatusCode::NotFound, Some("Key not found".to_string()))
            }
        }
        Err(e) => Response::new(
            StatusCode::InternalServerError,
            Some(format!("Failed to delete key: {}", e)),
        ),
    }
}
