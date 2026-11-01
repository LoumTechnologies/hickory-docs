//! Usage ingestion and balance query endpoints.

use std::sync::Arc;

use axum::extract::State;
use axum::http::HeaderMap;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use crate::billing::{FEE_RATE, estimate_cost_usd};
use crate::db::{UsageEventRow, insert_usage_events, monthly_totals};
use crate::error::AppError;
use crate::jwt::verify_user_token;

// ---------------------------------------------------------------------------
// POST /api/usage
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct UsageBatch {
    events: Vec<UsageEventPayload>,
}

#[derive(Deserialize)]
pub struct UsageEventPayload {
    machine_id: String,
    timestamp: DateTime<Utc>,
    provider: String,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cache_creation_tokens: u64,
    cache_read_tokens: u64,
}

pub async fn ingest_usage(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(batch): Json<UsageBatch>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Auth is optional.  Anonymous events are stored without a user_id.
    let user_id = extract_user_id(&state.jwt_secret, &headers);

    let rows: Vec<UsageEventRow> = batch
        .events
        .into_iter()
        .map(|ev| {
            let cost = estimate_cost_usd(
                &ev.provider,
                &ev.model,
                ev.input_tokens,
                ev.output_tokens,
                ev.cache_creation_tokens,
                ev.cache_read_tokens,
            );
            UsageEventRow {
                machine_id: ev.machine_id,
                call_timestamp: ev.timestamp,
                provider: ev.provider,
                model: ev.model,
                input_tokens: ev.input_tokens as i64,
                output_tokens: ev.output_tokens as i64,
                cache_creation_tokens: ev.cache_creation_tokens as i64,
                cache_read_tokens: ev.cache_read_tokens as i64,
                estimated_cost_usd: cost,
                fee_usd: cost * FEE_RATE,
            }
        })
        .collect();

    insert_usage_events(&state.db, user_id, rows)
        .await
        .map_err(AppError::internal)?;

    Ok(Json(serde_json::json!({ "ok": true })))
}

// ---------------------------------------------------------------------------
// GET /api/balance
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct BalanceResponse {
    this_month_input_tokens: i64,
    this_month_output_tokens: i64,
    this_month_fee_usd: f64,
}

pub async fn get_balance(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<BalanceResponse>, AppError> {
    let user_id = extract_user_id(&state.jwt_secret, &headers)
        .ok_or_else(|| AppError::unauthorized("authentication required"))?;

    let totals = monthly_totals(&state.db, user_id)
        .await
        .map_err(AppError::internal)?;

    Ok(Json(BalanceResponse {
        this_month_input_tokens: totals.total_input_tokens,
        this_month_output_tokens: totals.total_output_tokens,
        this_month_fee_usd: totals.total_fee_usd,
    }))
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn extract_user_id(jwt_secret: &str, headers: &HeaderMap) -> Option<Uuid> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    let claims = verify_user_token(jwt_secret, token).ok()?;
    claims.sub.parse().ok()
}
