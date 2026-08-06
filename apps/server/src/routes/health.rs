//! GET /api/health.

use axum::Json;
use axum::extract::State;
use serde_json::{Value, json};

use crate::AppState;

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db).await.is_ok();
    Json(json!({
        "ok": db_ok,
        "executor": state.config.executor.as_str(),
        "db": db_ok,
    }))
}
