// mergemint-backend/src/main.rs
//
// Application entry-point: builds the Axum router with middleware and starts
// the HTTP server.
//
// ## Request body size limits and timeout middleware (#476)
//
// Two middleware layers are added to the router to protect the service from
// slow clients and excessively large payloads:
//
//   * `RequestBodyLimitLayer` — rejects bodies larger than `MAX_BODY_BYTES`.
//     Without this, a malicious client could stream an arbitrarily large body
//     and exhaust server memory before any handler logic runs.
//
//   * `TimeoutLayer` — cancels any request (including body reads and handler
//     execution) that takes longer than `REQUEST_TIMEOUT`.  This prevents slow
//     clients or downstream Horizon calls from holding connections indefinitely
//     and starving the thread pool.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    routing::post,
    Router,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
};

mod db;
mod routes;

use db::new_shared_db;
use routes::tx::{resolve_dispute, self_claim, AppState};

/// Maximum allowed request body size (1 MiB).
const MAX_BODY_BYTES: usize = 1 * 1024 * 1024;

/// Maximum wall-clock time allowed for a single request, including body reads
/// and handler execution.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() {
    let shared_db = new_shared_db();
    let state = Arc::new(AppState { db: shared_db });

    let app = Router::new()
        .route("/tx/resolve-dispute", post(resolve_dispute))
        .route("/tx/self-claim", post(self_claim))
        .with_state(state)
        // Guard against slow-loris / oversized-body attacks (#476).
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        // Cancel requests that exceed the wall-clock budget (#476).
        .layer(TimeoutLayer::new(REQUEST_TIMEOUT));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind TCP listener");

    println!("mergemint-backend listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("server error");
}
