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

Ordinary resources prove that their original physical publication is still
current. A retained main transition also needs its live entry intent. The
[entry proof](workspace-entry-lineage.md) can carry authority through canceled
Move publications when their original physical files have been replaced by
their exact inverse producers.

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
        super::workspace_entry_authority::transitions::validate_restoration_origin(
            &state, &request.capture, &recovery,
        )?;
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
        let entry_transition=super::workspace_entry_authority::transitions::recovery_metadata(&root,&recovery);
        let mut value = summary(recovery, query, &workspace)?;value["entry_transition"]=entry_transition;
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

Create the canonical entry through the resource API, then review Undo to
absence and Redo through the explicit entry protocol. Move that same entry
to a different address. A generic inverse without a live entry intent must
refuse during preparation, before creating a publication record. The explicit
inverse still restores the source and role. Each accepted write advances the
original entry generation inside an owned temporary workspace.

The fresh engine witness observed creation, Undo to absence, Redo, and Move
to a `.spec` address. A generic inverse refused at preparation and left the
journal directory unchanged. A subsequent explicitly reviewed inverse restored
the canonical path, original logical identity and exact saved bytes.

<ui:copy id="resource-recovery-entry-service-witness">
"""Actual engine restores a canonical role only through explicit review."""
import importlib.util
import os
import tempfile
import urllib.error
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_script(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "scripts" / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    fixture = load_script("test-simple-browser")
    owned = load_script("test-agent-proposal-persistence")
    with tempfile.TemporaryDirectory(prefix="entry-restoration-") as temporary:
        folder = Path(temporary).resolve()
        root, state = folder / "workspace", folder / "state"
        root.mkdir()
        executable = os.environ.get("HICKORY_SIMPLE_TEST_ENGINE", str(ROOT / "target/source-edit-focus/debug/examples/engine_client"))
        engine = fixture.start([executable, str(root), "window", "http://127.0.0.1:1"], env={**os.environ, "HICKORY_STATE_DIR": str(state)})
        try:
            check(root, fixture.origin(engine), owned.call)
        finally:
            owned.stop_engine(fixture, engine, state)


def check(root, origin, request):
    source = '\x3ch:doc xmlns:h="http://www.hickorydocs.com/1.0">\x3ch:project sources=""/>\x3c/h:doc>'
    main_path, moved_path = "project.md", "moved.spec"

    def get(route):
        return request(origin, "GET", "/api/" + route)

    def post(route, body):
        return request(origin, "POST", "/api/" + route, body)

    def acquire(capture, token=None):
        if token is None:
            inputs = post("changes/resource-inputs", capture)
        else:
            for _ in range(2):
                inputs = post("workspace/entry/inputs", {"version": 1, "token": token, "capture": capture})
        assert not inputs["refusals"], inputs
        return {"inputs": inputs, "capture": capture}

    def entry_proposal(operation):
        selection = get("workspace/entry")
        intent = post("workspace/entry/intent", {"version": 1, "entry_selection": selection, "operation": operation})
        capture = {"context": intent["context"], "edit": intent["edit"]}
        if selection.get("source_guard"):
            capture["saved_guards"] = {selection["entry"]["path"]: selection["source_guard"]}
        return intent["title"], acquire(capture, intent["token"])

    def publish(title, proposal, identifier):
        proposal["plan"] = post("changes/plan-resources", {"context": proposal["capture"]["context"], "title": title, "edit": proposal["capture"]["edit"], "sources": proposal["inputs"]["sources"]})["plan"]
        facts = post("changes/resource-consequences", proposal)
        return post("changes/resource-save", {"request_id": identifier, "reviewed_fingerprint": facts["fingerprint"], "proposal": proposal})

    def recovery(result):
        identifier = Path(result["outcome"]["recovery"]).parent.name
        return get("changes/resource-recovery?id=" + identifier)

    def restore(result, identifier):
        record = recovery(result)
        _, proposal = entry_proposal({"kind": "restoration", "recovery_id": record["id"], "recovery_fingerprint": record["fingerprint"]})
        proposal["recovery_id"] = record["id"]
        reviewed = post("changes/prepare-restoration", proposal)
        return post("changes/restore-resources", {"request_id": identifier, "reviewed_fingerprint": reviewed["consequences"]["fingerprint"], "reviewed_recovery_fingerprint": reviewed["restoration"]["recovery_fingerprint"], "proposal": proposal})

    created = publish("Create canonical entry", acquire({"context": "create-canonical", "edit": {"documentChanges": [
        {"kind": "create", "uri": "hick:///project.md"},
        {"textDocument": {"uri": "hick:///project.md", "version": None}, "edits": [
            {"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}, "newText": source}
        ]},
    ]}}), "a" * 32)
    baseline = created["entry_outcome"]["current_entry"]

    def assert_entry(result, path, generation):
        assert result["outcome"]["complete"] and result["entry_outcome"]["state"] == "confirmed", result
        expected = {**baseline, "path": path, "generation": baseline["generation"] + generation}
        assert result["entry_outcome"]["current_entry"] == expected == get("workspace/entry")["entry"]
        for relative in (main_path, moved_path):
            if relative == path:
                assert (root / relative).read_text() == source
            else:
                assert not (root / relative).exists()

    assert_entry(created, main_path, 0)
    removed = restore(created, "b" * 32)
    assert_entry(removed, None, 1)
    assert_entry(restore(removed, "c" * 32), main_path, 2)
    title, proposal = entry_proposal({"kind": "move", "destination": moved_path})
    moved = publish(title, proposal, "d" * 32)
    assert_entry(moved, moved_path, 3)

    journals = root / ".hick-cache/resource-changes"
    original = set(journals.iterdir())
    record = recovery(moved)
    generic = acquire({"context": "unbound-inverse", "edit": {"changes": {"hick:///" + path: [] for path in (main_path, moved_path)}}})
    try:
        post("changes/prepare-restoration", {**generic, "recovery_id": record["id"]})
    except urllib.error.HTTPError as error:
        detail = error.read().decode()
        assert error.code == 409 and "fresh workspace-entry review" in detail, (error.code, detail)
    else:
        raise AssertionError("Unbound inverse acquired a retained main transition")
    assert set(journals.iterdir()) == original
    assert_entry(moved, moved_path, 3)
    assert_entry(restore(moved, "e" * 32), main_path, 4)
    print("PASS: explicit canonical creation Undo/Redo, unbound Move inverse refusal, guarded Move inverse, exact entry generations and bytes")


if __name__ == "__main__":
    main()
</ui:copy>

<ui:file path="scripts/test-resource-entry-restoration-service.py"><ui:paste select="#resource-recovery-entry-service-witness"/></ui:file>
</ui:doc>
