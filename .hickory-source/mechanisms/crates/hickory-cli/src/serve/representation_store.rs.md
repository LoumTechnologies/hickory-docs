<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/representation_store.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c77b1ea387e3"></a>
## `crates/hickory-cli/src/serve/representation_store.rs`

Related contracts: [A literate view writes through to ordinary source](../../../../../guarantees/authoring/a-literate-view-writes-through-to-ordinary-source/contract.md).

<a id="f-c77b1ea387e3-directory"></a>
### `directory`

Optional personal persistence. Repository files remain the only code authority.

<storycode:copy id="f-c77b1ea387e3-directory">
//! Optional personal persistence. Repository files remain the only code authority.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    representation::Representation,
};
use std::path::PathBuf;

fn directory(state: &LocalState) -> ApiResult<PathBuf> {
    Ok(
        hickory_workspace::WorkspaceStore::for_project(state.index.root())?
            .dir()
            .join("literate-views"),
    )
}
</storycode:copy>
<a id="f-c77b1ea387e3-path"></a>
### `path`

<storycode:copy id="f-c77b1ea387e3-path">
fn path(state: &LocalState, id: &str) -> ApiResult<PathBuf> {
    if id.len() != 16 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::bad_request("Invalid view identity."));
    }
    Ok(directory(state)?.join(format!("{id}.json")))
}
</storycode:copy>
<a id="f-c77b1ea387e3-load"></a>
### `load`

<storycode:copy id="f-c77b1ea387e3-load">
pub async fn load(state: &LocalState) -> ApiResult<()> {
    let dir = directory(state)?;
    if !dir.exists() {
        return Ok(());
    }
    let mut views = state.representations.lock().await;
    for entry in std::fs::read_dir(dir).map_err(anyhow::Error::from)? {
        let entry = entry.map_err(anyhow::Error::from)?;
        if entry.path().extension().is_none_or(|e| e != "json") {
            continue;
        }
        let bytes = std::fs::read(entry.path()).map_err(anyhow::Error::from)?;
        let mut view: Representation =
            serde_json::from_slice(&bytes).map_err(anyhow::Error::from)?;
        if entry.path() != path(state, &view.id)? {
            continue;
        }
        if views.contains_key(&view.id) {
            continue;
        }
        // A kept reading may move to a bisect worktree. Rebuild locations with this root.
        let compiled =
            super::representation::compile(state, &view.id, &view.source, &view.backing).await?;
        for file in &mut view.files {
            if let Some(next) = compiled
                .iter()
                .find(|f| f.path == file.path && f.content == file.content)
            {
                file.provenance = next.provenance.clone();
                file.source_path = next.source_path.clone();
            } else {
                return Err(ApiError::conflict(
                    "The kept view's metadata does not match its document. Reopen a source-backed view.",
                ));
            }
        }
        views.insert(view.id.clone(), view);
    }
    Ok(())
}
</storycode:copy>
<a id="f-c77b1ea387e3-keep"></a>
### `keep`

<storycode:copy id="f-c77b1ea387e3-keep">
pub fn keep(state: &LocalState, view: &Representation) -> ApiResult<PathBuf> {
    let metadata = path(state, &view.id)?;
    std::fs::create_dir_all(metadata.parent().unwrap()).map_err(anyhow::Error::from)?;
    let document = metadata.with_extension("md");
    super::store::write_atomic(&document, view.source.as_bytes())?;
    super::store::write_atomic(
        &metadata,
        &serde_json::to_vec(view).map_err(anyhow::Error::from)?,
    )?;
    Ok(document)
}
</storycode:copy>
<a id="f-c77b1ea387e3-update_if_kept"></a>
### `update_if_kept`

<storycode:copy id="f-c77b1ea387e3-update_if_kept">
pub fn update_if_kept(state: &LocalState, view: &Representation) -> ApiResult<()> {
    if path(state, &view.id)?.exists() {
        keep(state, view)?;
    }
    Ok(())
}
</storycode:copy>
<a id="f-c77b1ea387e3-discard"></a>
### `discard`

<storycode:copy id="f-c77b1ea387e3-discard">
pub fn discard(state: &LocalState, id: &str) -> ApiResult<()> {
    let metadata = path(state, id)?;
    for file in [metadata.with_extension("md"), metadata] {
        if file.exists() {
            std::fs::remove_file(file).map_err(anyhow::Error::from)?;
        }
    }
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/representation_store.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/representation_store.rs"><storycode:paste select="#f-c77b1ea387e3-directory"/><storycode:paste select="#f-c77b1ea387e3-path"/><storycode:paste select="#f-c77b1ea387e3-load"/><storycode:paste select="#f-c77b1ea387e3-keep"/><storycode:paste select="#f-c77b1ea387e3-update_if_kept"/><storycode:paste select="#f-c77b1ea387e3-discard"/></storycode:file>

</storycode:doc>
