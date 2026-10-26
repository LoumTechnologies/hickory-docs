<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/runtime_runs.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-649fac42ee3c"></a>
## `crates/hickory-cli/src/serve/runtime_runs.rs`

<a id="f-649fac42ee3c-access"></a>
### `access`

Shared durable observation store; no Markdown mutation or execution.

<storycode:copy id="f-649fac42ee3c-access">
//! Shared durable observation store; no Markdown mutation or execution.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_observe::{LocalStore, Retention, Retrieved, Run, Status};
use serde_json::{Value, json};

fn access<T>(state: &LocalState, f: impl FnOnce(&LocalStore) -> anyhow::Result<T>) -> ApiResult<T> {
    let mut store = state
        .observations
        .lock()
        .map_err(|_| ApiError::internal("Observation store unavailable"))?;
    if store.is_none() {
        *store = Some(LocalStore::open(state.index.root())?);
    }
    f(store.as_ref().unwrap()).map_err(|e| ApiError::unprocessable(e.to_string()))
}
</storycode:copy>
<a id="f-649fac42ee3c-list"></a>
### `list`

<storycode:copy id="f-649fac42ee3c-list">
pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    access(&state, |store| {
        Ok(Json(json!({"runs":store.ids()?.into_iter().map(|id| {
        match store.get(&id) {
            Ok(Retrieved::Present(run)) => json!({"id":id,"state":"present","run":run}),
            Ok(Retrieved::Missing) => json!({"id":id,"state":"missing"}),
            Ok(Retrieved::Corrupt(error)) => json!({"id":id,"state":"corrupt","error":error}),
            Err(error) => json!({"id":id,"state":"unavailable","error":error.to_string()}),
        }
    }).collect::<Vec<_>>()})))
    })
}
</storycode:copy>
<a id="f-649fac42ee3c-save"></a>
### `save`

<storycode:copy id="f-649fac42ee3c-save">
pub async fn save(
    State(state): State<LocalState>,
    Json(mut run): Json<Run>,
) -> ApiResult<Json<Value>> {
    if serde_json::to_vec(&run)
        .map_err(|e| ApiError::bad_request(e.to_string()))?
        .len()
        > 16 * 1024 * 1024
    {
        return Err(ApiError::bad_request(
            "Retained run exceeds 16 MiB; select a bounded capture",
        ));
    }
    access(&state, |store| {
        if let Retrieved::Present(old) = store.get(&run.id)? {
            run.pinned = old.pinned;
        }
        store.save(&run)?;
        if run.status != Status::Running {
            store.retain(
                Retention::default(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
            )?;
        }
        Ok(Json(json!({"id":run.id,"retained":true})))
    })
}
</storycode:copy>
<a id="f-649fac42ee3c-remove"></a>
### `remove`

<storycode:copy id="f-649fac42ee3c-remove">
pub async fn remove(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    access(&state, |store| {
        Ok(Json(json!({"deleted":store.delete(&id)?})))
    })
}
</storycode:copy>
<a id="f-649fac42ee3c-pin"></a>
### `pin`

<storycode:copy id="f-649fac42ee3c-pin">
pub async fn pin(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let pinned = body["pinned"]
        .as_bool()
        .ok_or_else(|| ApiError::bad_request("Specify pinned boolean"))?;
    access(&state, |store| {
        store.pin(&id, pinned)?;
        Ok(Json(json!({"id":id,"pinned":pinned})))
    })
}

</storycode:copy>
<a id="f-649fac42ee3c-python"></a>
### `python`

Explicit native-user operation; deliberately absent from agent tool capabilities.

<storycode:copy id="f-649fac42ee3c-python">
/// Explicit native-user operation; deliberately absent from agent tool capabilities.
pub async fn python(
    State(state): State<LocalState>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let root = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let relative = body["document"]
        .as_str()
        .ok_or_else(|| ApiError::bad_request("Specify document"))?;
    let path = root
        .join(relative.strip_prefix("hick:///").unwrap_or(relative))
        .canonicalize()
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    if !path.starts_with(&root) {
        return Err(ApiError::bad_request("Document escapes workspace"));
    }
    let mut run = crate::observation::capture_mode(
        &path,
        body["program"].as_str().filter(|s| !s.is_empty()),
        body["seconds"].as_u64().unwrap_or(10),
        body["profile"].as_bool().unwrap_or(false),
    )
    .await
    .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    run.sources[0].uri = path
        .strip_prefix(&root)
        .map_err(|e| ApiError::bad_request(e.to_string()))?
        .to_string_lossy()
        .replace('\\', "/");
    access(&state, |store| {
        store.save(&run)?;
        Ok(Json(json!({"run":run})))
    })
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/runtime_runs.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/runtime_runs.rs"><storycode:paste select="#f-649fac42ee3c-access"/><storycode:paste select="#f-649fac42ee3c-list"/><storycode:paste select="#f-649fac42ee3c-save"/><storycode:paste select="#f-649fac42ee3c-remove"/><storycode:paste select="#f-649fac42ee3c-pin"/><storycode:paste select="#f-649fac42ee3c-python"/></storycode:file>

</storycode:doc>
