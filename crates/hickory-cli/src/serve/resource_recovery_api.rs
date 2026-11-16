//! Read-only recovery inspection and reviewed saved/draft restoration admission.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    resource_changes::SourceState,
    resource_inputs::{Inputs, Request as Capture, SavedSource},
    resource_journal::{Manifest, Recovery},
    resource_restoration::Restoration,
};
use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrepareRequest {
    pub recovery_id: String,
    pub inputs: Inputs,
    pub capture: Capture,
}
#[derive(Serialize)]
pub struct Prepared {
    pub version: u32,
    pub restoration: Restoration,
    pub consequences: super::resource_candidates::Facts,
    #[serde(skip)]
    protected_entries: Vec<String>,
}
static ANALYSES: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);
async fn prepare_inner(state: LocalState, request: PrepareRequest) -> Result<Prepared, String> {
    let _permit = ANALYSES
        .try_acquire()
        .map_err(|_| "Restoration analysis is busy. Let it finish before preparing again.")?;
    let fresh =
        super::resource_inputs::inspect(State(state.clone()), Json(request.capture.clone()))
            .await
            .map_err(|error| error.detail().to_string())?
            .0;
    let root = state.index.root().to_path_buf();
    let identity = state.resource_root.clone();
    tokio::task::spawn_blocking(move || {
        if identity.check(&root)? != fresh.workspace {
            return Err(
                "Workspace changed before recovery analysis. Reconnect and prepare again.".into(),
            );
        }
        let recovery = super::resource_journal::read_recovery(&root, &request.recovery_id)?;
        let restoration = super::resource_restoration::prepare(&recovery, &request.inputs, &fresh)?;
        let consequences = restoration.analyze(&root, &fresh, &request.capture)?;
        if identity.check(&root)? != fresh.workspace {
            return Err(
                "Workspace changed during recovery analysis. Reconnect and prepare again.".into(),
            );
        }
        Ok(Prepared {
            version: 1,
            restoration,
            consequences,
            protected_entries: fresh.ownership.protected_entries,
        })
    })
    .await
    .map_err(|error| format!("Recovery analysis stopped: {error}. No restoration was published."))?
}
pub async fn prepare(
    State(state): State<LocalState>,
    Json(request): Json<PrepareRequest>,
) -> ApiResult<Json<Prepared>> {
    prepare_inner(state, request)
        .await
        .map(Json)
        .map_err(ApiError::conflict)
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequest {
    pub request_id: String,
    pub reviewed_fingerprint: String,
    pub reviewed_recovery_fingerprint: String,
    pub proposal: PrepareRequest,
}
async fn execute(state: LocalState, request: RestoreRequest) -> Result<Value, String> {
    let prepared = prepare_inner(state.clone(), request.proposal).await?;
    if prepared.restoration.recovery_fingerprint != request.reviewed_recovery_fingerprint
        || prepared.consequences.fingerprint != request.reviewed_fingerprint
    {
        return Err("Recovery evidence or source consequences changed after review. Prepare the complete restoration again.".into());
    }
    if !prepared.consequences.protected_targets.is_empty() {
        return Err(format!(
            "Restored source paths would become generated: {}. Reconcile their authored construction before restoring.",
            prepared
                .consequences
                .protected_targets
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    let manifest = Manifest::from_restoration(&prepared.restoration)?;
    let mut result = super::resource_saves::publish(
        state,
        manifest,
        prepared.consequences,
        request.request_id,
        prepared.protected_entries,
        super::resource_saves::EntryTransition::Historical,
    )
    .await?;
    result["draft_plan"] =
        serde_json::to_value(prepared.restoration.plan).map_err(|error| error.to_string())?;
    result["restored_from"] = json!(prepared.restoration.recovery_id);
    Ok(result)
}
pub async fn restore(
    State(state): State<LocalState>,
    Json(request): Json<RestoreRequest>,
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
            "Workspace changed. Reconnect and prepare recovery again.",
        ));
    }
    if !super::resource_saves::hex(&request.reviewed_recovery_fingerprint, 64) {
        return Err(ApiError::bad_request(
            "Review the exact retained recovery evidence before restoring.",
        ));
    }
    if let Some(result) = super::resource_saves::reserve(
        &workspace,
        &request.request_id,
        &request.reviewed_fingerprint,
        &request,
    )
    .map_err(ApiError::conflict)?
    {
        return result.map(Json).map_err(ApiError::conflict);
    }
    let id = request.request_id.clone();
    let result = execute(state, request).await;
    super::resource_saves::retain(&workspace, &id, &result);
    result.map(Json).map_err(ApiError::conflict)
}
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftSide {
    Before,
    After,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InspectRequest {
    pub id: String,
    pub path: Option<String>,
    pub draft: Option<DraftSide>,
    pub saved: Option<DraftSide>,
}
fn saved_summary(source: &SavedSource) -> Value {
    match source {
        SavedSource::Missing => json!({"kind":"missing"}),
        SavedSource::File {
            text,
            sha256,
            readonly,
            executable,
            mode,
        } => {
            json!({"kind":"file","bytes":text.len(),"sha256":sha256,"readonly":readonly,"executable":executable,"mode":mode})
        }
    }
}
fn draft_summary(source: &SourceState) -> Value {
    match source {
        SourceState::Missing { .. } => json!({"kind":"missing"}),
        SourceState::File { text, .. } => {
            json!({"kind":"file","bytes":text.len(),"sha256":super::workspace_fs::view::hash(text.as_bytes())})
        }
    }
}
fn summary(recovery: Recovery, query: InspectRequest, workspace: &str) -> Result<Value, String> {
    if (query.draft.is_some() || query.saved.is_some()) && query.path.is_none() {
        return Err("Select a recorded source path before requesting its draft text.".into());
    }
    if query
        .path
        .as_ref()
        .is_some_and(|path| !recovery.manifest.before.contains_key(path))
    {
        return Err("The selected source is not part of this recovery record.".into());
    }
    if query.draft.is_some() && query.saved.is_some() {
        return Err("Select either a saved baseline or a dirty draft.".into());
    }
    let mut paths = Vec::new();
    let mut text = None;
    let mut saved_text = None;
    for patch in &recovery.manifest.reviewed_plan.patches {
        paths.push(json!({"path":patch.path,"saved_before":saved_summary(&recovery.manifest.before[&patch.path]),"saved_after":saved_summary(&recovery.manifest.after[&patch.path]),"draft_before":draft_summary(&patch.before),"draft_after":draft_summary(&patch.after)}));
        if query.path.as_ref() == Some(&patch.path) {
            let saved = match query.saved {
                Some(DraftSide::Before) => Some(&recovery.manifest.before[&patch.path]),
                Some(DraftSide::After) => Some(&recovery.manifest.after[&patch.path]),
                None => None,
            };
            if let Some(SavedSource::File { text: source, .. }) = saved {
                saved_text = Some(source.clone());
            }
            let draft = match query.draft {
                Some(DraftSide::Before) => Some(&patch.before),
                Some(DraftSide::After) => Some(&patch.after),
                None => None,
            };
            if let Some(SourceState::File { text: source, .. }) = draft {
                text = Some(source.clone());
            }
        }
    }
    Ok(
        json!({"version":1,"workspace":workspace,"id":recovery.id,"fingerprint":recovery.fingerprint,"manifest_sha256":recovery.manifest_sha256,
        "recorded_workspace":recovery.manifest.workspace,"record_version":recovery.manifest.version,"request_id":recovery.manifest.request_id,"restore_of":recovery.manifest.restore_of,"context":recovery.manifest.context,"title":recovery.manifest.title.chars().take(256).collect::<String>(),"title_truncated":recovery.manifest.title.chars().count()>256,
        "paths":paths,"receipts":recovery.receipts,"unrecognized_files":recovery.unrecognized_files,"all_paths_recorded_applied":recovery.all_paths_recorded_applied,"draft_text":text,"saved_text":saved_text}),
    )
}
static READS: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(2);
pub async fn inspect(
    State(state): State<LocalState>,
    Query(query): Query<InspectRequest>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let permit = READS.try_acquire().map_err(|_| {
        ApiError::conflict(
            "Recovery inspection is busy. Let the current read finish before trying again.",
        )
    })?;
    let root = state.index.root().to_path_buf();
    let identity = state.resource_root.clone();
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let workspace = identity.check(&root)?.to_string();
        let recovery = super::resource_journal::read_recovery(&root, &query.id)?;
        let value = summary(recovery, query, &workspace)?;
        if identity.check(&root)? != workspace {
            return Err(
                "Workspace changed while reading recovery. Reconnect before using this evidence."
                    .into(),
            );
        }
        Ok::<_, String>(value)
    })
    .await
    .map_err(|error| ApiError::internal(format!("Recovery inspection stopped: {error}")))?
    .map(Json)
    .map_err(ApiError::conflict)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListRequest {
    pub after: Option<String>,
}
pub async fn list(
    State(state): State<LocalState>,
    Query(query): Query<ListRequest>,
) -> ApiResult<Json<Value>> {
    state
        .resource_root
        .ready()
        .await
        .map_err(ApiError::unavailable)?;
    let permit = READS.try_acquire().map_err(|_| {
        ApiError::conflict("Recovery inspection is busy. Let the current read finish.")
    })?;
    let root = state.index.root().to_path_buf();
    let identity = state.resource_root.clone();
    tokio::task::spawn_blocking(move || {
  let _permit=permit;let workspace=identity.check(&root)?.to_string();
  let page=super::resource_journal::list_recoveries(&root,query.after.as_deref())?;
  if identity.check(&root)?!=workspace{return Err("Workspace changed while listing recovery. Reconnect and refresh.".to_string());}
  Ok(serde_json::json!({"version":1,"workspace":workspace,"ids":page.ids,"next":page.next,"ignored":page.ignored}))
 }).await.map_err(|error|ApiError::internal(format!("Recovery discovery stopped: {error}")))?.map(Json).map_err(ApiError::conflict)
}
