//! A document's editable representation is independent of its durable backing.
//! Source-backed lenses never enter DocIndex and never publish during a read.
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::Mutex;

use super::workspace_fs::view::{confined, hash, writable};
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};

pub type Representations = Arc<Mutex<HashMap<String, Representation>>>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Backing {
    Files { paths: Vec<String> },
    Document { doc_id: String },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct File {
    pub path: String,
    pub content: String,
    #[serde(default)]
    pub source_path: String,
    pub hash: String,
    pub provenance: Vec<hickory_lineage::Provenance>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Representation {
    pub id: String,
    pub backing: Backing,
    pub source: String,
    pub revision: String,
    pub files: Vec<File>,
    pub explanation_stale: bool,
    #[serde(default)]
    pub local_warning: Option<String>,
}

#[derive(Deserialize)]
pub struct Create {
    pub backing: Backing,
}
#[derive(Deserialize)]
pub struct Edit {
    pub revision: String,
    pub source: String,
}

fn failure(e: impl std::fmt::Display) -> ApiError {
    ApiError::unprocessable(format!(
        "{e}. Review the literate view and try again; nothing was saved."
    ))
}

fn read(root: &std::path::Path, path: &str) -> ApiResult<String> {
    let path = confined(root, path).map_err(failure)?;
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(failure(e)),
    };
    if bytes.len() > 10 * 1024 * 1024 {
        return Err(failure("file exceeds the 10 MiB editor limit"));
    }
    String::from_utf8(bytes).map_err(|_| failure("this view requires UTF-8 text"))
}

fn initial(paths: &[String], contents: &BTreeMap<String, String>) -> ApiResult<String> {
    let mut source = "# Literate view\n\nCode is backed by the original files. Explanations and arrangement belong to this view.\n\n".to_string();
    for path in paths {
        // Attribute quoting is not a new escaping grammar.
        if path.contains(['"', '<', '>', '\n', '\r']) {
            return Err(failure(
                "this filename cannot be represented by a file element",
            ));
        }
        source.push_str(&format!(
            "## {path}\n\n<hick:file path=\"{path}\">\n{}</hick:file>\n\n",
            contents[path]
        ));
    }
    Ok(source)
}

pub(super) async fn compile(
    state: &LocalState,
    id: &str,
    source: &str,
    backing: &Backing,
) -> ApiResult<Vec<File>> {
    let doc = hick_lang::parse(source).map_err(failure)?;
    if matches!(backing, Backing::Files { .. })
        && doc
            .all_tags()
            .iter()
            .any(|tag| !["doc", "file", "copy", "cut", "paste"].contains(&tag.name.as_str()))
    {
        return Err(failure(
            "source-backed views support prose and literal file/copy/cut/paste elements; execution belongs in the repository's tools",
        ));
    }
    let name = match backing {
        Backing::Document { doc_id } => state
            .index
            .absolute(doc_id)
            .ok_or_else(|| failure("document is no longer open"))?,
        _ => state.index.root().join(format!("__lens-{id}.md")),
    };
    let cache = hick_literate::cache::CacheConfig::new(
        name.parent().unwrap(),
        hick_literate::cache::CacheMode::Reuse,
    );
    let result = hick_literate::run_pipeline_weave(
        &[(&name.display().to_string(), source)],
        &state.params,
        Some(&cache),
    )
    .await
    .map_err(failure)?;
    let mut files = Vec::new();
    for (path, content) in result.files {
        let relative = match backing {
            Backing::Document { .. } => name
                .parent()
                .unwrap()
                .join(&path)
                .strip_prefix(state.index.root())
                .map_err(failure)?
                .to_string_lossy()
                .replace('\\', "/"),
            _ => path.clone(),
        };
        confined(state.index.root(), &relative).map_err(failure)?;
        if let Some(text) = content.as_text() {
            let mut provenance = result
                .provenance_maps
                .get(&path)
                .map(hickory_lineage::from_provenance_map)
                .unwrap_or_default();
            if text.is_empty()
                && provenance.is_empty()
                && let Some(tag) = doc.all_tags().into_iter().find(|t| {
                    t.name == "file"
                        && t.get_attribute("path") == Some(path.as_str())
                        && t.children
                            .iter()
                            .all(|n| matches!(n, hick_lang::HickNode::Text(..)))
                })
                && let Some(close) = tag.close_span
            {
                provenance.push(hickory_lineage::Provenance {
                    start: 0,
                    end: 0,
                    origin: hickory_lineage::Origin::Literal {
                        doc_path: name.display().to_string(),
                        span: (close.start, close.start),
                    },
                });
            }
            files.push(File {
                path: relative,
                content: text.into(),
                source_path: name.display().to_string(),
                hash: hash(text.as_bytes()),
                provenance,
            });
        } else {
            return Err(failure("binary outputs cannot be edited through this view"));
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(files)
}

fn revision(source: &str, files: &[File]) -> String {
    hash(
        format!(
            "{source}{}",
            files
                .iter()
                .map(|f| format!("{}:{}", f.path, f.hash))
                .collect::<String>()
        )
        .as_bytes(),
    )
}

pub async fn create_value(state: &LocalState, backing: Backing) -> ApiResult<Representation> {
    let id = format!("{:016x}", super::rand_id());
    let source = match &backing {
        Backing::Files { paths } => {
            if paths.is_empty() || paths.len() > 32 {
                return Err(failure("choose between one and 32 source files"));
            }
            let mut originals = BTreeMap::new();
            for path in paths {
                writable(path).map_err(failure)?;
                if !confined(state.index.root(), path)
                    .map_err(failure)?
                    .exists()
                {
                    // Creation admits tracked deletions, but never invents a missing source.
                    super::revision::git(
                        state.index.root(),
                        &["ls-files", "--error-unmatch", "--", path],
                    )?;
                }
                if originals
                    .insert(path.clone(), read(state.index.root(), path)?)
                    .is_some()
                {
                    return Err(failure("a source file was selected twice"));
                }
            }
            initial(paths, &originals)?
        }
        Backing::Document { doc_id } => state.read_source(doc_id)?,
    };
    let mut files = compile(state, &id, &source, &backing).await?;
    if let Backing::Files { paths } = &backing {
        files.retain(|f| paths.contains(&f.path));
        if files.len() != paths.len()
            || files
                .iter()
                .any(|f| read(state.index.root(), &f.path).ok().as_deref() != Some(&f.content))
        {
            return Err(failure(
                "the literate view does not reconstruct the selected files byte-for-byte",
            ));
        }
    }
    let view = Representation {
        revision: revision(&source, &files),
        id: id.clone(),
        backing,
        source,
        files,
        explanation_stale: false,
        local_warning: None,
    };
    state.representations.lock().await.insert(id, view.clone());
    Ok(view)
}

pub async fn create(
    State(state): State<LocalState>,
    Json(body): Json<Create>,
) -> ApiResult<Json<Representation>> {
    Ok(Json(create_value(&state, body.backing).await?))
}

pub async fn list(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    super::representation_store::load(&state).await?;
    Ok(Json(
        json!({"views":state.representations.lock().await.values().cloned().collect::<Vec<_>>()}),
    ))
}

pub async fn preview(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Edit>,
) -> ApiResult<Json<Vec<File>>> {
    let view = state
        .representations
        .lock()
        .await
        .get(&id)
        .cloned()
        .ok_or_else(|| failure("view no longer exists"))?;
    if view.revision != body.revision {
        return Err(ApiError::conflict("Read the view again before previewing."));
    }
    let mut files = compile(&state, &id, &body.source, &view.backing).await?;
    if let Backing::Files { paths } = view.backing {
        files.retain(|f| paths.contains(&f.path));
    }
    Ok(Json(files))
}

pub async fn get(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Representation>> {
    super::representation_store::load(&state).await?;
    refresh(State(state), Path(id)).await
}

pub async fn edit_value(state: &LocalState, id: &str, body: Edit) -> ApiResult<Representation> {
    crate::engine::writes::during(&state.writes, edit_held(state, id, body)).await
}

async fn edit_held(state: &LocalState, id: &str, body: Edit) -> ApiResult<Representation> {
    let mut views = state.representations.lock().await;
    let before = views
        .get(id)
        .cloned()
        .ok_or_else(|| failure("view no longer exists"))?;
    if before.revision != body.revision {
        return Err(ApiError::conflict(
            "The view changed. Read it again before editing.",
        ));
    }
    let mut files = compile(state, id, &body.source, &before.backing).await?;
    match &before.backing {
        Backing::Files { paths } => {
            files.retain(|f| paths.contains(&f.path));
            // Reject undeclared outputs too, rather than silently dropping them.
            let parsed = hick_lang::parse(&body.source).map_err(failure)?;
            if parsed.all_tags().iter().any(|t| {
                t.name == "file"
                    && t.get_attribute("path")
                        .is_none_or(|p| !paths.iter().any(|path| path == p))
            }) || files.len() != paths.len()
            {
                return Err(failure(
                    "a view must continue to produce exactly its selected source files",
                ));
            }
            for file in &before.files {
                if read(state.index.root(), &file.path)? != file.content {
                    return Err(ApiError::conflict(format!(
                        "{} changed outside this view. Refresh before saving.",
                        file.path
                    )));
                }
            }
            let changed: Vec<_> = files
                .iter()
                .filter(|f| {
                    before
                        .files
                        .iter()
                        .any(|old| old.path == f.path && old.content != f.content)
                })
                .collect();
            // Validate the complete batch before publishing; rollback on IO failure.
            for file in &changed {
                let path = confined(state.index.root(), &file.path).map_err(failure)?;
                if path.exists()
                    && std::fs::metadata(path)
                        .map_err(failure)?
                        .permissions()
                        .readonly()
                {
                    return Err(failure(format!("{} is read-only", file.path)));
                }
            }
            crate::history::record(
                state.index.root(),
                hickory_workspace::history::ActKind::Saved,
                Some("Literate view edit".into()),
                &changed
                    .iter()
                    .map(|f| {
                        (
                            state.index.root().join(&f.path),
                            f.content.as_bytes().to_vec(),
                        )
                    })
                    .collect::<Vec<_>>(),
            );
            let absent: Vec<_> = changed
                .iter()
                .filter(|f| !state.index.root().join(&f.path).exists())
                .map(|f| &f.path)
                .collect();
            for (index, file) in changed.iter().enumerate() {
                let path = confined(state.index.root(), &file.path).map_err(failure)?;
                if let Err(error) = super::store::write_atomic(&path, file.content.as_bytes()) {
                    for prior in &changed[..index] {
                        let original = before.files.iter().find(|f| f.path == prior.path).unwrap();
                        if absent.contains(&&prior.path) {
                            std::fs::remove_file(state.index.root().join(&prior.path))
                                .map_err(failure)?;
                            continue;
                        }
                        super::store::write_atomic(
                            &state.index.root().join(&prior.path),
                            original.content.as_bytes(),
                        )
                        .map_err(|e| ApiError::internal(format!("Saving stopped and rollback failed: {e}. Inspect the selected files before continuing.")))?;
                    }
                    return Err(failure(error));
                }
            }
        }
        Backing::Document { doc_id } => {
            if state.read_source(doc_id)? != before.source {
                return Err(ApiError::conflict(
                    "The backing document changed. Refresh before saving.",
                ));
            }
            // The live room compares the same revision before publication.
            let saved = state
                .rooms
                .replace_source_if_current(doc_id, &before.source, &body.source, || {
                    state
                        .write_source(doc_id, &body.source)
                        .map_err(|e| anyhow::anyhow!("{}", e.message()))
                })
                .await
                .map_err(failure)?;
            if !saved {
                return Err(ApiError::conflict(
                    "The backing document changed in its editor. Refresh before saving.",
                ));
            }
        }
    }
    let code_changed = before.files.iter().any(|old| {
        files
            .iter()
            .any(|new| new.path == old.path && new.hash != old.hash)
    });
    let mut view = Representation {
        local_warning: None,
        revision: revision(&body.source, &files),
        source: body.source,
        files,
        explanation_stale: before.explanation_stale || code_changed,
        ..before
    };
    if let Err(e) = super::representation_store::update_if_kept(state, &view) {
        view.local_warning = Some(format!(
            "The code and live view are current, but the kept reading could not be updated: {}. Keep it again after fixing local storage.",
            e.message()
        ));
    }
    views.insert(id.into(), view.clone());
    Ok(view)
}

pub async fn edit(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<Edit>,
) -> ApiResult<Json<Representation>> {
    Ok(Json(edit_value(&state, &id, body).await?))
}

pub async fn refresh(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Representation>> {
    crate::engine::writes::during(&state.writes, refresh_held(&state, id))
        .await
        .map(Json)
}

async fn refresh_held(state: &LocalState, id: String) -> ApiResult<Representation> {
    let mut views = state.representations.lock().await;
    let before = views
        .get(&id)
        .cloned()
        .ok_or_else(|| failure("view no longer exists"))?;
    let mut source = before.source.clone();
    match &before.backing {
        Backing::Document { doc_id } => source = state.read_source(doc_id)?,
        Backing::Files { .. } => {
            let name = state
                .index
                .root()
                .join(format!("__lens-{id}.md"))
                .display()
                .to_string();
            let mut edits = Vec::new();
            for file in &before.files {
                edits.extend(
                    crate::up::reverse::source_edits_for_save(
                        &file.content,
                        &read(state.index.root(), &file.path)?,
                        &file.provenance,
                    )
                    .map_err(failure)?,
                );
            }
            if !edits.is_empty() {
                source = hickory_lineage::apply_source_edits(
                    &HashMap::from([(name.clone(), source)]),
                    &edits,
                )
                .map_err(failure)?
                .remove(&name)
                .ok_or_else(|| failure("external edits could not be mapped; reopen the view"))?;
            }
        }
    }
    let mut files = compile(state, &id, &source, &before.backing).await?;
    if let Backing::Files { paths } = &before.backing {
        files.retain(|f| paths.contains(&f.path));
    }
    if matches!(before.backing, Backing::Files { .. })
        && files
            .iter()
            .any(|f| read(state.index.root(), &f.path).ok().as_deref() != Some(&f.content))
    {
        return Err(ApiError::conflict(
            "External edits cannot be represented unambiguously in this arrangement. Open a new view; repository files were left intact.",
        ));
    }
    let changed = source != before.source;
    let mut view = Representation {
        local_warning: None,
        revision: revision(&source, &files),
        source,
        files,
        explanation_stale: before.explanation_stale || changed,
        ..before
    };
    if let Err(e) = super::representation_store::update_if_kept(state, &view) {
        view.local_warning = Some(format!(
            "The code and live view are current, but the kept reading could not be updated: {}. Keep it again after fixing local storage.",
            e.message()
        ));
    }
    views.insert(id, view.clone());
    Ok(view)
}

pub async fn discard(
    State(state): State<LocalState>,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    super::representation_store::discard(&state, &id)?;
    state.representations.lock().await.remove(&id);
    Ok(Json(json!({"ok":true})))
}
