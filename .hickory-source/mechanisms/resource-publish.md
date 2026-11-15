<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Publish reviewed resource sources with explicit recovery outcomes

For maintainers connecting a namespace save to current workspace authority.

The publication core is separate from proposal planning and recovery storage. A
caller must hold the mutation gate and check live editors, rooms and construction
ownership; the authority callback runs before recovery and before each path. It
receives already applied paths so a caller can account for this batch's own edits.
This core exposes no HTTP route and does not by itself grant permission to save.

Preflight compares exact saved bytes, metadata, filename spelling and ordinary
parent directories. Retain the complete saved and dirty-draft recovery record
before the first source write. Install candidate files before removing originals;
use a fresh synchronized temporary file and no-clobber installation for new paths.
Moved source identities keep their modeled permissions. No directories are
created for source destinations. Reobserve each result and append its receipt.

A refused later path leaves earlier confirmed writes in place and returns their
names plus the recovery record. A failure after an attempted write is explicitly
uncertain, including synchronization or receipt failures. Never claim a complete
save or automatically undo independently changed files. Checks observe concurrent
filesystem state; they are not an operating-system transaction or a lock against
external programs replacing paths between observations.

The focused temporary-root cases exercise dirty-draft overwrite/move, permission
preservation, retained before states, an independent edit between two writes and
stale preflight with no recovery/source side effects. They do not establish a
working frontend resource-save flow; that integration remains outstanding.

### Separate confirmed changes from uncertain attempts

Return complete only after every path is observed and receipted. Confirmed source changes and uncertain attempts are separate facts; a recovery path accompanies every publication that reached its record.

<ui:copy id="resource-source-publication-outcomes">
//! Checked per-path source publication; complete recovery evidence precedes writes.
use super::{
    resource_inputs::{RootIdentity, SavedSource},
    resource_journal::{Batch, Manifest, Receipt},
};
use serde::Serialize;
use std::{collections::BTreeSet, path::Path};
#[derive(Debug, Serialize)]
pub struct Outcome {
    pub complete: bool,
    pub source_files_modified: bool,
    pub source_files_uncertain: bool,
    pub recovery: String,
    pub applied: BTreeSet<String>,
    pub error: Option<String>,
}
</ui:copy>

### Check root and source authority again

Require the held workspace identity, exact saved content and metadata, exact filename spelling and writable ordinary parents. A stale preflight must fail before recovery IO, while a stale later path must preserve already reported outcomes.

<ui:copy id="resource-source-publication-admission">
fn root_matches(root: &Path, identity: &RootIdentity, expected: &str) -> Result<(), String> {
    if identity.check(root)? != expected {
        return Err("Workspace changed. Reconnect before saving resource changes.".into());
    }
    Ok(())
}
fn check(
    root: &Path,
    path: &str,
    expected: &SavedSource,
    desired: &SavedSource,
) -> Result<std::path::PathBuf, String> {
    let (full, saved) = super::resource_inputs::saved_target(root, path)?;
    if &saved != expected {
        return Err(format!(
            "Source changed since review: {path}. Refresh before retrying."
        ));
    }
    if expected != desired {
        if matches!(saved, SavedSource::File { readonly: true, .. }) {
            return Err(format!("Source is read-only: {path}"));
        }
        let parent = full.parent().ok_or("Source has no parent directory.")?;
        let metadata = std::fs::symlink_metadata(parent)
            .map_err(|error| format!("Source parent is unavailable: {path}: {error}"))?;
        if !metadata.is_dir()
            || metadata.file_type().is_symlink()
            || metadata.permissions().readonly()
        {
            return Err(format!(
                "Source parent is not a writable ordinary directory: {path}"
            ));
        }
    }
    Ok(full)
}
</ui:copy>

### Install one reviewed state

Fresh temporary files preserve the reviewed identity’s modeled permissions. New destinations use no-clobber installation. Recheck before replacement and inspect the result after directory synchronization; any failure after an attempt remains uncertain.

<ui:copy id="resource-source-publication-installation">
fn synchronize_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    std::fs::File::open(path.parent().ok_or("Source has no parent directory.")?)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("Source directory synchronization was not confirmed: {error}"))?;
    Ok(())
}
fn replace(
    root: &Path,
    path: &str,
    before: &SavedSource,
    after: &SavedSource,
) -> Result<bool, String> {
    let full = check(root, path, before, after)?;
    if before == after {
        return Ok(false);
    }
    match after {
        SavedSource::Missing => std::fs::remove_file(&full)
            .map_err(|error| format!("Source deletion was not confirmed: {path}: {error}"))?,
        SavedSource::File {
            text,
            readonly,
            mode,
            ..
        } => {
            use std::io::Write;
            let mut temporary = tempfile::NamedTempFile::new_in(full.parent().unwrap())
                .map_err(|error| format!("Cannot prepare source destination: {path}: {error}"))?;
            temporary
                .write_all(text.as_bytes())
                .map_err(|error| error.to_string())?;
            #[cfg(unix)]
            if let Some(mode) = mode {
                use std::os::unix::fs::PermissionsExt;
                temporary
                    .as_file()
                    .set_permissions(std::fs::Permissions::from_mode(*mode))
                    .map_err(|error| error.to_string())?;
            }
            #[cfg(not(unix))]
            {
                let _ = mode;
                let mut permissions = temporary
                    .as_file()
                    .metadata()
                    .map_err(|error| error.to_string())?
                    .permissions();
                permissions.set_readonly(*readonly);
                temporary
                    .as_file()
                    .set_permissions(permissions)
                    .map_err(|error| error.to_string())?;
            }
            #[cfg(unix)]
            let _ = readonly;
            temporary
                .as_file()
                .sync_all()
                .map_err(|error| error.to_string())?;
            check(root, path, before, after)?;
            match before {
                SavedSource::Missing => {
                    temporary.persist_noclobber(&full).map_err(|error| {
                        format!("New source destination was not confirmed: {path}: {error}")
                    })?;
                }
                SavedSource::File { .. } => {
                    temporary.persist(&full).map_err(|error| {
                        format!("Source replacement was not confirmed: {path}: {error}")
                    })?;
                }
            }
        }
    }
    synchronize_parent(&full)?;
    let (_, saved) = super::resource_inputs::saved_target(root, path)?;
    if &saved != after {
        return Err(format!(
            "Published source differs from its reviewed state: {path}. Inspect the retained recovery record."
        ));
    }
    Ok(true)
}
</ui:copy>

### Retain before writing and stop with explicit outcomes

Retain the whole namespace first. Publish files before deletions and recheck caller-owned authority before each step. A failure stops subsequent paths and keeps the manifest; receipt failures do not erase a source write that was already observed.

<ui:copy id="resource-source-publication-publication">
/// Caller holds the mutation gate and owns current editor/room/construction admission.
/// The authority callback receives already published paths so it can distinguish
/// this batch's source changes from independent changes to compiler dependencies.
pub fn publish(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    mut authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
) -> Result<Outcome, String> {
    root_matches(root, identity, &manifest.workspace)?;
    authority(&BTreeSet::new())?;
    for (path, before) in &manifest.before {
        check(
            root,
            path,
            before,
            manifest
                .after
                .get(path)
                .ok_or("Incomplete resource namespace.")?,
        )?;
    }
    let batch = Batch::begin(root, manifest)?;
    let mut outcome = Outcome {
        complete: false,
        source_files_modified: false,
        source_files_uncertain: false,
        recovery: batch.relative_path(),
        applied: BTreeSet::new(),
        error: None,
    };
    // Install candidate files before deleting displaced originals. Recovery retains
    // every original even for overwrites and swaps; no automatic rollback occurs.
    let mut paths: Vec<_> = manifest.before.keys().collect();
    paths.sort_by_key(|path| (matches!(manifest.after[*path], SavedSource::Missing), *path));
    for (sequence, path) in paths.into_iter().enumerate() {
        let mut attempted = false;
        let result = (|| {
            root_matches(root, identity, &manifest.workspace)?;
            authority(&outcome.applied)?;
            check(root, path, &manifest.before[path], &manifest.after[path])?;
            attempted = manifest.before[path] != manifest.after[path];
            let changed = replace(root, path, &manifest.before[path], &manifest.after[path])?;
            outcome.source_files_modified |= changed;
            root_matches(root, identity, &manifest.workspace)?;
            Ok::<_, String>(())
        })();
        match result {
            Ok(()) => {
                outcome.applied.insert(path.clone());
                if let Err(error) = batch.receipt(&Receipt {
                    sequence,
                    path: path.clone(),
                    state: "applied".into(),
                    error: None,
                }) {
                    outcome.error = Some(format!(
                        "Source was observed saved at {path}, but its recovery receipt was not confirmed: {error}"
                    ));
                    return Ok(outcome);
                }
            }
            Err(error) => {
                outcome.source_files_uncertain |= attempted;
                let receipt = batch.receipt(&Receipt {
                    sequence,
                    path: path.clone(),
                    state: if attempted { "uncertain" } else { "refused" }.into(),
                    error: Some(error.clone()),
                });
                outcome.error = Some(match receipt {
                    Ok(()) => error,
                    Err(record_error) => {
                        format!("{error} Recovery receipt was also not confirmed: {record_error}")
                    }
                });
                return Ok(outcome);
            }
        }
    }
    outcome.complete = true;
    Ok(outcome)
}
</ui:copy>

### Exercise overwrite, partial completion and stale preflight

Temporary roots demonstrate a dirty draft moving over another saved file, source permissions following identity, retained original bytes, and a later independent edit surviving partial publication. A stale initial baseline must create neither recovery files nor changes to other source.

<ui:copy id="resource-source-publication-tests">
#[cfg(test)]
mod tests {
    use super::super::{
        resource_changes::{self, SourceState},
        resource_inputs::{Inputs, Ownership},
    };
    use super::*;
    use std::collections::BTreeMap;
    fn fixture(root: &Path, identity: &RootIdentity, edit: serde_json::Value) -> Manifest {
        let saved_sources: BTreeMap<String, SavedSource> = resource_changes::targets(&edit)
            .unwrap()
            .into_iter()
            .map(|path| {
                let saved = super::super::resource_inputs::saved_target(root, &path)
                    .unwrap()
                    .1;
                (path, saved)
            })
            .collect();
        let sources = saved_sources
            .iter()
            .map(|(path, saved)| {
                let state = match saved {
                    SavedSource::Missing => SourceState::Missing { creatable: true },
                    SavedSource::File { text, .. } => SourceState::File {
                        identity: path.clone(),
                        text: if path == "a.py" {
                            "dirty a\n".into()
                        } else {
                            text.clone()
                        },
                        writable: true,
                        removable: true,
                        version: Some(3),
                    },
                };
                (path.clone(), state)
            })
            .collect();
        let inputs = Inputs {
            version: 1,
            context: "review".into(),
            workspace: identity.check(root).unwrap().into(),
            ownership: Ownership {
                mode: "fixture".into(),
                entry: None,
                protected_entries: vec![],
                saved_sources: BTreeMap::new(),
                draft_sources: BTreeMap::new(),
            },
            sources,
            saved_sources,
            refusals: BTreeMap::new(),
            source_files_modified: false,
        };
        let plan = resource_changes::prepare(resource_changes::Request {
            context: inputs.context.clone(),
            title: "Reviewed resources".into(),
            edit,
            sources: inputs.sources.clone(),
        })
        .unwrap();
        Manifest::from_review(&inputs, &plan).unwrap()
    }
    fn root() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.py"), "saved a\n").unwrap();
        std::fs::write(root.path().join("b.py"), "saved b\n").unwrap();
        root
    }
    #[test]
    fn publish_move_saves_reviewed_draft_and_retains_original_saved_namespace() {
        let root = root();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                root.path().join("a.py"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(
            root.path(),
            &identity,
            serde_json::json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py","options":{"overwrite":true}}]}),
        );
        let result = publish(root.path(), &identity, &manifest, |_| Ok(())).unwrap();
        assert!(result.complete && result.source_files_modified);
        assert_eq!(result.applied.len(), 2);
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "dirty a\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(root.path().join("b.py"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o755
            );
        }
        let record: Manifest =
            serde_json::from_slice(&std::fs::read(root.path().join(result.recovery)).unwrap())
                .unwrap();
        assert_eq!(record.before, manifest.before);
        assert_eq!(
            serde_json::to_value(record.reviewed_plan).unwrap(),
            serde_json::to_value(manifest.reviewed_plan).unwrap()
        );
    }
    #[test]
    fn stale_later_path_reports_partial_publication_and_keeps_independent_edit() {
        let root = root();
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(
            root.path(),
            &identity,
            serde_json::json!({"changes":{
                "hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"first "}],
                "hick:///b.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"second "}]
            }}),
        );
        let outcome = publish(root.path(), &identity, &manifest, |applied| {
            if applied.contains("a.py") {
                std::fs::write(root.path().join("b.py"), "independent edit\n").unwrap();
            }
            Ok(())
        })
        .unwrap();
        assert!(!outcome.complete && outcome.source_files_modified);
        assert_eq!(outcome.applied, BTreeSet::from(["a.py".into()]));
        assert!(outcome.error.as_ref().unwrap().contains("b.py"));
        assert_eq!(
            std::fs::read_to_string(root.path().join("a.py")).unwrap(),
            "first dirty a\n"
        );
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "independent edit\n"
        );
        assert!(root.path().join(outcome.recovery).is_file());
    }

    #[test]
    fn creation_and_deletion_keep_exact_recovery_and_private_new_file_permissions() {
        let root = root();
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(
            root.path(),
            &identity,
            serde_json::json!({"documentChanges":[
                {"kind":"create","uri":"hick:///new.py"},
                {"textDocument":{"uri":"hick:///new.py","version":null},"edits":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"new source\n"}]},
                {"kind":"delete","uri":"hick:///a.py"}
            ]}),
        );
        let outcome = publish(root.path(), &identity, &manifest, |_| Ok(())).unwrap();
        assert!(outcome.complete && !outcome.source_files_uncertain);
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("new.py")).unwrap(),
            "new source\n"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(root.path().join("new.py"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let record: Manifest =
            serde_json::from_slice(&std::fs::read(root.path().join(outcome.recovery)).unwrap())
                .unwrap();
        assert_eq!(record.before, manifest.before);
        assert!(matches!(record.before["new.py"], SavedSource::Missing));
    }
    #[test]
    fn a_new_destination_created_after_preflight_is_never_overwritten() {
        let root = root();
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(
            root.path(),
            &identity,
            serde_json::json!({"documentChanges":[{"kind":"create","uri":"hick:///new.py"}]}),
        );
        let mut visits = 0;
        let outcome = publish(root.path(), &identity, &manifest, |_| {
            visits += 1;
            if visits == 2 {
                std::fs::write(root.path().join("new.py"), "independent destination\n").unwrap();
            }
            Ok(())
        })
        .unwrap();
        assert!(
            !outcome.complete && !outcome.source_files_modified && !outcome.source_files_uncertain
        );
        assert!(outcome.applied.is_empty());
        assert_eq!(
            std::fs::read_to_string(root.path().join("new.py")).unwrap(),
            "independent destination\n"
        );
        assert!(root.path().join(outcome.recovery).is_file());
    }

    #[test]
    fn stale_preflight_does_not_create_recovery_or_modify_other_paths() {
        let root = root();
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(
            root.path(),
            &identity,
            serde_json::json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py","options":{"overwrite":true}}]}),
        );
        std::fs::write(root.path().join("a.py"), "independent\n").unwrap();
        assert!(publish(root.path(), &identity, &manifest, |_| Ok(())).is_err());
        assert!(!root.path().join(".hick-cache").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "saved b\n"
        );
    }
}
</ui:copy>


<ui:file path="crates/hickory-cli/src/serve/resource_publish.rs"><ui:paste select="#resource-source-publication-outcomes"/><ui:paste select="#resource-source-publication-admission"/><ui:paste select="#resource-source-publication-installation"/><ui:paste select="#resource-source-publication-publication"/><ui:paste select="#resource-source-publication-tests"/></ui:file>
</ui:doc>
