//! Durable quarantine admission, containing no target answers or execution logs.
//! A daemon restart must not silently turn a private root into an ordinary one.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retained {
    version: u32,
    roots: BTreeSet<PathBuf>,
    #[serde(default)]
    ended: BTreeSet<PathBuf>,
}
fn read() -> Result<Retained> {
    let path = super::directory()?.join("private-roots.json");
    let retained = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<Retained>(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Retained {
                version: 1,
                ..Default::default()
            });
        }
        Err(error) => return Err(error.into()),
    };
    ensure!(
        retained.version == 1
            && retained.roots.iter().all(|root| root.is_absolute())
            && retained.ended.is_subset(&retained.roots),
        "Invalid retained private policy; ordinary fallback refused"
    );
    Ok(retained)
}
pub(super) fn active() -> Result<bool> {
    let retained = read()?;
    Ok(retained
        .roots
        .iter()
        .any(|root| !retained.ended.contains(root)))
}
pub(super) fn overlaps(root: &Path) -> Result<bool> {
    Ok(read()?
        .roots
        .iter()
        .any(|other| root.starts_with(other) || other.starts_with(root)))
}
/// Called under the daemon's attach mutex and owner lock before serving a binding.
pub(super) fn retain(root: &Path) -> Result<()> {
    let mut retained = read()?;
    retained.roots.insert(root.to_path_buf());
    retained.ended.remove(root);
    crate::serve::store::write_atomic(
        &super::directory()?.join("private-roots.json"),
        &serde_json::to_vec(&retained)?,
    )
}

fn binding_path(root: &Path) -> Result<PathBuf> {
    let dir = super::directory()?.join("private-bindings");
    std::fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{}.json", hick_observe::prediction::digest(&root)?)))
}
pub(crate) fn store_binding(
    root: &Path,
    binding: &crate::serve::private_prediction_recovery::Recovery,
) -> Result<()> {
    ensure!(root == binding.root(), "Private recovery root mismatch");
    crate::serve::store::write_atomic(&binding_path(root)?, &serde_json::to_vec(binding)?)
}
pub(super) fn binding(
    root: &Path,
) -> Result<Option<crate::serve::private_prediction_recovery::Recovery>> {
    if !read()?.roots.contains(root) {
        return Ok(None);
    }
    match std::fs::read(binding_path(root)?) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Ending releases global context quarantine, never this root's private admission.
pub(super) fn end(root: &Path) -> Result<()> {
    let mut retained = read()?;
    ensure!(retained.roots.contains(root), "Private root unavailable");
    retained.ended.insert(root.to_path_buf());
    crate::serve::store::write_atomic(
        &super::directory()?.join("private-roots.json"),
        &serde_json::to_vec(&retained)?,
    )
}
