<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-fleet/src/store.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a15d9c64a842"></a>
## `crates/hickory-fleet/src/store.rs`

<a id="f-a15d9c64a842-lock"></a>
### `lock`

<storycode:copy id="f-a15d9c64a842-lock">
use super::{FleetFile, Machine, restrict};
use anyhow::{Context, Result};
use std::{fs::File, io::Write, path::Path};

/// All updated CLI/server writers lock the same stable sidecar, never a renamed inode.
pub(super) fn lock(path: &Path, exclusive: bool) -> Result<File> {
    let parent = path.parent().context("Fleet path has no parent")?;
    std::fs::create_dir_all(parent)?;
    let path = path.with_extension("lock");
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&path)?;
    restrict(&path)?;
    if exclusive {
        fs2::FileExt::lock_exclusive(&file)?;
    } else {
        fs2::FileExt::lock_shared(&file)?;
    }
    Ok(file)
}

</storycode:copy>
<a id="f-a15d9c64a842-load"></a>
### `load`

A failed read is safe as an empty *view*, but must never become an empty write.

<storycode:copy id="f-a15d9c64a842-load">
/// A failed read is safe as an empty *view*, but must never become an empty write.
pub(super) fn load(path: &Path) -> Result<Vec<Machine>> {
    let raw = match std::fs::read(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(error).with_context(|| format!("Could not read {}", path.display()));
        }
    };
    let value: FleetFile = serde_json::from_slice(&raw).with_context(|| {
        format!(
            "Refusing to replace unreadable fleet {}; repair it before changing grants",
            path.display()
        )
    })?;
    Ok(value.machines)
}

</storycode:copy>
<a id="f-a15d9c64a842-save"></a>
### `save`

Called under the exclusive sidecar lock. Readers see an entire old or new roster.

<storycode:copy id="f-a15d9c64a842-save">
/// Called under the exclusive sidecar lock. Readers see an entire old or new roster.
pub(super) fn save(path: &Path, machines: Vec<Machine>) -> Result<()> {
    let parent = path.parent().context("Fleet path has no parent")?;
    let body = serde_json::to_vec_pretty(&FleetFile { machines }).context("encoding the fleet")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    restrict(temporary.path())?;
    temporary.write_all(&body)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(path)
        .map_err(|error| error.error)
        .with_context(|| format!("Could not replace {}", path.display()))?;
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-fleet/src/store.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-fleet/src/store.rs"><storycode:paste select="#f-a15d9c64a842-lock"/><storycode:paste select="#f-a15d9c64a842-load"/><storycode:paste select="#f-a15d9c64a842-save"/></storycode:file>

</storycode:doc>
