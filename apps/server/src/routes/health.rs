//! GET /api/health and GET /api/executor.

use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};

use crate::AppState;
use crate::config::ExecutorKind;

#[utoipa::path(
    get,
    path = "/api/health",
    responses((status = 200, description = "liveness + DB reachability", body = Value)),
    tag = "health"
)]
pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    Json(json!({
        "ok": db_ok,
        "executor": state.config.executor.as_str(),
        "db": db_ok,
    }))
}

/// `GET /api/executor` — where cells run (api.md). `images` is the image
/// ref → store path map on canopy; `null` on local, where the image
/// attribute is recorded provenance, not an enforced sandbox.
#[utoipa::path(
    get,
    path = "/api/executor",
    responses((status = 200, description = "active executor kind + image map", body = Value)),
    tag = "health"
)]
pub async fn executor(State(state): State<AppState>) -> Json<Value> {
    let kind = state.config.executor;
    let images = match kind {
        // Docker honours `image=` directly — the ref IS the environment, so
        // there is no ref→store-path indirection to report.
        ExecutorKind::Local | ExecutorKind::Docker => Value::Null,
        ExecutorKind::Canopy => json!(crate::executor::canopy_image_map()),
    };
    Json(json!({ "kind": kind.as_str(), "images": images }))
}
