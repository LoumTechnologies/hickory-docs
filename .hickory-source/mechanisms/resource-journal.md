<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain resource save states before publication

For maintainers implementing recoverable namespace saves independently of any editor host.

An explicit resource save first records its complete before/after namespace, including missing files, exact text and modeled permissions. This record preserves overwritten or swapped source even if publication stops partway through. Versioned append-only receipts name observed per-path outcomes; they never authorize replay. Records are bounded and installed without clobbering existing evidence. Existing symlink/non-directory recovery paths refuse. The batch holds its directory identity and checks its recovery path before adding receipts. The publisher must check its held workspace identity around all operations and independently validate source before writing; this store is not a filesystem transaction or automatic rollback mechanism. Files are synchronized before publication; Unix also synchronizes containing directories. Other hosts do not gain a directory-fsync guarantee from this implementation.

Version 2 also retains the exact reviewed plan: its before states contain unsaved
drafts, stable source identities and preparation versions, while the separate
saved before states retain disk bytes and permissions. Applying the inverse can
therefore recover the draft without replacing its older saved baseline. Validate
the complete plan, namespace kinds, final source text and saved SHA-256 values
before creating recovery directories. An incomplete or contradictory record must
fail before publication can begin. Existing version-1 records have no draft
snapshot and cannot be promoted to complete undo records.

This store exposes no resource-save route. A publisher must recheck generated ownership in both candidate saved and draft construction namespaces, including explicit source absence, before writing any candidate destination. Retaining a record does not substitute for that authority check.

<ui:copy id="resource-save-recovery-record">
//! Explicit resource saves retain their before/after states before publishing source.
use super::{
    resource_changes::{ResourcePlan, SourceState},
    resource_inputs::SavedSource,
};
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
    pub reviewed_plan: ResourcePlan,
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
fn validate_saved(source: &SavedSource) -> Result<(), String> {
    if let SavedSource::File { text, sha256, .. } = source {
        if text.len() > 8 * 1024 * 1024
            || sha256 != &super::workspace_fs::view::hash(text.as_bytes())
        {
            return Err(
                "Resource recovery saved source has an invalid content fingerprint.".into(),
            );
        }
    }
    Ok(())
}
fn same_kind(saved: &SavedSource, draft: &SourceState) -> bool {
    matches!(
        (saved, draft),
        (SavedSource::Missing, SourceState::Missing { .. })
            | (SavedSource::File { .. }, SourceState::File { .. })
    )
}
fn same_saved_draft(saved: &SavedSource, draft: &SourceState) -> bool {
    match (saved, draft) {
        (SavedSource::Missing, SourceState::Missing { .. }) => true,
        (SavedSource::File { text: saved, .. }, SourceState::File { text: draft, .. }) => {
            saved == draft
        }
        _ => false,
    }
}
impl Batch {
    /// The caller must check its held workspace identity before and after creating this record.
    pub fn begin(root: &Path, manifest: &Manifest) -> Result<Self, String> {
        if manifest.version != 2
            || manifest.workspace.is_empty()
            || manifest.context.is_empty()
            || manifest.before.is_empty()
            || manifest.before.len() > 10_000
            || manifest.reviewed_plan.context != manifest.context
            || manifest.reviewed_plan.title != manifest.title
            || manifest.before.keys().ne(manifest.after.keys())
        {
            return Err(
                "Resource recovery requires one complete versioned before/after namespace.".into(),
            );
        }
        for path in manifest.before.keys() {
            super::resource_changes::validate_path(path)?;
        }
        // The plan retains the pre-save drafts independently of saved bytes.
        // Reconstruct it to reject duplicate paths or a truncated review record.
        let drafts: BTreeMap<_, _> = manifest
            .reviewed_plan
            .patches
            .iter()
            .map(|patch| (patch.path.clone(), patch.before.clone()))
            .collect();
        if drafts.keys().ne(manifest.before.keys()) {
            return Err("Resource recovery must retain every reviewed source path.".into());
        }
        let after = manifest.reviewed_plan.apply(&manifest.context, &drafts)?;
        for (path, saved) in &manifest.before {
            validate_saved(saved)?;
            validate_saved(&manifest.after[path])?;
            if !same_kind(saved, &drafts[path])
                || !same_saved_draft(&manifest.after[path], &after[path])
            {
                return Err(format!(
                    "Resource recovery differs from the reviewed namespace: {path}. Prepare again."
                ));
            }
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
    pub(super) fn manifest() -> Manifest {
        Manifest {
            version: 2,
            reviewed_plan: ResourcePlan {
                version: 1,
                context: "reviewed-change".into(),
                title: "Move draft".into(),
                annotations: vec![],
                patches: vec![
                    super::super::resource_changes::ResourcePatch {
                        path: "a.py".into(),
                        before: SourceState::File {
                            identity: "original-a".into(),
                            text: "😀 unsaved draft\n".into(),
                            writable: true,
                            removable: true,
                            version: Some(4),
                        },
                        after: SourceState::Missing { creatable: true },
                    },
                    super::super::resource_changes::ResourcePatch {
                        path: "b.py".into(),
                        before: SourceState::Missing { creatable: true },
                        after: SourceState::File {
                            identity: "original-a".into(),
                            text: "after\n".into(),
                            writable: true,
                            removable: true,
                            version: Some(4),
                        },
                    },
                ],
            },
            workspace: "held-root".into(),
            context: "reviewed-change".into(),
            title: "Move draft".into(),
            before: BTreeMap::from([
                (
                    "a.py".into(),
                    SavedSource::File {
                        text: "😀 before\n".into(),
                        sha256: super::super::workspace_fs::view::hash("😀 before\n".as_bytes()),
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
                        sha256: super::super::workspace_fs::view::hash(b"after\n"),
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
        assert_eq!(
            serde_json::to_value(&captured.reviewed_plan).unwrap(),
            serde_json::to_value(&expected.reviewed_plan).unwrap()
        );
        let SourceState::File {
            text,
            identity,
            version,
            ..
        } = &captured.reviewed_plan.patches[0].before
        else {
            panic!("missing draft")
        };
        assert_eq!(text, "😀 unsaved draft\n");
        assert_eq!(identity, "original-a");
        assert_eq!(*version, Some(4));
        let restored = captured
            .reviewed_plan
            .inverse()
            .apply(
                &captured.context,
                &captured
                    .reviewed_plan
                    .patches
                    .iter()
                    .map(|patch| (patch.path.clone(), patch.after.clone()))
                    .collect(),
            )
            .unwrap();
        assert_eq!(&restored["a.py"], &captured.reviewed_plan.patches[0].before);
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

## Reject incomplete reviewed draft recovery records

Corrupt one part at a time: discard a reviewed path, change final saved text, or supply a false saved fingerprint. Every case must refuse before even creating the recovery directory. The retained-state case above separately checks that the original dirty draft, identity and version survive serialization and inverse planning.

<ui:copy id="resource-journal-review-integrity-test">
#[cfg(test)]
#[test]
fn resource_recovery_rejects_truncated_drafts_and_inconsistent_saved_states_before_io() {
    let root = tempfile::tempdir().unwrap();
    let mut missing = tests::manifest();
    missing.reviewed_plan.patches.pop();
    assert!(Batch::begin(root.path(), &missing).is_err());
    let mut wrong_text = tests::manifest();
    if let SavedSource::File { text, sha256, .. } = wrong_text.after.get_mut("b.py").unwrap() {
        *text = "unreviewed".into();
        *sha256 = super::workspace_fs::view::hash(text.as_bytes());
    }
    assert!(Batch::begin(root.path(), &wrong_text).is_err());
    let mut wrong_hash = tests::manifest();
    if let SavedSource::File { sha256, .. } = wrong_hash.before.get_mut("a.py").unwrap() {
        *sha256 = "invented".into();
    }
    assert!(Batch::begin(root.path(), &wrong_hash).is_err());
    assert!(!root.path().join(".hick-cache").exists());
}
</ui:copy>

## Construct saved namespace from reviewed source identities

A moved draft inherits the saved permissions of its source identity, even when it replaces another path. Text comes from the reviewed final draft; saved baselines remain untouched in the before map. New identities start with private file permissions. This pure constructor validates the plan against captured source states and builds a complete record without filesystem access; it does not grant publication authority.

<ui:copy id="resource-journal-from-review">
impl Manifest {
    pub fn from_review(
        inputs: &super::resource_inputs::Inputs,
        plan: &ResourcePlan,
    ) -> Result<Self, String> {
        if inputs.version != 1
            || inputs.context != plan.context
            || inputs.workspace.is_empty()
            || inputs.source_files_modified
            || !inputs.refusals.is_empty()
        {
            return Err(
                "Acquire complete resource inputs before retaining the reviewed change.".into(),
            );
        }
        let final_drafts = plan.apply(&inputs.context, &inputs.sources)?;
        let mut originals = BTreeMap::new();
        for (path, source) in &inputs.sources {
            let saved = inputs
                .saved_sources
                .get(path)
                .ok_or("Saved resource baseline is incomplete.")?;
            if !same_kind(saved, source) {
                return Err("Saved resource namespace differs from its draft capture.".into());
            }
            validate_saved(saved)?;
            if let SourceState::File { identity, .. } = source {
                originals.insert(identity.as_str(), saved);
            }
        }
        let mut before = BTreeMap::new();
        let mut after = BTreeMap::new();
        for patch in &plan.patches {
            before.insert(
                patch.path.clone(),
                inputs.saved_sources[&patch.path].clone(),
            );
            let saved = match &final_drafts[&patch.path] {
                SourceState::Missing { .. } => SavedSource::Missing,
                SourceState::File { identity, text, .. } => {
                    let (readonly, executable, mode) = match originals.get(identity.as_str()) {
                        Some(SavedSource::File {
                            readonly,
                            executable,
                            mode,
                            ..
                        }) => (*readonly, *executable, *mode),
                        Some(SavedSource::Missing) => {
                            return Err("A reviewed source identity has no saved baseline.".into());
                        }
                        None => (false, false, if cfg!(unix) { Some(0o600) } else { None }),
                    };
                    SavedSource::File {
                        text: text.clone(),
                        sha256: super::workspace_fs::view::hash(text.as_bytes()),
                        readonly,
                        executable,
                        mode,
                    }
                }
            };
            after.insert(patch.path.clone(), saved);
        }
        Ok(Self {
            version: 2,
            workspace: inputs.workspace.clone(),
            context: inputs.context.clone(),
            title: plan.title.clone(),
            reviewed_plan: plan.clone(),
            before,
            after,
        })
    }
}

#[cfg(test)]
#[test]
fn reviewed_resource_record_follows_identity_permissions_and_keeps_dirty_baseline() {
    let expected = tests::manifest();
    let inputs = super::resource_inputs::Inputs {
        version: 1,
        context: expected.context.clone(),
        workspace: expected.workspace.clone(),
        ownership: super::resource_inputs::Ownership {
            mode: "test".into(),
            entry: None,
            protected_entries: vec![],
            saved_sources: BTreeMap::new(),
            draft_sources: BTreeMap::new(),
        },
        sources: expected
            .reviewed_plan
            .patches
            .iter()
            .map(|patch| (patch.path.clone(), patch.before.clone()))
            .collect(),
        saved_sources: expected.before.clone(),
        refusals: BTreeMap::new(),
        source_files_modified: false,
    };
    let record = Manifest::from_review(&inputs, &expected.reviewed_plan).unwrap();
    assert_eq!(record.before, expected.before);
    assert_eq!(record.after, expected.after);
    let root = tempfile::tempdir().unwrap();
    Batch::begin(root.path(), &record).unwrap();
    assert!(!root.path().join("a.py").exists());
    assert!(!root.path().join("b.py").exists());
    let mut wrong = inputs;
    wrong.saved_sources.remove("a.py");
    assert!(Manifest::from_review(&wrong, &expected.reviewed_plan).is_err());
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_journal.rs"><ui:paste select="#resource-save-recovery-record"/><ui:paste select="#resource-journal-review-integrity-test"/><ui:paste select="#resource-journal-from-review"/></ui:file>
</ui:doc>
