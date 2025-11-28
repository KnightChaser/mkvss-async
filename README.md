# mkvss-async

> High-Performance Async Key-Value Store Server written in Rust. (Asynchronous implementation of [`KnightChaser/mkvss`](https://github.com/KnightChaser/mkvss.git).)

`mkvss-async` is an educational implementation of a non-blocking, multi-threaded REST API server.

Unlike standard web projects that rely on frameworks like Axum or Actix, this project implements minimum HTTP/1.1 protocol parser, Routing logic, and Connection Handling manually. It demonstrates the raw power of Rust's `async/await` ecosystem by achieving 85k+ Requests Per Second(RPS) on local benchmarks.

API references and other usages are exactly same with the original repository; [`KnightChaser/mkvss`](https://github.com/KnightChaser/mkvss.git).

## Architecture

The server has evolved from a synchronous "Thread-Per-Request"(`mkvss`) model to a modern Asynchronous Event Loop architecture(`mkvss-async`).

### Core Components
* Runtime: [`Tokio`](https://tokio.rs/) (Multi-threaded Work Stealing Scheduler).
* Database: [`SQLx`](https://github.com/launchbadge/sqlx) (Async SQLite with WAL mode).
* Memory Management: extensive use of `Arc<str>` for zero-copy string passing.
* Protocol: Custom implementation of `AsyncBufRead` for parsing HTTP headers and bodies without blocking.

### The "Async" Difference
Instead of spawning a heavy OS thread for every connection, `mkvss-async` spawns a lightweight Tokio Task.
1.  **I/O is Non-Blocking:** When the database is queried, the CPU immediately switches to handling the next incoming HTTP request.
2.  **Low Overhead:** Thousands of connections can be handled by a few OS threads.

## Getting started

### Prerequisites
* Rust (latest stable)
* SQLite3 (bundled)

### Installation & Run
```bash
# Clone the repository
git clone https://github.com/KnightChaser/mkvss-async.git
cd mkvss-async

# Run the server (Initializes DB automatically)
cargo run --release
```

## Performance

Benchmarked using `oha` (Oha HTTP Load Generator) on my PC:

- Concurrency: 50
- Duration: 10 sec
- Database: SQLite in WAL mode

<img width="783" height="1070" alt="image" src="https://github.com/user-attachments/assets/2799ff9d-b2f8-48b4-8cce-96d405527b27" />
<br>

|                | `mkvss`   | `mkvss-async` | Improvement            |
|----------------|-----------|---------------|------------------------|
| Throughput     | 24,257    | 85,925        | 3.54x higher           |
| Avg. Latency   | 2.0562ms  | 0.5790ms      | 3.54x faster           |
| P99 Latency    | 4.9463ms  | 1.6252ms      | ~ 3x faster            |
| P99.9 Latency  | 6.0222ms  | 2.6841ms      | ~ 2.24x faster         |
| P99.99 Latency | 7.0840ms  | 3.7726ms      | ~ 1.88x faster         |
| Success rate   | 99.99093% | 99.99581%     | 2.16x less error-prone |

