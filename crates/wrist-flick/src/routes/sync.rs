use axum::http::StatusCode;
use axum::response::Json;

/// Sync is handled by waymark's gossip/QUIC layer, not via direct WebSocket.
/// These stubs remain for API compatibility during the transition.

#[derive(serde::Serialize)]
pub struct SyncStatusResponse {
    pub status: String,
    pub message: String,
}

pub async fn status() -> Json<SyncStatusResponse> {
    Json(SyncStatusResponse {
        status: "ok".into(),
        message: "Sync handled by waymark layer".into(),
    })
}

pub async fn ws_handler() -> StatusCode {
    // WebSocket-based grove sync replaced by waymark gossip protocol
    StatusCode::GONE
}

pub async fn state_vector() -> StatusCode {
    // State vector sync replaced by waymark gossip protocol
    StatusCode::GONE
}
