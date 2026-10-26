<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-observe/src/store.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-323322b54a07"></a>
## `crates/hick-observe/src/store.rs`

<a id="f-323322b54a07-LocalStore"></a>
### `LocalStore`

<storycode:copy id="f-323322b54a07-LocalStore">
use crate::*;
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// Single-writer lease. Opening twice fails rather than recovering an active writer.
/// Root is .hick/runs, deliberately outside the evictable .hick-cache tree.
pub struct LocalStore {
    root: PathBuf,
    _lease: File,
}
</storycode:copy>
<a id="f-323322b54a07-Retrieved"></a>
### `Retrieved`

<storycode:copy id="f-323322b54a07-Retrieved">
#[derive(Debug)]
pub enum Retrieved {
    Present(Run),
    Missing,
    Corrupt(String),
}
</storycode:copy>
<a id="f-323322b54a07-Retention"></a>
### `Retention`

<storycode:copy id="f-323322b54a07-Retention">
#[derive(Debug, Clone, Copy)]
pub struct Retention {
    pub max_unpinned_runs: usize,
    pub max_unpinned_bytes: u64,
    pub max_age_ms: u64,
}
</storycode:copy>
<a id="f-323322b54a07-impl-Default-for-Retention"></a>
### `impl-Default for Retention`

<storycode:copy id="f-323322b54a07-impl-Default-for-Retention">
impl Default for Retention {
    fn default() -> Self {
        Self {
            max_unpinned_runs: 100,
            max_unpinned_bytes: 256 * 1024 * 1024,
            max_age_ms: 30 * 86400 * 1000,
        }
    }
}
</storycode:copy>
<a id="f-323322b54a07-impl-LocalStore"></a>
### `impl-LocalStore`

<storycode:copy id="f-323322b54a07-impl-LocalStore">
impl LocalStore {
    /// Recovery marks persisted running records partial; it never claims success.
    pub fn open(project: &Path) -> Result<Self> {
        let root = project.join(".hick/runs");
        fs::create_dir_all(&root)?;
        let lease = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("writer.lock"))?;
        lease.try_lock_exclusive()?;
        let store = Self {
            root,
            _lease: lease,
        };
        for id in store.ids()? {
            if let Retrieved::Present(mut r) = store.get(&id)?
                && r.status == Status::Running
            {
                r.status = Status::Partial;
                r.detail = Some("writer interrupted; recovered on restart".into());
                store.save(&r)?;
            }
        }
        Ok(store)
    }
    fn path(&self, id: &str) -> Result<PathBuf> {
        ensure!(
            !id.is_empty() && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'),
            "invalid run id"
        );
        Ok(self.root.join(format!("{id}.json")))
    }
    pub fn ids(&self) -> Result<Vec<String>> {
        let mut ids = vec![];
        for e in fs::read_dir(&self.root)? {
            let path = e?.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Some(id) = path.file_stem().and_then(|s| s.to_str())
            {
                ids.push(id.to_owned());
            }
        }
        ids.sort();
        Ok(ids)
    }
    pub fn get(&self, id: &str) -> Result<Retrieved> {
        let path = self.path(id)?;
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Retrieved::Missing),
            Err(e) => return Err(e.into()),
        };
        match serde_json::from_slice::<Run>(&bytes) {
            Ok(run) => match run.validate() {
                Ok(()) if run.id == id => Ok(Retrieved::Present(run)),
                Ok(()) => Ok(Retrieved::Corrupt("run identity mismatch".into())),
                Err(e) => Ok(Retrieved::Corrupt(e.to_string())),
            },
            Err(e) => Ok(Retrieved::Corrupt(e.to_string())),
        }
    }
    /// Atomic, synced replacement. Captured sources and existing event prefix are
    /// immutable. A stale caller cannot discard observations or rewrite history.
    pub fn save(&self, run: &Run) -> Result<()> {
        run.validate()?;
        let path = self.path(&run.id)?;
        match self.get(&run.id)? {
            Retrieved::Present(old) => {
                ensure!(
                    run.sources == old.sources && run.started_unix_ms == old.started_unix_ms,
                    "captured run identity is immutable"
                );
                ensure!(run.events.starts_with(&old.events), "cannot rewrite events");
                ensure!(!old.truncated || run.truncated, "cannot erase truncation");
                if old.status != Status::Running {
                    ensure!(
                        run.status == old.status && run.events == old.events,
                        "finished run is immutable"
                    );
                }
            }
            Retrieved::Corrupt(_) => bail!("refusing to overwrite corrupt evidence"),
            Retrieved::Missing => (),
        }
        let mut tmp = tempfile::NamedTempFile::new_in(&self.root)?;
        tmp.write_all(&serde_json::to_vec(run)?)?;
        tmp.as_file().sync_all()?;
        tmp.persist(path)?;
        File::open(&self.root)?.sync_all()?;
        Ok(())
    }
    pub fn pin(&self, id: &str, pinned: bool) -> Result<()> {
        let Retrieved::Present(mut r) = self.get(id)? else {
            bail!("run unavailable")
        };
        r.pinned = pinned;
        self.save(&r)
    }
    /// Explicit deletion is allowed even for pinned runs. Retention never deletes pins.
    pub fn delete(&self, id: &str) -> Result<bool> {
        match fs::remove_file(self.path(id)?) {
            Ok(()) => {
                File::open(&self.root)?.sync_all()?;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }
    /// Oldest-first eviction; running, pinned and corrupt evidence is preserved.
    /// Limits apply to unpinned completed records; pins can exceed the budget.
    pub fn retain(&self, policy: Retention, now_unix_ms: u64) -> Result<Vec<String>> {
        let mut runs = vec![];
        for id in self.ids()? {
            if let Retrieved::Present(r) = self.get(&id)?
                && !r.pinned
                && r.status != Status::Running
            {
                let bytes = fs::metadata(self.path(&id)?)?.len();
                runs.push((r.started_unix_ms, id, bytes));
            }
        }
        runs.sort();
        let mut count = runs.len();
        let mut bytes: u64 = runs.iter().map(|r| r.2).sum();
        let mut removed = vec![];
        for (started, id, size) in runs {
            if count > policy.max_unpinned_runs
                || bytes > policy.max_unpinned_bytes
                || now_unix_ms.saturating_sub(started) > policy.max_age_ms
            {
                self.delete(&id)?;
                count -= 1;
                bytes -= size;
                removed.push(id);
            }
        }
        Ok(removed)
    }
}
</storycode:copy>

### Assemble `crates/hick-observe/src/store.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-observe/src/store.rs"><storycode:paste select="#f-323322b54a07-LocalStore"/><storycode:paste select="#f-323322b54a07-Retrieved"/><storycode:paste select="#f-323322b54a07-Retention"/><storycode:paste select="#f-323322b54a07-impl-Default-for-Retention"/><storycode:paste select="#f-323322b54a07-impl-LocalStore"/></storycode:file>

</storycode:doc>
