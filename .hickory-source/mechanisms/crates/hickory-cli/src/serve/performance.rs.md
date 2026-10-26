<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/performance.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-600004d8aa2e"></a>
## `crates/hickory-cli/src/serve/performance.rs`

<a id="f-600004d8aa2e-action"></a>
### `action`

<storycode:copy id="f-600004d8aa2e-action">
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
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/performance.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/performance.rs"><storycode:paste select="#f-600004d8aa2e-action"/></storycode:file>

</storycode:doc>
