//! Checked per-path source publication; complete recovery evidence precedes writes.
use super::{
    resource_inputs::{RootIdentity, SavedSource},
    resource_journal::{Batch, Manifest, Receipt},
};
use serde::Serialize;
use std::{collections::{BTreeMap,BTreeSet}, path::Path};
#[derive(Debug, Serialize)]
pub struct Outcome {
    pub complete: bool,
    pub source_files_modified: bool,
    pub source_files_uncertain: bool,
    pub recovery: String,
    pub applied: BTreeSet<String>,
    pub error: Option<String>,
    #[serde(skip)]
    pub observed:Option<BTreeMap<String,ObservedSource>>,
}

/// A producer-held file, not a name that can silently acquire another occupant.
#[derive(Debug)]
pub struct ObservedSource{file:std::fs::File,identity:String}
#[derive(Debug)]
pub(super) struct ObservedWrite{pub changed:bool,pub source:Option<ObservedSource>,pub proven:bool}
impl ObservedSource{
 fn from_file(file:std::fs::File)->Result<Self,String>{let identity=super::source_history::identity_for_metadata(&file.metadata().map_err(|e|e.to_string())?)?;Ok(Self{file,identity})}
 pub(super) fn verify(&self,root:&Path,path:&str,saved:&SavedSource)->Result<(),String>{
  let full=root.join(path);let held=same_file::Handle::from_file(self.file.try_clone().map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
  if same_file::Handle::from_path(&full).map_err(|e|e.to_string())?!=held||super::source_history::identity_for_metadata(&self.file.metadata().map_err(|e|e.to_string())?)?!=self.identity{return Err(format!("Published source identity changed: {path}. No editor guard or history connection was adopted."))}
  let(_,current)=super::resource_inputs::saved_target(root,path)?;if &current!=saved{return Err(format!("Published source bytes or permissions changed: {path}"))}
  if same_file::Handle::from_path(&full).map_err(|e|e.to_string())?!=held||super::source_history::identity_for_metadata(&self.file.metadata().map_err(|e|e.to_string())?)?!=self.identity{return Err(format!("Published source changed during confirmation: {path}"))}Ok(())
 }
}
fn capture_source(root:&Path,path:&str,saved:&SavedSource)->Result<Option<ObservedSource>,String>{
 if matches!(saved,SavedSource::Missing){return Ok(None)}
 let full=root.join(path);let mut options=std::fs::OpenOptions::new();options.read(true);
 #[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;options.custom_flags(libc::O_NOFOLLOW);}
 let held=ObservedSource::from_file(options.open(&full).map_err(|e|e.to_string())?)?;held.verify(root,path,saved)?;Ok(Some(held))
}
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
fn synchronize_parent(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    std::fs::File::open(path.parent().ok_or("Source has no parent directory.")?)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("Source directory synchronization was not confirmed: {error}"))?;
    Ok(())
}
pub(super) fn replace_observed(
    root: &Path,
    path: &str,
    before: &SavedSource,
    after: &SavedSource,
) -> Result<ObservedWrite, String> {
    replace_observed_for_identity(root,path,before,after,false)
}
pub(super) fn replace_observed_for_identity(
    root: &Path,
    path: &str,
    before: &SavedSource,
    after: &SavedSource,
    force_identity:bool,
) -> Result<ObservedWrite, String> {
    let full = check(root, path, before, after)?;
    if before == after && !force_identity {return Ok(ObservedWrite{changed:false,source:capture_source(root,path,after)?,proven:true})}
    if force_identity&&matches!(before,SavedSource::File{readonly:true,..}){return Err(format!("Read-only source identity cannot be replaced: {path}"))}
    let mut published=None;
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
            let file=match before {
                SavedSource::Missing => {
                    temporary.persist_noclobber(&full).map_err(|error| {
                        format!("New source destination was not confirmed: {path}: {error}")
                    })?
                }
                SavedSource::File { .. } => {
                    temporary.persist(&full).map_err(|error| {
                        format!("Source replacement was not confirmed: {path}: {error}")
                    })?
                }
            };
            published=Some(ObservedSource::from_file(file)?);
        }
    }
    synchronize_parent(&full)?;
    let (_, saved) = super::resource_inputs::saved_target(root, path)?;
    if &saved != after {
        return Err(format!(
            "Published source differs from its reviewed state: {path}. Inspect the retained recovery record."
        ));
    }
    if let Some(source)=&published{source.verify(root,path,after)?}
    Ok(ObservedWrite{changed:true,source:published,proven:true})
}
pub(super) fn replace(root:&Path,path:&str,before:&SavedSource,after:&SavedSource)->Result<bool,String>{replace_observed(root,path,before,after).map(|write|write.changed)}
/// Caller holds the mutation gate and owns current editor/room/construction admission.
/// The authority callback receives already published paths so it can distinguish
/// this batch's source changes from independent changes to compiler dependencies.
pub fn publish(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
) -> Result<Outcome, String> {
    publish_observed_guarded_with(root,identity,manifest,authority,|_|Ok(()),|path,before,after,force|replace_observed_for_identity(root,path,before,after,force),false)
}

pub(super) fn publish_with(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    write: impl FnMut(&str, &SavedSource, &SavedSource) -> Result<bool, String>,
) -> Result<Outcome, String> {
    publish_guarded_with(root, identity, manifest, authority, |_| Ok(()), write, false)
}
/// Conflict resolution retains its index lease through both forward publication
/// and rollback. Ordinary resource saves keep their existing partial-save policy.
pub(super) fn publish_guarded_with(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    rollback_authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    mut write: impl FnMut(&str, &SavedSource, &SavedSource) -> Result<bool, String>,
    rollback: bool,
) -> Result<Outcome, String> {
    publish_observed_guarded_with(root,identity,manifest,authority,rollback_authority,|path,before,after,_force|write(path,before,after).map(|changed|ObservedWrite{changed,source:None,proven:false}),rollback)
}
pub(super) fn publish_observed_guarded_with(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    rollback_authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    write: impl FnMut(&str, &SavedSource, &SavedSource,bool) -> Result<ObservedWrite, String>,
    rollback: bool,
) -> Result<Outcome, String> {
    publish_observed_guarded_with_lineage(root,identity,manifest,authority,rollback_authority,write,rollback,
        super::source_history::prepare_resource_lineage)
}

/// An entry inverse prepares its original-history journal while it borrows the
/// retained proof. Release that borrow before any publisher callback can write.
pub(super) fn publish_observed_guarded_with_lineage(
    root: &Path,
    identity: &RootIdentity,
    manifest: &Manifest,
    mut authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    mut rollback_authority: impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    mut write: impl FnMut(&str, &SavedSource, &SavedSource,bool) -> Result<ObservedWrite, String>,
    rollback: bool,
    prepare_lineage: impl FnOnce(&Path, &str, &Manifest, &BTreeMap<String,String>) -> Result<super::source_history::ResourceLineageCapture, String>,
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
    let mut before_sources=BTreeMap::new();for(path,saved)in &manifest.before{if let Some(source)=capture_source(root,path,saved)?{before_sources.insert(path.clone(),source);}}
    let before_identities=before_sources.iter().map(|(path,source)|(path.clone(),source.identity.clone())).collect();
    let batch = Batch::begin(root, manifest)?;
    let lineage=prepare_lineage(root,&batch.id,manifest,&before_identities)?;
    let mut all_proven=true;let mut after_sources=BTreeMap::new();
    let mut outcome = Outcome {
        complete: false,
        source_files_modified: false,
        source_files_uncertain: false,
        recovery: batch.relative_path(),
        applied: BTreeSet::new(),
        error: None,
        observed:None,
    };
    // Install candidate files before deleting displaced originals. Recovery retains
    // every original even for overwrites and swaps; no automatic rollback occurs.
    let mut paths: Vec<_> = manifest.before.keys().collect();
    paths.sort_by_key(|path| (matches!(manifest.after[*path], SavedSource::Missing), *path));
    let mut attempted_paths = Vec::new();
    for (sequence, path) in paths.into_iter().enumerate() {
        let mut attempted = false;
        let result = (|| {
            root_matches(root, identity, &manifest.workspace)?;
            authority(&outcome.applied)?;
            check(root, path, &manifest.before[path], &manifest.after[path])?;
            if let Some(source)=before_sources.get(path){source.verify(root,path,&manifest.before[path])?}
            let force_identity=manifest.reviewed_plan.patches.iter().find(|patch|patch.path==*path).is_some_and(|patch|matches!((&patch.before,&patch.after),(super::resource_changes::SourceState::File{identity:before,..},super::resource_changes::SourceState::File{identity:after,..})if before!=after));
            attempted = manifest.before[path] != manifest.after[path] || force_identity;
            if attempted { attempted_paths.push(path.clone()); }
            let written = write(path, &manifest.before[path], &manifest.after[path],force_identity)?;
            outcome.source_files_modified |= written.changed;all_proven &= written.proven;
            if let Some(source)=written.source{source.verify(root,path,&manifest.after[path])?;if !force_identity&&manifest.before[path]==manifest.after[path]&&before_sources.get(path).is_some_and(|before|before.identity!=source.identity){return Err(format!("Unchanged source was independently replaced: {path}"))}after_sources.insert(path.clone(),source);}
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
                    if rollback { rollback_attempt(root, identity, manifest, &batch, &attempted_paths, &after_sources, all_proven, &mut rollback_authority, &mut write, &mut outcome); }
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
                if rollback { rollback_attempt(root, identity, manifest, &batch, &attempted_paths, &after_sources, all_proven, &mut rollback_authority, &mut write, &mut outcome); }
                return Ok(outcome);
            }
        }
    }
    if all_proven{
        let confirmation=(||->Result<(),String>{root_matches(root,identity,&manifest.workspace)?;authority(&outcome.applied)?;
            for(path,saved)in &manifest.after{match saved{SavedSource::File{..}=>after_sources.get(path).ok_or_else(||format!("Published source has no held identity: {path}"))?.verify(root,path,saved)?,SavedSource::Missing=>{let(_,current)=super::resource_inputs::saved_target(root,path)?;if &current!=saved{return Err(format!("Deleted source name was independently recreated: {path}"))}}}}
            let identities=after_sources.iter().map(|(path,source)|(path.clone(),source.identity.clone())).collect();super::source_history::finish_resource_lineage(root,manifest,lineage,&identities)
        })();if let Err(error)=confirmation{outcome.error=Some(format!("Source publication completed, but its namespace/history confirmation was refused: {error}"));return Ok(outcome)}
        outcome.observed=Some(after_sources);
    }
    outcome.complete = true;
    Ok(outcome)
}
/// Rollback never adopts a foreign edit. The original receipts remain immutable
/// history; this separate report describes what was actually reobserved afterward.
fn rollback_attempt(
    root: &Path, identity: &RootIdentity, manifest: &Manifest, batch: &Batch,
    attempted: &[String], after_sources:&BTreeMap<String,ObservedSource>,proven:bool,authority: &mut impl FnMut(&BTreeSet<String>) -> Result<(), String>,
    write: &mut impl FnMut(&str, &SavedSource, &SavedSource,bool) -> Result<ObservedWrite, String>,
    outcome: &mut Outcome,
) {
    let mut remaining: BTreeSet<String> = attempted.iter().cloned().collect();
    let owned=remaining.clone();let mut restored_sources=BTreeMap::new();
    let mut confirmed = Vec::new();
    let mut errors = Vec::new();
    for path in attempted.iter().rev() {
        let result = (|| {
            root_matches(root, identity, &manifest.workspace)?;
            authority(if proven{&owned}else{&remaining})?;
            if proven{match &manifest.after[path]{SavedSource::File{..}=>after_sources.get(path).ok_or_else(||format!("Rollback has no producer-held identity for uncertain source: {path}"))?.verify(root,path,&manifest.after[path])?,SavedSource::Missing=>{let(_,current)=super::resource_inputs::saved_target(root,path)?;if current!=SavedSource::Missing{return Err(format!("Rollback preserved an independently recreated source: {path}"))}}}}
            let (_, current) = super::resource_inputs::saved_target(root, path)?;
            if current != manifest.before[path] {
                if current != manifest.after[path] {
                    return Err(format!("Rollback preserved an independently changed source: {path}"));
                }
                outcome.source_files_modified = true;
                let restored=write(path, &manifest.after[path], &manifest.before[path],false)?;
                if proven{if !restored.proven{return Err(format!("Rollback returned no producer proof: {path}"))}if matches!(manifest.before[path],SavedSource::File{..}){let source=restored.source.ok_or_else(||format!("Rollback returned no held source: {path}"))?;source.verify(root,path,&manifest.before[path])?;restored_sources.insert(path.clone(),source);}}
            }else if proven{if let Some(source)=after_sources.get(path){source.verify(root,path,&manifest.before[path])?;}}
            root_matches(root, identity, &manifest.workspace)?;
            let (_, restored) = super::resource_inputs::saved_target(root, path)?;
            if restored != manifest.before[path] {
                return Err(format!("Rollback was not confirmed: {path}"));
            }
            Ok::<(), String>(())
        })();
        match result {
            Ok(()) => { remaining.remove(path); confirmed.push(path.clone()); }
            Err(error) => errors.push(error),
        }
    }
    if proven{confirmed.retain(|path|{let result=match &manifest.before[path]{SavedSource::File{..}=>restored_sources.get(path).or_else(||after_sources.get(path)).ok_or_else(||format!("Restored source has no held producer identity: {path}")).and_then(|source|source.verify(root,path,&manifest.before[path])),SavedSource::Missing=>super::resource_inputs::saved_target(root,path).and_then(|(_,saved)|if saved==SavedSource::Missing{Ok(())}else{Err(format!("Restored absent name was independently recreated: {path}"))})};if let Err(error)=result{errors.push(error);false}else{true}})}
    let report = serde_json::json!({"version":1,"confirmed":confirmed,"errors":errors});
    if let Err(error) = batch.record_rollback(&report) {
        errors.push(format!("Rollback report was not retained: {error}"));
    }
    outcome.source_files_uncertain = !errors.is_empty();
    let prior = outcome.error.take().unwrap_or_else(|| "Publication did not complete.".into());
    outcome.error = Some(if errors.is_empty() {
        format!("{prior} This attempt's source writes were observed restored; its save remains incomplete. Inspect the retained rollback report before preparing another review.")
    } else {
        format!("{prior} Rollback is incomplete or uncertain: {}. Inspect retained recovery before further changes.", errors.join("; "))
    });
}
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
                        text: if path == "a.py" && text == "saved a\n" {
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
            saved_guards:BTreeMap::new(),
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
    fn same_byte_replacement_after_writer_retires_namespace_confirmation(){
        let root=root();let identity=RootIdentity::bind(root.path());
        let manifest=fixture(root.path(),&identity,serde_json::json!({"changes":{"hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"new "}]}}));
        let outcome=publish(root.path(),&identity,&manifest,|applied|{if applied.len()==manifest.after.len(){let path=root.path().join("a.py");let bytes=std::fs::read(&path).unwrap();std::fs::remove_file(&path).unwrap();std::fs::write(path,bytes).unwrap()}Ok(())}).unwrap();
        assert!(!outcome.complete);assert!(outcome.observed.is_none());assert!(outcome.error.unwrap().contains("identity changed"));
    }
    #[test]
    fn retained_producer_identity_refuses_equal_text_recreation_before_editor_guard(){
        let root=root();let identity=RootIdentity::bind(root.path());
        let manifest=fixture(root.path(),&identity,serde_json::json!({"changes":{"hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"new "}]}}));
        let outcome=publish(root.path(),&identity,&manifest,|_|Ok(())).unwrap();assert!(outcome.complete,"{:?}",outcome.error);
        let held=&outcome.observed.as_ref().unwrap()["a.py"];held.verify(root.path(),"a.py",&manifest.after["a.py"]).unwrap();let path=root.path().join("a.py");let bytes=std::fs::read(&path).unwrap();std::fs::remove_file(&path).unwrap();std::fs::write(path,bytes).unwrap();assert!(held.verify(root.path(),"a.py",&manifest.after["a.py"]).is_err());
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
    fn guarded_failure_restores_only_its_own_writes_and_retains_rollback_evidence() {
        for foreign in [false, true] {
            let root = root();
            let identity = RootIdentity::bind(root.path());
            let manifest = fixture(root.path(), &identity, serde_json::json!({"changes":{
                "hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"first "}],
                "hick:///b.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"second "}]
            }}));
            let outcome = publish_guarded_with(root.path(), &identity, &manifest,
                |applied| {
                    if applied.contains("a.py") {
                        if foreign { std::fs::write(root.path().join("a.py"), "foreign\n").unwrap(); }
                        return Err("Index authority changed before second write".into());
                    }
                    Ok(())
                }, |_| Ok(()),
                |path, before, after| replace(root.path(), path, before, after), true).unwrap();
            assert!(!outcome.complete && outcome.source_files_modified);
            assert_eq!(outcome.source_files_uncertain, foreign);
            assert_eq!(std::fs::read_to_string(root.path().join("a.py")).unwrap(), if foreign { "foreign\n" } else { "saved a\n" });
            assert_eq!(std::fs::read_to_string(root.path().join("b.py")).unwrap(), "saved b\n");
            let report: serde_json::Value = serde_json::from_slice(&std::fs::read(root.path().join(&outcome.recovery).parent().unwrap().join("rollback.json")).unwrap()).unwrap();
            assert_eq!(report["confirmed"].as_array().unwrap().len(), usize::from(!foreign));
            assert_eq!(report["errors"].as_array().unwrap().len(), usize::from(foreign));
        }
    }
    #[test]
    fn observed_rollback_preserves_same_byte_foreign_recreation() {
        let root=root();let identity=RootIdentity::bind(root.path());
        let manifest=fixture(root.path(),&identity,serde_json::json!({"changes":{
            "hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"first "}],
            "hick:///b.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"second "}]
        }}));
        let outcome=publish_observed_guarded_with(root.path(),&identity,&manifest,|applied|{
            if applied.contains("a.py"){let path=root.path().join("a.py");let bytes=std::fs::read(&path).unwrap();std::fs::remove_file(&path).unwrap();std::fs::write(path,bytes).unwrap();return Err("Stop before second source".into())}Ok(())
        },|_|Ok(()),|path,before,after,force|replace_observed_for_identity(root.path(),path,before,after,force),true).unwrap();
        assert!(!outcome.complete&&outcome.source_files_uncertain);
        assert_eq!(std::fs::read_to_string(root.path().join("a.py")).unwrap(),"first dirty a\n");
        let report:serde_json::Value=serde_json::from_slice(&std::fs::read(root.path().join(outcome.recovery).parent().unwrap().join("rollback.json")).unwrap()).unwrap();
        assert!(report["confirmed"].as_array().unwrap().is_empty());assert!(!report["errors"].as_array().unwrap().is_empty());
    }
    #[test]
    fn equal_byte_logical_swap_rewrites_both_physical_epochs() {
        let root=root();std::fs::write(root.path().join("a.py"),"identical\n").unwrap();std::fs::write(root.path().join("b.py"),"identical\n").unwrap();
        let identity=RootIdentity::bind(root.path());let manifest=fixture(root.path(),&identity,serde_json::json!({"documentChanges":[
            {"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///temporary.py"},
            {"kind":"rename","oldUri":"hick:///b.py","newUri":"hick:///a.py"},
            {"kind":"rename","oldUri":"hick:///temporary.py","newUri":"hick:///b.py"}
        ]}));
        let a=capture_source(root.path(),"a.py",&manifest.before["a.py"]).unwrap().unwrap();let b=capture_source(root.path(),"b.py",&manifest.before["b.py"]).unwrap().unwrap();
        let outcome=publish(root.path(),&identity,&manifest,|_|Ok(())).unwrap();assert!(outcome.complete,"{:?}",outcome.error);assert!(outcome.source_files_modified);
        let after=outcome.observed.unwrap();assert_ne!(a.identity,after["a.py"].identity);assert_ne!(b.identity,after["b.py"].identity);
    }
    #[test]
    fn rollback_authority_failure_never_rewrites_sources() {
        let root = root();
        let identity = RootIdentity::bind(root.path());
        let manifest = fixture(root.path(), &identity, serde_json::json!({"changes":{
            "hick:///a.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"first "}],
            "hick:///b.py":[{"range":{"start":{"line":0,"character":0},"end":{"line":0,"character":0}},"newText":"second "}]
        }}));
        let outcome = publish_guarded_with(root.path(), &identity, &manifest,
            |applied| if applied.is_empty() { Ok(()) } else { Err("Stop publication".into()) },
            |_| Err("Git index lease changed".into()),
            |path, before, after| replace(root.path(), path, before, after), true).unwrap();
        assert!(!outcome.complete && outcome.source_files_uncertain);
        assert!(outcome.error.unwrap().contains("Git index lease changed"));
        assert_eq!(std::fs::read_to_string(root.path().join("a.py")).unwrap(), "first dirty a\n");
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
