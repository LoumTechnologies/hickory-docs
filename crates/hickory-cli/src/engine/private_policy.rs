//! Durable quarantine admission, containing no target answers or execution logs.
//! A daemon restart must not silently turn a private root into an ordinary one.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retained {
    version: u32,
    roots: BTreeSet<PathBuf>,
    #[serde(default)]
    ended: BTreeSet<PathBuf>,
    #[serde(default)]
    stores: BTreeMap<PathBuf, PathBuf>,
}
fn read() -> Result<Retained> {
    let path = super::directory()?.join("private-roots.json");
    let mut retained = match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice::<Retained>(&bytes)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Retained {
                version: 1,
                ..Default::default()
            });
        }
        Err(error) => return Err(error.into()),
    };
    // Older recovery records already contain the trusted ledger location.
    // Recover that protection before interpreting an old End declaration.
    for root in &retained.roots {
        if !retained.stores.contains_key(root) {
            match std::fs::read(binding_path(root)?) {
                Ok(bytes) => {
                    let recovery: crate::serve::private_prediction_recovery::Recovery =
                        serde_json::from_slice(&bytes)?;
                    retained
                        .stores
                        .insert(root.clone(), recovery.protected_store().canonicalize()?);
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => ensure!(
                    !retained.ended.contains(root),
                    "Ended private policy lacks its evidence-storage identity; trusted host recovery required"
                ),
                Err(error) => return Err(error.into()),
            }
        }
    }
    ensure!(
        retained.version == 1
            && retained.roots.iter().all(|root| root.is_absolute())
            && retained.ended.is_subset(&retained.roots)
            && retained
                .stores
                .iter()
                .all(|(root, path)| retained.roots.contains(root) && path.is_absolute()),
        "Invalid retained private policy; ordinary fallback refused"
    );
    Ok(retained)
}
pub(crate) fn active() -> Result<bool> {
    let retained = read()?;
    Ok(retained
        .roots
        .iter()
        .any(|root| !retained.ended.contains(root)))
}
pub(crate) fn overlaps(root: &Path) -> Result<bool> {
    let retained = read()?;
    Ok(retained
        .roots
        .iter()
        .chain(retained.stores.values())
        .any(|other| root.starts_with(other) || other.starts_with(root)))
}
/// Called under the daemon's attach mutex and owner lock before serving a binding.
pub(super) fn retain(root: &Path, store: &Path) -> Result<()> {
    let mut retained = read()?;
    retained.roots.insert(root.to_path_buf());
    retained
        .stores
        .insert(root.to_path_buf(), store.canonicalize()?);
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

// In-process ordinary requests admitted before quarantine must drain before a
// private startup can publish. Background jobs are checked separately afterwards.
static ORDINARY: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub(crate) struct Ordinary;
pub(crate) fn ordinary() -> Ordinary {
    ORDINARY.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    Ordinary
}
impl Drop for Ordinary {
    fn drop(&mut self) {
        ORDINARY.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}
pub(super) async fn drain() -> Result<()> {
    tokio::time::timeout(std::time::Duration::from_secs(5),async {
        while ORDINARY.load(std::sync::atomic::Ordering::SeqCst)>0 {tokio::time::sleep(std::time::Duration::from_millis(10)).await;}
    }).await.map_err(|_|anyhow::anyhow!("Ordinary requests or debugger operations are still active; stop them before opening private practice"))
}
