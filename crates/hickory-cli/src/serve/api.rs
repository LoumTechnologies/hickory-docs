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
pub struct ApiError(StatusCode, String, Option<Value>);

impl ApiError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self(StatusCode::NOT_FOUND, msg.into(), None)
    }
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, msg.into(), None)
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
    let rel = body.path.trim().trim_start_matches(['/', '\\']).to_string();
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
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<TreeNode>>,
}

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
    let index = state.index.clone();
    // Walking a working tree is filesystem work; keep it off the runtime.
    let (tree, truncated) = tokio::task::spawn_blocking(move || file_tree(&root, &index))
        .await
        .map_err(|e| ApiError::internal(format!("file listing task failed: {e}")))?;
    Ok(Json(json!({
        "root": root_name,
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
        insert_tree_node(&mut top, &rel, dir, index);
    }
    sort_tree(&mut top);
    (top, truncated)
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
fn language_of(path: &str) -> String {
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
    json!({ "window_title": store.window_title })
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
            (other, _) => {
                return Err(ApiError::bad_request(format!(
                    "unknown UI setting {other:?}; the only setting is \"window_title\""
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
