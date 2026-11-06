use super::LocalState;
use axum::{Json, extract::State};

pub async fn status(State(state): State<LocalState>) -> Json<crate::checks::Snapshot> {
    Json(crate::checks::snapshot(state.index.root()).await)
}
