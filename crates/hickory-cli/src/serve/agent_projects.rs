//! One captured A/B boundary for every dock harness, including native ACP writes.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use crate::workspace_capture::{CAPTURE_BYTES, CAPTURE_ENTRIES};
use axum::{
    Json,
    extract::{Path, State},
};
use hick_lang::{AgentProjectCapture, ProjectInventory, ProjectSnapshot, ProjectTree};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct CapturedTurn {
    #[serde(skip)]
    persisted: bool,
    #[serde(default)]
    pub workspace: Option<crate::workspace_capture::WorkspaceCapture>,
    #[serde(skip)]
    pub live_editors: Option<std::sync::Arc<super::acp::edits::Edits>>,
    pub root: PathBuf,
    pub entry: String,
    pub documents: BTreeMap<String, String>,
    pub inventory: ProjectInventory,
    pub starting: ProjectSnapshot,
    pub completed: Option<ProjectSnapshot>,
    pub ambiguous: BTreeSet<String>,
    pub error: Option<String>,
    pub response: Option<String>,
    #[serde(default)]
    pub artifact: Option<crate::changeset::Artifact>,
    pub session: PathBuf,
    #[serde(default)]
    rooms: BTreeMap<String, String>,
}

pub async fn begin(
    state: &LocalState,
    doc: &str,
    session: PathBuf,
) -> anyhow::Result<Option<CapturedTurn>> {
    let Some(document) = state.index.absolute(doc) else {
        return Ok(None);
    };
    let source = if let Some(room) = state.rooms.get(doc).await {
        room.text().await
    } else {
        state
            .read_source(doc)
            .map_err(|e| anyhow::anyhow!("{}", e.detail()))?
    };
    if hick_lang::parse(&source)?.find_tags("project").is_empty() {
        return Ok(None);
    }
    let root = document
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Project needs a directory"))?
        .canonicalize()?;
    let entry = document
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Project needs a UTF-8 entry"))?
        .to_owned();
    let mut live = BTreeMap::new();
    let mut rooms = BTreeMap::new();
    for (id, _) in state.index.entries() {
        if let Some(room) = state.rooms.get(&id).await {
            let text = room.text().await;
            rooms.insert(id.clone(), text.clone());
            if let Some(path) = state.index.absolute(&id).and_then(|p| {
                p.strip_prefix(&root)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"))
            }) {
                live.insert(path, text);
            }
        }
    }
    let documents = hick_lang::load_project_sources(&entry, &|name| {
        if name == entry {
            return Ok(source.clone());
        }
        if let Some(source) = live.get(name) {
            return Ok(source.clone());
        }
        let path = root.join(name);
        if !std::fs::symlink_metadata(&path)
            .map_err(|e| e.to_string())?
            .is_file()
            || !path
                .canonicalize()
                .map_err(|e| e.to_string())?
                .starts_with(&root)
        {
            return Err(format!("Unconfined Markdown source: {name}"));
        }
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    })
    .map_err(anyhow::Error::msg)?;
    let project = hick_lang::evaluate_project(&entry, &documents, &ProjectTree::default())
        .map_err(anyhow::Error::msg)?;
    let starting = crate::project_revision::directory_bounded(
        &root,
        &project.inventory,
        CAPTURE_BYTES,
        CAPTURE_ENTRIES,
    )?;
    Ok(Some(CapturedTurn {
        persisted: false,
        workspace: None,
        live_editors: None,
        root,
        entry,
        documents,
        inventory: project.inventory,
        starting: ProjectSnapshot::capture(&starting),
        completed: None,
        ambiguous: BTreeSet::new(),
        error: None,
        response: None,
        artifact: None,
        session,
        rooms,
    }))
}

pub(super) fn editors(
    state: &LocalState,
    id: &str,
    python: bool,
    context: &super::agent_context::EditorContext,
) -> std::sync::Arc<super::acp::edits::Edits> {
    let key = if python {
        format!("python:{id}")
    } else {
        id.into()
    };
    let edits = super::agent_review::queue(state, &key).edits.clone();
    edits
        .live_context
        .store(context.live, std::sync::atomic::Ordering::Relaxed);
    *edits.buffers.lock().unwrap() = context.buffers.clone();
    edits
}

/// ACP workspace fallback: the folder flag, never the execution cwd, defines disk scope.
pub async fn begin_editors(
    state: &LocalState,
    doc: &str,
    session: PathBuf,
    edits: std::sync::Arc<super::acp::edits::Edits>,
) -> anyhow::Result<Option<CapturedTurn>> {
    anyhow::ensure!(
        edits
            .live_context
            .load(std::sync::atomic::Ordering::Relaxed)
            || edits.buffers.lock().unwrap().is_empty(),
        "This editor client cannot provide fresh completion snapshots. Update the editor client to send context.live and answer capture requests; ordinary conversation remains available without verified capture."
    );
    let declared = begin(state, doc, session.clone()).await?;
    // Editor overlays need the ordinary workspace inventory, including Markdown bytes.
    if edits.buffers.lock().unwrap().is_empty() && declared.is_some() {
        return Ok(declared);
    }
    let root = state
        .folder_open
        .then(|| state.index.root().canonicalize())
        .transpose()?;
    let buffers = edits.buffers.lock().unwrap().clone();
    let (workspace, starting) = tokio::task::spawn_blocking(move || {
        let (workspace, tree) = crate::workspace_capture::WorkspaceCapture::freeze(root, &buffers)?;
        Ok::<_, anyhow::Error>((workspace, ProjectSnapshot::capture(&tree)))
    })
    .await??;
    Ok(Some(CapturedTurn {
        persisted: false,
        root: workspace.root.clone().unwrap_or_default(),
        workspace: Some(workspace),
        live_editors: Some(edits),
        entry: "response.md".into(),
        documents: BTreeMap::new(),
        inventory: ProjectInventory::default(),
        starting,
        completed: None,
        ambiguous: BTreeSet::new(),
        error: None,
        response: None,
        artifact: None,
        session,
        rooms: BTreeMap::new(),
    }))
}

pub(super) fn cancelled(state: &LocalState, turn: &str) -> bool {
    state
        .agent
        .cancels
        .lock()
        .unwrap()
        .get(turn)
        .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Relaxed))
}

fn capture_sidecar(session: &std::path::Path, token: &str) -> anyhow::Result<PathBuf> {
    anyhow::ensure!(token.len() == 32 && token.bytes().all(|b| b.is_ascii_hexdigit()),
        "Invalid project capture reference");
    Ok(session.with_extension(format!("capture-{token}.json.gz")))
}

fn record_capture(turn: &str, capture: &CapturedTurn, resolution: Option<&str>) -> anyhow::Result<()> {
    let record = super::acp::record::Record::existing(capture.session.clone());
    record.check()?;
    // Publish the small transcript reference only after the complete capture is durable.
    let token = format!("{:016x}{:016x}", super::rand_id(), super::rand_id());
    let path = capture_sidecar(&capture.session, &token)?;
    let mut file = tempfile::NamedTempFile::new_in(path.parent().unwrap_or(std::path::Path::new(".")))?;
    let writer = std::io::BufWriter::new(file.as_file_mut());
    let mut compressed = flate2::write::GzEncoder::new(writer, flate2::Compression::fast());
    serde_json::to_writer(&mut compressed, capture)?;
    compressed.finish()?.into_inner().map_err(|error| error.into_error())?.sync_all()?;
    file.persist_noclobber(&path)?;
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        std::fs::File::open(parent)?.sync_all()?;
    }
    record.context("literate-project-capture", &json!({"turn":turn,"capture_ref":token,"resolution":resolution}))
}

pub async fn finish(
    state: &LocalState,
    turn: &str,
    mut capture: CapturedTurn,
    explanation: Option<&str>,
) {
    let result = async {
        let completed = if let Some(scope) = &capture.workspace {
            let editors = capture
                .live_editors
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Live editor capture unavailable"))?;
            if explanation.is_some() {
                tokio::select! {
                    result = editors.refresh() => result?,
                    _ = async { while !cancelled(state, turn) { tokio::time::sleep(std::time::Duration::from_millis(50)).await; } } => {
                        editors.cancel();
                        anyhow::bail!(hickory_agent::STOPPED_BY_USER);
                    }
                }
            }
            let scope = scope.clone();
            let buffers = editors.buffers.lock().unwrap().clone();
            tokio::task::spawn_blocking(move || scope.current(&buffers)).await??
        } else {
            crate::project_revision::directory_bounded(
                &capture.root, &capture.inventory, CAPTURE_BYTES, CAPTURE_ENTRIES,
            )?
        };
        // Native filesystem writes have no exclusive writer identity. Do not
        // silently call an external editor's write the agent's successful act.
        let starting = capture.starting.restore().map_err(anyhow::Error::msg)?;
        capture
            .ambiguous
            .extend(starting.compare(&completed).into_iter().map(|d| d.path));
        for (id, source) in &capture.rooms {
            if let Some(room) = state.rooms.get(id).await
                && room.text().await != *source
            {
                capture.ambiguous.insert(format!(
                    "editor:{}",
                    state.index.path_of(id).unwrap_or(id.clone())
                ));
            }
        }
        if capture
            .artifact
            .as_ref()
            .is_some_and(|a| a.expected != crate::changeset::fingerprint(&completed))
        {
            capture.artifact = None;
            capture.response = None;
        }
        capture.completed = Some(ProjectSnapshot::capture(&completed));
        if explanation.is_none() {
            anyhow::bail!("Turn did not complete successfully; its actual writes remain captured.");
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;
    capture.error = result.err().map(|e| format!("{e:#}"));
    let turn_id = turn.to_owned();
    let Ok(capture) = tokio::task::spawn_blocking(move || {
        match record_capture(&turn_id, &capture, None) {
            Ok(()) => capture.persisted = true,
            Err(e) => capture.error = Some(format!("Project capture could not be recorded: {e:#}")),
        }
        capture
    }).await else { return };
    state
        .agent
        .projects
        .lock()
        .unwrap()
        .insert(turn.into(), capture);
}

/// Completed captures are read lazily from retained evidence, not kept forever in RAM.
pub(super) fn release(state: &LocalState, turn: &str) {
    let mut projects = state.agent.projects.lock().unwrap();
    if projects.get(turn).is_some_and(|capture| capture.persisted) {
        projects.remove(turn);
    }
}

pub(super) fn captured(state: &LocalState, doc: &str, turn: &str) -> ApiResult<CapturedTurn> {
    let record = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(doc)
        .and_then(|ts| ts.iter().find(|t| t.id == turn))
        .cloned()
        .ok_or_else(|| ApiError::not_found("No turn in this conversation."))?;
    if let Some(capture) = state.agent.projects.lock().unwrap().get(turn).cloned() {
        return Ok(capture);
    }
    let session = state.index.root().join(record.session);
    let source = std::fs::read_to_string(&session)
        .map_err(|_| ApiError::not_found("This turn has no declared-project capture."))?;
    let parsed = hick_lang::parse(&source).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let value = parsed
        .all_tags()
        .into_iter()
        .filter(|t| {
            t.name == "context"
                && matches!(
                    t.get_attribute("kind"),
                    Some("literate-project-capture" | "literate-project-capture-error")
                )
        })
        .filter_map(|t| serde_json::from_str::<Value>(&t.text_content()).ok())
        .rfind(|v| v["turn"] == turn)
        .ok_or_else(|| {
            ApiError::not_found(
                "This turn has no complete project capture. Start a turn on a declared project.",
            )
        })?;
    if let Some(error) = value["error"].as_str() {
        return Err(ApiError::unprocessable(format!(
            "Incomplete project capture: {error}"
        )));
    }
    let capture = if let Some(reference) = value["capture_ref"].as_str() {
        read_capture(&session, reference).map_err(|e| ApiError::unprocessable(format!("Project capture unavailable: {e:#}")))?
    } else {
        // Inline evidence from older releases remains readable.
        serde_json::from_value(value["capture"].clone())
            .map_err(|e| ApiError::unprocessable(e.to_string()))?
    };
    Ok(capture)
}

fn read_capture(session: &std::path::Path, reference: &str) -> anyhow::Result<CapturedTurn> {
    use std::io::{BufReader, Read};
    let path = capture_sidecar(session, reference)?;
    anyhow::ensure!(std::fs::symlink_metadata(&path)?.is_file(), "Capture is not a regular file");
    let file = std::fs::File::open(path)?;
    let reader = flate2::read::GzDecoder::new(file);
    // Covers both maximum folder snapshots serialized as byte arrays, plus metadata.
    const MAX_CAPTURE_JSON: u64 = 2 * 1024 * 1024 * 1024;
    let mut reader = BufReader::new(reader.take(MAX_CAPTURE_JSON + 1));
    let capture = serde_json::from_reader(&mut reader)?;
    anyhow::ensure!(reader.get_ref().limit() > 0, "Capture exceeds its decoding budget");
    Ok(capture)
}

pub async fn snapshot(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
) -> ApiResult<Json<Value>> {
    let capture = captured(&state, &doc, &turn)?;
    Ok(Json(
        json!({"workspace":capture.workspace,"entry":capture.entry,"ambiguous":capture.ambiguous,"error":capture.error,"response":capture.response,"artifact":capture.artifact,"starting":capture.starting,"completed":capture.completed}),
    ))
}

#[derive(Deserialize)]
pub struct Finalize {
    #[serde(default)]
    pub resolve: Vec<String>,
}
pub async fn finalize(
    State(state): State<LocalState>,
    Path((doc, turn)): Path<(String, String)>,
    Json(body): Json<Finalize>,
) -> ApiResult<Json<Value>> {
    let mut capture = captured(&state, &doc, &turn)?;
    if let Some(error) = &capture.error {
        return Err(ApiError::unprocessable(format!(
            "Incomplete capture: {error}"
        )));
    }
    for path in body.resolve {
        if !capture.ambiguous.remove(&path) {
            return Err(ApiError::bad_request(format!(
                "No unresolved captured change: {path}"
            )));
        }
    }
    let mut boundary = AgentProjectCapture::new(
        capture
            .starting
            .restore()
            .map_err(ApiError::unprocessable)?,
    );
    for path in &capture.ambiguous {
        boundary.concurrent_write(path);
    }
    let completed = capture
        .completed
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("Actual completed output state is unavailable."))?
        .restore()
        .map_err(ApiError::unprocessable)?;
    let explanation = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(&doc)
        .and_then(|ts| ts.iter().find(|t| t.id == turn))
        .and_then(|t| t.answer.clone())
        .ok_or_else(|| ApiError::unprocessable("No successful final answer."))?;
    let response = if let Some(artifact) = &capture.artifact {
        if !capture.ambiguous.is_empty() {
            return Err(ApiError::conflict(
                "Resolve concurrent writes before attributing the transition.",
            ));
        }
        let differences = crate::changeset::verify_sources(
            &capture
                .starting
                .restore()
                .map_err(ApiError::unprocessable)?,
            &completed,
            &artifact.entry,
            &artifact.sources,
        )
        .map_err(|e| ApiError::conflict(e.to_string()))?;
        if !differences.is_empty() {
            return Err(ApiError::conflict(
                "Submitted artifact does not reproduce the captured completed tree.",
            ));
        }
        artifact
            .sources
            .get(&artifact.entry)
            .cloned()
            .ok_or_else(|| ApiError::unprocessable("Artifact entry is missing."))?
    } else {
        boundary
            .finalize(&completed, &explanation)
            .map_err(ApiError::conflict)?
    };
    if capture.artifact.is_none() {
        let entry = crate::changeset::response_entry(
            &capture
                .starting
                .restore()
                .map_err(ApiError::unprocessable)?,
            &completed,
        );
        capture.artifact = Some(crate::changeset::Artifact {
            sources: BTreeMap::from([(entry.clone(), response.clone())]),
            entry,
            expected: crate::changeset::fingerprint(&completed),
        });
    }
    capture.response = Some(response.clone());
    record_capture(&turn, &capture, Some("user-declared"))
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    state.agent.projects.lock().unwrap().remove(&turn);
    Ok(Json(json!({"response":response,"verified":true})))
}

pub(super) fn completed_capture(
    state: &LocalState,
    doc: &str,
    turn: &str,
) -> ApiResult<CapturedTurn> {
    let capture = captured(state, doc, turn)?;
    if capture.workspace.is_some() {
        return Err(ApiError::conflict(
            "Workspace/editor captures have no maintained literate source destination. Save a declared project explicitly before incorporation.",
        ));
    }
    if capture.response.is_none() || capture.error.is_some() || !capture.ambiguous.is_empty() {
        return Err(ApiError::conflict(
            "Finalize the captured transition and resolve concurrent changes before incorporation.",
        ));
    }
    Ok(capture)
}
#[cfg(test)]
mod capture_lifetime_tests {
    use super::*;

    // Guarantee: .hickory-source/guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md
    #[tokio::test]
    async fn release_preserves_unrecorded_evidence_and_drops_durable_snapshots() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("note.md"), "note").unwrap();
        let prepared = super::super::prepare(super::super::ServeOptions {
            target: root.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let state = prepared.state;
        let doc = state.index.sole().unwrap().0;
        let session = root.path().join("session.md");
        let capture = begin_editors(&state, &doc, session.clone(), Default::default())
            .await
            .unwrap()
            .unwrap();
        state
            .agent
            .projects
            .lock()
            .unwrap()
            .insert("turn".into(), capture.clone());
        release(&state, "turn");
        assert!(state.agent.projects.lock().unwrap().contains_key("turn"));
        // A missing session exercises a real recording failure, not just the flag.
        finish(&state, "turn", capture.clone(), None).await;
        assert!(!state.agent.projects.lock().unwrap()["turn"].persisted);
        release(&state, "turn");
        assert!(state.agent.projects.lock().unwrap().contains_key("turn"));
        super::super::acp::record::Record::open(session.clone(), &root.path().join("note.md"))
            .unwrap();
        finish(&state, "turn", capture, None).await;
        assert!(state.agent.projects.lock().unwrap()["turn"].persisted);
        release(&state, "turn");
        assert!(!state.agent.projects.lock().unwrap().contains_key("turn"));
        let source = std::fs::read_to_string(&session).unwrap();
        let parsed = hick_lang::parse(&source).unwrap();
        let row = parsed.all_tags().into_iter().find(|tag| {
            tag.get_attribute("kind") == Some("literate-project-capture")
        }).unwrap();
        let row: Value = serde_json::from_str(&row.text_content()).unwrap();
        assert!(row.get("capture").is_none(), "history should not embed snapshot bytes");
        let loaded = read_capture(&session, row["capture_ref"].as_str().unwrap()).unwrap();
        assert_eq!(&*loaded.starting.restore().unwrap().get("note.md").unwrap().bytes, b"note");
        assert!(read_capture(&session, "../foreign.json.gz").is_err());
    }
}
pub(super) struct RecordedPatchInspection {
    pub entry: String,
    pub document: String,
    pub operation_id: String,
    pub opening_byte: usize,
    pub completed_fingerprint: String,
    pub changes: Vec<Value>,
}

pub(super) fn inspect_recorded_patch(
    capture: &CapturedTurn,
    source: &str,
    opening_line: usize,
) -> ApiResult<RecordedPatchInspection> {
    const MAX_ROWS: usize = 128;
    const MAX_PATH: usize = 1024;
    const MAX_INLINE_ENTRY: usize = 16 * 1024;
    const MAX_INLINE_TOTAL: usize = 64 * 1024;
    if capture.error.is_some() || !capture.ambiguous.is_empty() || source.is_empty() {
        return Err(ApiError::conflict("Finalize the original capture and resolve attribution before inspecting a declared patch."));
    }
    let artifact = capture.artifact.as_ref()
        .ok_or_else(|| ApiError::conflict("This turn has no verified captured construction."))?;
    if capture.response.as_ref() != artifact.sources.get(&artifact.entry) || capture.response.is_none() {
        return Err(ApiError::conflict("This turn has no finalized captured construction."));
    }
    let completed = capture.completed.as_ref()
        .ok_or_else(|| ApiError::conflict("The completed tree for this construction is unavailable."))?;
    let source_bytes = artifact.sources.values().try_fold(0_u64, |total, text| total.checked_add(text.len() as u64));
    if artifact.sources.len() > CAPTURE_ENTRIES || !source_bytes.is_some_and(|bytes| bytes <= CAPTURE_BYTES) {
        return Err(ApiError::unprocessable("The captured Markdown closure exceeds the inspection limit."));
    }
    let mut owners = artifact.sources.iter().filter(|(_, text)| text.as_str() == source);
    let (document, _) = owners.next()
        .ok_or_else(|| ApiError::conflict("This message is not an exact document in the captured construction."))?;
    if owners.next().is_some() {
        return Err(ApiError::conflict("The message matches more than one captured document."));
    }
    let starting = capture.starting.restore().map_err(ApiError::unprocessable)?;
    let completed = completed.restore().map_err(ApiError::unprocessable)?;
    let fingerprint = crate::changeset::fingerprint(&completed);
    if fingerprint != artifact.expected {
        return Err(ApiError::conflict("The captured construction fingerprint differs from its completed tree."));
    }
    let (project, patch) = hick_lang::inspect_project_patch(
        &artifact.entry, &artifact.sources, &starting, document, opening_line,
    ).map_err(ApiError::unprocessable)?;
    if !artifact.sources.keys().all(|path| project.inventory.documents.contains(path)) ||
        !project.new.compare(&completed).is_empty()
    {
        return Err(ApiError::conflict("The captured construction does not reproduce its complete recorded result."));
    }
    let differences = patch.before.compare(&patch.after);
    if differences.is_empty() {
        return Err(ApiError::unprocessable("The selected declared patch has no output changes."));
    }
    if differences.len() > MAX_ROWS || differences.iter().any(|row| row.path.len() > MAX_PATH)
        || artifact.entry.len() > MAX_PATH || document.len() > MAX_PATH || patch.id.len() > MAX_PATH
    {
        return Err(ApiError::unprocessable("The declared patch exceeds the bounded comparison size."));
    }
    let mut inline_remaining = MAX_INLINE_TOTAL;
    let mut describe = |entry: Option<&std::sync::Arc<hick_lang::ProjectEntry>>| {
        let Some(entry) = entry else { return Value::Null };
        let (text, text_status) = match std::str::from_utf8(&entry.bytes) {
            Ok(text) if text.len() <= MAX_INLINE_ENTRY && text.len() <= inline_remaining => {
                inline_remaining -= text.len();
                (Some(text), "exact")
            }
            Ok(_) => (None, "over_limit"),
            Err(_) => (None, "non_utf8"),
        };
        let kind = match entry.kind { hick_lang::EntryKind::File => "file", hick_lang::EntryKind::Symlink => "symlink" };
        json!({
            "kind": kind, "executable": entry.executable, "bytes": entry.bytes.len(),
            "sha256": super::source_tests::digest(&entry.bytes), "text": text, "text_status": text_status,
        })
    };
    let changes = differences.into_iter().map(|row| {
        let before = describe(patch.before.get(&row.path));
        let after = describe(patch.after.get(&row.path));
        json!({"path": row.path, "before": before, "after": after})
    }).collect();
    Ok(RecordedPatchInspection {
        entry: artifact.entry.clone(), document: document.clone(), operation_id: patch.id,
        opening_byte: patch.opening_byte, completed_fingerprint: fingerprint, changes,
    })
}
#[cfg(test)]
mod recorded_declared_patch_tests {
    use super::*;

    fn capture(
        root: &std::path::Path,
        session: PathBuf,
        sources: BTreeMap<String, String>,
        starting: &ProjectTree,
    ) -> CapturedTurn {
        let entry = "response.md";
        let evaluated = hick_lang::evaluate_project(entry, &sources, starting).unwrap();
        CapturedTurn {
            persisted: true,
            workspace: None,
            live_editors: None,
            root: root.to_path_buf(),
            entry: entry.into(),
            documents: BTreeMap::new(),
            inventory: evaluated.inventory,
            starting: ProjectSnapshot::capture(starting),
            completed: Some(ProjectSnapshot::capture(&evaluated.new)),
            ambiguous: BTreeSet::new(),
            error: None,
            response: Some(sources[entry].clone()),
            artifact: Some(crate::changeset::Artifact {
                entry: entry.into(),
                sources,
                expected: crate::changeset::fingerprint(&evaluated.new),
            }),
            session,
            rooms: BTreeMap::new(),
        }
    }

    fn inputs() -> ProjectTree {
        let mut tree = ProjectTree::default();
        tree.insert("alpha.txt", hick_lang::ProjectEntry::file(&b"old\n"[..]))
            .unwrap();
        tree.insert("beta.txt", hick_lang::ProjectEntry::file(&b"other\n"[..]))
            .unwrap();
        tree
    }

    fn declarations() -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                "response.md".into(),
                concat!(
                    "<hick:project sources=\"other.md\"/>\n",
                    "Explanation.\n",
                    "<hick:patch id=\"selected\">\n",
                    "--- a/alpha.txt\n+++ b/alpha.txt\n@@ -1 +1 @@\n-old\n+middle\n",
                    "</hick:patch>\n",
                    "<hick:patch id=\"later\" after=\"selected\">\n",
                    "--- a/alpha.txt\n+++ b/alpha.txt\n@@ -1 +1 @@\n-middle\n+final\n",
                    "</hick:patch>\n",
                )
                .into(),
            ),
            (
                "other.md".into(),
                concat!(
                    "Separate document.\n",
                    "Different patch on the same line.\n",
                    "<hick:patch id=\"unrelated\">\n",
                    "--- a/beta.txt\n+++ b/beta.txt\n@@ -1 +1 @@\n-other\n+changed\n",
                    "</hick:patch>\n",
                )
                .into(),
            ),
        ])
    }

    #[tokio::test]
    async fn recorded_declared_patch_binds_its_owning_document_and_local_transition() {
        let temporary = tempfile::tempdir().unwrap();
        let source = temporary.path().join("alpha.txt");
        std::fs::write(&source, "independent working bytes\n").unwrap();
        let prepared = super::super::prepare(super::super::ServeOptions {
            target: temporary.path().to_path_buf(),
            port: 0,
            params: vec![],
            executor: crate::ExecutorChoice::Local,
            key_store_path: None,
            ui_settings_path: None,
        })
        .await
        .unwrap();
        let state = prepared.state;
        let root = state.index.root().canonicalize().unwrap();
        let session_address = "sessions/owned.md";
        let session = state.index.root().join(session_address);
        std::fs::create_dir_all(session.parent().unwrap()).unwrap();
        std::fs::write(&session, "retained conversation fixture\n").unwrap();

        let sources = declarations();
        let selected = sources["response.md"].clone();
        let captured = capture(&root, session.clone(), sources, &inputs());
        let turn = super::super::agent::TurnRecord {
            backend: "fixture".into(),
            id: "owned".into(),
            parent_id: None,
            parent_is_user: false,
            prompt: "Different recorded user message".into(),
            answer: Some(selected.clone()),
            status: "ok".into(),
            error: None,
            created_at: String::new(),
            provider: String::new(),
            model: String::new(),
            usage: None,
            session: session_address.into(),
        };
        state
            .agent
            .turns
            .lock()
            .unwrap()
            .insert("workspace".into(), vec![turn]);
        state
            .agent
            .projects
            .lock()
            .unwrap()
            .insert("owned".into(), captured.clone());
        let request = |source: &str| {
            serde_json::from_value(json!({
                "side": "assistant", "opening_line": 2,
                "source_hash": super::super::source_tests::digest(source.as_bytes()),
            }))
            .unwrap()
        };
        let (doc, turn) = ("workspace".to_owned(), "owned".to_owned());
        let read = super::super::agent::inspect_fence_construction(
            State(state.clone()),
            Path((doc.clone(), turn.clone())),
            Json(request(&selected)),
        )
        .await
        .unwrap()
        .0;
        assert_eq!(read["read_only"], true);
        assert_eq!(read["side"], "assistant");
        assert_eq!(read["entry"], "response.md");
        assert_eq!(read["document"], "response.md");
        assert_eq!(read["operation_id"], "selected");
        assert_eq!(read["opening_byte"], selected.find("<hick:patch").unwrap());
        let changes = read["changes"].as_array().unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0]["path"], "alpha.txt");
        assert_eq!(changes[0]["before"]["text"], "old\n");
        assert_eq!(changes[0]["after"]["text"], "middle\n");
        assert_eq!(changes[0]["after"]["text_status"], "exact");
        assert!(read.get("token").is_none());

        let stale = super::super::agent::inspect_fence_construction(
            State(state.clone()),
            Path((doc.clone(), turn.clone())),
            Json(request("stale bytes")),
        )
        .await;
        assert!(stale.unwrap_err().detail().contains("message changed"));
        state.agent.projects.lock().unwrap().remove("owned");
        let absent = super::super::agent::inspect_fence_construction(
            State(state),
            Path((doc, turn)),
            Json(request(&selected)),
        )
        .await;
        assert!(absent.is_err());
        assert_eq!(
            std::fs::read_to_string(source).unwrap(),
            "independent working bytes\n"
        );
        assert_eq!(
            std::fs::read_to_string(session).unwrap(),
            "retained conversation fixture\n"
        );
        assert!(!root.join("beta.txt").exists());
    }

    #[test]
    fn recorded_declared_patch_rejects_ambiguous_stale_bare_and_empty_transitions() {
        let root = std::path::Path::new("/owned");
        let start = inputs();
        let mut sources = declarations();
        let selected = sources["response.md"]
            .replacen(
                "<hick:patch id=\"selected\">",
                "```patch id=\"selected\"",
                1,
            )
            .replacen("</hick:patch>", "```", 1);
        sources.insert("response.md".into(), selected.clone());
        let captured = capture(root, root.join("sessions/owned.md"), sources, &start);
        let exact = inspect_recorded_patch(&captured, &selected, 2).unwrap();
        assert_eq!(exact.operation_id, "selected");
        assert_eq!(exact.changes[0]["after"]["text"], "middle\n");
        assert!(inspect_recorded_patch(&captured, &format!("Changed.\n{selected}"), 2).is_err());
        assert!(inspect_recorded_patch(&captured, &selected, 3).is_err());

        let mut duplicate = captured.clone();
        duplicate
            .artifact
            .as_mut()
            .unwrap()
            .sources
            .insert("duplicate.md".into(), selected.clone());
        assert!(inspect_recorded_patch(&duplicate, &selected, 2).is_err());
        let mut stale = captured;
        stale.completed = Some(ProjectSnapshot::capture(&start));
        stale.artifact.as_mut().unwrap().expected = crate::changeset::fingerprint(&start);
        assert!(inspect_recorded_patch(&stale, &selected, 2).is_err());

        let patch = "--- a/alpha.txt\n+++ b/alpha.txt\n";
        let bare = format!("<hick:project/>\n```patch\n{patch}@@ -1 +1 @@\n-old\n+changed\n```\n");
        let empty = format!(
            "<hick:project/>\n<hick:patch id=\"empty\">\n{patch}@@ -0,0 +0,0 @@\n</hick:patch>\n"
        );
        for message in [bare, empty] {
            let fixture = capture(
                root,
                root.join("sessions/owned.md"),
                BTreeMap::from([("response.md".into(), message.clone())]),
                &start,
            );
            assert!(inspect_recorded_patch(&fixture, &message, 1).is_err());
        }
    }
}
