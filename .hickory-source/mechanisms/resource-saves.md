<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Save reviewed source resources through one engine model

For maintainers connecting all simple hosts to source namespace publication.

POST `/api/changes/resource-save` is an explicit source save; GET on the same path reads its retained attempt. Clients must validate current reviewed buffers before sending, then adopt source identities only after a confirmed complete result. This engine API alone is not frontend resource parity.

Focused evidence (2026-11-15): the rebuilt engine and
[actual protocol witness](resource-inputs.md#exercise-the-actual-acquisition-and-planning-routes)
pass dirty-draft move/save, separate saved/draft recovery, duplicate result and
status lookup, changed-request identity refusal, stale source and generated target
refusal, and a document discovered after startup moving through room retirement.
Five focused publication cases continue to pass with the live-document adapter
seam. Dirty collaborative input races and native frontend resource application
are not established by this witness.

### Bind one attempt to one reviewed request

A unique client identity binds the complete typed request and the held engine-root session. Pending or uncertain attempts refuse replay. Confirmed responses are retained within explicit count and byte limits; exhausting response capacity retains a refusal rather than authorizing another write. Restarting the engine changes the root nonce, so an old capture cannot silently become a fresh operation.

<ui:copy id="resource-save-attempts">
//! Explicit source resource saves share one guarded outcome across client retries.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_inputs::SavedSource,
    resource_journal::Manifest,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Mutex,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: String,
    pub reviewed_fingerprint: String,
    pub proposal: super::resource_candidates::Request,
}
struct Attempt {
    hash: String,
    result: Option<Result<Value, String>>,
}
#[derive(Default)]
struct Attempts {
    records: BTreeMap<String, Attempt>,
    retained_bytes: usize,
}
static ATTEMPTS: std::sync::LazyLock<Mutex<Attempts>> =
    std::sync::LazyLock::new(|| Mutex::new(Attempts::default()));
fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn reserve(workspace: &str, request: &Request) -> Result<Option<Result<Value, String>>, String> {
    if !hex(workspace, 32)
        || !hex(&request.request_id, 32)
        || !hex(&request.reviewed_fingerprint, 64)
    {
        return Err("Resource saves require a current workspace, unique request identity and reviewed consequence fingerprint.".into());
    }
    let hash = super::workspace_fs::view::hash(
        &serde_json::to_vec(request).map_err(|error| error.to_string())?,
    );
    let key = format!("{workspace}:{}", request.request_id);
    let mut attempts = ATTEMPTS.lock().map_err(|_| "Resource save outcomes are unavailable. Inspect retained recovery records before retrying.")?;
    if let Some(attempt) = attempts.records.get(&key) {
        if attempt.hash != hash {
            return Err("This save identity already names a different reviewed request. Prepare a new proposal.".into());
        }
        return attempt.result.clone().map(Some).ok_or_else(|| "The previous save outcome is not confirmed. Inspect retained resource recovery records; do not repeat the operation with a new identity.".into());
    }
    if attempts.records.len() >= 4096 {
        return Err("This engine has retained its resource save identity limit. Inspect outstanding saves, then reconnect to a fresh engine before new resource saves.".into());
    }
    attempts.records.insert(key, Attempt { hash, result: None });
    Ok(None)
}
fn retain(workspace: &str, id: &str, result: &Result<Value, String>) {
    let Ok(mut attempts) = ATTEMPTS.lock() else {
        return;
    };
    let bytes = serde_json::to_vec(result).map_or(usize::MAX, |bytes| bytes.len());
    let retained = if bytes <= (32 * 1024 * 1024usize).saturating_sub(attempts.retained_bytes) {
        attempts.retained_bytes += bytes;
        result.clone()
    } else {
        Err(format!(
            "The save outcome exceeded retained response capacity. Inspect its recovery record {} before retrying.",
            result
                .as_ref()
                .ok()
                .and_then(|value| value["outcome"]["recovery"].as_str())
                .unwrap_or("under .hick-cache/resource-changes")
        ))
    };
    if let Some(attempt) = attempts.records.get_mut(&format!("{workspace}:{id}")) {
        attempt.result = Some(retained);
    }
}
</ui:copy>

### Check live source while changing its saved path

Run per-path publication through the existing live-document comparison. Replacements update the live text under the same room lock as the durable write; deletions retire the old room so a debounced save cannot recreate its source. A missing path with an existing live room refuses. Plain files keep their saved-state checks. The engine write gate remains held while the blocking publisher awaits these room operations.

<ui:copy id="resource-save-rooms">
fn room_write(
    state: &LocalState,
    runtime: &tokio::runtime::Handle,
    root: &std::path::Path,
    path: &str,
    before: &SavedSource,
    after: &SavedSource,
) -> Result<bool, String> {
    if !path.ends_with(".md") {
        return super::resource_publish::replace(root, path, before, after);
    }
    let id = if matches!(before, SavedSource::File { .. }) {
        state.store.register_source(&state.index, path)
    } else {
        state.index.id_for_path(path)
    };
    runtime.block_on(async {
        let mut changed = false;
        let mut persist = || -> anyhow::Result<()> {
            changed = super::resource_publish::replace(root, path, before, after).map_err(anyhow::Error::msg)?;
            Ok(())
        };
        let accepted = match (before, after) {
            (SavedSource::File { text: expected, .. }, SavedSource::File { text, .. }) => state.rooms.replace_source_if_current(&id, expected, text, persist).await.map_err(|error|error.to_string())?,
            (SavedSource::File { text: expected, .. }, SavedSource::Missing) => state.rooms.retire_source_if_current(&id, expected, persist).await.map_err(|error|error.to_string())?,
            (SavedSource::Missing, _) => {
                if state.rooms.get(&id).await.is_some() { return Err(format!("A live editor owns the missing source path: {path}. Reconcile it before saving resources.")); }
                persist().map_err(|error|error.to_string())?;
                true
            }
        };
        if !accepted { return Err(format!("A live document changed after review: {path}. Save or reconcile it before resource changes.")); }
        if matches!(after, SavedSource::File { .. }) { state.store.register_source(&state.index, path); }
        Ok(changed)
    })
}
</ui:copy>

The current default construction entry is address-bound to `project.md`. Until an
explicit workspace-entry transition is implemented, save refuses to remove or
move that entry or a physical alias. A successful file move must not silently
remove main-project ownership from subsequent requests. Read-only construction
analysis can still describe such a hypothetical move; it is not save permission.

### Recompute the reviewed consequences before publishing

Repeat candidate saved/draft construction analysis and exact source acquisition. Refuse changed fingerprints, newly generated targets and positive static output claims from other visible documents. Check non-target compiler dependencies before every path, with a bounded cumulative IO budget. Candidate target source changes are this plan’s own transitions; final saved states are reobserved before confirming the result. This remains selected/main compiler scope with additional static claims, not discovery of every hidden construction in the workspace.

<ui:copy id="resource-save-admission">
fn dependency_budget(
    root: &std::path::Path,
    dependencies: &BTreeMap<String, String>,
    paths: &BTreeSet<String>,
) -> Result<(), String> {
    let mut bytes = 0u64;
    for path in dependencies.keys().filter(|path| !paths.contains(*path)) {
        let metadata = std::fs::metadata(root.join(path)).map_err(|error| error.to_string())?;
        bytes = bytes.saturating_add(metadata.len());
    }
    if bytes.saturating_mul(paths.len() as u64 + 1) > 256 * 1024 * 1024 {
        return Err("Resource save dependency checks exceed their interactive budget. Narrow the proposal before saving.".into());
    }
    Ok(())
}
async fn execute(state: LocalState, request: Request) -> Result<Value, String> {
    let facts =
        super::resource_candidates::inspect(State(state.clone()), Json(request.proposal.clone()))
            .await
            .map_err(|error| error.detail().to_string())?
            .0;
    if facts.fingerprint != request.reviewed_fingerprint {
        return Err("Resource consequences changed after review. Prepare and review the complete proposal again.".into());
    }
    if !facts.protected_targets.is_empty() {
        return Err(format!(
            "Resource targets would become generated: {}. Change their authored construction instead.",
            facts
                .protected_targets
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let fresh = super::resource_inputs::inspect(
        State(state.clone()),
        Json(request.proposal.capture.clone()),
    )
    .await
    .map_err(|error| error.detail().to_string())?
    .0;
    super::resource_inputs::validate_candidate(
        &request.proposal.inputs,
        &fresh,
        &request.proposal.plan,
    )?;
    let manifest = Manifest::from_review(&request.proposal.inputs, &request.proposal.plan)?;
    let runtime = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        let root = state.index.root();
        let paths: BTreeSet<_> = manifest.before.keys().cloned().collect();
        // The default entry is still address-bound. Do not silently lose its
        // ownership scope before an explicit workspace-entry transition exists.
        if request.proposal.inputs.ownership.protected_entries.iter().any(|entry| entry == "project.md") {
            for patch in &manifest.reviewed_plan.patches {
                if matches!(patch.after, super::resource_changes::SourceState::Missing { .. })
                    && same_file::is_same_file(root.join(&patch.path), root.join("project.md")).unwrap_or(false)
                {
                    return Err("The canonical project.md entry or its physical alias cannot be moved or deleted by a resource save yet. Keep that entry in place and move its source members instead.".into());
                }
            }
        }
        dependency_budget(root, &facts.checked_saved_sources, &paths)?;
        let claims = super::api::generated_outputs(root, &state.index);
        for path in &paths {
            if let Some(owner) = super::api::generated_by(&claims, path) {
                return Err(format!("Source path is declared generated by {owner}: {path}. Change its authored construction instead."));
            }
        }
        let dependencies: BTreeMap<_, _> = facts.checked_saved_sources.into_iter().filter(|(path, _)| !paths.contains(path)).collect();
        let mut outcome = super::resource_publish::publish_with(root, &state.resource_root, &manifest,
            |_| { super::source_tests::check_saved_sources(root, &dependencies).map(|_| ()).map_err(|error|error.detail().to_string()) },
            |path, before, after| room_write(&state, &runtime, root, path, before, after))?;
        if outcome.complete {
            for (path, expected) in &manifest.after {
                match super::resource_inputs::saved_target(root, path) {
                    Ok((_, current)) if &current == expected => {},
                    _ => { outcome.complete = false; outcome.error = Some(format!("Saved source changed before confirmation: {path}. Inspect current files and the recovery record before adopting resource changes.")); break; }
                }
            }
        }
        Ok(json!({"version":1,"request_id":request.request_id,"workspace":manifest.workspace,"context":manifest.context,"outcome":outcome,"saved_sources":manifest.after}))
    }).await.map_err(|error|format!("Resource publication stopped unexpectedly: {error}. Inspect retained recovery before retrying."))?
}
</ui:copy>

### Retain the outcome even after a response disconnects

The dispatch task owns mutation ordering through completion. Reserve the request identity before admission and retain either its result or refusal. A complete reply reports exact saved baselines and recovery evidence; a partial reply keeps confirmed paths and uncertainty visible. The endpoint does not mutate client buffers or claim a coordinated UI undo.

<ui:copy id="resource-save-save">
/// Dispatch already owns the process-lifetime mutation gate, including after disconnect.
pub async fn save(
    State(state): State<LocalState>,
    Json(request): Json<Request>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let workspace = request.proposal.inputs.workspace.clone();
    if state
        .resource_root
        .check(state.index.root())
        .map_err(ApiError::conflict)?
        != workspace
    {
        return Err(ApiError::conflict(
            "Workspace identity changed. Reconnect and prepare again.",
        ));
    }
    if let Some(result) = reserve(&workspace, &request).map_err(ApiError::conflict)? {
        return result.map(Json).map_err(ApiError::conflict);
    }
    let id = request.request_id.clone();
    let result = execute(state, request).await;
    retain(&workspace, &id, &result);
    result.map(Json).map_err(ApiError::conflict)
}
</ui:copy>

### Read the original attempt without repeating it

The status query is read-only and bound to the current held root. Missing, pending or uncertain outcomes require inspecting recovery; none is permission to create a replacement request automatically.

<ui:copy id="resource-save-status">
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub request_id: String,
}
pub async fn status(
    State(state): State<LocalState>,
    Query(query): Query<Status>,
) -> ApiResult<Json<Value>> {
    if !hex(&query.request_id, 32) {
        return Err(ApiError::bad_request(
            "Use the original resource save request identity.",
        ));
    }
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let workspace = state
        .resource_root
        .check(state.index.root())
        .map_err(ApiError::conflict)?;
    let attempts = ATTEMPTS.lock().map_err(|_| {
        ApiError::unavailable(
            "Resource save outcomes are unavailable. Inspect retained recovery records.",
        )
    })?;
    let attempt = attempts.records.get(&format!("{workspace}:{}",query.request_id)).ok_or_else(||ApiError::not_found("No retained save outcome in this engine session. Inspect recovery records before retrying a request from an earlier session."))?;
    attempt
        .result
        .clone()
        .ok_or_else(|| {
            ApiError::conflict(
                "Save outcome is pending or uncertain. Inspect recovery records before retrying.",
            )
        })?
        .map(Json)
        .map_err(ApiError::conflict)
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_saves.rs"><ui:paste select="#resource-save-attempts"/><ui:paste select="#resource-save-rooms"/><ui:paste select="#resource-save-admission"/><ui:paste select="#resource-save-save"/><ui:paste select="#resource-save-status"/></ui:file>
</ui:doc>
