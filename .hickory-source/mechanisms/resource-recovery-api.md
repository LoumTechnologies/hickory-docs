<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Admit reviewed recovery through the shared engine model

For maintainers connecting existing menu-based change previews to saved/draft restoration.

The engine exposes read-only inspection and preparation, followed by an explicit guarded restore action. These routes do not mutate client buffers. A host must capture and validate current buffer identities, keep relevant drafts stable during publication and adopt the returned namespace only after complete confirmation. Invalid historical construction drafts, partial-state recovery and UI adoption are not established here.

Focused evidence (2026-11-15): the rebuilt real-engine resource witness passes
inspection, independent-source refusal, separate saved/draft restore and redo,
changed parent/consequence refusal, repeated restore identity and read-only status.
The entry witness passes canonical creation/undo-to-absence and refuses historical
canonical moves before new recovery IO. Two outcome/attempt cases, four write-gate
cases and four startup cases pass. Recovery reads/preparation remain independent;
publication remains a guarded write. No frontend namespace/undo acceptance is
claimed by these service witnesses.

### Prepare against current source and checked recovery

Read one bounded recovery record and reacquire all current source and ownership facts. The pure inverse validates saved baselines, drafts and current preparation versions. Compile saved and dirty restoration layers independently and return their exact consequences and parent fingerprint. Root identity is checked around blocking work; this named preparation is read-only and independent of the file-write gate.

<ui:copy id="resource-recovery-api-preparation">
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
</ui:copy>

### Repeat admission and retain one restoration outcome

Require both the reviewed parent evidence fingerprint and reviewed consequences. Rebuild the inverse against fresh source, refuse protected targets, retain a restoration-aware record and use the same live-document/dependency/root publisher as ordinary saves. Request identity prevents replay after a lost response. The response returns the draft transition separately from confirmed saved baselines; a partial result is not namespace adoption permission.

<ui:copy id="resource-recovery-api-publication">
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestoreRequest {
    pub request_id: String,
    pub reviewed_fingerprint: String,
    pub reviewed_recovery_fingerprint: String,
    pub proposal: PrepareRequest,
}
async fn execute(state: LocalState, request: RestoreRequest) -> Result<Value, String> {
    let saved_guards=request.proposal.inputs.saved_guards.clone();
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
        saved_guards,
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
        return match result{Ok(value)=>super::resource_saves::checked_retained(&state,value).await.map(Json).map_err(ApiError::conflict),Err(error)=>Err(ApiError::conflict(error))};
    }
    let id = request.request_id.clone();
    let result = execute(state, request).await;
    super::resource_saves::retain(&workspace, &id, &result);
    result.map(Json).map_err(ApiError::conflict)
}
</ui:copy>

### Inspect metadata and explicitly selected historical draft text

Return recorded saved/draft hashes, sizes, path presence and receipts without claiming current disk equality. Historical workspace identity is labeled separately from the current root. Full text is returned only for an explicitly selected recorded draft side and path; default inspection avoids sending every retained source body. Reads are bounded and never create cache directories or publish source.

<ui:copy id="resource-recovery-api-inspection">
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
</ui:copy>

## Read a bounded discovery page under the held workspace

Discovery shares the inspection budget and checks the held root before and after its blocking read. Returning the current workspace nonce lets an open reading reject responses from a different root session. The route remains available independently of unrelated project preparation; no source or recovery file is created.

<ui:copy id="resource-recovery-list-route">
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
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_recovery_api.rs"><ui:paste select="#resource-recovery-api-preparation"/><ui:paste select="#resource-recovery-api-publication"/><ui:paste select="#resource-recovery-api-inspection"/><ui:paste select="#resource-recovery-list-route"/></ui:file>
## Check canonical entry restoration through the actual engine

Create the canonical entry through the resource API, then restore its recorded
prior absence. That historical deletion must remain supported. Next move an
explicitly selected entry to the canonical address: restoring it to another
address would silently lose implicit ownership, so it must refuse before creating
a recovery record. All files and records live under a temporary workspace.

<ui:copy id="resource-recovery-entry-service-witness">
"""Actual recovery admission keeps the address-bound canonical entry coherent."""
import importlib.util,json,os,tempfile,urllib.request,urllib.error
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
 spec=importlib.util.spec_from_file_location("fixture",ROOT/"scripts/test-simple-browser.py");fixture=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture)
 with tempfile.TemporaryDirectory(prefix="entry-restoration-") as temporary:
  folder=Path(temporary);root=folder/"workspace";root.mkdir();engine=None
  source='<h:doc xmlns:h="http://www.hickorydocs.com/1.0"><h:project sources=""/></h:doc>'
  try:
   engine=fixture.start([os.environ.get("HICKORY_SIMPLE_TEST_ENGINE",str(ROOT/"target/source-edit-focus/debug/examples/engine_client")),str(root),"window","http://127.0.0.1:1"],env={**os.environ,"HICKORY_STATE_DIR":str(folder/"state")});origin=fixture.origin(engine)
   def call(route,body):
    request=urllib.request.Request(origin+"/api/"+route,data=json.dumps(body).encode(),headers={"Content-Type":"application/json"},method="POST")
    with urllib.request.urlopen(request,timeout=30) as reply:return json.load(reply)
   def save(edit,context,identifier,entry=None):
    capture={"context":context,"edit":edit}
    if entry is not None:capture["project_entry"]=entry
    inputs=call("changes/resource-inputs",capture)
    plan=call("changes/plan-resources",{"context":context,"title":context,"edit":edit,"sources":inputs["sources"]})["plan"]
    proposal={"inputs":inputs,"capture":capture,"plan":plan};facts=call("changes/resource-consequences",proposal)
    return call("changes/resource-save",{"request_id":identifier,"reviewed_fingerprint":facts["fingerprint"],"proposal":proposal})
   def restoration(result,context,identifier):
    record=json.loads((root/result["outcome"]["recovery"]).read_text())
    capture={"context":context,"edit":{"changes":{"hick:///"+path:[] for path in record["after"]}}}
    proposal={"recovery_id":Path(result["outcome"]["recovery"]).parent.name,"inputs":call("changes/resource-inputs",capture),"capture":capture}
    prepared=call("changes/prepare-restoration",proposal)
    return {"request_id":identifier,"proposal":proposal,"reviewed_fingerprint":prepared["consequences"]["fingerprint"],"reviewed_recovery_fingerprint":prepared["restoration"]["recovery_fingerprint"]}
   created=save({"documentChanges":[{"kind":"create","uri":"hick:///project.md"},{"textDocument":{"uri":"hick:///project.md","version":None},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":source}]}]},"Create canonical entry","a"*32)
   assert created["outcome"]["complete"] and (root/"project.md").read_text()==source
   removed=call("changes/restore-resources",restoration(created,"Undo entry creation","b"*32))
   assert removed["outcome"]["complete"] and not (root/"project.md").exists()
   (root/"entry.md").write_text(source)
   moved=save({"documentChanges":[{"kind":"rename","oldUri":"hick:///entry.md","newUri":"hick:///project.md"}]},"Move selected entry to canonical address","c"*32,"entry.md")
   assert moved["outcome"]["complete"] and not (root/"entry.md").exists()
   record_folder=root/".hick-cache/resource-changes";before=set(record_folder.iterdir())
   try:call("changes/restore-resources",restoration(moved,"Refuse implicit canonical entry move","d"*32))
   except urllib.error.HTTPError as error:assert error.code==409 and "canonical project.md" in error.read().decode()
   else:raise AssertionError("Historical restoration silently moved canonical entry ownership")
   assert (root/"project.md").read_text()==source and not (root/"entry.md").exists() and set(record_folder.iterdir())==before
   print("PASS: canonical creation can be restored to absence, but historical moves cannot silently discard canonical entry ownership")
  finally:
   if engine:fixture.stop(engine)
if __name__=="__main__":main()
</ui:copy>

<ui:file path="scripts/test-resource-entry-restoration-service.py"><ui:paste select="#resource-recovery-entry-service-witness"/></ui:file>
</ui:doc>
