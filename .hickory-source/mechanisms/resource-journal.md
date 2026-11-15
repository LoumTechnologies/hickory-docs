<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain resource save states before publication

For maintainers implementing recoverable namespace saves independently of any editor host.

An explicit resource save first records its complete before/after namespace, including missing files, exact text and modeled permissions. This record preserves overwritten or swapped source even if publication stops partway through. Versioned append-only receipts name observed per-path outcomes; they never authorize replay. Records are bounded and installed without clobbering existing evidence. Existing symlink/non-directory recovery paths refuse. The batch holds its directory identity and checks its recovery path before adding receipts. The publisher must check its held workspace identity around all operations and independently validate source before writing; this store is not a filesystem transaction or automatic rollback mechanism. Files are synchronized before publication; Unix also synchronizes containing directories. Other hosts do not gain a directory-fsync guarantee from this implementation.

This store exposes no resource-save route. A publisher must recheck generated ownership in both candidate saved and draft construction namespaces, including explicit source absence, before writing any candidate destination. Retaining a record does not substitute for that authority check.

<ui:copy id="resource-save-recovery-record">
//! Explicit resource saves retain their before/after states before publishing source.
use super::resource_inputs::SavedSource;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub workspace: String,
    pub context: String,
    pub title: String,
    pub before: BTreeMap<String, SavedSource>,
    pub after: BTreeMap<String, SavedSource>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub sequence: usize,
    pub path: String,
    pub state: String,
    pub error: Option<String>,
}
pub struct Batch {
    directory: PathBuf,
    handle: same_file::Handle,
    paths: std::collections::BTreeSet<String>,
    pub id: String,
}
fn regular_directory(path: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(path){
  Ok(meta) if meta.is_dir()&&!meta.file_type().is_symlink()=>Ok(()),
  Ok(_)=>Err("Resource recovery directory is not an ordinary folder. Restore it before saving resource changes.".into()),
  Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{
   match std::fs::create_dir(path){Ok(())=>Ok(()),Err(error) if error.kind()==std::io::ErrorKind::AlreadyExists=>regular_directory(path),Err(error)=>Err(format!("Cannot create resource recovery folder: {error}. No source files were changed."))}
  }
  Err(error)=>Err(format!("Cannot inspect resource recovery folder: {error}. No source files were changed.")),
 }
}
fn sync_directory(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        std::fs::File::open(path)
            .and_then(|file| file.sync_all())
            .map_err(|error| format!("Cannot synchronize resource recovery folder: {error}"))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn write_new(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    let mut temporary = tempfile::NamedTempFile::new_in(directory)
        .map_err(|error| format!("Cannot prepare resource recovery record: {error}"))?;
    temporary
        .write_all(bytes)
        .and_then(|_| temporary.as_file().sync_all())
        .map_err(|error| format!("Cannot synchronize resource recovery record: {error}"))?;
    temporary
        .persist_noclobber(directory.join(name))
        .map_err(|error| format!("Cannot publish resource recovery record: {error}"))?;
    sync_directory(directory)
}
impl Batch {
    /// The caller must check its held workspace identity before and after creating this record.
    pub fn begin(root: &Path, manifest: &Manifest) -> Result<Self, String> {
        if manifest.version != 1
            || manifest.workspace.is_empty()
            || manifest.context.is_empty()
            || manifest.before.len() > 10_000
            || manifest.before.keys().ne(manifest.after.keys())
        {
            return Err(
                "Resource recovery requires one complete versioned before/after namespace.".into(),
            );
        }
        for path in manifest.before.keys() {
            super::resource_changes::validate_path(path)?;
        }
        let bytes = serde_json::to_vec(manifest).map_err(|error| error.to_string())?;
        if bytes.len() > 40 * 1024 * 1024 {
            return Err(
                "Resource recovery record exceeds 40 MiB. Narrow the change before saving.".into(),
            );
        }
        let mut directory = root.to_path_buf();
        for part in [".hick-cache", "resource-changes"] {
            directory.push(part);
            regular_directory(&directory)?;
        }
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce)
            .map_err(|error| format!("Cannot allocate resource recovery identity: {error}"))?;
        let id = hex::encode(nonce);
        directory.push(&id);
        std::fs::create_dir(&directory)
            .map_err(|error| format!("Cannot create a new resource recovery batch: {error}"))?;
        write_new(&directory, "manifest.json", &bytes)?;
        sync_directory(directory.parent().unwrap())?;
        let handle = same_file::Handle::from_path(&directory).map_err(|error| error.to_string())?;
        Ok(Self {
            directory,
            id,
            handle,
            paths: manifest.before.keys().cloned().collect(),
        })
    }
    /// A receipt is append-only evidence, never authority to replay a filesystem write.
    pub fn receipt(&self, receipt: &Receipt) -> Result<(), String> {
        if receipt.sequence >= 10_000
            || !matches!(receipt.state.as_str(), "applied" | "refused" | "uncertain")
        {
            return Err("Invalid resource recovery receipt.".into());
        }
        super::resource_changes::validate_path(&receipt.path)?;
        if !self.paths.contains(&receipt.path) {
            return Err("Recovery receipt names an uncaptured source path.".into());
        }
        for directory in [
            &self.directory,
            self.directory.parent().unwrap(),
            self.directory.parent().unwrap().parent().unwrap(),
        ] {
            let meta = std::fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
            if !meta.is_dir() || meta.file_type().is_symlink() {
                return Err("Resource recovery folder changed. Stop saving and inspect the retained manifest.".into());
            }
        }
        if same_file::Handle::from_path(&self.directory).map_err(|error| error.to_string())?
            != self.handle
        {
            return Err("Resource recovery batch identity changed. Stop saving and inspect the retained manifest.".into());
        }
        let bytes = serde_json::to_vec(receipt).map_err(|error| error.to_string())?;
        if bytes.len() > 64 * 1024 {
            return Err("Resource recovery receipt is too large.".into());
        }
        let meta = std::fs::symlink_metadata(&self.directory).map_err(|error| error.to_string())?;
        if !meta.is_dir() || meta.file_type().is_symlink() {
            return Err(
                "Resource recovery folder changed. Stop saving and inspect the retained manifest."
                    .into(),
            );
        }
        write_new(
            &self.directory,
            &format!("{:05}.json", receipt.sequence),
            &bytes,
        )
    }
    pub fn relative_path(&self) -> String {
        format!(".hick-cache/resource-changes/{}/manifest.json", self.id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn manifest() -> Manifest {
        Manifest {
            version: 1,
            workspace: "held-root".into(),
            context: "reviewed-change".into(),
            title: "Move draft".into(),
            before: BTreeMap::from([
                (
                    "a.py".into(),
                    SavedSource::File {
                        text: "😀 before\n".into(),
                        sha256: "before".into(),
                        readonly: false,
                        executable: true,
                        mode: Some(0o755),
                    },
                ),
                ("b.py".into(), SavedSource::Missing),
            ]),
            after: BTreeMap::from([
                ("a.py".into(), SavedSource::Missing),
                (
                    "b.py".into(),
                    SavedSource::File {
                        text: "after\n".into(),
                        sha256: "after".into(),
                        readonly: false,
                        executable: true,
                        mode: Some(0o755),
                    },
                ),
            ]),
        }
    }
    #[test]
    fn exact_recovery_states_exist_before_any_source_write_and_receipts_never_clobber() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("a.py"), "saved\n").unwrap();
        let expected = manifest();
        let batch = Batch::begin(temp.path(), &expected).unwrap();
        let captured: Manifest = serde_json::from_slice(
            &std::fs::read(temp.path().join(batch.relative_path())).unwrap(),
        )
        .unwrap();
        assert_eq!(captured.before, expected.before);
        assert_eq!(captured.after, expected.after);
        let receipt = Receipt {
            sequence: 0,
            path: "b.py".into(),
            state: "applied".into(),
            error: None,
        };
        batch.receipt(&receipt).unwrap();
        assert!(batch.receipt(&receipt).is_err());
        let first = std::fs::read(batch.directory.join("00000.json")).unwrap();
        assert_eq!(
            serde_json::from_slice::<Receipt>(&first).unwrap().path,
            "b.py"
        );
        assert_eq!(std::fs::read(temp.path().join("a.py")).unwrap(), b"saved\n");
        assert!(!temp.path().join("b.py").exists());
        let second = Batch::begin(temp.path(), &expected).unwrap();
        assert_ne!(batch.id, second.id);
    }
    #[test]
    fn incomplete_namespaces_and_non_directory_recovery_paths_refuse_without_source_writes() {
        let temp = tempfile::tempdir().unwrap();
        let mut incomplete = manifest();
        incomplete.after.remove("b.py");
        assert!(Batch::begin(temp.path(), &incomplete).is_err());
        assert!(!temp.path().join(".hick-cache").exists());
        std::fs::write(temp.path().join(".hick-cache"), "ordinary source").unwrap();
        assert!(Batch::begin(temp.path(), &manifest()).is_err());
        assert_eq!(
            std::fs::read(temp.path().join(".hick-cache")).unwrap(),
            b"ordinary source"
        );
    }
    #[cfg(unix)]
    #[test]
    fn recovery_never_follows_a_preexisting_symlink() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), temp.path().join(".hick-cache")).unwrap();
        assert!(Batch::begin(temp.path(), &manifest()).is_err());
        assert!(std::fs::read_dir(outside.path()).unwrap().next().is_none());
    }
    #[cfg(unix)]
    #[test]
    fn a_replaced_batch_directory_cannot_receive_receipts() {
        let temp = tempfile::tempdir().unwrap();
        let batch = Batch::begin(temp.path(), &manifest()).unwrap();
        let retained = temp.path().join("retained-batch");
        std::fs::rename(&batch.directory, &retained).unwrap();
        std::fs::create_dir(&batch.directory).unwrap();
        assert!(
            batch
                .receipt(&Receipt {
                    sequence: 0,
                    path: "a.py".into(),
                    state: "applied".into(),
                    error: None
                })
                .is_err()
        );
        assert!(
            std::fs::read_dir(&batch.directory)
                .unwrap()
                .next()
                .is_none()
        );
        assert!(retained.join("manifest.json").is_file());
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_journal.rs"><ui:paste select="#resource-save-recovery-record"/></ui:file>
</ui:doc>
