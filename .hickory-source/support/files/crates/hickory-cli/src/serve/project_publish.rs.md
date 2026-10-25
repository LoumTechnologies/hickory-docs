<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/serve/project_publish.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Finalization and incorporation check distinct equalities](../../../../../../guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities/contract.md)

Captured SHA-256: `b8638283d76f89878cc65c1f06a7e7596987f0e15eaa26ee74e97924b48721a4`.

<book:entry path="crates/hickory-cli/src/serve/project_publish.rs">
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
    confined_file(root, relative)
}

pub fn confined_file(root: &Path, relative: &str) -> Result<std::path::PathBuf> {
    hick_lang::validate_project_path(relative).map_err(anyhow::Error::msg)?;
    ensure!(
        relative != hick_lang::EDITOR_BUFFER_PREFIX.trim_end_matches('/')
            && !relative.starts_with(hick_lang::EDITOR_BUFFER_PREFIX),
        "Editor addresses have no filesystem destination"
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
    publish_text(state, root, baseline, candidate, None).await
}

/// Shared guarded text publication, also used for captured B-to-C outputs.
pub async fn publish_text(
    state: &LocalState,
    root: &Path,
    baseline: &BTreeMap<String, String>,
    candidate: &BTreeMap<String, String>,
    expected_tree: Option<&hick_lang::ProjectTree>,
) -> Result<Vec<String>> {
    let files = expected_tree.is_some();
    let paths: BTreeSet<_> = baseline.keys().chain(candidate.keys()).cloned().collect();
    let mut current = BTreeMap::new();
    for path in &paths {
        let full = if files {
            confined_file(root, path)?
        } else {
            confined(root, path)?
        };
        if full.exists() {
            ensure!(
                std::fs::symlink_metadata(&full)?.is_file(),
                "Not a regular destination: {path}"
            );
            current.insert(path.clone(), std::fs::read_to_string(full)?);
        }
    }
    if !files {
        current = documents(state, root, paths.clone()).await?;
    }
    ensure!(
        current == *baseline,
        "Literate destinations changed; review a new candidate."
    );
    let mut applied = vec![];
    for path in paths {
        if baseline.get(&path) == candidate.get(&path) {
            continue;
        }
        let result: Result<bool> = async {
            let full = if files {
                confined_file(root, &path)?
            } else {
                confined(root, &path)?
            };
            if let Some(tree) = expected_tree {
                check_entry(root, &path, tree)?;
            }
            let rel = full
                .strip_prefix(state.index.root().canonicalize()?)?
                .to_string_lossy()
                .replace('\\', "/");
            let id = if path.ends_with(".md") {
                state.index.add(&rel)
            } else {
                state.index.id_for_path(&rel)
            };
            let expected = baseline.get(&path).map_or("", String::as_str);
            if full.exists() {
                ensure!(
                    !std::fs::metadata(&full)?.permissions().readonly(),
                    "Destination is read-only: {path}"
                );
            }
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
                let persist = || -> Result<()> {
                    if let Some(tree) = expected_tree {
                        check_entry(root, &path, tree)?;
                    }
                    ensure!(
                        std::fs::read_to_string(&full)? == expected,
                        "Destination changed: {path}"
                    );
                    if path.ends_with(".md") {
                        state
                            .write_source(&id, source)
                            .map_err(|e| anyhow::anyhow!("{}", e.detail()))
                    } else {
                        crate::history::record(
                            state.index.root(),
                            hickory_workspace::history::ActKind::Saved,
                            Some("Authored final changeset revision".into()),
                            &[(full.clone(), source.as_bytes().to_vec())],
                        );
                        super::store::write_atomic(&full, source.as_bytes())
                    }
                };
                if path.ends_with(".md") {
                    state
                        .rooms
                        .replace_source_if_current(&id, expected, source, persist)
                        .await
                } else {
                    persist()?;
                    Ok(true)
                }
            } else {
                let persist = || -> Result<()> {
                    if let Some(tree) = expected_tree {
                        check_entry(root, &path, tree)?;
                    }
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
                };
                if path.ends_with(".md") {
                    state
                        .rooms
                        .retire_source_if_current(&id, expected, persist)
                        .await
                } else {
                    persist()?;
                    Ok(true)
                }
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

/// Check modeled metadata as well as bytes immediately before each publication.
fn check_entry(root: &Path, path: &str, expected: &hick_lang::ProjectTree) -> Result<()> {
    let full = confined_file(root, path)?;
    let metadata = std::fs::symlink_metadata(&full);
    if let Some(entry) = expected.get(path) {
        let metadata = metadata?;
        ensure!(metadata.is_file(), "Destination kind changed: {path}");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            ensure!(
                (metadata.permissions().mode() & 0o111 != 0) == entry.executable,
                "Destination mode changed: {path}"
            );
        }
        ensure!(
            std::fs::read(&full)?.as_slice() == entry.bytes.as_ref(),
            "Destination bytes changed: {path}"
        );
    } else {
        ensure!(
            metadata.is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
            "A destination was created: {path}"
        );
    }
    Ok(())
}
</book:entry>

</book:doc>
