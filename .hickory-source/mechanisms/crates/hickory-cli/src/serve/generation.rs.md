<gencode:doc xmlns:gencode="http://www.hickorydocs.com/1.0">
# Preview and publish an explicitly selected project

For: maintainers changing safe document opening and explicit generation.

The session confines a user-selected root and entry. Neither the current document nor its containing directory silently supplies generation context. Private practice has no ordinary publication authority.

<gencode:copy id="generation-http-context">
//! Generation is an explicit root/entry operation, never a side effect of reads.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{Json, extract::State};
use hick_project_env::generation::{GenerationContext, GenerationPlan};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Request {
    pub root: String,
    pub entry: String,
    pub revision: Option<String>,
}

fn capture(state: &LocalState, request: &Request) -> ApiResult<GenerationContext> {
    if state.private_prediction.is_some() {
        return Err(ApiError::forbidden(
            "Ordinary generation is unavailable in private practice",
        ));
    }
    let scope = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    // Root is chosen relative to this session, never relative to a scanned page.
    let root = if request.root == "." {
        scope.clone()
    } else {
        hick_lang::validate_project_path(&request.root).map_err(ApiError::unprocessable)?;
        scope
            .join(&request.root)
            .canonicalize()
            .map_err(|e| ApiError::unprocessable(e.to_string()))?
    };
    if !root.starts_with(&scope) {
        return Err(ApiError::forbidden(
            "Generation root escapes the selected folder",
        ));
    }
    GenerationContext::capture(&root, &request.entry).map_err(ApiError::unprocessable)
}

</gencode:copy>

Preview is read-only. Publication requires the source revision disclosed by preview. Reuse the existing write act when HTTP middleware already holds it; reacquiring its mutex would deadlock. Filesystem work runs off the async runtime, and only successful publication notifies open editors.

<gencode:copy id="generation-http-actions">
pub async fn preview(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<GenerationPlan>> {
    let writes = state.writes.clone();
    crate::engine::writes::during(&writes, async move {
        let captured_state = state.clone();
        let plan = tokio::task::spawn_blocking(move || {
            capture(&captured_state, &request).map(|context| context.plan())
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))??;
        Ok(Json(plan))
    })
    .await
}

pub async fn generate(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<Value>> {
    let revision = request
        .revision
        .clone()
        .ok_or_else(|| ApiError::bad_request("Preview the project before generating"))?;
    let writes = state.writes.clone();
    crate::engine::writes::during(&writes, async move {
        let captured_state = state.clone();
        let written = tokio::task::spawn_blocking(move || {
            capture(&captured_state, &request)?
                .publish(&revision)
                .map_err(ApiError::conflict)
        })
        .await
        .map_err(|e| ApiError::internal(e.to_string()))??;
        for (id, _) in state.index.entries() {
            state
                .rooms
                .publish_run_event(&id, &json!({"files_changed": true, "doc": id}))
                .await;
        }
        Ok(Json(json!({"written": written})))
    })
    .await
}
</gencode:copy>

<gencode:file path="crates/hickory-cli/src/serve/generation.rs"><gencode:paste select="#generation-http-context"/><gencode:paste select="#generation-http-actions"/></gencode:file>
</gencode:doc>
