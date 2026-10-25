<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/test_coverage.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Coverage retains its measured scope](../../../../../../guarantees/verification/coverage-retains-its-measured-scope/contract.md)

Captured SHA-256: `27ae5fb8a9e1c18caccc282f77383c5415a1fce743a5e114cb72f9e658235e80`.

<book:entry path="crates/hickory-cli/src/serve/test_coverage.rs">
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
</book:entry>

</book:doc>
