<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/performance.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `89ea903526e651706879b64c4affb122f51b420c7e3a37fac1092982da8a48a1`.

<book:entry path="crates/hickory-cli/src/serve/performance.rs">
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
</book:entry>

</book:doc>
