/// MergeMint Backend
///
/// Axum HTTP server that exposes the MergeMint API.
///
/// Versioning strategy (issue #483)
/// ---------------------------------
/// All stable endpoints live under `/api/v1/`. The unprefixed paths
/// (`/api/bounties`, `/api/tx/*`) are kept as deprecated aliases during the
/// migration window so existing clients continue to work. Aliases will be
/// removed in a future minor release — clients should migrate to `/api/v1/`.
///
/// Route inventory
/// ---------------
/// GET  /api/v1/bounties                       list bounties
/// GET  /api/v1/bounties/stream                SSE push channel (issue #482)
/// GET  /api/v1/bounties/assignee/{address}    bounties by assignee (issue #481)
/// GET  /api/v1/bounties/{id}                  single bounty
/// POST /api/v1/bounties                       create bounty
/// POST /api/v1/bounties/{id}/claim            claim bounty
/// GET  /health                                health check
///
/// Deprecated aliases (same handlers, will emit Deprecation header in future)
/// GET  /api/bounties                → /api/v1/bounties
/// GET  /api/bounties/stream         → /api/v1/bounties/stream
/// GET  /api/bounties/assignee/{a}   → /api/v1/bounties/assignee/{a}

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::env;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod routes;

use db::Db;
use routes::bounties;

// ── Shared application state ──────────────────────────────────────────────────

/// State cloned into every Axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Database client (backed by a `sqlx::PgPool`).
    pub db: Db,
    /// Broadcast channel used to push bounty IDs to SSE subscribers whenever
    /// an on-chain state change is indexed. The indexer calls `.send(id)` and
    /// every open `/api/v1/bounties/stream` connection receives the event.
    pub bounty_broadcast: broadcast::Sender<String>,
}

// ── Router construction ───────────────────────────────────────────────────────

fn build_router(state: AppState) -> Router {
    // Versioned routes under /api/v1
    let v1_bounties = Router::new()
        .route("/", get(bounties::list_bounties).post(bounties::list_bounties))
        // NOTE: /stream and /assignee/:address must be registered before /:id
        // so Axum's router gives them priority over the dynamic segment.
        .route("/stream", get(bounties::bounty_stream))
        .route("/assignee/:address", get(bounties::list_bounties_by_assignee))
        .route("/:id/claim", post(bounties::claim_bounty));

    let v1 = Router::new().nest("/bounties", v1_bounties);

    // Deprecated /api/* aliases — same handlers, same state
    let legacy_bounties = Router::new()
        .route("/", get(bounties::list_bounties))
        .route("/stream", get(bounties::bounty_stream))
        .route("/assignee/:address", get(bounties::list_bounties_by_assignee))
        .route("/:id/claim", post(bounties::claim_bounty));

    let legacy = Router::new().nest("/bounties", legacy_bounties);

    Router::new()
        .nest("/api/v1", v1)
        .nest("/api", legacy)
        .route("/health", get(health_handler))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({ "status": "ok" }))
}

// ── Entry point ───────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Broadcast channel with a buffer of 256 messages. Slow SSE subscribers
    // that lag behind will receive a `RecvError::Lagged` and should reconnect.
    let (tx, _) = broadcast::channel::<String>(256);

    let state = AppState {
        db: Db::new(pool),
        bounty_broadcast: tx,
    };

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("mergemint-backend listening on {}", bind_addr);

    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
