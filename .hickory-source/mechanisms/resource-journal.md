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

When an engine publication has a request identity, retain its canonical lowercase
identity in the manifest and use it as the batch address. A disconnected client
can inspect that exact address after restart through the existing recovery reader.
Creating the directory still uses no-clobber semantics: the same identity cannot
publish again over its old evidence. Older records without request identities
keep randomly allocated addresses. A missing record does not prove that a request
never started; inspection is historical evidence and never automatic replay.

Focused verification (2026-11-15): the resource-specific CLI selection executed
47 passing cases, including exact request-addressed reopening, duplicate batch
refusal and invalid identity refusal before recovery IO. The rebuilt actual-engine
resource service witness passed original-request lookup across process restart
with unchanged saved files and retained evidence fingerprints. These checks do
not recover a request that stopped before its manifest was retained, and never
infer a successful save from a historical receipt.

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_of: Option<RestorationOrigin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    pub before: BTreeMap<String, SavedSource>,
    pub after: BTreeMap<String, SavedSource>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestorationOrigin {
    pub recovery_id: String,
    pub fingerprint: String,
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
pub(super) fn regular_directory(path: &Path) -> Result<(), String> {
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
fn manifest_bytes(manifest: &Manifest) -> Result<Vec<u8>, String> {
    if manifest.request_id.as_ref().is_some_and(|id| {
        id.len() != 32 || !id.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }) {
        return Err("Recovery request identity must be 32 lowercase hexadecimal characters.".into());
    }
    if !matches!(manifest.version, 2 | 3)
        || (manifest.version == 2 && manifest.restore_of.is_some())
        || (manifest.version == 3 && manifest.restore_of.is_none())
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
    if let Some(origin) = &manifest.restore_of {
        if origin.recovery_id.len() != 32
            || !origin
                .recovery_id
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
            || origin.fingerprint.len() != 64
            || !origin
                .fingerprint
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
        {
            return Err("A restoration record requires its exact recovery identity and evidence fingerprint.".into());
        }
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
            || if manifest.restore_of.is_some() {
                !same_kind(&manifest.after[path], &after[path])
            } else {
                !same_saved_draft(&manifest.after[path], &after[path])
            }
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
    Ok(bytes)
}
impl Batch {
    /// The caller must check its held workspace identity before and after creating this record.
    pub fn begin(root: &Path, manifest: &Manifest) -> Result<Self, String> {
        let bytes = manifest_bytes(manifest)?;
        let mut directory = root.to_path_buf();
        for part in [".hick-cache", "resource-changes"] {
            directory.push(part);
            regular_directory(&directory)?;
        }
        let id = if let Some(id) = &manifest.request_id {
            id.clone()
        } else {
            let mut nonce = [0u8; 16];
            getrandom::fill(&mut nonce)
                .map_err(|error| format!("Cannot allocate resource recovery identity: {error}"))?;
            hex::encode(nonce)
        };
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
    /// Ordinary saves append one bounded observed outcome to this same held batch.
    /// The extra file is deliberately not a generic resource restoration receipt.
    pub(super) fn append_source_evidence(&self,bytes:&[u8])->Result<(),String>{
        self.append_evidence("source-save.json",bytes)
    }
    /// A rollback report is separate immutable evidence; an applied receipt is
    /// never rewritten to claim the inverse was also confirmed.
    pub(super) fn record_rollback(&self,value:&impl Serialize)->Result<(),String>{
        self.append_evidence("rollback.json",&serde_json::to_vec(value).map_err(|e|e.to_string())?)
    }
    pub(super) fn append_resource_lineage(&self,bytes:&[u8])->Result<(),String>{
        self.append_evidence_bounded("source-lineage.json",bytes,2*1024*1024)
    }
    fn append_evidence(&self,name:&str,bytes:&[u8])->Result<(),String>{self.append_evidence_bounded(name,bytes,64*1024)}
    fn append_evidence_bounded(&self,name:&str,bytes:&[u8],limit:usize)->Result<(),String>{
        if bytes.len()>limit{return Err("Publication evidence exceeds its bounded record size.".into())}
        for path in [&self.directory,self.directory.parent().unwrap(),self.directory.parent().unwrap().parent().unwrap()]{
            let metadata=std::fs::symlink_metadata(path).map_err(|e|e.to_string())?;
            if !metadata.is_dir()||metadata.file_type().is_symlink(){return Err("Source history folder changed before its outcome was recorded.".into())}
        }
        if same_file::Handle::from_path(&self.directory).map_err(|e|e.to_string())?!=self.handle{return Err("Source history batch identity changed.".into())}
        write_new(&self.directory,name,bytes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn manifest() -> Manifest {
        Manifest {
            version: 2,
            request_id: None,
            restore_of: None,
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
    fn publication_request_addresses_survive_restart_without_replay() {
        let root = tempfile::tempdir().unwrap();
        let id = "0123456789abcdef0123456789abcdef";
        let mut expected = manifest();
        expected.request_id = Some(id.into());
        let batch = Batch::begin(root.path(), &expected).unwrap();
        assert_eq!(batch.id, id);
        drop(batch);
        // Reopening evidence needs no process-held attempt table or batch handle.
        let retained = read_recovery(root.path(), id).unwrap();
        assert_eq!(retained.manifest.request_id.as_deref(), Some(id));
        assert!(!retained.all_paths_recorded_applied);
        assert!(retained.receipts.is_empty());
        assert!(Batch::begin(root.path(), &expected).is_err());
        assert_eq!(read_recovery(root.path(), id).unwrap().fingerprint, retained.fingerprint);
        assert!(!root.path().join("a.py").exists());
        for invalid in ["../outside", "ABCDEF0123456789abcdef0123456789", ""] {
            let fresh = tempfile::tempdir().unwrap();
            expected.request_id = Some(invalid.into());
            assert!(Batch::begin(fresh.path(), &expected).is_err());
            assert!(!fresh.path().join(".hick-cache").exists());
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
            request_id: None,
            workspace: inputs.workspace.clone(),
            context: inputs.context.clone(),
            title: plan.title.clone(),
            reviewed_plan: plan.clone(),
            restore_of: None,
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
        saved_guards:BTreeMap::new(),
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

Focused evidence (2026-11-15): all nine recovery-store cases pass after sharing
the pure manifest validator with the reader. The three reader cases cover fresh,
partial and fully receipted records, independent current source remaining intact,
unknown files, changed evidence fingerprints, invalid paths/receipts, oversized
receipts and Unix symlink refusal. This is internal read-only evidence access;
no recovery route, UI restore command or saved/draft undo is established.

## Read retained recovery evidence without replay

A recovery inspection reads one exact batch identity, validates the same complete manifest used by publication, and retains the separate saved and draft states. Receipt sequences, paths and outcomes are checked; unknown files are reported without being read as receipts. Listing, file and cumulative receipt sizes are bounded. Manifest and overall evidence fingerprints distinguish a changed observation. Hold and recheck directory identities around the read; Unix final-file opens also reject symlinks. Reading missing records never creates cache directories. All-paths-recorded-applied describes receipts only, not current saved files or authority to replay. Callers must still check their held workspace identity around this internal reader. No recovery HTTP route or restore command is introduced here.

<ui:copy id="resource-recovery-reading">
#[derive(Debug, Serialize)]
pub struct Recovery {
    pub id: String,
    pub manifest_sha256: String,
    pub fingerprint: String,
    pub manifest: Manifest,
    pub receipts: Vec<Receipt>,
    pub unrecognized_files: Vec<String>,
    /// Literal recorded outcomes, not current filesystem state or replay authority.
    pub all_paths_recorded_applied: bool,
}
pub(super) fn read_regular(path: &Path, limit: usize) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Recovery record is unavailable: {error}"))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > limit as u64 {
        return Err("Recovery record is not a bounded ordinary file. Inspect the retained files before restoring.".into());
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("Cannot open recovery record: {error}"))?;
    if !file
        .metadata()
        .map_err(|error| error.to_string())?
        .is_file()
    {
        return Err("Recovery record changed file kind.".into());
    }
    let held = same_file::Handle::from_file(file.try_clone().map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let current = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if bytes.len() > limit
        || !current.is_file()
        || current.file_type().is_symlink()
        || same_file::Handle::from_path(path).map_err(|error| error.to_string())? != held
    {
        return Err(
            "Recovery record changed while reading. Inspect it again before restoring.".into(),
        );
    }
    Ok(bytes)
}
/// Read evidence only. The caller must check its held workspace identity around this read.
pub fn read_recovery(root: &Path, id: &str) -> Result<Recovery, String> {
    if id.len() != 32
        || !id
            .bytes()
            .all(|byte| matches!(byte,b'0'..=b'9'|b'a'..=b'f'))
    {
        return Err("Use the exact resource recovery identity from the saved outcome.".into());
    }
    let cache = root.join(".hick-cache");
    let changes = cache.join("resource-changes");
    let directory = changes.join(id);
    let mut held = Vec::new();
    for path in [&cache, &changes, &directory] {
        let metadata = std::fs::symlink_metadata(path)
            .map_err(|error| format!("Recovery directory is unavailable: {error}"))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("Recovery path is not an ordinary directory.".into());
        }
        held.push(same_file::Handle::from_path(path).map_err(|error| error.to_string())?);
    }
    let bytes = read_regular(&directory.join("manifest.json"), 40 * 1024 * 1024)?;
    let manifest_sha256 = super::workspace_fs::view::hash(&bytes);
    let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "Recovery manifest is incomplete, invalid or lacks its reviewed draft snapshot: {error}"
        )
    })?;
    drop(bytes);
    // Reuse publication's complete versioned record validation without any IO.
    let _ = manifest_bytes(&manifest)?;
    let mut names = Vec::new();
    let mut name_bytes = 0usize;
    for entry in std::fs::read_dir(&directory).map_err(|error| error.to_string())? {
        let name = entry
            .map_err(|error| error.to_string())?
            .file_name()
            .into_string()
            .map_err(|_| "Recovery directory contains an unrepresentable filename.")?;
        name_bytes = name_bytes.saturating_add(name.len());
        if names.len() >= 20_000 || name_bytes > 1024 * 1024 {
            return Err("Recovery directory exceeds its interactive listing budget.".into());
        }
        names.push(name);
    }
    names.sort();
    let mut receipts = Vec::new();
    let mut unrecognized_files = Vec::new();
    let mut total = 0usize;
    let mut paths = std::collections::BTreeSet::new();
    for name in names {
        if name == "manifest.json" {
            continue;
        }
        if name.len() != 10
            || !name.ends_with(".json")
            || !name.as_bytes()[..5].iter().all(u8::is_ascii_digit)
        {
            unrecognized_files.push(name);
            continue;
        }
        let bytes = read_regular(&directory.join(&name), 64 * 1024)?;
        total = total.saturating_add(bytes.len());
        if total > 16 * 1024 * 1024 {
            return Err("Recovery receipts exceed their interactive byte budget. Inspect retained records directly.".into());
        }
        let receipt: Receipt = serde_json::from_slice(&bytes)
            .map_err(|error| format!("Recovery receipt is invalid: {error}"))?;
        if receipt.sequence
            != name[..5]
                .parse::<usize>()
                .map_err(|error| error.to_string())?
            || receipt.sequence >= manifest.before.len()
            || !manifest.before.contains_key(&receipt.path)
            || !paths.insert(receipt.path.clone())
            || !matches!(receipt.state.as_str(), "applied" | "refused" | "uncertain")
        {
            return Err(
                "Recovery receipts do not describe one complete captured source namespace.".into(),
            );
        }
        receipts.push(receipt);
    }
    for (path, original) in [&cache, &changes, &directory].into_iter().zip(held) {
        let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || same_file::Handle::from_path(path).map_err(|error| error.to_string())? != original
        {
            return Err(
                "Recovery directory changed during reading. Inspect it again before restoring."
                    .into(),
            );
        }
    }
    let all_paths_recorded_applied = paths.len() == manifest.before.len()
        && receipts
            .iter()
            .enumerate()
            .all(|(sequence, receipt)| receipt.sequence == sequence && receipt.state == "applied");
    let fingerprint = super::workspace_fs::view::hash(
        &serde_json::to_vec(&(&manifest_sha256, &receipts, &unrecognized_files))
            .map_err(|error| error.to_string())?,
    );
    Ok(Recovery {
        id: id.into(),
        manifest_sha256,
        fingerprint,
        manifest,
        receipts,
        unrecognized_files,
        all_paths_recorded_applied,
    })
}
</ui:copy>

## Check reading, incomplete evidence and unsafe cache paths

Inspect a fresh, partially receipted and fully receipted record while leaving independent source bytes unchanged. Compare fingerprints as evidence changes and preserve the original draft transition. Missing caches must remain absent; uncaptured paths, oversized receipts, manifest symlinks and symlinked batch folders refuse without creating source files.

<ui:copy id="resource-recovery-reading-tests">
#[cfg(test)]
mod recovery_read_tests {
    use super::*;
    #[test]
    fn read_retains_drafts_and_literal_receipts_without_replaying_source() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.py"), "independent current source\n").unwrap();
        let batch = Batch::begin(root.path(), &tests::manifest()).unwrap();
        let initial = read_recovery(root.path(), &batch.id).unwrap();
        assert!(!initial.all_paths_recorded_applied && initial.receipts.is_empty());
        batch
            .receipt(&Receipt {
                sequence: 0,
                path: "b.py".into(),
                state: "applied".into(),
                error: None,
            })
            .unwrap();
        let partial = read_recovery(root.path(), &batch.id).unwrap();
        assert!(!partial.all_paths_recorded_applied);
        assert_ne!(initial.fingerprint, partial.fingerprint);
        assert_eq!(initial.manifest_sha256, partial.manifest_sha256);
        batch
            .receipt(&Receipt {
                sequence: 1,
                path: "a.py".into(),
                state: "applied".into(),
                error: None,
            })
            .unwrap();
        std::fs::write(batch.directory.join("inspection-note.txt"), "a note").unwrap();
        let recorded = read_recovery(root.path(), &batch.id).unwrap();
        assert!(recorded.all_paths_recorded_applied);
        assert_eq!(recorded.unrecognized_files, ["inspection-note.txt"]);
        assert_eq!(
            serde_json::to_value(&recorded.manifest.reviewed_plan).unwrap(),
            serde_json::to_value(&initial.manifest.reviewed_plan).unwrap()
        );
        // Recorded outcomes are not a claim about independently changed disk.
        assert_eq!(
            std::fs::read_to_string(root.path().join("a.py")).unwrap(),
            "independent current source\n"
        );
        assert!(!root.path().join("b.py").exists());
    }
    #[test]
    fn read_refuses_unknown_paths_and_inconsistent_or_oversized_receipts_without_creating_folders()
    {
        let root = tempfile::tempdir().unwrap();
        assert!(read_recovery(root.path(), "../outside").is_err());
        assert!(read_recovery(root.path(), &"a".repeat(32)).is_err());
        assert!(!root.path().join(".hick-cache").exists());
        let batch = Batch::begin(root.path(), &tests::manifest()).unwrap();
        std::fs::write(
            batch.directory.join("00000.json"),
            serde_json::to_vec(&Receipt {
                sequence: 0,
                path: "uncaptured.py".into(),
                state: "applied".into(),
                error: None,
            })
            .unwrap(),
        )
        .unwrap();
        assert!(read_recovery(root.path(), &batch.id).is_err());
        std::fs::write(
            batch.directory.join("00000.json"),
            vec![b' '; 64 * 1024 + 1],
        )
        .unwrap();
        assert!(read_recovery(root.path(), &batch.id).is_err());
        assert!(!root.path().join("a.py").exists() && !root.path().join("b.py").exists());
    }
    #[cfg(unix)]
    #[test]
    fn read_never_accepts_a_manifest_symlink_or_symlinked_batch_directory() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let batch = Batch::begin(root.path(), &tests::manifest()).unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::rename(
            batch.directory.join("manifest.json"),
            outside.path().join("manifest.json"),
        )
        .unwrap();
        symlink(
            outside.path().join("manifest.json"),
            batch.directory.join("manifest.json"),
        )
        .unwrap();
        assert!(read_recovery(root.path(), &batch.id).is_err());
        let other_id = "e".repeat(32);
        symlink(
            outside.path(),
            batch.directory.parent().unwrap().join(&other_id),
        )
        .unwrap();
        assert!(read_recovery(root.path(), &other_id).is_err());
    }
}
</ui:copy>

Focused evidence (2026-11-15): four restoration model/publication cases and all
nine existing recovery-store cases pass with mixed version-2/version-3 records.
Temporary-root restore and redo retain separate saved/draft state and parent
fingerprints; invalid origins, versions and saved hashes refuse before another
batch is created. This proves the shared record/publisher path, not a recovery
HTTP action or frontend undo flow.

## Retain both sides of a saved/draft restoration

An ordinary version-2 save records a saved after state equal to its reviewed draft. A version-3 restoration instead retains the exact historical saved baseline independently of the restored dirty draft, and names its parent recovery observation. File presence, hashes, complete paths and the reviewed transition remain validated. The parent ID and fingerprint are evidence references, never permission to replay: a publisher must recheck the parent and current source authority before writing. Version-2 records without the optional field remain readable. This constructor creates only a record value; it does not publish source.

<ui:copy id="resource-restoration-record">
impl Manifest {
    pub fn from_restoration(
        restoration: &super::resource_restoration::Restoration,
    ) -> Result<Self, String> {
        if restoration.version != 1 {
            return Err("Unsupported saved/draft restoration version.".into());
        }
        let manifest = Self {
            version: 3,
            request_id: None,
            workspace: restoration.workspace.clone(),
            context: restoration.context.clone(),
            title: restoration.plan.title.clone(),
            reviewed_plan: restoration.plan.clone(),
            restore_of: Some(RestorationOrigin {
                recovery_id: restoration.recovery_id.clone(),
                fingerprint: restoration.recovery_fingerprint.clone(),
            }),
            before: restoration.saved_before.clone(),
            after: restoration.saved_after.clone(),
        };
        let _ = manifest_bytes(&manifest)?;
        Ok(manifest)
    }
}
</ui:copy>

## Discover retained identities without replay

A recovery reading must be reachable after the originating HTTP response or engine session is lost. List bounded ordinary batch directories without opening their source snapshots. A directory identity is a discovery hint, not validated evidence or permission to restore; selecting it still runs the complete record reader. Cursor pages sort exact ASCII identities and never imply chronological order. Missing history is empty; symlink parents refuse. Hold and recheck existing parents around enumeration, and refuse an oversized scan rather than silently choosing a filesystem-dependent subset.

<ui:copy id="resource-recovery-discovery">
#[derive(Serialize)]
pub struct RecoveryPage {
    pub ids: Vec<String>,
    pub next: Option<String>,
    pub ignored: usize,
}
pub fn list_recoveries(root: &Path, after: Option<&str>) -> Result<RecoveryPage, String> {
    let valid = |id: &str| {
        id.len() == 32
            && id
                .bytes()
                .all(|byte| matches!(byte,b'0'..=b'9'|b'a'..=b'f'))
    };
    if after.is_some_and(|id| !valid(id)) {
        return Err("Recovery cursor must be an exact retained identity.".into());
    }
    let cache = root.join(".hick-cache");
    let changes = cache.join("resource-changes");
    let mut held = Vec::new();
    for path in [&cache, &changes] {
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RecoveryPage {
                    ids: Vec::new(),
                    next: None,
                    ignored: 0,
                });
            }
            Err(error) => return Err(format!("Recovery directory is unavailable: {error}")),
        };
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("Recovery path is not an ordinary directory.".into());
        }
        held.push(same_file::Handle::from_path(path).map_err(|error| error.to_string())?);
    }
    let mut ids = Vec::new();
    let mut ignored = 0;
    for (index, entry) in std::fs::read_dir(&changes)
        .map_err(|error| error.to_string())?
        .enumerate()
    {
        if index >= 20_000 {
            return Err("Recovery history exceeds its interactive discovery budget. Inspect retained history on disk.".into());
        }
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            ignored += 1;
            continue;
        };
        if !valid(name) || !kind.is_dir() || kind.is_symlink() {
            ignored += 1;
            continue;
        }
        if after.is_none_or(|cursor| name > cursor) {
            ids.push(name.to_string());
        }
    }
    for (path, expected) in [&cache, &changes].into_iter().zip(held) {
        let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || same_file::Handle::from_path(path).map_err(|error| error.to_string())? != expected
        {
            return Err(
                "Recovery history changed while listing. Refresh before selecting a record.".into(),
            );
        }
    }
    ids.sort();
    let more = ids.len() > 100;
    ids.truncate(100);
    let next = if more { ids.last().cloned() } else { None };
    Ok(RecoveryPage { ids, next, ignored })
}
#[cfg(test)]
mod discovery_tests {
    use super::*;
    #[test]
    fn missing_history_is_read_only_and_pages_are_exact_sorted_identities() {
        let root = tempfile::tempdir().unwrap();
        assert!(list_recoveries(root.path(), None).unwrap().ids.is_empty());
        assert!(!root.path().join(".hick-cache").exists());
        let changes = root.path().join(".hick-cache/resource-changes");
        std::fs::create_dir_all(&changes).unwrap();
        for n in (0..102).rev() {
            std::fs::create_dir(changes.join(format!("{n:032x}"))).unwrap();
        }
        std::fs::write(changes.join("not-a-record"), "keep").unwrap();
        let first = list_recoveries(root.path(), None).unwrap();
        assert_eq!(first.ids.len(), 100);
        assert_eq!(first.ids[0], format!("{:032x}", 0));
        assert_eq!(first.ignored, 1);
        let second = list_recoveries(root.path(), first.next.as_deref()).unwrap();
        assert_eq!(
            second.ids,
            vec![format!("{:032x}", 100), format!("{:032x}", 101)]
        );
        assert!(second.next.is_none());
        assert!(list_recoveries(root.path(), Some("../outside")).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn discovery_refuses_symlink_parents_and_ignores_symlink_batches() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join(".hick-cache")).unwrap();
        assert!(list_recoveries(root.path(), None).is_err());
        std::fs::remove_file(root.path().join(".hick-cache")).unwrap();
        let changes = root.path().join(".hick-cache/resource-changes");
        std::fs::create_dir_all(&changes).unwrap();
        std::os::unix::fs::symlink(outside.path(), changes.join("a".repeat(32))).unwrap();
        let page = list_recoveries(root.path(), None).unwrap();
        assert!(page.ids.is_empty());
        assert_eq!(page.ignored, 1);
        assert!(std::fs::read_dir(outside.path()).unwrap().next().is_none());
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_journal.rs"><ui:paste select="#resource-save-recovery-record"/><ui:paste select="#resource-journal-review-integrity-test"/><ui:paste select="#resource-journal-from-review"/><ui:paste select="#resource-recovery-reading"/><ui:paste select="#resource-recovery-reading-tests"/><ui:paste select="#resource-restoration-record"/><ui:paste select="#resource-recovery-discovery"/></ui:file>
</ui:doc>
