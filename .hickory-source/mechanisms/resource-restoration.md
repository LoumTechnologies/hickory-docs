<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Prepare a saved-baseline and dirty-draft restoration

For maintainers connecting resource history to guarded undo without coupling it to a renderer.

Use the checked recovery reader to obtain historical data and independent source acquisition for current authority. This model prepares an inverse; it does not write, expose a recovery endpoint or mutate a client buffer. A later publisher must verify the recovery fingerprint again, analyze both candidate construction layers and retain a restoration-aware record before saving.

Focused evidence (2026-11-15): both model cases pass. A moved dirty draft can
prepare separate old saved and dirty states, with independent fresh acquisition
identities and current capture stamps retained in the inverse. Shared candidate
analysis runs without changing files or the recovery inventory. Independent
drafts, saved metadata, permissions, versions and root scope refuse. Publication,
recovery endpoints and frontend namespace/undo adoption remain unimplemented by
this model.

### Keep saved and draft transitions independent

A restoration retains the original recovery identity and evidence fingerprint, a reviewed inverse draft plan and separate saved before/after states. The saved after text can be older than the restored dirty draft. Normalized acquisition inputs remain internal; they are not serialized as new client authority.

<ui:copy id="resource-restoration-states">
//! Restore saved baselines and dirty drafts as distinct reviewed transitions.
use super::{
    resource_changes::{ResourcePlan, SourceState},
    resource_inputs::{Inputs, SavedSource},
    resource_journal::Recovery,
};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Debug, Serialize)]
pub struct Restoration {
    pub version: u32,
    pub recovery_id: String,
    pub recovery_fingerprint: String,
    pub workspace: String,
    pub context: String,
    pub plan: ResourcePlan,
    pub saved_before: BTreeMap<String, SavedSource>,
    pub saved_after: BTreeMap<String, SavedSource>,
    /// Historical identities label the inverse transition, not fresh IO authority.
    #[serde(skip)]
    pub(crate) inputs: Inputs,
}
</ui:copy>

### Require the current result before proposing its inverse

Compare every saved byte and modeled permission against the recorded result. Require the exact captured source namespace and a fresh matching root, ownership, capabilities and source version. Historical identities label the inverse; rebind only those labels in a copy of the current capture, then use the ordinary complete planner. A historical preparation version never substitutes for a fresh current capture. This pure step performs no IO and grants no permission to publish.

<ui:copy id="resource-restoration-preparation">
/// Pure preparation; callers obtain recovery from the checked reader and repeat
/// current source/ownership acquisition before any later publication.
pub fn prepare(
    recovery: &Recovery,
    current: &Inputs,
    fresh: &Inputs,
) -> Result<Restoration, String> {
    if current.context.is_empty()
        || current.workspace.is_empty()
        || !current.refusals.is_empty()
        || !fresh.refusals.is_empty()
        || current.saved_sources != recovery.manifest.after
        || current.sources.keys().ne(recovery.manifest.after.keys())
    {
        return Err("Current saved sources do not match the retained resource result. Preserve independent edits and review recovery before restoring.".into());
    }
    let mut plan = recovery.manifest.reviewed_plan.inverse();
    plan.context = current.context.clone();
    let mut inputs = current.clone();
    for patch in &plan.patches {
        if let (
            SourceState::File {
                identity: expected, ..
            },
            Some(SourceState::File { identity, .. }),
        ) = (&patch.before, inputs.sources.get_mut(&patch.path))
        {
            *identity = expected.clone();
        }
    }
    super::resource_inputs::validate_candidate(&inputs, fresh, &plan)?;
    // Preserve the current capture's preparation stamps in the inverse record,
    // after proving that its text and capabilities match the historical result.
    for patch in &mut plan.patches {
        patch.before = inputs.sources[&patch.path].clone();
    }
    Ok(Restoration {
        version: 1,
        recovery_id: recovery.id.clone(),
        recovery_fingerprint: recovery.fingerprint.clone(),
        workspace: current.workspace.clone(),
        context: current.context.clone(),
        plan,
        saved_before: current.saved_sources.clone(),
        saved_after: recovery.manifest.before.clone(),
        inputs,
    })
}
</ui:copy>

### Analyze the two restored construction layers

Use the normalized current capture to analyze the inverse draft alongside its
separate saved baseline. This repeats complete freshness checks and uses the
shared compiler model; it does not reinterpret the dirty draft as saved text.
Invalid or unowned construction layers still refuse. The caller must check the
returned protected targets and current root again before a later publisher acts.

<ui:copy id="resource-restoration-consequences">
impl Restoration {
    pub fn analyze(
        &self,
        root: &std::path::Path,
        fresh: &Inputs,
        capture: &super::resource_inputs::Request,
    ) -> Result<super::resource_candidates::Facts, String> {
        super::resource_candidates::analyze_with_saved(
            root,
            &self.inputs,
            fresh,
            capture,
            &self.plan,
            Some(&self.saved_after),
        )
    }
}
</ui:copy>

### Restore the draft without saving it

First save a moved dirty draft in a temporary workspace and read its retained record. Prepare its inverse against independent fresh acquisition identities and current version counters. The desired disk state must be the old saved baseline while the desired editor state is the old unsaved draft. Preparation must leave files and the recovery inventory unchanged. Independent drafts, saved metadata, permissions, versions and root scopes refuse.

<ui:copy id="resource-restoration-tests">
#[cfg(test)]
mod tests {
    use super::super::{
        resource_changes,
        resource_inputs::{Ownership, RootIdentity},
        resource_journal::{Manifest, read_recovery},
        resource_publish,
    };
    use super::*;
    pub(super) fn fixture() -> (tempfile::TempDir, Recovery, Inputs) {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("a.py"), "saved baseline\n").unwrap();
        let identity = RootIdentity::bind(root.path());
        let sources = BTreeMap::from([
            (
                "a.py".into(),
                SourceState::File {
                    identity: "original-a".into(),
                    text: "unsaved draft\n".into(),
                    writable: true,
                    removable: true,
                    version: Some(7),
                },
            ),
            ("b.py".into(), SourceState::Missing { creatable: true }),
        ]);
        let inputs = Inputs {
            version: 1,
            context: "initial-save".into(),
            workspace: identity.check(root.path()).unwrap().into(),
            ownership: Ownership {
                mode: "fixture".into(),
                entry: None,
                protected_entries: vec![],
                saved_sources: BTreeMap::new(),
                draft_sources: BTreeMap::new(),
            },
            sources: sources.clone(),
            saved_sources: ["a.py", "b.py"]
                .into_iter()
                .map(|path| {
                    (
                        path.into(),
                        super::super::resource_inputs::saved_target(root.path(), path)
                            .unwrap()
                            .1,
                    )
                })
                .collect(),
            refusals: BTreeMap::new(),
            source_files_modified: false,
        };
        let plan = resource_changes::prepare(resource_changes::Request { context: inputs.context.clone(), title: "Move draft".into(), edit: serde_json::json!({"documentChanges":[{"kind":"rename","oldUri":"hick:///a.py","newUri":"hick:///b.py"}]}), sources }).unwrap();
        let manifest = Manifest::from_review(&inputs, &plan).unwrap();
        let outcome =
            resource_publish::publish(root.path(), &identity, &manifest, |_| Ok(())).unwrap();
        assert!(outcome.complete);
        let id = std::path::Path::new(&outcome.recovery)
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let recovery = read_recovery(root.path(), id).unwrap();
        let mut current = Inputs {
            version: 1,
            context: "current-undo".into(),
            workspace: inputs.workspace,
            ownership: inputs.ownership,
            sources: plan
                .patches
                .iter()
                .map(|patch| (patch.path.clone(), patch.after.clone()))
                .collect(),
            saved_sources: manifest.after,
            refusals: BTreeMap::new(),
            source_files_modified: false,
        };
        if let SourceState::File {
            identity, version, ..
        } = current.sources.get_mut("b.py").unwrap()
        {
            *identity = "current-buffer-b".into();
            *version = Some(99);
        }
        (root, recovery, current)
    }
    #[test]
    fn preparation_restores_saved_baseline_and_dirty_draft_separately_without_io() {
        let (root, recovery, current) = fixture();
        let mut fresh = current.clone();
        if let SourceState::File { identity, .. } = fresh.sources.get_mut("b.py").unwrap() {
            *identity = "fresh-acquisition-b".into();
        }
        let restoration = prepare(&recovery, &current, &fresh).unwrap();
        let drafts = restoration
            .plan
            .apply(&restoration.context, &restoration.inputs.sources)
            .unwrap();
        let SourceState::File { identity, text, .. } = &drafts["a.py"] else {
            panic!("draft missing")
        };
        assert_eq!(identity, "original-a");
        assert_eq!(text, "unsaved draft\n");
        let SavedSource::File { text: saved, .. } = &restoration.saved_after["a.py"] else {
            panic!("saved baseline missing")
        };
        assert_eq!(saved, "saved baseline\n");
        assert_ne!(text, saved);
        assert!(matches!(
            restoration.saved_after["b.py"],
            SavedSource::Missing
        ));
        assert_eq!(restoration.context, current.context);
        let captured = restoration
            .plan
            .patches
            .iter()
            .find(|patch| patch.path == "b.py")
            .unwrap();
        assert!(matches!(
            captured.before,
            SourceState::File {
                version: Some(99),
                ..
            }
        ));
        assert_eq!(restoration.recovery_fingerprint, recovery.fingerprint);
        assert!(
            serde_json::to_value(&restoration)
                .unwrap()
                .get("inputs")
                .is_none()
        );
        let capture = super::super::resource_inputs::Request {
            context: current.context.clone(),
            edit: serde_json::json!({"changes":{"hick:///a.py":[],"hick:///b.py":[]}}),
            drafts: BTreeMap::new(),
            project_entry: None,
            ownership_sources: BTreeMap::new(),
        };
        let facts = restoration.analyze(root.path(), &fresh, &capture).unwrap();
        assert!(facts.protected_targets.is_empty());
        assert!(!facts.source_files_modified);
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "unsaved draft\n"
        );
        assert_eq!(
            std::fs::read_dir(root.path().join(".hick-cache/resource-changes"))
                .unwrap()
                .count(),
            1
        );
    }
    #[test]
    fn independent_source_saved_metadata_permissions_versions_and_root_scope_refuse() {
        let (root, recovery, current) = fixture();
        let mut changed = current.clone();
        if let SourceState::File { text, .. } = changed.sources.get_mut("b.py").unwrap() {
            *text = "independent draft".into();
        }
        assert!(prepare(&recovery, &changed, &changed).is_err());
        let mut changed = current.clone();
        if let SavedSource::File { readonly, .. } = changed.saved_sources.get_mut("b.py").unwrap() {
            *readonly = true;
        }
        assert!(prepare(&recovery, &changed, &changed).is_err());
        let mut changed = current.clone();
        if let SourceState::File { writable, .. } = changed.sources.get_mut("b.py").unwrap() {
            *writable = false;
        }
        assert!(prepare(&recovery, &changed, &changed).is_err());
        let mut changed = current.clone();
        if let SourceState::File { version, .. } = changed.sources.get_mut("b.py").unwrap() {
            *version = Some(100);
        }
        assert!(prepare(&recovery, &current, &changed).is_err());
        changed = current.clone();
        changed.workspace = "different-root".into();
        assert!(prepare(&recovery, &current, &changed).is_err());
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "unsaved draft\n"
        );
    }
}
</ui:copy>

## Publish saved baselines while retaining dirty draft restoration

Drive a real temporary-root save, restoration and redo through the shared publisher. Each restoration record must retain its parent evidence and both saved/draft transitions. Restoring must write the old saved baseline, while its reviewed editor after state remains the original dirty text; redo returns the saved moved draft. The original record remains unchanged. A version-3 record without valid origin, a version-2 origin override and invalid restored fingerprints must fail before creating another batch. These are publisher/model checks, not frontend or HTTP undo acceptance.

<ui:copy id="resource-restoration-publication-tests">
#[cfg(test)]
mod publication_tests {
    use super::super::{
        resource_inputs::RootIdentity,
        resource_journal::{Batch, Manifest, read_recovery},
        resource_publish,
    };
    use super::*;
    fn read_outcome(root: &std::path::Path, outcome: &resource_publish::Outcome) -> Recovery {
        let id = std::path::Path::new(&outcome.recovery)
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        read_recovery(root, id).unwrap()
    }
    #[test]
    fn publication_restores_saved_baseline_separately_from_dirty_draft_and_redo() {
        let (root, original, mut current) = tests::fixture();
        // Explicit recovery may use a newly bound engine session at the same root.
        let identity = RootIdentity::bind(root.path());
        current.workspace = identity.check(root.path()).unwrap().into();
        let restoration = prepare(&original, &current, &current).unwrap();
        let record = Manifest::from_restoration(&restoration).unwrap();
        assert_eq!(record.version, 3);
        assert_eq!(record.restore_of.as_ref().unwrap().recovery_id, original.id);
        let outcome =
            resource_publish::publish(root.path(), &identity, &record, |_| Ok(())).unwrap();
        assert!(outcome.complete && !outcome.source_files_uncertain);
        assert_eq!(
            std::fs::read_to_string(root.path().join("a.py")).unwrap(),
            "saved baseline\n"
        );
        assert!(!root.path().join("b.py").exists());
        let restored = read_outcome(root.path(), &outcome);
        assert!(restored.all_paths_recorded_applied);
        assert_eq!(restored.manifest.after, restoration.saved_after);
        let SourceState::File { text, .. } = &restored
            .manifest
            .reviewed_plan
            .patches
            .iter()
            .find(|patch| patch.path == "a.py")
            .unwrap()
            .after
        else {
            panic!("restored draft missing")
        };
        assert_eq!(text, "unsaved draft\n");
        assert_eq!(
            restored.manifest.restore_of.as_ref().unwrap().fingerprint,
            original.fingerprint
        );
        let mut redo_current = current.clone();
        redo_current.context = "redo-current".into();
        redo_current.sources = restoration
            .plan
            .patches
            .iter()
            .map(|patch| (patch.path.clone(), patch.after.clone()))
            .collect();
        redo_current.saved_sources = restored.manifest.after.clone();
        if let SourceState::File {
            identity, version, ..
        } = redo_current.sources.get_mut("a.py").unwrap()
        {
            *identity = "current-editor-a".into();
            *version = Some(101);
        }
        let redo = prepare(&restored, &redo_current, &redo_current).unwrap();
        let redo_record = Manifest::from_restoration(&redo).unwrap();
        let redo_outcome =
            resource_publish::publish(root.path(), &identity, &redo_record, |_| Ok(())).unwrap();
        assert!(redo_outcome.complete);
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "unsaved draft\n"
        );
        let redone = read_outcome(root.path(), &redo_outcome);
        assert_eq!(redone.manifest.after, original.manifest.after);
        assert_eq!(
            redone.manifest.restore_of.as_ref().unwrap().recovery_id,
            restored.id
        );
        assert_eq!(
            read_recovery(root.path(), &original.id)
                .unwrap()
                .fingerprint,
            original.fingerprint
        );
        assert_eq!(
            std::fs::read_dir(root.path().join(".hick-cache/resource-changes"))
                .unwrap()
                .count(),
            3
        );
    }
    #[test]
    fn restoration_records_require_versioned_origin_and_valid_saved_states_before_io() {
        let (root, original, current) = tests::fixture();
        let restoration = prepare(&original, &current, &current).unwrap();
        let valid = Manifest::from_restoration(&restoration).unwrap();
        let mut missing = valid.clone();
        missing.restore_of = None;
        assert!(Batch::begin(root.path(), &missing).is_err());
        let mut old = valid.clone();
        old.version = 2;
        assert!(Batch::begin(root.path(), &old).is_err());
        let mut bad = valid.clone();
        bad.restore_of.as_mut().unwrap().recovery_id = "../outside".into();
        assert!(Batch::begin(root.path(), &bad).is_err());
        let mut bad = valid;
        if let SavedSource::File { sha256, .. } = bad.after.get_mut("a.py").unwrap() {
            *sha256 = "invalid".into();
        }
        assert!(Batch::begin(root.path(), &bad).is_err());
        assert_eq!(
            std::fs::read_dir(root.path().join(".hick-cache/resource-changes"))
                .unwrap()
                .count(),
            1
        );
        assert!(!root.path().join("a.py").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("b.py")).unwrap(),
            "unsaved draft\n"
        );
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/resource_restoration.rs"><ui:paste select="#resource-restoration-states"/><ui:paste select="#resource-restoration-preparation"/><ui:paste select="#resource-restoration-consequences"/><ui:paste select="#resource-restoration-tests"/><ui:paste select="#resource-restoration-publication-tests"/></ui:file>
</ui:doc>
