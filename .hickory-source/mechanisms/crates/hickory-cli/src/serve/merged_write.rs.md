<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/merged_write.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-1635aa80d82b"></a>
## `crates/hickory-cli/src/serve/merged_write.rs`

Related contracts: [A Write Through The Merged View Reports Per Target, And Undo Reaches All Of Them](../../../../../guarantees/collaboration/a-merged-view-write-reports-per-target/contract.md), [Comparison edit permissions do not establish replay evidence](../../../../../guarantees/literate-projects/comparison-permissions-and-evidence-are-independent/contract.md).

<a id="f-1635aa80d82b-TargetStatus"></a>
### `TargetStatus`

Independent targets: exact displayed snapshots, explicit permissions and
per-target undo guarded against subsequent edits. No filesystem transaction.

<storycode:copy id="f-1635aa80d82b-TargetStatus">
//! Independent targets: exact displayed snapshots, explicit permissions and
//! per-target undo guarded against subsequent edits. No filesystem transaction.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    merged::{Worktree, worktrees},
};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
pub struct TargetStatus {
    pub source: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
</storycode:copy>
<a id="f-1635aa80d82b-MergedWrite"></a>
### `MergedWrite`

<storycode:copy id="f-1635aa80d82b-MergedWrite">
#[derive(Deserialize)]
pub struct MergedWrite {
    pub path: String,
    pub targets: Vec<String>,
    pub region: usize,
    pub text: String,
    pub route: Option<String>,
    pub source: Option<String>,
    /// Comparison membership does not grant permission to write.
    #[serde(default)]
    pub writable: Vec<String>,
    pub expected: BTreeMap<String, String>,
}
</storycode:copy>
<a id="f-1635aa80d82b-MergedUndo"></a>
### `MergedUndo`

<storycode:copy id="f-1635aa80d82b-MergedUndo">
#[derive(Deserialize)]
pub struct MergedUndo {
    pub path: String,
    pub undo: Vec<UndoEntry>,
}
</storycode:copy>
<a id="f-1635aa80d82b-UndoEntry"></a>
### `UndoEntry`

<storycode:copy id="f-1635aa80d82b-UndoEntry">
#[derive(Deserialize)]
pub struct UndoEntry {
    pub source: String,
    pub text: String,
    pub expected: String,
}

</storycode:copy>
<a id="f-1635aa80d82b-relative"></a>
### `relative`

<storycode:copy id="f-1635aa80d82b-relative">
fn relative(root: &Path, path: &str) -> anyhow::Result<PathBuf> {
    hick_lang::validate_project_path(path).map_err(anyhow::Error::msg)?;
    let mut full = root.to_path_buf();
    for part in path.split('/') {
        full.push(part);
        let meta = std::fs::symlink_metadata(&full)?;
        anyhow::ensure!(
            !meta.file_type().is_symlink(),
            "Worktree path follows a symlink: {path}"
        );
    }
    anyhow::ensure!(
        std::fs::symlink_metadata(&full)?.is_file(),
        "Not a regular worktree file: {path}"
    );
    Ok(full)
}
</storycode:copy>
<a id="f-1635aa80d82b-chosen"></a>
### `chosen`

<storycode:copy id="f-1635aa80d82b-chosen">
fn chosen(root: &Path, names: &[String]) -> anyhow::Result<Vec<Worktree>> {
    let trees = worktrees(root);
    let mut selected = vec![];
    let mut seen = BTreeSet::new();
    for name in names {
        anyhow::ensure!(seen.insert(name), "Duplicate worktree target: {name}");
        let matches: Vec<_> = trees.iter().filter(|t| &t.name == name).collect();
        anyhow::ensure!(
            matches.len() == 1,
            "Worktree must name one existing unambiguous target: {name}"
        );
        selected.push(matches[0].clone());
    }
    Ok(selected)
}
</storycode:copy>
<a id="f-1635aa80d82b-publish"></a>
### `publish`

<storycode:copy id="f-1635aa80d82b-publish">
async fn publish(
    state: &LocalState,
    tree: &Worktree,
    path: &str,
    expected: &str,
    text: &str,
) -> anyhow::Result<()> {
    let root = Path::new(&tree.path);
    let full = relative(root, path)?;
    let meta = std::fs::metadata(&full)?;
    anyhow::ensure!(
        std::fs::read_to_string(&full)? == expected,
        "Target changed since this comparison: {}",
        tree.name
    );
    if text == expected {
        if path.ends_with(".md")
            && let Ok(local) = full.strip_prefix(state.index.root().canonicalize()?)
            && let Some(room) = state
                .rooms
                .get(
                    &state
                        .index
                        .id_for_path(&local.to_string_lossy().replace('\\', "/")),
                )
                .await
        {
            anyhow::ensure!(
                room.text().await == expected,
                "A live editor changed target {}",
                tree.name
            );
        }
        // A target a partial write never reached is already restored. Do not
        // perform a write (or require write permission) to report that fact.
        return Ok(());
    }
    anyhow::ensure!(
        !meta.permissions().readonly(),
        "Target is read-only: {}",
        tree.name
    );
    // A generated file is an edit destination only through lineage. Never
    // bypass that boundary with a plain worktree write.
    let index = super::store::DocIndex::scan(root)?;
    if let Some(owner) = super::api::generated_outputs(root, &index).get(path) {
        anyhow::bail!(
            "Generated output {path}; edit contributing source {} through its output comparison",
            index.path_of(owner).unwrap_or_else(|| owner.clone())
        );
    }
    if path.ends_with(".md") {
        let doc = hick_lang::parse(text)?;
        let errors = hick_blocks::attribute_errors(text, &doc);
        anyhow::ensure!(errors.is_empty(), "Invalid document attributes: {errors:?}");
        if !doc.find_tags("project").is_empty() {
            let entry = full
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 entry"))?;
            let parent = full.parent().unwrap();
            let sources = hick_lang::load_project_sources(entry, &|name| {
                if name == entry {
                    Ok(text.into())
                } else {
                    std::fs::read_to_string(parent.join(name)).map_err(|e| e.to_string())
                }
            })
            .map_err(anyhow::Error::msg)?;
            hick_lang::evaluate_project(entry, &sources, &hick_lang::ProjectTree::default())
                .map_err(anyhow::Error::msg)?;
        }
    }
    let persist = || -> anyhow::Result<()> {
        anyhow::ensure!(
            std::fs::read_to_string(&full)? == expected,
            "Target changed before publication: {}",
            tree.name
        );
        use std::io::Write;
        let mut temporary = tempfile::NamedTempFile::new_in(
            full.parent()
                .ok_or_else(|| anyhow::anyhow!("Target needs a directory"))?,
        )?;
        temporary.write_all(text.as_bytes())?;
        temporary.as_file().set_permissions(meta.permissions())?;
        temporary.as_file().sync_all()?;
        anyhow::ensure!(
            std::fs::read_to_string(&full)? == expected,
            "Target changed before publication: {}",
            tree.name
        );
        temporary.persist(&full)?;
        Ok(())
    };
    if path.ends_with(".md")
        && let Ok(local) = full.strip_prefix(state.index.root().canonicalize()?)
    {
        let id = state.index.add(&local.to_string_lossy().replace('\\', "/"));
        anyhow::ensure!(
            state
                .rooms
                .replace_source_if_current(&id, expected, text, persist)
                .await?,
            "A live editor changed target {}",
            tree.name
        );
    } else {
        persist()?;
    }
    Ok(())
}
</storycode:copy>
<a id="f-1635aa80d82b-write"></a>
### `write`

<storycode:copy id="f-1635aa80d82b-write">
pub async fn write(
    State(state): State<LocalState>,
    Json(body): Json<MergedWrite>,
) -> ApiResult<Json<Value>> {
    hick_lang::validate_project_path(&body.path).map_err(ApiError::bad_request)?;
    let writes = state.writes.clone();
    crate::engine::writes::during(&writes, write_locked(state, body)).await
}
</storycode:copy>
<a id="f-1635aa80d82b-write_locked"></a>
### `write_locked`

<storycode:copy id="f-1635aa80d82b-write_locked">
async fn write_locked(state: LocalState, body: MergedWrite) -> ApiResult<Json<Value>> {
    let trees = chosen(state.index.root(), &body.targets).map_err(ApiError::from)?;
    let route = match body.route.as_deref() {
        None | Some("shared") => hick_merge::Route::Shared,
        Some("just-here") => hick_merge::Route::JustHere {
            source: body
                .source
                .ok_or_else(|| ApiError::bad_request("Choose one worktree target."))?,
        },
        Some(other) => return Err(ApiError::bad_request(format!("Unknown route {other}"))),
    };
    let mut sources = vec![];
    for tree in &trees {
        let path = relative(Path::new(&tree.path), &body.path).map_err(ApiError::from)?;
        let text =
            std::fs::read_to_string(path).map_err(|e| ApiError::unprocessable(e.to_string()))?;
        if body.expected.get(&tree.name) != Some(&text) {
            return Err(ApiError::conflict(format!(
                "{} changed; rebuild this comparison before editing its region",
                tree.name
            )));
        }
        sources.push(hick_merge::Source {
            name: tree.name.clone(),
            text,
        });
    }
    let view = hick_merge::merged_view(&sources);
    let writes = hick_merge::writes_for_edit(&view, body.region, &body.text, &route)
        .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    let mut statuses = vec![];
    let mut undo = vec![];
    for w in &writes {
        let tree = trees.iter().find(|t| t.name == w.source).unwrap();
        let before = &body.expected[&w.source];
        let result = if body.writable.contains(&w.source) {
            publish(&state, tree, &body.path, before, &w.text).await
        } else {
            Err(anyhow::anyhow!(
                "Target is read-only; explicitly open {} for writing",
                w.source
            ))
        };
        let expected = if result.is_ok() { &w.text } else { before };
        undo.push(json!({"source":w.source,"text":before,"expected":expected}));
        statuses.push(TargetStatus {
            source: w.source.clone(),
            ok: result.is_ok(),
            error: result.err().map(|e| format!("{e:#}")),
        });
    }
    let landed = statuses.iter().filter(|s| s.ok).count();
    Ok(Json(
        json!({"path":body.path,"targets":statuses,"landed":landed,"partial":landed>0 && landed<writes.len(),"undo":undo}),
    ))
}
</storycode:copy>
<a id="f-1635aa80d82b-undo"></a>
### `undo`

<storycode:copy id="f-1635aa80d82b-undo">
pub async fn undo(
    State(state): State<LocalState>,
    Json(body): Json<MergedUndo>,
) -> ApiResult<Json<Value>> {
    hick_lang::validate_project_path(&body.path).map_err(ApiError::bad_request)?;
    let writes = state.writes.clone();
    crate::engine::writes::during(&writes, undo_locked(state, body)).await
}
</storycode:copy>
<a id="f-1635aa80d82b-undo_locked"></a>
### `undo_locked`

<storycode:copy id="f-1635aa80d82b-undo_locked">
async fn undo_locked(state: LocalState, body: MergedUndo) -> ApiResult<Json<Value>> {
    let mut statuses = vec![];
    for item in body.undo {
        let result = match chosen(state.index.root(), std::slice::from_ref(&item.source)) {
            Ok(trees) => publish(&state, &trees[0], &body.path, &item.expected, &item.text).await,
            Err(e) => Err(e),
        };
        statuses.push(TargetStatus {
            source: item.source,
            ok: result.is_ok(),
            error: result.err().map(|e| format!("{e:#}")),
        });
    }
    let restored = statuses.iter().filter(|s| s.ok).count();
    Ok(Json(
        json!({"path":body.path,"targets":statuses,"restored":restored,"partial":restored>0 && restored<statuses.len()}),
    ))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/merged_write.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/merged_write.rs"><storycode:paste select="#f-1635aa80d82b-TargetStatus"/><storycode:paste select="#f-1635aa80d82b-MergedWrite"/><storycode:paste select="#f-1635aa80d82b-MergedUndo"/><storycode:paste select="#f-1635aa80d82b-UndoEntry"/><storycode:paste select="#f-1635aa80d82b-relative"/><storycode:paste select="#f-1635aa80d82b-chosen"/><storycode:paste select="#f-1635aa80d82b-publish"/><storycode:paste select="#f-1635aa80d82b-write"/><storycode:paste select="#f-1635aa80d82b-write_locked"/><storycode:paste select="#f-1635aa80d82b-undo"/><storycode:paste select="#f-1635aa80d82b-undo_locked"/></storycode:file>

</storycode:doc>
