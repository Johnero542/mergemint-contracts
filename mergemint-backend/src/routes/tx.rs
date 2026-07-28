// mergemint-backend/src/routes/tx.rs
//
// Transaction / bounty route handlers.

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::{acquire_db, SharedDb};

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

pub struct AppState {
    pub db: SharedDb,
}

// ---------------------------------------------------------------------------
// Error helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: u16,
    pub message: String,
}

impl AppError {
    pub fn bad_request(msg: impl Into<String>) -> (StatusCode, Json<AppError>) {
        let err = AppError {
            code: 400,
            message: msg.into(),
        };
        (StatusCode::BAD_REQUEST, Json(err))
    }

    pub fn not_found(msg: impl Into<String>) -> (StatusCode, Json<AppError>) {
        let err = AppError {
            code: 404,
            message: msg.into(),
        };
        (StatusCode::NOT_FOUND, Json(err))
    }
}

// ---------------------------------------------------------------------------
// Domain types (minimal stubs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounty {
    pub id: String,
    pub creator: String,
    pub amount: u64,
    /// Unix timestamp (seconds) after which a self-claim is considered stale.
    pub expires_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct ResolveDisputeRequest {
    pub bounty_id: String,
    pub arbitrator: String,
    pub winner: String,
}

#[derive(Debug, Serialize)]
pub struct ResolveDisputeResponse {
    pub ok: bool,
    pub xdr: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SelfClaimRequest {
    pub bounty_id: String,
    pub claimant: String,
}

// ---------------------------------------------------------------------------
// resolve_dispute handler (#474 + #475)
// ---------------------------------------------------------------------------

/// Resolve a disputed bounty by paying out the `winner`.
///
/// ## Short-circuit precheck (#474)
///
/// Only the bounty creator is authorised to act as arbitrator.  We reject
/// requests where `arbitrator != bounty.creator` *before* building XDR so we
/// never waste Horizon round-trips on unauthorised calls.
pub async fn resolve_dispute(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResolveDisputeRequest>,
) -> Result<Json<ResolveDisputeResponse>, (StatusCode, Json<AppError>)> {
    let bounty = {
        let db = acquire_db(&state.db);
        let raw = db
            .records
            .get(&req.bounty_id)
            .ok_or_else(|| AppError::not_found("bounty not found"))?
            .clone();
        serde_json::from_str::<Bounty>(&raw)
            .map_err(|_| AppError::bad_request("corrupt bounty record"))?
    };

    // -- precheck: only the bounty creator may arbitrate (#474) --------------
    if req.arbitrator != bounty.creator {
        return Err(AppError::bad_request(
            "only the bounty creator may act as arbitrator",
        ));
    }

    // Build the payout XDR (stub — real implementation invokes Stellar SDK).
    let xdr = build_payout_xdr(&bounty, &req.winner);

    Ok(Json(ResolveDisputeResponse {
        ok: true,
        xdr: Some(xdr),
    }))
}

/// Stub XDR builder.  Replace with real Stellar `TransactionBuilder` logic.
fn build_payout_xdr(bounty: &Bounty, winner: &str) -> String {
    format!(
        "XDR:bounty={},winner={},amount={}",
        bounty.id, winner, bounty.amount
    )
}

// ---------------------------------------------------------------------------
// self_claim handler (#475)
// ---------------------------------------------------------------------------

/// Allow a claimant to self-claim a bounty after the creator has not resolved
/// it within the agreed window.
///
/// ## Staleness window note (#475)
///
/// The `expires_at` field on the bounty marks the Unix timestamp (in seconds)
/// after which the bounty is considered unresolved by the creator and the
/// claimant may collect it unilaterally.  We check the current time against
/// this threshold *before* any on-chain interaction to avoid wasting gas on
/// claims that would be rejected by the contract anyway.
///
/// The window is set by the bounty creator at creation time and is stored
/// on-chain; this server-side check is an optimistic guard only — the contract
/// enforces the same rule authoritatively.
pub async fn self_claim(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SelfClaimRequest>,
) -> Result<Json<ResolveDisputeResponse>, (StatusCode, Json<AppError>)> {
    let bounty = {
        let db = acquire_db(&state.db);
        let raw = db
            .records
            .get(&req.bounty_id)
            .ok_or_else(|| AppError::not_found("bounty not found"))?
            .clone();
        serde_json::from_str::<Bounty>(&raw)
            .map_err(|_| AppError::bad_request("corrupt bounty record"))?
    };

    // -- self-claim staleness precheck (#475) ---------------------------------
    // The staleness window is the period between bounty creation and
    // `expires_at`.  A claimant may only self-claim once that window has
    // elapsed, i.e. when the current time is strictly past `expires_at`.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    if now <= bounty.expires_at {
        return Err(AppError::bad_request(
            "self-claim not yet available: staleness window has not elapsed",
        ));
    }

    let xdr = build_payout_xdr(&bounty, &req.claimant);

    Ok(Json(ResolveDisputeResponse {
        ok: true,
        xdr: Some(xdr),
    }))
}
