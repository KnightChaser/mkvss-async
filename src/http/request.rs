// src/http/request.rs

use super::method::Method;

use std::collections::HashMap;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::net::TcpStream;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
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
        let path = parts[1].to_string();

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
                    body = Some(String::from_utf8_lossy(&buffer).to_string());
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
