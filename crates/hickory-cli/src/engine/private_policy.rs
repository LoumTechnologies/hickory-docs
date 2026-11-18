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

fn recorded_private_store(retained:&Retained,root:&Path)->Result<Option<PathBuf>>{
 if !retained.roots.contains(root){return Ok(None)}
 let store=retained.stores.get(root).ok_or_else(||anyhow::anyhow!("Retained private root has no original protected store; ordinary fallback refused"))?;
 ensure!(store.is_absolute(),"Retained private store has no exact absolute identity");Ok(Some(store.clone()))
}
/// This is a refusal predicate, not permission to run private tools. Only the
/// already captured private provider owns isolated source edits.
pub(crate) fn exact_private_root(root:&Path)->Result<bool>{
 let root=root.canonicalize()?;let retained=read()?;let Some(store)=recorded_private_store(&retained,&root)? else{return Ok(false)};
 let recovery=binding(&root)?.ok_or_else(||anyhow::anyhow!("Original private source binding is unavailable; ordinary fallback refused"))?;
 ensure!(recovery.root()==root,"Retained private source provider belongs to another root");
 let protected=recovery.protected_store().canonicalize()?;
 ensure!(protected==store&&store.canonicalize()?==store,"Retained private provider and protected store no longer correspond");
 let metadata=std::fs::symlink_metadata(&store)?;ensure!(metadata.is_dir()&&!metadata.file_type().is_symlink(),"Private provider store is no longer its ordinary directory");Ok(true)
}
#[cfg(test)]mod exact_source_scope_tests{
 use super::*;
 #[test]fn retained_private_selection_is_exact_even_after_end_and_never_selects_ancestors(){let temporary=tempfile::tempdir().unwrap();let root=temporary.path().join("private");let sibling=temporary.path().join("ordinary");let store=temporary.path().join("store");std::fs::create_dir(&root).unwrap();std::fs::create_dir(&sibling).unwrap();std::fs::create_dir(&store).unwrap();let root=root.canonicalize().unwrap();let store=store.canonicalize().unwrap();let retained=Retained{version:1,roots:BTreeSet::from([root.clone()]),ended:BTreeSet::from([root.clone()]),stores:BTreeMap::from([(root.clone(),store.clone())])};assert_eq!(recorded_private_store(&retained,&root).unwrap(),Some(store));assert!(recorded_private_store(&retained,&sibling).unwrap().is_none());assert!(recorded_private_store(&retained,root.parent().unwrap()).unwrap().is_none());let missing=Retained{stores:BTreeMap::new(),..retained};assert!(recorded_private_store(&missing,&root).is_err());}
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
