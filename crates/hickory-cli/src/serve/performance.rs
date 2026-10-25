use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use serde_json::Value;
pub async fn action(
    State(state): State<LocalState>,
    Json(request): Json<crate::performance::Request>,
) -> ApiResult<Json<Value>> {
    crate::performance::dispatch(state.index.root(), request)
        .await
        .map(Json)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}
