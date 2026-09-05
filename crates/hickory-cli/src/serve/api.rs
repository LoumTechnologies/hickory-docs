//! The REST subset the document view needs, answered from files.
//!
//! Every handler here has a hosted counterpart in `apps/server/src/routes/`.
//! They agree on the wire shape — the same React client talks to both, and
//! `docs/specs/freeform/api.md` is the contract — but not on where the answer
//! comes from: there is no database, no run history, and no account. A weave
//! is computed from the file in front of us, which makes the local answers
//! *fresher* than the hosted ones (which serve the last successful run).
//!
//! What is deliberately absent: billing and analytics. The agent is real —
//! see [`super::agent`] — the same ReAct loop `hick agent` runs, wired to the
//! chat dock.

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};

use super::{LocalState, RunRecord};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// The error body shape the web client parses (`{"error": "…"}`).
/// Debug so a `Result<_, ApiError>` can be unwrapped in tests.
#[derive(Debug)]
pub struct ApiError(StatusCode, String, Option<Value>);

impl ApiError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into(), None)
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into(), None)
    }
    /// The message, for a caller that has to fold it into another error.
    pub fn detail(&self) -> &str {
        &self.1
    }
    pub fn unprocessable(msg: impl Into<String>) -> Self {
        Self(StatusCode::UNPROCESSABLE_ENTITY, msg.into(), None)
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self(StatusCode::FORBIDDEN, msg.into(), None)
    }
    pub fn conflict(msg: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, msg.into(), None)
    }
    pub fn unavailable(msg: impl Into<String>) -> Self {
        Self(StatusCode::SERVICE_UNAVAILABLE, msg.into(), None)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, msg.into(), None)
    }
    /// The human-readable half, for a test that wants to assert an error
    /// SAYS the thing rather than merely has a status.
    pub fn message(&self) -> &str {
        &self.1
    }
    pub fn with_detail(mut self, detail: Value) -> Self {
        self.2 = Some(detail);
        self
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({ "error": self.1 });
        if let Some(Value::Object(extra)) = self.2
            && let Some(obj) = body.as_object_mut()
        {
            obj.extend(extra);
        }
        (self.0, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::internal(format!("{e:#}"))
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

// ---------------------------------------------------------------------------
// Identity and project shape
// ---------------------------------------------------------------------------

/// `GET /api/projects` — the served directory, as one project.
pub async fn projects(State(state): State<LocalState>) -> Json<Value> {
    Json(json!([project_of(&state)]))
}

fn project_of(state: &LocalState) -> Value {
    let name = state
        .index
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "hick".to_string());
    json!({
        "id": "local",
        "name": name,
        "visibility": "private",
        "created_at": "1970-01-01T00:00:00Z",
    })
}

/// `GET /api/projects/:id/docs` — every `.hick` file under the root.
pub async fn project_docs(State(state): State<LocalState>) -> Json<Value> {
    let docs: Vec<Value> = state
        .index
        .entries()
        .into_iter()
        .map(|(id, path)| {
            json!({
                "id": id,
                "path": path,
                "updated_at": modified_at(&state, &id),
            })
        })
        .collect();
    Json(json!(docs))
}

/// Where a new document would go, refusing the paths that are not a place to
/// put one.
///
/// Shared by [`create_doc`] and the New Project route
/// ([`super::scaffold::create`]): the rules about what a document's path may
/// be belong to the folder, not to whichever button was pressed, and two
/// copies of them would drift the moment one grew a rule.
pub(crate) fn new_doc_target(
    state: &LocalState,
    path: &str,
) -> ApiResult<(String, std::path::PathBuf)> {
    let rel = path.trim().trim_start_matches(['/', '\\']).to_string();
    if rel.is_empty() || rel.contains("..") {
        return Err(ApiError::bad_request(
            "a document's path must be inside this folder, and cannot be empty",
        ));
    }
    if !rel.ends_with(".hick") {
        return Err(ApiError::bad_request(
            "a document's path must end in `.hick` — that is what makes it a document rather \
             than one of the files it generates",
        ));
    }

    let absolute = state.index.root().join(&rel);
    if absolute.exists() {
        return Err(ApiError::unprocessable(format!(
            "{rel} already exists. Open it, or choose another name."
        )));
    }
    if let Some(parent) = absolute.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ApiError::internal(format!("could not create {}: {e}", parent.display()))
        })?;
    }
    Ok((rel, absolute))
}

/// `POST /api/projects/:id/docs` — create a document in the served folder.
///
/// The app's "new document" button. A document is a file, so this writes one
/// and tells the index about it: the scan happens at startup, and a file
/// nothing knows about is a file nothing can open. Refuses to overwrite,
/// because "new" is not a way to lose something.
pub async fn create_doc(
    State(state): State<LocalState>,
    Json(body): Json<NewDoc>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let (rel, absolute) = new_doc_target(&state, &body.path)?;
    std::fs::write(&absolute, &body.source)
        .map_err(|e| ApiError::internal(format!("could not write {}: {e}", absolute.display())))?;

    let id = state.index.add(&rel);
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id,
            "path": rel,
            "source": body.source,
            "updated_at": modified_at(&state, &id),
        })),
    ))
}

/// Body of `POST /projects/:id/docs`.
#[derive(serde::Deserialize)]
pub struct NewDoc {
    pub path: String,
    #[serde(default)]
    pub source: String,
}

fn modified_at(state: &LocalState, id: &str) -> String {
    let stamp = state
        .index
        .absolute(id)
        .and_then(|p| std::fs::metadata(p).ok())
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // RFC 3339 without pulling in a date library: the client only displays it.
    let days = stamp / 86_400;
    let secs = stamp % 86_400;
    let (y, m, d) = civil_from_days(days as i64);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Days since the Unix epoch → (year, month, day). Howard Hinnant's
/// `civil_from_days`, which is exact and needs no table.
pub fn civil_from_days_public(z: i64) -> (i64, u32, u32) {
    civil_from_days(z)
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ---------------------------------------------------------------------------
// Documents
// ---------------------------------------------------------------------------

fn doc_json(state: &LocalState, id: &str) -> ApiResult<Value> {
    let path = state
        .index
        .path_of(id)
        .ok_or_else(|| ApiError::not_found(format!("no document {id} under the served directory")))?
        .to_string();
    let source = state.read_source(id)?;
    Ok(json!({
        "id": id,
        "path": path,
        "source": source,
        "updated_at": modified_at(state, id),
    }))
}

/// `GET /api/docs/:id`
pub async fn get_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    Ok(Json(doc_json(&state, &id)?))
}

#[derive(Deserialize)]
pub struct SaveDoc {
    pub source: String,
}

/// `PUT /api/docs/:id` — write the file, and tell the live room.
pub async fn put_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<SaveDoc>,
) -> ApiResult<Json<Value>> {
    state.write_source(&id, &body.source)?;
    // A room holding the pre-save text would write it straight back over this
    // on its next debounce.
    state.rooms.apply_external_source(&id, &body.source).await;
    Ok(Json(doc_json(&state, &id)?))
}

/// `GET /api/elements` — the vocabulary the app draws, as data: every
/// element's name, the component that draws it, its attributes and its
/// actions. Read from the registry, so it cannot disagree with `/render`.
pub async fn elements() -> Json<Value> {
    Json(json!({ "elements": hick_literate::render::describe_elements() }))
}

/// `POST /api/docs/:id/blocks/:at/:action` — one action on one element,
/// addressed by the byte offset its tag starts at.
///
/// The element answers with what it wants (`hick_blocks::ActionOutcome`)
/// and this handler carries it out with the machinery the older routes
/// already use: a `run` starts a run exactly as `POST /run` does and
/// answers `202` with the run id; an `edit` writes the document as `PUT`
/// does; an `answer` is returned as is. An element that does not know the
/// action is a `404` naming the element; one that refuses is a `422` in its
/// own words.
pub async fn block_action(
    State(state): State<LocalState>,
    Path((id, at, action)): Path<(String, usize, String)>,
    body: Option<Json<Value>>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    use hick_blocks::{ActionError, ActionOutcome};
    let run = state.weave(&id).await?;
    let body = body.map(|Json(v)| v).unwrap_or(Value::Null);
    let outcome = crate::block_action(&run, at, &action, body).map_err(|e| match e {
        ActionError::Unknown { .. } => ApiError::not_found(e.to_string()),
        ActionError::Refused(_) => ApiError::unprocessable(e.to_string()),
        ActionError::Failed(_) => ApiError::internal(e.to_string()),
    })?;
    match outcome {
        ActionOutcome::Answer { value } => Ok((
            StatusCode::OK,
            Json(json!({ "outcome": "answer", "value": value })),
        )),
        ActionOutcome::Run { cells } => {
            let run_id = state.start_run(&id, false).await?;
            Ok((
                StatusCode::ACCEPTED,
                Json(json!({ "outcome": "run", "run_id": run_id, "cells": cells })),
            ))
        }
        ActionOutcome::Edit { span, replacement } => {
            let source = run.doc.source.clone();
            if span.0 > span.1
                || span.1 > source.len()
                || !source.is_char_boundary(span.0)
                || !source.is_char_boundary(span.1)
            {
                return Err(ApiError::unprocessable(format!(
                    "the element asked to replace bytes {}..{} of a {}-byte document",
                    span.0,
                    span.1,
                    source.len()
                )));
            }
            let mut next = String::with_capacity(source.len() + replacement.len());
            next.push_str(&source[..span.0]);
            next.push_str(&replacement);
            next.push_str(&source[span.1..]);
            state.write_source(&id, &next)?;
            state.rooms.apply_external_source(&id, &next).await;
            Ok((
                StatusCode::OK,
                Json(
                    json!({ "outcome": "edit", "span": [span.0, span.1], "doc": doc_json(&state, &id)? }),
                ),
            ))
        }
    }
}

/// `GET /api/docs/:id/render` — the block model for the current source.
///
/// Weave, never execute: the client asks for this on every load and after
/// every save, and a render that ran the document would turn scrolling into
/// arbitrary code execution.
pub async fn render_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    Ok(Json(crate::block_model_json(&run).map_err(|e| {
        ApiError::unprocessable(format!("render failed: {e}"))
    })?))
}

// ---------------------------------------------------------------------------
// Outputs — the lineage ribbons
// ---------------------------------------------------------------------------

/// `GET /api/docs/:id/outputs`
pub async fn list_outputs(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let mut files: Vec<Value> = run
        .result
        .files
        .iter()
        .filter(|(_, content)| content.as_text().is_some())
        .map(|(path, _)| json!({ "path": path, "language": language_of(path) }))
        .collect();
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(Json(json!({ "files": files })))
}

/// `GET /api/structure` — definitions, references, and the links between
/// them, for every generated file in the session.
///
/// Structural, not exact: resolution is by name (see `hick_structure`). It is
/// here because it is the only navigation that works on a machine with no
/// language servers and no toolchain, which is most machines that just
/// installed a download. The client draws these links as a different kind
/// from the ones the weaver computed, because they are a different kind of
/// claim.
pub async fn structure(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let mut files = Vec::new();
    for (id, _) in state.index.entries() {
        // A document that will not weave right now contributes no structure
        // rather than failing the whole request: the reader asked about the
        // code, not about which document is mid-edit.
        let Ok(run) = state.weave(&id).await else {
            continue;
        };
        for (path, content) in &run.result.files {
            let Some(text) = content.as_text() else {
                continue;
            };
            if let Some(structure) = hick_structure::analyze(path, text) {
                files.push(structure);
            }
        }
    }
    let links = hick_structure::resolve(&files);
    Ok(Json(json!({
        "files": files,
        "links": links,
    })))
}

// ---------------------------------------------------------------------------
// The folder tree
// ---------------------------------------------------------------------------

/// One entry in the `GET /api/files` tree. Serialized shape:
/// `{"name", "path", "dir", "doc_id"?, "children"?}` — `doc_id` only on
/// `.hick` files, `children` only on directories.
#[derive(serde::Serialize)]
struct TreeNode {
    name: String,
    /// Root-relative, forward slashes on every platform.
    path: String,
    dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    doc_id: Option<String>,
    /// The document that generates this file, when one does.
    ///
    /// This is what stops the app offering to "make literate" a file that
    /// already is — a woven `cards.md` beside the `cards.hick` that writes it.
    /// It also lets such a file open as the generated thing it is, with its
    /// lineage and its refusal to be edited, rather than as a plain file that
    /// happens to be overwritten from time to time.
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_by: Option<String>,
    /// Why the disk does not hold what the document produces — the loop is
    /// leaving the file as it is. Absent for every file the document and the
    /// disk agree about. Axis 3 of docs/specs/freeform/three-axes.md.
    #[serde(skip_serializing_if = "Option::is_none")]
    diverged: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<TreeNode>>,
}

/// The `weave` attribute of a document's root element, and the `path` of a
/// `hick:file` block. Both are plain literal attributes in the source, which
/// is what makes reading them without a parse honest — see `declared_outputs`.
static WEAVE_ATTR: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"\bweave\s*=\s*"([^"]*)""#).expect("the weave-attribute pattern compiles")
});
static FILE_PATH_ATTR: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"<hick:file\b[^>]*?\bpath\s*=\s*"([^"]*)""#)
        .expect("the file-path-attribute pattern compiles")
});
/// A `<hick:volume output="DIR">` — a whole DIRECTORY a cell writes into.
///
/// This is the third way a document produces a file and the one that was
/// missing. A `hick:file` names one path; a volume names a directory whose
/// entire contents a program wrote, and nobody knows their names in advance.
/// Without this the file tree showed a generated API layer as ordinary
/// hand-editable source, while showing the hand-written domain beside it as
/// generated — exactly backwards for the question a reader is asking.
static VOLUME_OUTPUT_ATTR: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"<hick:volume\b[^>]*?\boutput\s*=\s*"([^"]*)""#)
        .expect("the volume-output-attribute pattern compiles")
});

/// A folder tree past this many entries answers what it has, flagged
/// `"truncated": true`, instead of walking (and shipping) a monster.
const FILE_TREE_CAP: usize = 10_000;

/// `GET /api/files` — the served root's file tree, for the folder pane.
///
/// Names only, never contents. Gitignore-aware, and skips what the search
/// index skips (hidden files, `.hick-cache`, `node_modules`), so the tree and
/// search agree on which files exist.
pub async fn files(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let root_name = state
        .index
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "hick".to_string());
    let root = state.index.root().to_path_buf();
    let root_path = root.canonicalize().unwrap_or_else(|_| root.clone());
    let index = state.index.clone();
    // Walking a working tree is filesystem work; keep it off the runtime.
    let (mut tree, truncated) = tokio::task::spawn_blocking(move || file_tree(&root, &index))
        .await
        .map_err(|e| ApiError::internal(format!("file listing task failed: {e}")))?;
    {
        let held = state.held.lock().expect("held outputs").clone();
        if !held.is_empty() {
            mark_held(&mut tree, &held);
        }
    }
    Ok(Json(json!({
        "root": root_name,
        // The absolute path, the separator that joins it to a node's path,
        // and what this desktop calls its file manager: everything the tree's
        // context menu needs to say "copy the absolute path" and "reveal in
        // Finder" without guessing which machine it is running on.
        "root_path": root_path.to_string_lossy(),
        "separator": std::path::MAIN_SEPARATOR_STR,
        "file_manager": super::reveal::file_manager_name(),
        "tree": tree,
        "truncated": truncated,
    })))
}

/// Walk the root and assemble the tree. Same walker configuration as
/// `hick-search`'s indexer: gitignore honoured even outside a git repository,
/// hidden files (which covers `.git`) skipped, `.hick-cache` and
/// `node_modules` never entered.
fn file_tree(root: &std::path::Path, index: &super::store::DocIndex) -> (Vec<TreeNode>, bool) {
    let mut top: Vec<TreeNode> = Vec::new();
    let mut documents: Vec<(String, std::path::PathBuf)> = Vec::new();
    let mut count = 0usize;
    let mut truncated = false;
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        // Honour .gitignore files even when the root is not (yet) a git
        // repository — the intent of the file is the same either way.
        .require_git(false)
        .git_global(false)
        .filter_entry(|e| e.file_name() != ".hick-cache" && e.file_name() != "node_modules")
        .build();
    for entry in walker.flatten() {
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        if rel.as_os_str().is_empty() {
            // The walker yields the root itself first; the tree starts below it.
            continue;
        }
        let file_type = entry.file_type();
        let dir = file_type.is_some_and(|t| t.is_dir());
        if !dir && !file_type.is_some_and(|t| t.is_file()) {
            continue; // broken symlinks and other non-files
        }
        if count >= FILE_TREE_CAP {
            truncated = true;
            break;
        }
        count += 1;
        // Forward slashes even on Windows: the path is a tree key and a
        // display string, not an OS path.
        let rel = rel.to_string_lossy().replace('\\', "/");
        if !dir && rel.ends_with(".hick") {
            documents.push((rel.clone(), entry.path().to_path_buf()));
        }
        insert_tree_node(&mut top, &rel, dir, index);
    }
    // Which files the documents in this folder write. Done after the walk so
    // a document is credited with an output that was listed before it.
    let generated = outputs_of(&documents, index);
    mark_generated(&mut top, &generated);
    sort_tree(&mut top);
    (top, truncated)
}

/// Output path → the id of the document that writes it, for a set of
/// documents already located.
fn outputs_of(
    documents: &[(String, std::path::PathBuf)],
    index: &super::store::DocIndex,
) -> HashMap<String, String> {
    let mut generated = HashMap::new();
    for (rel, absolute) in documents {
        let Ok(source) = std::fs::read_to_string(absolute) else {
            continue;
        };
        let id = index.add(rel);
        for output in declared_outputs(rel, &source) {
            // First document wins. Two documents writing one file is a
            // conflict the weaver reports; a caller asking "is this
            // generated" does not need to pick a side.
            generated.entry(output).or_insert_with(|| id.clone());
        }
    }
    generated
}

/// Every generated file under `root`, and which document writes it.
///
/// The folder tree gets this from its own walk; find-and-replace needs the
/// same answer without one, to refuse a write that the next weave would undo.
pub fn generated_outputs(
    root: &std::path::Path,
    index: &super::store::DocIndex,
) -> HashMap<String, String> {
    let mut documents = Vec::new();
    let walker = ignore::WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .require_git(false)
        .git_global(false)
        .filter_entry(|e| e.file_name() != ".hick-cache" && e.file_name() != "node_modules")
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        if rel.ends_with(".hick") {
            documents.push((rel, entry.path().to_path_buf()));
        }
    }
    outputs_of(&documents, index)
}

/// The files a document DECLARES it writes: its `weave` target and the path
/// of every `hick:file` block.
///
/// Read from the source with a regex rather than by weaving. Weaving every
/// document in the folder would be correct and far too slow for a listing
/// that refetches whenever the window regains focus — and what this is used
/// for (do not offer to adopt a file that is already generated; open it as
/// the generated thing it is) degrades safely if it is ever wrong.
///
/// Which means the honest boundary is: a path built from a variable
/// (`path="{{name}}.rs"`) is not recognised here. Such a file keeps behaving
/// the way every generated file did before this existed.
/// One declared path, joined onto the document's directory and normalised
/// into a tree key.
///
/// `None` for a path built from a variable: the tree cannot know what it will
/// be, and neither can anything reading the source instead of weaving it.
pub(crate) fn join_under(dir: &str, value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.contains("{{") {
        return None;
    }
    let joined = if dir.is_empty() {
        value.to_string()
    } else {
        format!("{dir}/{value}")
    };
    // Normalise `a/./b` and `a/b/../c` so the key matches a tree path.
    let mut parts: Vec<&str> = Vec::new();
    for part in joined.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    Some(parts.join("/"))
}

pub(crate) fn declared_outputs(doc_rel: &str, source: &str) -> Vec<String> {
    let dir = doc_rel.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let join = |value: &str| -> Option<String> { join_under(dir, value) };

    let mut out = Vec::new();
    let mut declared_weave = false;
    for capture in WEAVE_ATTR.captures_iter(source) {
        declared_weave = true;
        // `weave="none"` is the opt-out, not a file called `none`.
        if capture[1].trim() == hick_lang::WEAVE_NONE {
            continue;
        }
        if let Some(path) = join(&capture[1]) {
            out.push(path);
        }
    }
    // A BARE document declares no `weave=` and still weaves: the target
    // defaults to its own name (`bare-documents.md`). Reading only the
    // attribute meant every bare document's markdown was invisible here — not
    // marked generated in the file tree, and merged by git as if a person had
    // written it, which produced a second copy of a conflict already being
    // resolved in the document itself.
    if !declared_weave
        && let Some(stem) = doc_rel
            .rsplit('/')
            .next()
            .and_then(|f| f.strip_suffix(".hick"))
        && let Some(path) = join(&format!("{stem}.md"))
    {
        out.push(path);
    }
    for capture in FILE_PATH_ATTR.captures_iter(source) {
        if let Some(path) = join(&capture[1]) {
            out.push(path);
        }
    }
    // A volume's output is a directory, recorded with a trailing `/` so a
    // lookup can tell "this exact file" from "anything under here".
    //
    // Except once the document has INGESTED that volume. Ingest is the moment
    // the bytes stop being a flush and start being the document's own
    // `hick:file` blocks — which the pattern above already found, one path at
    // a time — and the volume is no longer flushed as a pipeline output at
    // all. Claiming the directory anyway would mark a whole tree as written
    // by a cell that no longer writes it.
    let ingested = source.contains("<hick:ingested");
    for capture in VOLUME_OUTPUT_ATTR.captures_iter(source) {
        if ingested {
            break;
        }
        if let Some(path) = join(&capture[1])
            && !path.is_empty()
        {
            out.push(format!("{path}/"));
        }
    }
    out
}

/// Which document generates `path`, if any.
///
/// An exact key is a `weave` target or a `hick:file`. A key ending in `/` is
/// a volume's output directory, and claims everything beneath it — so this
/// falls back to walking the path's ancestors. Longest prefix wins, so a
/// narrow volume nested inside a wider one is reported as the one that
/// actually wrote the file.
pub fn generated_by(generated: &HashMap<String, String>, path: &str) -> Option<String> {
    if let Some(doc) = generated.get(path) {
        return Some(doc.clone());
    }
    let mut best: Option<(usize, &String)> = None;
    for (key, doc) in generated {
        let Some(dir) = key.strip_suffix('/') else {
            continue;
        };
        if path.starts_with(dir)
            && path.as_bytes().get(dir.len()) == Some(&b'/')
            && best.is_none_or(|(len, _)| dir.len() > len)
        {
            best = Some((dir.len(), doc));
        }
    }
    best.map(|(_, doc)| doc.clone())
}

/// Stamp `diverged` onto every node whose disk bytes are not the document's.
fn mark_held(nodes: &mut [TreeNode], held: &HashMap<String, super::DivergedOutput>) {
    for node in nodes {
        if !node.dir
            && let Some(diverged) = held.get(&node.path)
        {
            node.diverged = Some(diverged.reason.clone());
        }
        if let Some(children) = node.children.as_mut() {
            mark_held(children, held);
        }
    }
}

/// `GET /api/outputs/diverged` — every produced file whose disk bytes are
/// not what its document produces: why, and the versions a merge needs. See
/// `docs/guarantees/authoring/an-output-that-cannot-be-carried-back-is-held.md`.
pub async fn diverged_outputs(State(state): State<LocalState>) -> Json<Value> {
    let held = state.held.lock().expect("held outputs").clone();
    Json(json!({ "diverged": held }))
}

#[derive(serde::Deserialize)]
pub struct ResolveBody {
    pub path: String,
    pub content: String,
}

/// `POST /api/outputs/resolve` — write the bytes a person chose (a merge's
/// result) over a diverged file. The loop then treats them as an ordinary
/// save: carried back into the document where they can be.
pub async fn resolve_output(
    State(state): State<LocalState>,
    Json(body): Json<ResolveBody>,
) -> ApiResult<Json<Value>> {
    if body.path.is_empty()
        || std::path::Path::new(&body.path).is_absolute()
        || body.path.split('/').any(|p| p == "..")
    {
        return Err(ApiError::bad_request(format!(
            "{:?} is not a path inside the open folder",
            body.path
        )));
    }
    let root = state
        .index
        .root()
        .canonicalize()
        .unwrap_or_else(|_| state.index.root().to_path_buf());
    let sender = state.up_commands.lock().expect("up command inbox").clone();
    let Some(sender) = sender else {
        return Err(ApiError::unavailable(
            "the folder is not being watched, so nothing can take the resolution",
        ));
    };
    sender
        .send(crate::up::UpCommand::Resolve(
            root.join(&body.path),
            body.content,
        ))
        .map_err(|_| ApiError::unavailable("the watch loop has stopped"))?;
    Ok(Json(json!({ "ok": true, "path": body.path })))
}

#[derive(serde::Deserialize)]
pub struct RegenerateBody {
    pub path: String,
}

/// `POST /api/outputs/regenerate` — overwrite a held file with what its
/// document produces. The one write over held bytes, taken only when asked.
pub async fn regenerate_output(
    State(state): State<LocalState>,
    Json(body): Json<RegenerateBody>,
) -> ApiResult<Json<Value>> {
    if body.path.is_empty()
        || std::path::Path::new(&body.path).is_absolute()
        || body.path.split('/').any(|p| p == "..")
    {
        return Err(ApiError::bad_request(format!(
            "{:?} is not a path inside the open folder",
            body.path
        )));
    }
    let held = state
        .held
        .lock()
        .expect("held outputs")
        .contains_key(&body.path);
    if !held {
        return Err(ApiError::unprocessable(format!(
            "{} is not held: the document and the disk already agree, so there is nothing to \
             regenerate",
            body.path
        )));
    }
    let root = state
        .index
        .root()
        .canonicalize()
        .unwrap_or_else(|_| state.index.root().to_path_buf());
    let sender = state.up_commands.lock().expect("up command inbox").clone();
    let Some(sender) = sender else {
        return Err(ApiError::unavailable(
            "the folder is not being watched, so nothing can regenerate the file; run \
             `hick weave` on the document instead",
        ));
    };
    sender
        .send(crate::up::UpCommand::Regenerate(root.join(&body.path)))
        .map_err(|_| ApiError::unavailable("the watch loop has stopped"))?;
    Ok(Json(json!({ "ok": true, "path": body.path })))
}

/// Stamp `generated_by` onto every node whose path a document writes.
fn mark_generated(nodes: &mut [TreeNode], generated: &HashMap<String, String>) {
    for node in nodes {
        if !node.dir && node.doc_id.is_none() {
            node.generated_by = generated_by(generated, &node.path);
        }
        if let Some(children) = node.children.as_mut() {
            mark_generated(children, generated);
        }
    }
}

/// Place one walked entry. The walker yields a directory before its contents,
/// so ancestors already exist; they are still created on demand so a missed
/// parent can never panic the listing.
fn insert_tree_node(top: &mut Vec<TreeNode>, rel: &str, dir: bool, index: &super::store::DocIndex) {
    let mut siblings = top;
    let mut parts = rel.split('/').peekable();
    let mut prefix = String::new();
    while let Some(part) = parts.next() {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(part);
        if parts.peek().is_none() {
            // `index.add` rather than `id_for_path`: the startup scan only saw
            // documents that existed then, and a `.hick` file it missed must
            // still be openable the moment the tree shows it.
            let doc_id = (!dir && rel.ends_with(".hick")).then(|| index.add(rel));
            siblings.push(TreeNode {
                name: part.to_string(),
                path: prefix.clone(),
                dir,
                doc_id,
                generated_by: None,
                diverged: None,
                children: dir.then(Vec::new),
            });
            return;
        }
        let pos = siblings
            .iter()
            .position(|n| n.dir && n.name == part)
            .unwrap_or_else(|| {
                siblings.push(TreeNode {
                    name: part.to_string(),
                    path: prefix.clone(),
                    dir: true,
                    doc_id: None,
                    generated_by: None,
                    diverged: None,
                    children: Some(Vec::new()),
                });
                siblings.len() - 1
            });
        siblings = siblings[pos]
            .children
            .as_mut()
            .expect("directory nodes always carry children");
    }
}

/// Directories first, then files, both case-insensitive alphabetical — the
/// order every file pane a user has ever seen puts them in.
fn sort_tree(nodes: &mut [TreeNode]) {
    nodes.sort_by(|a, b| {
        b.dir
            .cmp(&a.dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    for node in nodes {
        if let Some(children) = &mut node.children {
            sort_tree(children);
        }
    }
}

#[derive(Deserialize)]
pub struct FileQuery {
    pub path: String,
}

/// `GET /api/docs/:id/outputs/file?path=…` — content plus byte-precise
/// provenance. This is what the ribbon layer draws.
pub async fn get_output_file(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Query(q): Query<FileQuery>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let content = run
        .result
        .files
        .get(&q.path)
        .and_then(|c| c.as_text())
        .ok_or_else(|| {
            let mut available: Vec<&str> = run.result.files.keys().map(String::as_str).collect();
            available.sort();
            ApiError::not_found(format!(
                "this document produces no output named {:?} — it produces: {}",
                q.path,
                if available.is_empty() {
                    "(nothing)".to_string()
                } else {
                    available.join(", ")
                }
            ))
        })?
        .to_string();
    let provenance = crate::output_lineage(&run, &q.path)?;
    Ok(Json(json!({
        "path": q.path,
        "language": language_of(&q.path),
        "content": content,
        "provenance": provenance,
    })))
}

/// `GET /api/docs/:id/context` — context provenance: every run of lines an
/// agent wrote in this document, with what was in front of the model when it
/// wrote them, derived from the session files near the document. A second
/// provenance beside the weave's lineage; the app draws it as its own family
/// of ribbons and never the same way (`docs/specs/freeform/three-provenances.md`).
pub async fn get_context(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let path = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found(format!("no document with id {id}")))?;
    let source = std::fs::read_to_string(&path)
        .map_err(|e| ApiError::not_found(format!("cannot read {}: {e}", path.display())))?;
    let root = state.index.root().to_path_buf();
    let writes = tokio::task::spawn_blocking(move || {
        hickory_agent::context::context_for_document(&path, &source)
    })
    .await
    .map_err(|e| ApiError::internal(format!("context derivation failed: {e}")))?;
    // Session paths relative to the project root, which is how the app
    // names files — the tree, the tabs, the ports.
    let writes: Vec<Value> = writes
        .into_iter()
        .map(|w| {
            let mut v = serde_json::to_value(&w).unwrap_or(Value::Null);
            if let Some(obj) = v.as_object_mut()
                && let Some(Value::String(s)) = obj.get("session")
            {
                let rel = std::path::Path::new(s)
                    .strip_prefix(&root)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| s.clone());
                obj.insert("session".into(), Value::String(rel));
            }
            v
        })
        .collect();
    Ok(Json(json!({ "writes": writes })))
}

/// `GET /api/docs/:id/cites` — declared provenance: every `cites=` in the
/// document and what it resolves to, paths root-relative. The author's
/// assertion, drawn by the app as one.
pub async fn get_cites(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    let path = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found(format!("no document with id {id}")))?;
    let source = std::fs::read_to_string(&path)
        .map_err(|e| ApiError::not_found(format!("cannot read {}: {e}", path.display())))?;
    let root = state.index.root().to_path_buf();
    let rel = |p: &str| -> String {
        let pb = std::path::Path::new(p);
        let canon = pb.canonicalize().unwrap_or_else(|_| pb.to_path_buf());
        canon
            .strip_prefix(&root)
            .map(|q| q.display().to_string())
            .unwrap_or_else(|_| p.to_string())
    };
    let cites = crate::declared_cites(&path, &source)
        .map_err(|e| ApiError::internal(format!("resolving cites: {e:#}")))?
        .into_iter()
        .map(|mut c| {
            c.from.path = rel(&c.from.path);
            for t in &mut c.to {
                t.path = rel(&t.path);
            }
            c
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "cites": cites })))
}

#[derive(Deserialize)]
pub struct SessionQuery {
    pub path: String,
}

/// `GET /api/sessions/view?path=…` — a session file read back as the
/// conversation it records: turns (with their parent turn, for the tree),
/// and within each turn the steps — reasoning, prose, scripts and their
/// output, tool calls and results, files shown, lines written. The same
/// shape the chat dock renders, so a session opened as a document looks like
/// the chat it was.
pub async fn session_view(
    State(state): State<LocalState>,
    Query(q): Query<SessionQuery>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let rel = std::path::Path::new(&q.path);
    let abs = if rel.is_absolute() {
        rel.to_path_buf()
    } else {
        root.join(rel)
    };
    let canon = abs
        .canonicalize()
        .map_err(|e| ApiError::not_found(format!("no session at {}: {e}", q.path)))?;
    let root_canon = root.canonicalize().unwrap_or(root.clone());
    if !canon.starts_with(&root_canon) {
        return Err(ApiError::not_found(format!(
            "{} is outside this folder",
            q.path
        )));
    }
    let source = std::fs::read_to_string(&canon)
        .map_err(|e| ApiError::not_found(format!("cannot read {}: {e}", q.path)))?;
    if !hick_lang::is_session_source(&source) {
        return Err(ApiError::bad_request(format!(
            "{} is not a hick:session document",
            q.path
        )));
    }
    let view = hickory_agent::session_view::session_view(&source);
    let rel_path = canon
        .strip_prefix(&root_canon)
        .map(|p| p.display().to_string())
        .unwrap_or(q.path.clone());
    Ok(Json(json!({ "path": rel_path, "view": view })))
}

#[derive(Deserialize)]
pub struct EditRequest {
    pub path: String,
    pub edits: Vec<hickory_lineage::OutputEdit>,
}

/// `POST /api/docs/:id/outputs/edit` — edit the generated file; the change is
/// mapped back into the documents that produced it.
///
/// Simpler than the hosted counterpart in one way that matters: the weave was
/// computed from the files in this same request, so the "doc changed since the
/// run this output came from" conflict cannot arise. There is no stale
/// provenance to guard against, because there is no stored run.
pub async fn edit_outputs(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<EditRequest>,
) -> ApiResult<Json<Value>> {
    let run = state.weave(&id).await?;
    let content = run
        .result
        .files
        .get(&body.path)
        .and_then(|c| c.as_text())
        .ok_or_else(|| ApiError::not_found(format!("no output named {:?}", body.path)))?
        .to_string();
    let provenance = crate::output_lineage(&run, &body.path)?;

    let source_edits =
        hickory_lineage::map_edits(&content, &body.edits, &provenance).map_err(|e| match e {
            hickory_lineage::LineageError::SyntheticOverlap { start, end } => {
                ApiError::unprocessable(format!(
                    "edit overlaps a synthetic (non-editable) output range at bytes {start}..{end}"
                ))
                .with_detail(json!({ "range": { "start": start, "end": end } }))
            }
            hickory_lineage::LineageError::InvalidEdit(m) => ApiError::bad_request(m),
            hickory_lineage::LineageError::Conflict(m) => ApiError::unprocessable(m),
        })?;

    // Provenance names documents by the path the weave knew them as; read each
    // one from disk.
    let mut sources: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    for edit in &source_edits {
        if sources.contains_key(&edit.doc_path) {
            continue;
        }
        let text = state.read_source_by_doc_path(&edit.doc_path)?;
        sources.insert(edit.doc_path.clone(), text);
    }

    let updated = hickory_lineage::apply_source_edits(&sources, &source_edits)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;

    // Reject an edit that breaks the document: the next run must reproduce it.
    for (doc_path, new_source) in &updated {
        hick_lang::parse(new_source).map_err(|e| {
            ApiError::unprocessable(format!("edit would make {doc_path} unparseable: {e}"))
        })?;
    }

    for (doc_path, new_source) in &updated {
        let target_id = state.id_for_doc_path(doc_path);
        state.write_source_by_doc_path(doc_path, new_source)?;
        state
            .rooms
            .apply_external_source(&target_id, new_source)
            .await;
    }

    Ok(Json(
        json!({ "source_edits": source_edits, "applied": true }),
    ))
}

/// Language tag for an output path, matching the hosted server's mapping.
/// `pub(super)` because plain files (serve/plain_file.rs) tag with the same
/// mapping — a `.py` file must highlight the same whether woven or plain.
pub(super) fn language_of(path: &str) -> String {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    match ext {
        "rs" => "rust",
        "py" => "python",
        "ts" => "typescript",
        "tsx" => "typescript",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "javascript",
        "go" => "go",
        "sh" | "bash" => "shell",
        "json" => "json",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "md" | "markdown" => "markdown",
        "html" => "html",
        "css" => "css",
        "sql" => "sql",
        other => other,
    }
    .to_string()
}

// ---------------------------------------------------------------------------
// Running
// ---------------------------------------------------------------------------

#[derive(Deserialize, Default)]
pub struct RunRequest {
    #[serde(default)]
    pub cells: Option<Vec<String>>,
}

/// `POST /api/docs/:id/run` — execute for real, streaming events on the run
/// channel.
pub async fn run_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let run_id = state.start_run(&id, false).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

/// `POST /api/docs/:id/check` — verify without writing outputs.
pub async fn check_doc(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let run_id = state.start_run(&id, true).await?;
    Ok((StatusCode::ACCEPTED, Json(json!({ "run_id": run_id }))))
}

/// `GET /api/runs/:id`
pub async fn get_run(
    State(state): State<LocalState>,
    Path(run_id): Path<String>,
) -> ApiResult<Json<Value>> {
    let record: RunRecord = state
        .runs
        .lock()
        .unwrap()
        .get(&run_id)
        .cloned()
        .ok_or_else(|| ApiError::not_found("no such run in this session"))?;
    Ok(Json(json!({
        "id": run_id,
        "status": record.status,
        "started_at": record.started_at,
        "blocks": record.blocks,
    })))
}

// ---------------------------------------------------------------------------
// Static answers
// ---------------------------------------------------------------------------

/// `GET /api/executor` — which backend this session runs on. The web app shows
/// it, and in a shared session it is the difference between "sandboxed" and
/// "as the host".
pub async fn executor(State(state): State<LocalState>) -> Json<Value> {
    Json(json!({ "kind": state.executor_kind(), "images": Value::Null }))
}

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    #[serde(default = "default_search_k")]
    k: usize,
}

fn default_search_k() -> usize {
    12
}

/// `GET /api/search?q=…&k=…` — project-wide search over the served folder.
///
/// Lexical (BM25) always; semantic as well when the project has an embedding
/// model installed (`hick search --install-model`). The response says which,
/// so the client can offer the upgrade instead of silently ranking worse.
pub async fn search(
    State(state): State<LocalState>,
    Query(params): Query<SearchParams>,
) -> ApiResult<Json<Value>> {
    let query = params.q.trim().to_string();
    if query.is_empty() {
        return Err(ApiError::bad_request(
            "the search query q= must not be empty",
        ));
    }
    let k = params.k.clamp(1, 50);
    let root = state.index.root().to_path_buf();
    let shared = state.search.clone();
    // Index refresh and embedding are CPU work; keep them off the runtime.
    let (semantic, hits) = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut slot = shared.lock().expect("search mutex poisoned");
        // Build once; after that only re-walk for changed files. If a model
        // was installed since the engine was built, rebuild to pick it up.
        let rebuild = match slot.as_ref() {
            None => true,
            Some(engine) => !engine.semantic() && hick_search::model_available(&root),
        };
        if rebuild {
            *slot = Some(hick_search::SearchEngine::open(&root)?);
        } else if let Some(engine) = slot.as_mut() {
            engine.refresh()?;
        }
        let engine = slot.as_ref().expect("just built");
        let hits = engine.search(&query, k);
        Ok((engine.semantic(), hits))
    })
    .await
    .map_err(|e| ApiError::internal(format!("search task failed: {e}")))?
    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;

    Ok(Json(json!({ "semantic": semantic, "hits": hits })))
}

/// `GET /api/health`
pub async fn health(State(state): State<LocalState>) -> Json<Value> {
    Json(json!({ "ok": true, "executor": state.executor_kind(), "db": false }))
}

// ---------------------------------------------------------------------------
// Settings: LLM provider keys
// ---------------------------------------------------------------------------

/// The listing both key routes answer with. Key material never crosses the
/// wire: `configured` and a masked fragment (first 4 + last 2 characters at
/// most, nothing for short keys) are all the Settings page needs to render
/// "a key is installed" — the user already has the value; they got it from
/// the vendor.
///
/// `configured` counts the environment too, store first — the same
/// precedence the agent route resolves with — so the page tells the truth
/// about whether the agent would run, not just about this one file.
fn keys_listing(store: &hickory_agent::KeyStore) -> Value {
    let providers: Vec<Value> = hickory_agent::ProviderSelection::all()
        .into_iter()
        .map(|sel| {
            let key = store.key_for(sel.name()).or_else(|| {
                std::env::var(sel.key_env())
                    .ok()
                    .filter(|k| !k.trim().is_empty())
            });
            json!({
                "id": sel.name(),
                "label": sel.label(),
                "configured": key.is_some(),
                "masked": key.as_deref().map(hickory_agent::masked_key),
            })
        })
        .collect();
    json!({ "providers": providers })
}

/// `GET /api/settings/keys` — every provider, whether a key is configured,
/// and a masked fragment. Never the key itself.
pub async fn get_settings_keys(State(state): State<LocalState>) -> Json<Value> {
    let store = state.keys.store.read().expect("key store lock poisoned");
    Json(keys_listing(&store))
}

/// `PUT /api/settings/keys` — set or clear keys for the named providers
/// only: `{"anthropic": "sk-new", "openai": null}`. Everything is validated
/// before anything is applied, the file is persisted (0600 on Unix), and the
/// in-memory store is swapped so the next agent turn uses the new key with
/// no restart. Answers the same listing `GET` does.
pub async fn put_settings_keys(
    State(state): State<LocalState>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let Value::Object(entries) = body else {
        return Err(ApiError::bad_request(
            "the body must be a JSON object mapping provider ids to a key string \
             (to set) or null (to clear), e.g. {\"anthropic\": \"sk-…\"}",
        ));
    };

    // Stage on a copy so a bad entry — or a failed write — changes nothing:
    // the live store never holds half of a rejected request.
    let mut staged = state
        .keys
        .store
        .read()
        .expect("key store lock poisoned")
        .clone();
    for (id, value) in &entries {
        let key = match value {
            Value::Null => None,
            Value::String(s) => Some(s.clone()),
            // Deliberately vague about the value: it may be a mistyped key,
            // and an error body is a thing that gets pasted into bug reports.
            _ => {
                return Err(ApiError::bad_request(format!(
                    "the value for {id:?} must be a key string, or null to clear it"
                )));
            }
        };
        staged
            .set(id, key)
            .map_err(|e| ApiError::bad_request(format!("{e:#}")))?;
    }

    if let Some(path) = &state.keys.path {
        // The error carries the path and the OS failure, never key material.
        staged.save(path).map_err(|e| {
            ApiError::internal(format!(
                "could not save the key file: {e:#}. Check that the directory is \
                 writable and the disk is not full; the keys were not changed."
            ))
        })?;
    }

    let listing = keys_listing(&staged);
    *state.keys.store.write().expect("key store lock poisoned") = staged;
    Ok(Json(listing))
}

/// What GET and PUT `/api/settings/ui` both answer.
fn ui_listing(store: &crate::serve::UiStore) -> Value {
    json!({
        "window_title": store.window_title,
        "format_on_save": store.format_on_save,
    })
}

/// `GET /api/settings/ui` — the UI settings: the custom window title, or
/// null for the default. Mirrors `/api/settings/keys`.
pub async fn get_settings_ui(State(state): State<LocalState>) -> Json<Value> {
    let store = state.ui.store.read().expect("ui settings lock poisoned");
    Json(ui_listing(&store))
}

/// `PUT /api/settings/ui` — set or clear the custom window title:
/// `{"window_title": "My Notes"}` or `{"window_title": null}`. Everything is
/// validated before anything is applied, the file is persisted (`ui.json`
/// beside `llm-keys.json`), and the in-memory store is swapped so the page's
/// next read sees the new value with no restart. Answers the same listing
/// `GET` does. (The NATIVE window title is read from the file at the next
/// desktop launch; live native updates are out of scope — see
/// apps/desktop/src-tauri/src/lib.rs.)
pub async fn put_settings_ui(
    State(state): State<LocalState>,
    Json(body): Json<Value>,
) -> ApiResult<Json<Value>> {
    let Value::Object(entries) = body else {
        return Err(ApiError::bad_request(
            "the body must be a JSON object, e.g. {\"window_title\": \"My Notes\"} \
             to set a custom window title or {\"window_title\": null} to clear it",
        ));
    };

    // Stage on a copy so a bad entry — or a failed write — changes nothing.
    let mut staged = state
        .ui
        .store
        .read()
        .expect("ui settings lock poisoned")
        .clone();
    for (field, value) in &entries {
        match (field.as_str(), value) {
            ("window_title", Value::Null) => staged.window_title = None,
            ("window_title", Value::String(s)) => {
                let trimmed = s.trim();
                staged.window_title = (!trimmed.is_empty()).then(|| trimmed.to_string());
            }
            ("window_title", _) => {
                return Err(ApiError::bad_request(
                    "window_title must be a string, or null to clear it",
                ));
            }
            ("format_on_save", Value::Bool(on)) => staged.format_on_save = *on,
            ("format_on_save", _) => {
                return Err(ApiError::bad_request(
                    "format_on_save must be true or false",
                ));
            }
            (other, _) => {
                return Err(ApiError::bad_request(format!(
                    "unknown UI setting {other:?}; the settings are \"window_title\" \
                     and \"format_on_save\""
                )));
            }
        }
    }

    if let Some(path) = &state.ui.path {
        staged.save(path).map_err(|e| {
            ApiError::internal(format!(
                "could not save the UI settings file: {e:#}. Check that the \
                 directory is writable and the disk is not full; the settings \
                 were not changed."
            ))
        })?;
    }

    let listing = ui_listing(&staged);
    *state.ui.store.write().expect("ui settings lock poisoned") = staged;
    Ok(Json(listing))
}

/// Arc-friendly alias used by the router module.
pub type Shared = Arc<LocalState>;

#[cfg(test)]
mod tree_tests {
    use super::*;

    #[test]
    fn a_documents_weave_target_is_one_of_its_outputs() {
        // The case that started this: `cards.md` beside the `cards.hick` that
        // writes it, offered a button to "make it literate" when it already
        // is.
        let outputs = declared_outputs(
            "cards.hick",
            r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="cards.md">"#,
        );
        assert_eq!(outputs, vec!["cards.md".to_string()]);
    }

    #[test]
    fn every_generated_file_block_is_an_output() {
        let outputs = declared_outputs(
            "app.hick",
            r#"
            <hick:file path="src/main.rs">fn main() {}</hick:file>
            <hick:file path="Cargo.toml">[package]</hick:file>
            "#,
        );
        assert_eq!(
            outputs,
            vec![
                // The default weave: this document declares no `weave=`, so
                // it writes `app.md` beside itself.
                "app.md".to_string(),
                "src/main.rs".to_string(),
                "Cargo.toml".to_string()
            ]
        );
    }

    #[test]
    fn outputs_are_relative_to_the_documents_own_directory() {
        // A tree key is root-relative, so a document three folders down that
        // says `path="main.rs"` must not claim the root's `main.rs`.
        let outputs = declared_outputs(
            "notes/deep/app.hick",
            r#"<hick:doc weave="README.md"><hick:file path="src/main.rs"/>"#,
        );
        assert_eq!(
            outputs,
            vec![
                "notes/deep/README.md".to_string(),
                "notes/deep/src/main.rs".to_string(),
            ]
        );
    }

    #[test]
    fn a_dot_dot_in_a_path_is_resolved_rather_than_left_in_the_key() {
        // `notes/deep/../out.rs` would never match the tree's `notes/out.rs`.
        let outputs = declared_outputs("notes/deep/app.hick", r#"<hick:file path="../out.rs"/>"#);
        assert_eq!(
            outputs,
            vec!["notes/deep/app.md".to_string(), "notes/out.rs".to_string()]
        );
    }

    #[test]
    fn a_path_built_from_a_variable_is_not_claimed() {
        // The honest boundary of reading declarations instead of weaving: an
        // interpolated path is left alone rather than recorded as the literal
        // `{{name}}.rs`, which would mark a file nobody has.
        // The default weave is still claimed; only the interpolated path is
        // left alone.
        assert_eq!(
            declared_outputs("app.hick", r#"<hick:file path="{{name}}.rs"/>"#),
            vec!["app.md".to_string()]
        );
    }

    /// A volume's `output=` directory claims everything under it.
    ///
    /// This is how a document says "a program wrote these and I do not know
    /// their names". Before it was read, the app showed a generated API
    /// layer as ordinary source you could edit — and the next run replaced
    /// it — while marking the hand-written domain beside it as generated.
    #[test]
    fn a_volume_output_directory_claims_the_files_under_it() {
        let declared = declared_outputs(
            "30-api.hick",
            r#"<hick:volume name="apiout" output="src/Api/Generated" />"#,
        );
        assert_eq!(
            declared,
            vec!["30-api.md".to_string(), "src/Api/Generated/".to_string()]
        );

        let generated = HashMap::from([
            ("src/Api/Generated/".to_string(), "d1".to_string()),
            ("src/Api/".to_string(), "d0".to_string()),
        ]);
        // Under the directory: generated, and by the NARROWEST volume that
        // claims it.
        assert_eq!(
            generated_by(&generated, "src/Api/Generated/Endpoints.g.cs").as_deref(),
            Some("d1")
        );
        assert_eq!(
            generated_by(&generated, "src/Api/Other.cs").as_deref(),
            Some("d0")
        );
        // A prefix match is on a path BOUNDARY, never on spelling.
        // `src/ApiOther/` is not under `src/Api/`, and
        // `src/Api/GeneratedThing.cs` is under `src/Api/` but NOT under
        // `src/Api/Generated/` — so the wider volume claims it and the
        // narrower one does not.
        assert_eq!(generated_by(&generated, "src/ApiOther/x.cs"), None);
        assert_eq!(
            generated_by(&generated, "src/Api/GeneratedThing.cs").as_deref(),
            Some("d0")
        );
    }

    /// Only `weave="none"` generates nothing.
    ///
    /// A document of pure prose still writes its markdown — that is the
    /// default `bare-documents.md` adopted, and "a note that has no readable
    /// form is not a note". The opt-out is the one case with no outputs at
    /// all, and it is a keyword rather than a path: before this it claimed a
    /// file literally named `none`.
    #[test]
    fn only_weave_none_generates_nothing() {
        assert_eq!(
            declared_outputs("notes.hick", "# Just prose\n"),
            vec!["notes.md".to_string()]
        );
        assert!(
            declared_outputs(
                "gen.hick",
                r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">"#
            )
            .is_empty()
        );
    }

    #[test]
    fn marking_skips_documents_and_directories() {
        // A `.hick` file is a document, never somebody else's output, and a
        // directory is not a file at all.
        let generated = HashMap::from([
            ("cards.md".to_string(), "d1".to_string()),
            ("src".to_string(), "d1".to_string()),
            ("other.hick".to_string(), "d1".to_string()),
        ]);
        let mut tree = vec![
            TreeNode {
                name: "cards.md".into(),
                path: "cards.md".into(),
                dir: false,
                doc_id: None,
                generated_by: None,
                diverged: None,
                children: None,
            },
            TreeNode {
                name: "other.hick".into(),
                path: "other.hick".into(),
                dir: false,
                doc_id: Some("d2".into()),
                generated_by: None,
                diverged: None,
                children: None,
            },
            TreeNode {
                name: "src".into(),
                path: "src".into(),
                dir: true,
                doc_id: None,
                generated_by: None,
                diverged: None,
                children: Some(vec![TreeNode {
                    name: "main.rs".into(),
                    path: "src/main.rs".into(),
                    dir: false,
                    doc_id: None,
                    generated_by: None,
                    diverged: None,
                    children: None,
                }]),
            },
        ];
        mark_generated(&mut tree, &generated);
        assert_eq!(tree[0].generated_by.as_deref(), Some("d1"));
        assert_eq!(tree[1].generated_by, None, "a document is not an output");
        assert_eq!(tree[2].generated_by, None, "a directory is not a file");
        let children = tree[2].children.as_ref().unwrap();
        assert_eq!(children[0].generated_by, None, "nothing claims src/main.rs");
    }
}

// ---------------------------------------------------------------------------
// Blame
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct BlameParams {
    pub path: String,
}

/// `GET /api/blame?path=…` — who last touched each line.
///
/// One `git blame` for the whole file, never one per line: a thousand-line
/// file would otherwise fork a thousand processes to fill a column that is
/// off by default.
///
/// A folder that is not a repository, a file that is untracked, a machine
/// with no git — all answer `{"lines": []}` rather than an error. The column
/// is an optional annotation; refusing to open a file because its history is
/// unavailable would be absurd.
pub async fn blame(
    State(state): State<LocalState>,
    Query(params): Query<BlameParams>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    // Bounds-checked the same way every other path parameter is: relative, no
    // `..`, inside the folder.
    if params.path.is_empty()
        || params.path.starts_with('/')
        || params.path.split('/').any(|part| part == "..")
    {
        return Err(ApiError::bad_request(format!(
            "{} is not a path inside this folder",
            params.path
        )));
    }
    let rel = std::path::PathBuf::from(&params.path);
    let lines = tokio::task::spawn_blocking(move || crate::agent_lineage::blame_file(&root, &rel))
        .await
        .map_err(|e| ApiError::internal(format!("the blame task failed: {e}")))?;

    Ok(Json(json!({ "path": params.path, "lines": lines })))
}

// ---------------------------------------------------------------------------
// Completions drawn from the project itself
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct CompleteParams {
    /// What has been typed so far.
    pub prefix: String,
    /// The lines around the caret, for the semantic ranking. Optional: with
    /// no model installed it is not read at all.
    #[serde(default)]
    pub context: String,
    #[serde(default = "default_complete_k")]
    pub k: usize,
}

fn default_complete_k() -> usize {
    8
}

/// `GET /api/complete?prefix=…&context=…` — what this project calls things.
///
/// Deliberately NOT a language server's answer and never presented as one.
/// An LSP knows what is in scope and what its type is; it has no opinion
/// about whether this codebase says `cfg`, `config` or `settings`. This does,
/// and knows nothing about types. The two are shown together and labelled,
/// because neither subsumes the other.
pub async fn complete(
    State(state): State<LocalState>,
    Query(params): Query<CompleteParams>,
) -> ApiResult<Json<Value>> {
    let root = state.index.root().to_path_buf();
    let shared = state.search.clone();
    let k = params.k.clamp(1, 25);
    let suggestions = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
        let mut slot = shared.lock().expect("search mutex poisoned");
        let rebuild = match slot.as_ref() {
            None => true,
            Some(engine) => !engine.semantic() && hick_search::model_available(&root),
        };
        if rebuild {
            *slot = Some(hick_search::SearchEngine::open(&root)?);
        } else if let Some(engine) = slot.as_mut() {
            engine.refresh()?;
        }
        let engine = slot.as_ref().expect("just built");
        Ok(engine.completions(&params.prefix, &params.context, k))
    })
    .await
    .map_err(|e| ApiError::internal(format!("the completion task failed: {e}")))?
    .map_err(|e| ApiError::internal(format!("{e:#}")))?;

    Ok(Json(json!({ "suggestions": suggestions })))
}
