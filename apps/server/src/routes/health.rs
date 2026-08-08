//! GET /api/health and GET /api/executor.

use std::collections::HashMap;

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use utoipa::ToSchema;

use crate::AppState;
use crate::config::ExecutorKind;

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthOut {
    pub ok: bool,
    pub executor: String,
    pub db: bool,
}

#[utoipa::path(
    get,
    path = "/api/health",
    responses((status = 200, description = "liveness + DB reachability", body = HealthOut)),
    tag = "health"
)]
pub async fn health(State(state): State<AppState>) -> Json<HealthOut> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    Json(HealthOut {
        ok: db_ok,
        executor: state.config.executor.as_str().to_string(),
        db: db_ok,
    })
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ExecutorOut {
    pub kind: String,
    pub images: Option<HashMap<String, String>>,
}

/// `GET /api/executor` — where cells run (api.md). `images` is the image
/// ref → store path map on canopy; `null` on local, where the image
/// attribute is recorded provenance, not an enforced sandbox.
#[utoipa::path(
    get,
    path = "/api/executor",
    responses((status = 200, description = "active executor kind + image map", body = ExecutorOut)),
    tag = "health"
)]
pub async fn executor(State(state): State<AppState>) -> Json<ExecutorOut> {
    let kind = state.config.executor;
    let images = match kind {
        // Docker honours `image=` directly — the ref IS the environment, so
        // there is no ref→store-path indirection to report.
        ExecutorKind::Local | ExecutorKind::Docker => None,
        ExecutorKind::Canopy => Some(crate::executor::canopy_image_map()),
    };
    Json(ExecutorOut {
        kind: kind.as_str().to_string(),
        images,
    })
}
