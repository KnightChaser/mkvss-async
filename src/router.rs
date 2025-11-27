// src/router.rs

use super::http::method::Method;
use super::http::request::Request;
use super::http::response::Response;
use super::http::status_code::StatusCode;

// Main router function that directs requests to appropriate handlers
pub fn route(req: Request) -> Response {
    // Split the path into parts
    // e.g., "/keys/1" => ["", "keys", "1"]
    let path_parts: Vec<&str> = req.path.split('/').collect();

    // Route based on the "resouce" (the second part of the path)
    match path_parts.get(1) {
        Some(&"keys") => handle_keys_route(&req, &path_parts),
        _ => Response::new(
            StatusCode::NotFound,
            Some("404 Route Not Found (Did you mean /keys or /keys/{id}?)".to_string()),
        ),
    }
}

// Sub-router specifically for "/keys" routes
fn handle_keys_route(req: &Request, path_parts: &Vec<&str>) -> Response {
    // Check if we have an ID (Length 3: ["", "keys", "{id}"])
    let id = path_parts.get(2);

    match (&req.method, id) {
        // GET /keys/{id}
        (Method::GET, Some(id)) if !id.is_empty() => get_key(id),

        // POST /keys
        (Method::POST, None) | (Method::POST, Some(&"")) => create_key(req.body.clone()),

        // PUT /keys/some_id
        (Method::PUT, Some(id)) if !id.is_empty() => update_key(id, req.body.clone()),

        // DELETE /keys/some_id
        (Method::DELETE, Some(id)) if !id.is_empty() => delete_key(id),

        // Anything else is invalid for this resource
        _ => Response::new(
            StatusCode::BadRequest,
            Some("Invalid Method for Path".to_string()),
        ),
    }
}

// --- MOCK CONTROLLERS (We will connect these to DB later) ---

fn get_key(id: &str) -> Response {
    // TODO: Look up in SQLite
    let msg = format!("(Mock) Fetching value for key: {}", id);
    Response::new(StatusCode::Ok, Some(msg))
}

fn create_key(body: Option<String>) -> Response {
    // TODO: Insert into SQLite
    let content = body.unwrap_or_default();
    let msg = format!("(Mock) Creating key with value: {}", content);
    Response::new(StatusCode::Created, Some(msg))
}

fn update_key(id: &str, body: Option<String>) -> Response {
    // TODO: Update SQLite
    let content = body.unwrap_or_default();
    let msg = format!("(Mock) Updating key {} with value: {}", id, content);
    Response::new(StatusCode::Ok, Some(msg))
}

fn delete_key(id: &str) -> Response {
    // TODO: Delete from SQLite
    let msg = format!("(Mock) Deleting key: {}", id);
    Response::new(StatusCode::Ok, Some(msg))
}
