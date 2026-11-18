<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep saved and logical entry recovery namespaces separate

For engine maintainers and simple-host adapters preserving code while a guarded optional recovery capability loads.

The old path is a logical File over saved Missing only when retained_capture carries an actual live editor there. Cold recovery keeps it Missing. A clean editor can retain its identity while the preview offers the original dirty draft. An independently edited live draft keeps its current text. When the old file is absent, the sealed old saved body is used only to detect local edits. It cannot become a physical preimage. Exact saved bytes and modes remain independent.

<ui:copy id="workspace-entry-completion-model">
//! Pure dual-namespace preparation. This builder does not grant IO authority.
use crate::serve::{
    resource_changes::{ResourcePatch, ResourcePlan, SourceState},
    resource_inputs::SavedSource,
    resource_journal::Manifest,
};
use std::collections::{BTreeMap, BTreeSet};
pub(super) const TITLE: &str = "Complete interrupted construction Move";
#[derive(Clone, Debug)]
pub(super) struct Completion {
    pub plan: ResourcePlan,
    pub saved_before: BTreeMap<String, SavedSource>,
    pub saved_after: BTreeMap<String, SavedSource>,
}
fn file_identity(source: &SourceState) -> Option<&str> {
    match source {
        SourceState::File { identity, .. } => Some(identity),
        _ => None,
    }
}
fn source_text(source: &SourceState) -> Option<&str> {
    match source {
        SourceState::File { text, .. } => Some(text),
        _ => None,
    }
}
fn dirty(source: &SourceState, saved: &SavedSource) -> bool {
    match (source, saved) {
        (SourceState::File { text, .. }, SavedSource::File { text: saved, .. }) => text != saved,
        (SourceState::File { .. }, SavedSource::Missing) => true,
        _ => false,
    }
}
/// The caller must separately authenticate the sealed pending identity, verify
/// its producer checkpoints, freeze every retained source owner, and repeat
/// this exact preparation before publication. A Missing saved preimage never
/// receives authority from the logical File representing its orphan editor.
pub(super) fn prepare(
    original: &Manifest,
    old: &str,
    new: &str,
    current_saved: &BTreeMap<String, SavedSource>,
    logical: &BTreeMap<String, SourceState>,
    retained: &BTreeSet<String>,
    context: &str,
) -> Result<Completion, String> {
    if old == new
        || context.is_empty()
        || original.before.len() != 2
        || original.before.keys().ne(original.after.keys())
        || current_saved.keys().ne(original.before.keys())
        || logical.keys().ne(original.before.keys())
        || !retained.iter().all(|path| logical.contains_key(path))
    {
        return Err("Acquire one complete original entry Move namespace before recovery".into());
    }
    crate::serve::resource_changes::validate_path(old)?;
    crate::serve::resource_changes::validate_path(new)?;
    let old_patch = original
        .reviewed_plan
        .patches
        .iter()
        .find(|patch| patch.path == old)
        .ok_or("Interrupted Move lacks its original logical source")?;
    let new_patch = original
        .reviewed_plan
        .patches
        .iter()
        .find(|patch| patch.path == new)
        .ok_or("Interrupted Move lacks its destination")?;
    if !matches!(original.before.get(old), Some(SavedSource::File { .. }))
        || !matches!(original.before.get(new), Some(SavedSource::Missing))
        || !matches!(original.after.get(old), Some(SavedSource::Missing))
        || original.after.get(new) != original.before.get(old)
        || !matches!(old_patch.after, SourceState::Missing { .. })
        || !matches!(new_patch.before, SourceState::Missing { .. })
        || file_identity(&old_patch.before).is_none()
        || file_identity(&old_patch.before) != file_identity(&new_patch.after)
        || source_text(&old_patch.before) != source_text(&new_patch.after)
    {
        return Err("Recovery is confined to the sealed document-preserving entry Move".into());
    }
    let mut identities = BTreeSet::new();
    for (path, source) in logical {
        match (source, &current_saved[path]) {
            (SourceState::File { identity, .. }, SavedSource::File { .. }) => {
                if !identities.insert(identity) {
                    return Err("Captured logical source identities are ambiguous".into());
                }
            }
            (SourceState::File { identity, .. }, SavedSource::Missing)
                if path == old && retained.contains(path) =>
            {
                if !identities.insert(identity) {
                    return Err("Captured orphan source identity is ambiguous".into());
                }
            }
            (SourceState::Missing { .. }, SavedSource::Missing) => {}
            _ => {
                return Err(format!(
                    "A retained logical source has no sealed namespace slot: {path}"
                ))
            }
        }
    }
    if retained.contains(new) && dirty(&logical[new], &current_saved[new]) && retained.contains(old)
    {
        return Err("Two retained construction editors need explicit conflict review; neither dirty draft was discarded".into());
    }
    let selected = if retained.contains(old) && file_identity(&logical[old]).is_some() {
        old
    } else if retained.contains(new) && file_identity(&logical[new]).is_some() {
        new
    } else if file_identity(&logical[old]).is_some() {
        old
    } else {
        new
    };
    let mut destination = logical
        .get(selected)
        .ok_or("Interrupted Move has no surviving logical construction")?
        .clone();
    let SourceState::File { text, writable, .. } = &mut destination else {
        return Err("No retained construction survives in the proven interrupted namespace".into());
    };
    if !*writable {
        return Err(
            "Retained construction is read-only; recovery cannot silently change its permission"
                .into(),
        );
    }
    let draft_baseline = if matches!(current_saved[selected], SavedSource::Missing) {
        &original.before[selected]
    } else {
        &current_saved[selected]
    };
    if !retained.contains(selected) || !dirty(&logical[selected], draft_baseline) {
        // A host may open the surviving saved text before review. That editor
        // still keeps its identity, while a genuinely dirty live draft wins.
        *text = source_text(&new_patch.after)
            .ok_or("Original Move lost its retained dirty source")?
            .into();
    }
    let patches = logical
        .iter()
        .map(|(path, before)| ResourcePatch {
            path: path.clone(),
            before: before.clone(),
            after: if path == new {
                destination.clone()
            } else {
                SourceState::Missing { creatable: true }
            },
        })
        .collect();
    let plan = ResourcePlan {
        version: 1,
        context: context.into(),
        title: TITLE.into(),
        patches,
        annotations: vec![],
    };
    plan.apply(context, logical)?;
    Ok(Completion {
        plan,
        saved_before: current_saved.clone(),
        saved_after: original.after.clone(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn file(identity: &str, text: &str) -> SourceState {
        SourceState::File {
            identity: identity.into(),
            text: text.into(),
            writable: true,
            removable: true,
            version: Some(7),
        }
    }
    fn fixture() -> (Manifest, SavedSource) {
        let saved = SavedSource::File {
            text: "saved\r\n".into(),
            sha256: crate::serve::workspace_fs::view::hash(b"saved\r\n"),
            readonly: false,
            executable: false,
            mode: Some(0o640),
        };
        let source = file("original", "retained dirty\r\n");
        let missing = SourceState::Missing { creatable: true };
        let original = Manifest {
            version: 2,
            request_id: Some("a".repeat(32)),
            workspace: "original-root".into(),
            context: "original".into(),
            title: "Move".into(),
            restore_of: None,
            before: BTreeMap::from([
                ("main.md".into(), saved.clone()),
                ("next.spec".into(), SavedSource::Missing),
            ]),
            after: BTreeMap::from([
                ("main.md".into(), SavedSource::Missing),
                ("next.spec".into(), saved.clone()),
            ]),
            reviewed_plan: ResourcePlan {
                version: 1,
                context: "original".into(),
                title: "Move".into(),
                annotations: vec![],
                patches: vec![
                    ResourcePatch {
                        path: "main.md".into(),
                        before: source.clone(),
                        after: missing.clone(),
                    },
                    ResourcePatch {
                        path: "next.spec".into(),
                        before: missing,
                        after: source,
                    },
                ],
            },
        };
        (original, saved)
    }
    #[test]
    fn copied_destination_keeps_retained_original_editor_and_saved_baseline() {
        let (original, saved) = fixture();
        let current = BTreeMap::from([
            ("main.md".into(), saved.clone()),
            ("next.spec".into(), saved.clone()),
        ]);
        let logical = BTreeMap::from([
            ("main.md".into(), file("original-editor", "later dirty\r\n")),
            ("next.spec".into(), file("copy", "saved\r\n")),
        ]);
        let result = prepare(
            &original,
            "main.md",
            "next.spec",
            &current,
            &logical,
            &BTreeSet::from(["main.md".into()]),
            "fresh",
        )
        .unwrap();
        let after = result.plan.apply("fresh", &logical).unwrap();
        assert_eq!(file_identity(&after["next.spec"]), Some("original-editor"));
        assert_eq!(source_text(&after["next.spec"]), Some("later dirty\r\n"));
        assert_eq!(result.saved_after["next.spec"], saved);
    }
    #[test]
    fn original_orphan_is_logical_file_without_saved_preimage_authority() {
        let (original, saved) = fixture();
        let current = BTreeMap::from([
            ("main.md".into(), SavedSource::Missing),
            ("next.spec".into(), saved),
        ]);
        let logical = BTreeMap::from([
            ("main.md".into(), file("orphan-editor", "later dirty")),
            ("next.spec".into(), file("copy", "saved\r\n")),
        ]);
        let result = prepare(
            &original,
            "main.md",
            "next.spec",
            &current,
            &logical,
            &BTreeSet::from(["main.md".into()]),
            "fresh",
        )
        .unwrap();
        assert_eq!(result.saved_before["main.md"], SavedSource::Missing);
        assert_eq!(
            file_identity(&result.plan.apply("fresh", &logical).unwrap()["next.spec"]),
            Some("orphan-editor")
        );
    }
    #[test]
    fn cold_restart_restores_historical_dirty_text_only_in_reviewed_after_draft() {
        let (original, saved) = fixture();
        let current = BTreeMap::from([
            ("main.md".into(), SavedSource::Missing),
            ("next.spec".into(), saved.clone()),
        ]);
        let logical = BTreeMap::from([
            ("main.md".into(), SourceState::Missing { creatable: true }),
            ("next.spec".into(), file("surviving-copy", "saved\r\n")),
        ]);
        let result = prepare(
            &original,
            "main.md",
            "next.spec",
            &current,
            &logical,
            &BTreeSet::new(),
            "fresh",
        )
        .unwrap();
        assert_eq!(
            source_text(&result.plan.apply("fresh", &logical).unwrap()["next.spec"]),
            Some("retained dirty\r\n")
        );
        assert_eq!(result.saved_after["next.spec"], saved);
    }
    #[test]
    fn clean_original_editor_retains_identity_and_receives_historical_draft() {
        let (original, saved) = fixture();
        for old_saved in [saved.clone(), SavedSource::Missing] {
            let current = BTreeMap::from([
                ("main.md".into(), old_saved.clone()),
                ("next.spec".into(), saved.clone()),
            ]);
            let logical = BTreeMap::from([
                ("main.md".into(), file("original-editor", "saved\r\n")),
                ("next.spec".into(), file("copy", "saved\r\n")),
            ]);
            let result = prepare(
                &original,
                "main.md",
                "next.spec",
                &current,
                &logical,
                &BTreeSet::from(["main.md".into(), "next.spec".into()]),
                "fresh",
            )
            .unwrap();
            let after = result.plan.apply("fresh", &logical).unwrap();
            assert_eq!(file_identity(&after["next.spec"]), Some("original-editor"));
            assert_eq!(source_text(&after["next.spec"]), Some("retained dirty\r\n"));
            assert_eq!(result.saved_before["main.md"], old_saved);
            assert_eq!(result.saved_after["next.spec"], saved);
        }
    }
    #[test]
    fn independent_dirty_peer_never_gets_silently_merged_or_removed() {
        let (original, saved) = fixture();
        let current = BTreeMap::from([
            ("main.md".into(), saved.clone()),
            ("next.spec".into(), saved),
        ]);
        let logical = BTreeMap::from([
            ("main.md".into(), file("original-editor", "first dirty")),
            ("next.spec".into(), file("peer-editor", "independent dirty")),
        ]);
        assert!(prepare(
            &original,
            "main.md",
            "next.spec",
            &current,
            &logical,
            &BTreeSet::from(["main.md".into(), "next.spec".into()]),
            "fresh"
        )
        .unwrap_err()
        .contains("conflict review"));
    }
    #[test]
    fn unknown_orphan_scope_and_invented_identity_refuse() {
        let (original, saved) = fixture();
        let current = BTreeMap::from([
            ("main.md".into(), saved),
            ("next.spec".into(), SavedSource::Missing),
        ]);
        let logical = BTreeMap::from([
            ("main.md".into(), file("one", "saved\r\n")),
            ("next.spec".into(), file("foreign", "unproved")),
        ]);
        assert!(prepare(
            &original,
            "main.md",
            "next.spec",
            &current,
            &logical,
            &BTreeSet::from(["next.spec".into()]),
            "fresh"
        )
        .is_err());
        assert!(prepare(
            &original,
            "main.md",
            "../escape",
            &current,
            &logical,
            &BTreeSet::new(),
            "fresh"
        )
        .is_err());
    }
}
</ui:copy>

<ui:file path="crates/hickory-cli/src/serve/workspace_entry_completion.rs"><ui:paste select="#workspace-entry-completion-model"/></ui:file>
</ui:doc>
