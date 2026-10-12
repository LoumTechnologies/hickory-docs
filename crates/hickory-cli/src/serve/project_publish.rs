//! Reviewed Markdown batches share the live-room CAS publication boundary.
use super::LocalState;
use anyhow::{Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

pub async fn documents(
    state: &LocalState,
    root: &Path,
    paths: impl IntoIterator<Item = String>,
) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for path in paths {
        let full = confined(root, &path)?;
        if !full.exists() {
            continue;
        }
        ensure!(
            std::fs::symlink_metadata(&full)?.is_file(),
            "Markdown destination is not a regular file: {path}"
        );
        let id = state.id_for_doc_path(
            full.to_str()
                .ok_or_else(|| anyhow::anyhow!("Non-UTF-8 source destination"))?,
        );
        let source = if let Some(room) = state.rooms.get(&id).await {
            room.text().await
        } else {
            std::fs::read_to_string(full)?
        };
        result.insert(path, source);
    }
    Ok(result)
}

pub fn confined(root: &Path, relative: &str) -> Result<std::path::PathBuf> {
    hick_lang::validate_project_path(relative).map_err(anyhow::Error::msg)?;
    ensure!(
        relative.ends_with(".md"),
        "Literate destinations must be Markdown: {relative}"
    );
    let mut path = root.to_path_buf();
    for component in relative.split('/') {
        path.push(component);
        if let Ok(meta) = std::fs::symlink_metadata(&path) {
            ensure!(
                !meta.file_type().is_symlink(),
                "Source destination follows a symlink: {relative}"
            );
        }
    }
    Ok(path)
}

/// Every item is guarded independently, and a partial failure names what landed.
/// The caller holds `state.writes`; this is not a cross-file transaction.
pub async fn publish(
    state: &LocalState,
    root: &Path,
    baseline: &BTreeMap<String, String>,
    candidate: &BTreeMap<String, String>,
) -> Result<Vec<String>> {
    let paths: BTreeSet<_> = baseline.keys().chain(candidate.keys()).cloned().collect();
    ensure!(
        documents(state, root, paths.clone()).await? == *baseline,
        "Literate destinations changed; review a new candidate."
    );
    let mut applied = vec![];
    for path in paths {
        if baseline.get(&path) == candidate.get(&path) {
            continue;
        }
        let result: Result<bool> = async {
            let full = confined(root, &path)?;
            let rel = full
                .strip_prefix(state.index.root().canonicalize()?)?
                .to_string_lossy()
                .replace('\\', "/");
            let id = state.index.add(&rel);
            let expected = baseline.get(&path).map_or("", String::as_str);
            if let Some(source) = candidate.get(&path) {
                if !full.exists() {
                    ensure!(
                        !baseline.contains_key(&path),
                        "Source destination disappeared: {path}"
                    );
                    ensure!(
                        state.rooms.get(&id).await.is_none(),
                        "A live editor still owns the missing source: {path}"
                    );
                    let parent = full
                        .parent()
                        .ok_or_else(|| anyhow::anyhow!("Source needs a directory"))?;
                    std::fs::create_dir_all(parent)?;
                    use std::io::Write;
                    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
                    temporary.write_all(source.as_bytes())?;
                    crate::history::record(
                        state.index.root(),
                        hickory_workspace::history::ActKind::Saved,
                        Some("Literate source incorporation".into()),
                        &[(full.clone(), source.as_bytes().to_vec())],
                    );
                    temporary.persist_noclobber(&full)?;
                    return Ok(true);
                }
                state
                    .rooms
                    .replace_source_if_current(&id, expected, source, || {
                        ensure!(
                            std::fs::read_to_string(&full)? == expected,
                            "Destination changed: {path}"
                        );
                        state
                            .write_source(&id, source)
                            .map_err(|e| anyhow::anyhow!("{}", e.detail()))
                    })
                    .await
            } else {
                state
                    .rooms
                    .retire_source_if_current(&id, expected, || {
                        ensure!(
                            std::fs::read_to_string(&full)? == expected,
                            "Destination changed: {path}"
                        );
                        crate::history::record(
                            state.index.root(),
                            hickory_workspace::history::ActKind::Saved,
                            Some("Literate source incorporation".into()),
                            &[(full.clone(), vec![])],
                        );
                        std::fs::remove_file(&full)?;
                        Ok(())
                    })
                    .await
            }
        }
        .await;
        match result {
            Ok(true) => applied.push(path),
            Ok(false) => anyhow::bail!(
                "Concurrent edit refused at {path}; applied destinations: {applied:?}. Undo these individually against their reviewed bytes."
            ),
            Err(e) => anyhow::bail!(
                "Publication failed at {path}: {e:#}; applied destinations: {applied:?}. Undo these individually against their reviewed bytes."
            ),
        }
    }
    Ok(applied)
}
