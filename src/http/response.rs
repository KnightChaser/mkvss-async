// src/http/response.rs

use super::status_code::StatusCode;

use std::io::Result as IoResult;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct Response {
    pub status_code: StatusCode,
    pub body: Option<String>,
}

impl Response {
    pub fn new(status_code: StatusCode, body: Option<String>) -> Self {
        Response { status_code, body }
    }

    pub async fn send(&self, stream: &mut TcpStream) -> IoResult<()> {
        let body = match &self.body {
            Some(body) => body,
            None => "",
        };

        // Format: "HTTP/1.1 <status_code> <reason_phrase>\r\n\r\n<body>"
        let response_string = format!(
            "HTTP/1.1 {} {}\r\n\r\n{}",
            self.status_code as u16,
            self.status_code.reason_phrase(),
            body
        );

        // Await the write
        stream.write_all(response_string.as_bytes()).await?;

        // Flush the stream to ensure all data is sent
        stream.flush().await
    }
}
