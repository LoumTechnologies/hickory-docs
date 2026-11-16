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
pub(super) fn hex(value: &str, length: usize) -> bool {
    value.len() == length && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
pub(super) fn reserve<T: Serialize>(
    workspace: &str,
    request_id: &str,
    reviewed_fingerprint: &str,
    request: &T,
) -> Result<Option<Result<Value, String>>, String> {
    if !hex(workspace, 32) || !hex(request_id, 32) || !hex(reviewed_fingerprint, 64) {
        return Err("Resource saves require a current workspace, unique request identity and reviewed consequence fingerprint.".into());
    }
    let hash = super::workspace_fs::view::hash(
        &serde_json::to_vec(request).map_err(|error| error.to_string())?,
    );
    let key = format!("{workspace}:{request_id}");
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
pub(super) fn retain(workspace: &str, id: &str, result: &Result<Value, String>) {
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
) -> Result<super::resource_publish::ObservedWrite, String> {
    if !path.ends_with(".md") {
        return super::resource_publish::replace_observed(root, path, before, after);
    }
    let id = if matches!(before, SavedSource::File { .. }) {
        state.store.register_source(&state.index, path)
    } else {
        state.index.id_for_path(path)
    };
    runtime.block_on(async {
        let mut written = None;
        let mut persist = || -> anyhow::Result<()> {
            written = Some(super::resource_publish::replace_observed(root, path, before, after).map_err(anyhow::Error::msg)?);
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
        written.ok_or_else(||format!("Source publication callback did not confirm an observed writer: {path}"))
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
    let entries = request.proposal.inputs.ownership.protected_entries.clone();
    publish(
        state,
        manifest,
        facts,
        request.request_id,
        request.proposal.inputs.saved_guards.clone(),
        entries,
        EntryTransition::Stable,
    )
    .await
}
</ui:copy>

### Share checked publication across save and restoration

Both operations use the same bounded dependency checks, static output claims,
root checks, per-path live-document writer and final saved-state observation.
Ordinary saves keep the canonical entry fixed. A historical restoration may
remove a newly created entry, but cannot move the canonical entry identity to a
different address while workspace-entry transitions remain unsupported.

Only a complete, reobserved outcome returns confirmed `saved_sources`. A partial
or uncertain response instead returns `intended_saved_sources` and a null saved
map; planned states must not be adopted as though every file was saved.

<ui:copy id="resource-save-publication">
pub(super) enum EntryTransition {
    Stable,
    Historical,
}
pub(super) async fn publish(
    state: LocalState,
    manifest: Manifest,
    facts: super::resource_candidates::Facts,
    request_id: String,
    saved_guards:BTreeMap<String,String>,
    protected_entries: Vec<String>,
    entry_policy: EntryTransition,
) -> Result<Value, String> {
    publish_guarded(state,manifest,facts,request_id,saved_guards,protected_entries,entry_policy,Box::new(|_|Ok(())),false).await
}
/// A conflict wrapper adds its independent index lease without weakening any
/// existing room, saved-source, canonical-entry or compiler dependency guard.
pub(super) async fn publish_guarded(
    state: LocalState,
    mut manifest: Manifest,
    facts: super::resource_candidates::Facts,
    request_id: String,
    saved_guards:BTreeMap<String,String>,
    protected_entries: Vec<String>,
    entry_policy: EntryTransition,
    admission: Box<dyn Fn(&std::path::Path)->Result<(),String>+Send>,
    transactional_rollback: bool,
) -> Result<Value,String> {
    // The durable record can be found by the original request after a lost reply
    // or process restart. It is evidence, never permission to repeat publication.
    manifest.request_id = Some(request_id.to_ascii_lowercase());
    let runtime = tokio::runtime::Handle::current();
    tokio::task::spawn_blocking(move || {
        let root = state.index.root();
        let paths: BTreeSet<_> = manifest.before.keys().cloned().collect();
        // The default entry is still address-bound. Do not silently lose its
        // ownership scope before an explicit workspace-entry transition exists.
        if protected_entries.iter().any(|entry| entry == "project.md") {
            for patch in &manifest.reviewed_plan.patches {
                if matches!(patch.after, super::resource_changes::SourceState::Missing { .. })
                    && same_file::is_same_file(root.join(&patch.path), root.join("project.md")).unwrap_or(false)
                {
                    let survives = match &patch.before {
                        super::resource_changes::SourceState::File { identity, .. } => manifest.reviewed_plan.patches.iter().any(|candidate| matches!(&candidate.after, super::resource_changes::SourceState::File { identity: after, .. } if after == identity)),
                        _ => false,
                    };
                    if matches!(entry_policy, EntryTransition::Stable) || survives {
                        return Err("The canonical project.md entry or its physical alias cannot be moved or deleted by a resource save yet. Keep that entry in place and move its source members instead.".into());
                    }
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
        let mut outcome = super::resource_publish::publish_observed_guarded_with(root, &state.resource_root, &manifest,
            |published| { admission(root)?;super::resource_inputs::validate_guard_map(root,&saved_guards,published)?; super::source_tests::check_saved_sources(root, &dependencies).map(|_| ()).map_err(|error|error.detail().to_string()) },
            |published| { admission(root)?;super::resource_inputs::validate_guard_map(root,&saved_guards,published)?; super::source_tests::check_saved_sources(root, &dependencies).map(|_| ()).map_err(|error|error.detail().to_string()) },
            |path, before, after| room_write(&state, &runtime, root, path, before, after), transactional_rollback)?;
        if outcome.complete {
            for (path, expected) in &manifest.after {
                match super::resource_inputs::saved_target(root, path) {
                    Ok((_, current)) if &current == expected => {},
                    _ => { outcome.complete = false; outcome.error = Some(format!("Saved source changed before confirmation: {path}. Inspect current files and the recovery record before adopting resource changes.")); break; }
                }
            }
        }
        let files = if outcome.complete {
            match source_receipts(&state, &manifest.after, &outcome) {
                Ok(files) => files,
                Err(error) => { outcome.complete = false; outcome.error = Some(format!("Source publication was observed, but editor metadata is unavailable: {error}. Inspect recovery before adopting the namespace.")); Vec::new() }
            }
        } else { Vec::new() };
        publication_reply(request_id, manifest.workspace, manifest.context, outcome, manifest.after, files)
    }).await.map_err(|error|format!("Resource publication stopped unexpectedly: {error}. Inspect retained recovery before retrying."))?
}
fn publication_reply(
    request_id: String,
    workspace: String,
    context: String,
    outcome: super::resource_publish::Outcome,
    after: BTreeMap<String, SavedSource>,
    files: Vec<Value>,
) -> Result<Value, String> {
    let confirmed = outcome.complete;let adoption_current=confirmed && outcome.observed.is_some();
    let mut reply = json!({"version":2,"request_id":request_id,"workspace":workspace,"context":context,"outcome":outcome,"adoption_current":adoption_current,"adoption_reason":null,"saved_sources":null});
    let key = if confirmed {
        "saved_sources"
    } else {
        "intended_saved_sources"
    };
    reply[key] = serde_json::to_value(after).map_err(|error| error.to_string())?;
    reply["source_files"] = json!(if confirmed { files } else { Vec::new() });
    Ok(reply)
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
    if let Some(result) = reserve(
        &workspace,
        &request.request_id,
        &request.reviewed_fingerprint,
        &request,
    )
    .map_err(ApiError::conflict)?
    {
        return match result{Ok(value)=>checked_retained(&state,value).await.map(Json).map_err(ApiError::conflict),Err(error)=>Err(ApiError::conflict(error))};
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
/// Retained completion describes a past publication. Revalidate the exact physical
/// saved namespace before granting a new editor adoption; never execute a writer.
fn current_receipt(state:&LocalState,value:&Value)->Result<(),String>{
 let root=state.index.root();let workspace=state.resource_root.check(root)?;
 if value["workspace"].as_str()!=Some(workspace)||value["version"].as_u64()!=Some(2)||(value["outcome"]["complete"].as_bool()!=Some(true)||value["adoption_current"].as_bool()!=Some(true)){return Err("This retained outcome does not confirm a complete current source namespace.".into())}
 let saved:BTreeMap<String,SavedSource>=serde_json::from_value(value["saved_sources"].clone()).map_err(|_|"Retained publication has no complete saved namespace.")?;
 if saved.is_empty()||saved.len()>10000{return Err("Retained source namespace is empty or exceeds its bounded reading.".into())}
 let rows=value["source_files"].as_array().ok_or("Retained publication has no source metadata.")?;let mut guards=BTreeMap::new();
 for row in rows{let path=row["file"]["path"].as_str().ok_or("Retained source metadata has no exact path.")?;let guard=row["save_guard"].as_str().ok_or("Retained source has no captured physical guard.")?;let metadata:SavedSource=serde_json::from_value(row["saved"].clone()).map_err(|_|"Retained source metadata is malformed.")?;
  if !matches!(metadata,SavedSource::File{..})||saved.get(path)!=Some(&metadata)||guard.len()>16*1024||guards.insert(path.to_owned(),guard.to_owned()).is_some(){return Err("Retained source metadata is ambiguous or differs from its saved namespace.".into())}
 }
 let files=saved.iter().filter(|(_,value)|matches!(value,SavedSource::File{..})).count();if files!=guards.len(){return Err("Retained source metadata does not cover every saved file.".into())}
 let claims=super::api::generated_outputs(root,&state.index);
 for(path,expected)in &saved{let(full,current)=super::resource_inputs::saved_target(root,path)?;if &current!=expected{return Err(format!("Saved namespace changed after publication: {path}. Its old completion remains historical."))}
  if let Some(guard)=guards.get(path){if super::api::generated_by(&claims,path).is_some()||claims.keys().any(|claim|same_file::is_same_file(&full,root.join(claim)).unwrap_or(false)){return Err(format!("Current generated ownership changed at {path}. Inspect its authored source before editor adoption."))}
   let observed=super::source_history::read_for_api(state,&full,path)?;if observed.save_guard.as_deref()!=Some(guard){return Err(format!("Saved physical identity changed after publication: {path}. No old editor identity was attached to its new occupant."))}
  }
 }
 state.resource_root.check(root)?;Ok(())
}
pub(super) async fn checked_retained(state:&LocalState,mut value:Value)->Result<Value,String>{
 if value.get("saved_sources").is_none(){return Ok(value)} // Stage-only receipts have separate index authority.
 let state=state.clone();tokio::task::spawn_blocking(move||{let legacy=value["version"].as_u64()!=Some(2);if legacy{value["version"]=json!(2);value["adoption_current"]=json!(false)}match current_receipt(&state,&value){Ok(())=>{value["adoption_current"]=json!(true);value["adoption_reason"]=Value::Null},Err(error)=>{value["adoption_current"]=json!(false);value["adoption_reason"]=json!(error)}}value}).await.map_err(|error|format!("Current source adoption could not be checked: {error}. The original historical receipt remains retained."))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub request_id: String,
}
/// Conflict recovery distinguishes an observed refusal from a missing or in-flight outcome.
/// Only this engine's retained, root-scoped result can establish completion.
pub(super) async fn retained_status(state: &LocalState, request_id: &str) -> Result<Value,String> {
    if !hex(request_id,32) { return Err("Use the original publication request identity.".into()); }
    state.resource_root.ready().await?;
    let workspace=state.resource_root.retained_epoch()?;
    let (status,mut outcome,error)={let attempts=ATTEMPTS.lock().map_err(|_|"Publication status is unavailable; inspect retained recovery.".to_string())?;
    let attempt=attempts.records.get(&format!("{workspace}:{request_id}"));
    match attempt.and_then(|v|v.result.as_ref()) {
        Some(Ok(value)) => ("completed",value.clone(),Value::Null),
        Some(Err(error)) => ("completed",Value::Null,json!(error)),
        None if attempt.is_some() => ("pending",Value::Null,Value::Null),
        None => ("unknown",Value::Null,Value::Null),
    }};
    if !outcome.is_null(){outcome=checked_retained(state,outcome).await?}
    Ok(json!({"version":1,"request_id":request_id,"state":status,"outcome":outcome,"error":error}))
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
        .retained_epoch()
        .map_err(ApiError::conflict)?;
    let retained={let attempts=ATTEMPTS.lock().map_err(|_|ApiError::unavailable("Resource save outcomes are unavailable. Inspect retained recovery records."))?;
        attempts.records.get(&format!("{workspace}:{}",query.request_id)).ok_or_else(||ApiError::not_found("No retained save outcome in this engine session. Inspect recovery records before retrying a request from an earlier session."))?.result.clone().ok_or_else(||ApiError::conflict("Save outcome is pending or uncertain. Inspect recovery records before retrying."))?};
    match retained{Ok(value)=>checked_retained(&state,value).await.map(Json).map_err(ApiError::conflict),Err(error)=>Err(ApiError::conflict(error))}
}
</ui:copy>

The conflict-specific retained status reader exposes completed refusal, pending and unknown as typed states. A refusal is not proof of rollback: callers must additionally revalidate the complete original source and index before retiring a refused attempt. An unknown record never permits replay. The ordinary save endpoint retains its existing API.

## Do not adopt partial plans as saved baselines

An incomplete publication can have confirmed earlier paths while later destinations remain unwritten or uncertain. Its response must expose the plan as intended data and withhold the confirmed saved-baseline map. A completed result exposes that map. The shared attempt test also proves pending requests refuse replay, identical completed requests reuse their result, and changed typed content cannot reuse an identity across save/restore operations.

<ui:copy id="resource-published-outcome-contract-tests">
#[cfg(test)]
mod outcome_contract_tests {
    use super::*;
    fn outcome(complete: bool) -> super::super::resource_publish::Outcome {
        super::super::resource_publish::Outcome {
            complete,
            source_files_modified: true,
            source_files_uncertain: !complete,
            recovery: ".hick-cache/resource-changes/record/manifest.json".into(),
            applied: BTreeSet::from(["a.py".into()]),
            error: (!complete).then(|| "b.py was not confirmed".into()),
            observed:None,
        }
    }
    #[test]
    fn partial_publication_exposes_intent_without_confirmed_saved_baselines() {
        let after = BTreeMap::from([("a.py".into(), SavedSource::Missing)]);
        let partial = publication_reply(
            "request".into(),
            "workspace".into(),
            "context".into(),
            outcome(false),
            after.clone(),
            vec![json!({"file":{"path":"not-confirmed.py"}})],
        )
        .unwrap();
        assert!(partial["saved_sources"].is_null());
        assert_eq!(partial["source_files"], json!([]));
        assert_eq!(partial["intended_saved_sources"]["a.py"]["kind"], "missing");
        assert_eq!(partial["outcome"]["applied"], json!(["a.py"]));
        let complete = publication_reply(
            "request".into(),
            "workspace".into(),
            "context".into(),
            outcome(true),
            after,
            vec![json!({"file":{"path":"confirmed.py"}})],
        )
        .unwrap();
        assert_eq!(complete["saved_sources"]["a.py"]["kind"], "missing");
        assert!(complete.get("intended_saved_sources").is_none());
        assert_eq!(complete["source_files"][0]["file"]["path"], "confirmed.py");
    }
    #[test]
    fn shared_attempt_identity_refuses_pending_and_changed_content_and_retains_completed_result() {
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).unwrap();
        let workspace = hex::encode(nonce);
        let id = "a".repeat(32);
        let fingerprint = "b".repeat(64);
        let body = json!({"request_id":id,"reviewed_fingerprint":fingerprint,"kind":"save"});
        assert!(
            reserve(&workspace, &id, &fingerprint, &body)
                .unwrap()
                .is_none()
        );
        assert!(reserve(&workspace, &id, &fingerprint, &body).is_err());
        let result = Ok(json!({"outcome":{"complete":true},"saved_sources":{}}));
        retain(&workspace, &id, &result);
        assert_eq!(
            reserve(&workspace, &id, &fingerprint, &body).unwrap(),
            Some(result.clone())
        );
        let changed = json!({"request_id":id,"reviewed_fingerprint":fingerprint,"kind":"restore"});
        assert!(reserve(&workspace, &id, &fingerprint, &changed).is_err());
        assert_eq!(
            reserve(&workspace, &id, &fingerprint, &body).unwrap(),
            Some(result)
        );
    }
}
</ui:copy>

## Return editor baselines without a second file read

A successful namespace action already observed its exact final saved states. Return an array of file descriptors, saved baselines and the ordinary plain-file save fingerprint from that observation. Arrays retain byte-distinct path values across client runtimes. Markdown IDs come from the index registered under the live-document writer; no late filesystem canonicalization can retarget the receipt. The language hint uses the same router as the existing file tree. An incomplete result withholds these adoption receipts. This is saved-state metadata, not completed semantic analysis or client adoption.

<ui:copy id="resource-source-editor-receipts">
fn source_receipts(
    state: &LocalState,
    after: &BTreeMap<String, SavedSource>,
    outcome:&super::resource_publish::Outcome,
) -> Result<Vec<Value>, String> {
    let observed=outcome.observed.as_ref().ok_or("Resource publication has no producer-held source identity. Retain readable recovery; no editor guards were issued.")?;
    let document_ids: BTreeMap<_, _> = state
        .index
        .entries()
        .into_iter()
        .map(|(id, path)| (path, id))
        .collect();
    let mut files = Vec::new();
    for (path, saved) in after {
        let SavedSource::File { text, .. } = saved else {
            continue;
        };
        let mut file = json!({"name":path.rsplit('/').next().unwrap_or(path),"path":path,"dir":false,"language":super::api::file_language::language_of(path)});
        let hash = if path.ends_with(".md") {
            let id = document_ids.get(path).ok_or_else(|| {
                format!("Saved document has no registered source identity: {path}")
            })?;
            file["doc_id"] = json!(id);
            String::new()
        } else {
            super::plain_file::content_hash(text.as_bytes())
        };
        let proven=observed.get(path).ok_or_else(||format!("Saved source has no held producer identity: {path}"))?;proven.verify(state.index.root(),path,saved)?;
        let (full, current) = super::resource_inputs::saved_target(state.index.root(), path)?;
        if &current != saved { return Err(format!("Saved source changed before issuing its editor guard: {path}")); }
        let guard = super::source_history::issue_guard(state, &full, text)?;
        proven.verify(state.index.root(),path,saved)?;
        files.push(json!({"file":file,"saved":saved,"hash":hash,"save_guard":guard}));
    }
    Ok(files)
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_saves.rs"><ui:paste select="#resource-save-attempts"/><ui:paste select="#resource-save-rooms"/><ui:paste select="#resource-save-admission"/><ui:paste select="#resource-save-publication"/><ui:paste select="#resource-save-save"/><ui:paste select="#resource-save-status"/><ui:paste select="#resource-published-outcome-contract-tests"/><ui:paste select="#resource-source-editor-receipts"/></ui:file>
</ui:doc>
