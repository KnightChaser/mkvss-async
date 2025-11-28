// src/http/request.rs

use super::method::Method;

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: Arc<str>,
    pub headers: HashMap<String, String>,
    pub body: Option<Arc<str>>,
}

impl Request {
    pub async fn parse(stream: &mut TcpStream) -> Option<Self> {
        // Read request line
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();

        if reader.read_line(&mut request_line).await.is_err() || request_line.is_empty() {
            return None;
        }

        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() < 2 {
            return None;
        }

        let method = Method::from_str(parts[0]).ok()?;

        // NOTE:
        // Convert String to Arc<str> to avoid unnecessary allocations at further stages
        let path = Arc::from(parts[1]);

        // Read headers into HashMap
        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await.ok()?;

            if line == "\r\n" || line.is_empty() {
                break;
            }

            // Parse "Key: Value"
            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        // Ready body
        let mut body = None;
        if let Some(content_length) = headers.get("Content-Length") {
            if let Ok(length) = content_length.parse::<usize>() {
                if length > 0 {
                    let mut buffer = vec![0; length];
                    reader.read_exact(&mut buffer).await.ok()?;

                    // Convert String -> Arc<str>
                    let body_string = String::from_utf8_lossy(&buffer).into_owned();
                    body = Some(Arc::from(body_string));
                }
            }
        }

        Some(Request {
            method,
            path,
            headers,
            body,
        })
    }
}
