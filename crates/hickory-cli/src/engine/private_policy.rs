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
        retained.version == 1 && retained.roots.iter().all(|root| root.is_absolute()),
        "Invalid retained private policy; ordinary fallback refused"
    );
    Ok(retained)
}
pub(super) fn active() -> Result<bool> {
    Ok(!read()?.roots.is_empty())
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
    crate::serve::store::write_atomic(
        &super::directory()?.join("private-roots.json"),
        &serde_json::to_vec(&retained)?,
    )
}
