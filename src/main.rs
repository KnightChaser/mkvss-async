// src/main.rs

mod db;
mod http;
mod router;

use db::DbPool;

use tokio::net::{TcpListener, TcpStream};

async fn handle_client(mut stream: TcpStream, pool: DbPool) {
    use http::request::Request;
    use http::response::Response;
    use http::status_code::StatusCode;

    let response = match Request::parse(&mut stream).await {
        Some(req) => router::route(req, &pool).await,
        None => Response::new(StatusCode::BadRequest, Some("400 Bad Request".to_string())),
    };

    if let Err(e) = response.send(&mut stream).await {
        eprintln!("Failed to send response: {}", e);
    }
}

#[tokio::main]
async fn main() {
    let pool = db::init_pool("sqlite::mkvss_async.db").await;

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");
    println!("Server listening on {}", addr);

    loop {
        // Async accept
        let (stream, _) = listener.accept().await.unwrap();
        let pool = pool.clone();

        tokio::spawn(async move {
            handle_client(stream, pool).await;
        });
    }
}
