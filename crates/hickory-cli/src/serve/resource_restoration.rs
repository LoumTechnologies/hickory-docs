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
#[cfg(test)]
mod tests {
    use super::super::{
        resource_changes,
        resource_inputs::{Ownership, RootIdentity},
        resource_journal::{Manifest, read_recovery},
        resource_publish,
    };
    use super::*;
    fn fixture() -> (tempfile::TempDir, Recovery, Inputs) {
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
