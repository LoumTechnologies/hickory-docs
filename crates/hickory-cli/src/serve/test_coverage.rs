use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde_json::{Value, json};

pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    crate::coverage::dispatch(
        state.index.root(),
        serde_json::from_value(json!({"action":"list"})).unwrap(),
    )
    .await
    .map(Json)
    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}
pub async fn action(
    State(state): State<LocalState>,
    Json(request): Json<crate::coverage::Request>,
) -> ApiResult<Json<Value>> {
    crate::coverage::dispatch(state.index.root(), request)
        .await
        .map(Json)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}
