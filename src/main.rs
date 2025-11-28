// src/main.rs

mod db;
mod http;
mod router;

use db::DbPool;

use std::net::{TcpListener, TcpStream};
use std::thread;

fn handle_client(mut stream: TcpStream, pool: DbPool) {
    use http::request::Request;
    use http::response::Response;
    use http::status_code::StatusCode;

    let response = match Request::parse(&mut stream) {
        Some(req) => {
            // Delegate the task to router
            router::route(req, &pool)
        }
        None => Response::new(StatusCode::BadRequest, Some("400 Bad Request".to_string())),
    };

    // Send response
    if let Err(e) = response.send(&mut stream) {
        println!("Failed to send response: {}", e);
    }
}

fn main() {
    let pool = db::init_pool("mkvss.db");

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server running on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let pool_handle = pool.clone();
                thread::spawn(move || {
                    handle_client(stream, pool_handle);
                });
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }
}
