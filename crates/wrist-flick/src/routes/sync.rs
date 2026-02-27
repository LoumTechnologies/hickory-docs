//! Sync status and device linking REST endpoints.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::app_state::AppState;

// MARK: - VSLICE: DriftSignalSync @Routes

#[derive(serde::Serialize)]
pub struct SyncStatusResponse {
    pub status: String,
    pub node_id: Option<String>,
}

pub async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let core = state.core.lock().await;
    let connection_status = core.connection_status();
    let node_id = core.node_id();
    Json(SyncStatusResponse {
        status: connection_status,
        node_id,
    })
}

#[derive(Deserialize)]
pub struct StartSyncRequest {
    pub relay_url: Option<String>,
}

pub async fn start_sync(
    State(state): State<AppState>,
    Json(body): Json<StartSyncRequest>,
) -> impl IntoResponse {
    let mut core = state.core.lock().await;
    let mut config = drift_signal_core::SyncConfig::default();
    if let Some(relay) = body.relay_url {
        config = config.with_cloud(relay);
    }
    match core.start_sync(config).await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "started" })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn stop_sync(State(state): State<AppState>) -> impl IntoResponse {
    let mut core = state.core.lock().await;
    match core.stop_sync().await {
        Ok(()) => (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "stopped" })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn generate_link_token(State(state): State<AppState>) -> impl IntoResponse {
    let core = state.core.lock().await;
    let token = core.generate_link_token();
    Json(serde_json::json!({ "token": token }))
}
