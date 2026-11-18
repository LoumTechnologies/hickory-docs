<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Retain observed interrupted entry publication without guessing

For engine maintainers handling a room refusal, failed source writer or process restart while a reviewed construction entry Move is in flight.

## Distinguish original, attempted and observed source authority

The original saved namespace keeps its physical source guards and source-parent identities. A durable attempted path is recorded before its filesystem writer. A successful writer contributes an observed checkpoint only after its original held producer verifies exact bytes, modes and identity. A restart can then inspect the mixed original/observed namespace. Saved-byte equality and ordinary journal receipts cannot manufacture a missing checkpoint.

These records are sealed evidence inside the existing private entry binding, never live application grants. Missing optional evidence in older bindings retains the old strict refusal. A no-effect cleanup requires zero attempted or observed paths and unchanged original guards; it consumes a generation instead of reviving old intent tokens.

<ui:copy id="workspace-entry-interrupted-evidence">
mod failures {
    use super::*;
    use crate::serve::{resource_inputs::SavedSource, resource_journal::Manifest};
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::Path,
    };

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct PathProof {
        version: u32,
        saved_sha256: String,
        parent: String,
        guard: Option<String>,
    }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct Evidence {
        version: u32,
        original: BTreeMap<String, PathProof>,
        attempted: BTreeSet<String>,
        observed: BTreeMap<String, PathProof>,
    }
    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    pub(super) struct NoEffect {
        pub request_id: String,
        pub manifest_sha256: String,
        pub before: EntryStamp,
        pub after: EntryStamp,
    }
    fn saved_hash(saved: &SavedSource) -> Result<String, String> {
        Ok(digest(
            &serde_json::to_vec(saved).map_err(|error| error.to_string())?,
        ))
    }
    impl PathProof {
        fn capture(root: &Path, path: &str, expected: &SavedSource) -> Result<Self, String> {
            let (full, current) = crate::serve::resource_inputs::saved_target(root, path)?;
            if &current != expected {
                return Err(format!(
                    "Source changed before retaining interrupted-publication evidence: {path}"
                ));
            }
            let parent = crate::serve::resource_inputs::RootIdentity::fingerprint(
                full.parent().ok_or("Source has no parent")?,
            )?;
            let guard = if matches!(expected, SavedSource::File { .. }) {
                Some(
                    crate::serve::source_history::read_observed(root, &full, path)?
                        .save_guard
                        .ok_or("Source has no original physical guard")?,
                )
            } else {
                None
            };
            let proof = Self {
                version: 1,
                saved_sha256: saved_hash(expected)?,
                parent,
                guard,
            };
            proof.check(root, path, expected)?;
            Ok(proof)
        }
        fn check(&self, root: &Path, path: &str, expected: &SavedSource) -> Result<(), String> {
            if self.version != 1 || self.saved_sha256 != saved_hash(expected)? {
                return Err(
                    "Interrupted source proof differs from its exact saved namespace".into(),
                );
            }
            let (full, current) = crate::serve::resource_inputs::saved_target(root, path)?;
            if &current != expected
                || crate::serve::resource_inputs::RootIdentity::fingerprint(
                    full.parent().ok_or("Source has no parent")?,
                )? != self.parent
            {
                return Err(format!("Interrupted source namespace changed: {path}"));
            }
            match (expected, &self.guard) {
                (SavedSource::File { .. }, Some(guard)) => {
                    crate::serve::source_history::validate_observed_guard(root, &full, path, guard)
                }
                (SavedSource::Missing, None) => Ok(()),
                _ => Err("Interrupted source proof is incomplete".into()),
            }
        }
    }
    impl Evidence {
        pub(super) fn prepare(root: &Path, manifest: &Manifest) -> Result<Self, String> {
            let original = manifest
                .before
                .iter()
                .map(|(path, saved)| Ok((path.clone(), PathProof::capture(root, path, saved)?)))
                .collect::<Result<_, String>>()?;
            Ok(Self {
                version: 1,
                original,
                attempted: BTreeSet::new(),
                observed: BTreeMap::new(),
            })
        }
        pub(super) fn validate_shape(&self) -> Result<(), String> {
            if self.version != 1
                || self.original.len() > 10_000
                || !self
                    .attempted
                    .iter()
                    .all(|path| self.original.contains_key(path))
                || !self
                    .observed
                    .keys()
                    .all(|path| self.attempted.contains(path))
            {
                return Err("Interrupted publication evidence has an invalid namespace".into());
            }
            for (path, proof) in &self.original {
                crate::serve::resource_changes::validate_path(path)?;
                if proof.version != 1 || !crate::serve::resource_saves::hex(&proof.saved_sha256, 64)
                {
                    return Err("Interrupted source evidence is malformed".into());
                }
            }
            Ok(())
        }
        pub(super) fn attempt(&mut self, path: &str) -> Result<(), String> {
            if !self.original.contains_key(path) {
                return Err("Writer is outside the retained interrupted namespace".into());
            }
            self.attempted.insert(path.into());
            Ok(())
        }
        pub(super) fn observe(
            &mut self,
            root: &Path,
            path: &str,
            saved: &SavedSource,
            write: &crate::serve::resource_publish::ObservedWrite,
        ) -> Result<(), String> {
            if !self.attempted.contains(path) || !write.proven {
                return Err("Interrupted checkpoint lacks its original admitted producer".into());
            }
            match (saved, &write.source) {
                (SavedSource::File { .. }, Some(source)) => source.verify(root, path, saved)?,
                (SavedSource::Missing, None) => {}
                _ => return Err("Interrupted checkpoint has no matching held producer".into()),
            }
            let proof = PathProof::capture(root, path, saved)?;
            if let Some(source) = &write.source {
                source.verify(root, path, saved)?;
            }
            self.observed.insert(path.into(), proof);
            Ok(())
        }
        pub(super) fn no_effect(&self, root: &Path, manifest: &Manifest) -> Result<(), String> {
            self.validate_shape()?;
            if !self.attempted.is_empty() || !self.observed.is_empty() {
                return Err(
                    "Attempted writers remain uncertain; no no-effect cleanup was inferred".into(),
                );
            }
            if self.original.keys().ne(manifest.before.keys()) || manifest.after.keys().ne(manifest.before.keys()) {
                return Err("Interrupted original namespace is incomplete".into());
            }
            for (path, proof) in &self.original {
                proof.check(root, path, &manifest.before[path])?;
            }
            Ok(())
        }
        pub(super) fn current(
            &self,
            root: &Path,
            manifest: &Manifest,
        ) -> Result<BTreeMap<String, SavedSource>, String> {
            self.validate_shape()?;
            if self.original.keys().ne(manifest.before.keys()) || manifest.after.keys().ne(manifest.before.keys()) {
                return Err("Interrupted namespace does not match its original manifest".into());
            }
            self.original
                .iter()
                .map(|(path, original)| {
                    let (proof, saved) = match self.observed.get(path) {
                        Some(proof) => (proof, &manifest.after[path]),
                        None => (original, &manifest.before[path]),
                    };
                    proof.check(root, path, saved)?;
                    Ok((path.clone(), saved.clone()))
                })
                .collect()
        }
    }
}
#[cfg(all(test, unix))]
mod interrupted_evidence_tests {
    use super::*;
    use crate::serve::{
        resource_changes::{ResourcePatch, ResourcePlan, SourceState},
        resource_inputs::SavedSource,
        resource_journal::Manifest,
    };
    use failures::Evidence;
    fn fixture() -> (tempfile::TempDir, Manifest) {
        use std::os::unix::fs::PermissionsExt;
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path();
        std::fs::write(root.join("main.md"), b"saved\r\n").unwrap();
        std::fs::set_permissions(root.join("main.md"), std::fs::Permissions::from_mode(0o640))
            .unwrap();
        let source = SourceState::File {
            identity: "original".into(),
            text: "dirty\r\n".into(),
            writable: true,
            removable: true,
            version: Some(7),
        };
        let missing = SourceState::Missing { creatable: true };
        let saved = crate::serve::resource_inputs::saved_target(root, "main.md")
            .unwrap()
            .1;
        let before = BTreeMap::from([
            ("main.md".into(), saved.clone()),
            ("next.spec".into(), SavedSource::Missing),
        ]);
        let after = BTreeMap::from([
            ("main.md".into(), SavedSource::Missing),
            ("next.spec".into(), saved),
        ]);
        let manifest = Manifest {
            version: 2,
            workspace: "fixture".into(),
            context: "original".into(),
            title: "Move".into(),
            restore_of: None,
            request_id: None,
            before,
            after,
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
        (temporary, manifest)
    }
    #[test]
    fn unchanged_namespace_is_provably_no_effect_and_equal_byte_replacement_is_not() {
        let (root, manifest) = fixture();
        let evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        evidence.no_effect(root.path(), &manifest).unwrap();
        let path = root.path().join("main.md");
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::write(&path, bytes).unwrap();
        assert!(evidence.no_effect(root.path(), &manifest).is_err());
    }
    #[test]
    fn unresolved_writer_attempt_cannot_claim_no_effect() {
        let (root, manifest) = fixture();
        let mut evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        evidence.attempt("next.spec").unwrap();
        assert!(evidence.no_effect(root.path(), &manifest).is_err());
        assert!(evidence.attempt("foreign.txt").is_err());
    }
    #[test]
    fn producer_checkpoint_retains_exact_mixed_namespace_across_serialization() {
        let (root, manifest) = fixture();
        let mut evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        evidence.attempt("next.spec").unwrap();
        let written = crate::serve::resource_publish::replace_observed_for_identity(
            root.path(),
            "next.spec",
            &manifest.before["next.spec"],
            &manifest.after["next.spec"],
            false,
        )
        .unwrap();
        evidence
            .observe(
                root.path(),
                "next.spec",
                &manifest.after["next.spec"],
                &written,
            )
            .unwrap();
        let restarted: Evidence =
            serde_json::from_slice(&serde_json::to_vec(&evidence).unwrap()).unwrap();
        let namespace = restarted.current(root.path(), &manifest).unwrap();
        assert_eq!(namespace["main.md"], manifest.before["main.md"]);
        assert_eq!(namespace["next.spec"], manifest.after["next.spec"]);
        assert_eq!(
            std::fs::read(root.path().join("next.spec")).unwrap(),
            b"saved\r\n"
        );
        assert!(restarted.no_effect(root.path(), &manifest).is_err());
    }
    #[test]
    fn writer_to_checkpoint_crash_does_not_guess_progress() {
        let (root, manifest) = fixture();
        let mut evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        evidence.attempt("next.spec").unwrap();
        let _written = crate::serve::resource_publish::replace_observed_for_identity(
            root.path(),
            "next.spec",
            &manifest.before["next.spec"],
            &manifest.after["next.spec"],
            false,
        )
        .unwrap();
        assert!(evidence.current(root.path(), &manifest).is_err());
    }
    #[test]
    fn substituted_producer_never_becomes_checkpoint_authority() {
        let (root, manifest) = fixture();
        let mut evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        evidence.attempt("next.spec").unwrap();
        let written = crate::serve::resource_publish::replace_observed_for_identity(
            root.path(),
            "next.spec",
            &manifest.before["next.spec"],
            &manifest.after["next.spec"],
            false,
        )
        .unwrap();
        let path = root.path().join("next.spec");
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        std::fs::write(path, bytes).unwrap();
        assert!(evidence
            .observe(
                root.path(),
                "next.spec",
                &manifest.after["next.spec"],
                &written
            )
            .is_err());
    }
    #[test]
    fn changed_saved_permissions_or_parent_refuse_recovery() {
        use std::os::unix::fs::PermissionsExt;
        let (root, manifest) = fixture();
        let evidence = Evidence::prepare(root.path(), &manifest).unwrap();
        std::fs::set_permissions(
            root.path().join("main.md"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        assert!(evidence.current(root.path(), &manifest).is_err());
    }
}
</ui:copy>
</ui:doc>
