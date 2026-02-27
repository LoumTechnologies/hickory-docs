//! Capability REST endpoints.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::Value;

use crate::app_state::AppState;

// MARK: - VSLICE: DriftSignalCapabilities @Routes

pub async fn list_registered(State(state): State<AppState>) -> impl IntoResponse {
    let core = state.core.lock().await;
    match core.list_registered_capabilities() {
        Ok(caps) => Json(caps).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn find_providers(
    State(state): State<AppState>,
    Path(cap_type): Path<String>,
) -> impl IntoResponse {
    let core = state.core.lock().await;
    match core.find_capability_providers(&cap_type) {
        Ok(providers) => Json(providers).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct InvokeRequest {
    pub cap_type: String,
    pub target_device: Option<String>,
    #[serde(default)]
    pub params: Value,
}

pub async fn invoke(
    State(state): State<AppState>,
    Json(body): Json<InvokeRequest>,
) -> impl IntoResponse {
    let mut core = state.core.lock().await;
    match core
        .invoke_capability(&body.cap_type, body.target_device.as_deref(), body.params)
        .await
    {
        Ok(invocation_id) => (
            StatusCode::ACCEPTED,
            Json(serde_json::json!({ "invocation_id": invocation_id })),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn get_invocation(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let core = state.core.lock().await;
    match core.get_invocation(&id) {
        Ok(Some(inv)) => Json(inv).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
