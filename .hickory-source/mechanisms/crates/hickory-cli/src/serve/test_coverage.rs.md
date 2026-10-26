<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/test_coverage.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cdbb6e20bcf7"></a>
## `crates/hickory-cli/src/serve/test_coverage.rs`

Related contracts: [Coverage retains its measured scope](../../../../../guarantees/verification/coverage-retains-its-measured-scope/contract.md).

<a id="f-cdbb6e20bcf7-list"></a>
### `list`

<storycode:copy id="f-cdbb6e20bcf7-list">
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
</storycode:copy>
<a id="f-cdbb6e20bcf7-action"></a>
### `action`

<storycode:copy id="f-cdbb6e20bcf7-action">
pub async fn action(
    State(state): State<LocalState>,
    Json(request): Json<crate::coverage::Request>,
) -> ApiResult<Json<Value>> {
    crate::coverage::dispatch(state.index.root(), request)
        .await
        .map(Json)
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/test_coverage.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/test_coverage.rs"><storycode:paste select="#f-cdbb6e20bcf7-list"/><storycode:paste select="#f-cdbb6e20bcf7-action"/></storycode:file>

</storycode:doc>
