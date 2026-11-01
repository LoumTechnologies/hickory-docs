<gencode:doc xmlns:gencode="http://www.hickorydocs.com/1.0">
# Bind publication to a captured project

For: maintainers changing safe document opening and explicit generation.

A context stores a canonical root, explicit entry and complete source revision. Private fields prevent a discovered page from minting publication authority.

<gencode:copy id="generation-context-identity">
//! A publication capability bound to one explicit project root and entry.
use hick_lang::{ProjectTree, evaluate_project};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// A captured plan is read-only. Its revision covers every source in the closure.
#[derive(Clone, Debug, Serialize)]
pub struct GenerationPlan {
    pub root: PathBuf,
    pub entry: String,
    pub revision: String,
    pub outputs: Vec<String>,
}

/// Private fields prevent a discovered page from becoming a publication root.
pub struct GenerationContext {
    root: PathBuf,
    entry: String,
    sources: BTreeMap<String, String>,
    tree: ProjectTree,
    revision: String,
}

</gencode:copy>

Capture reads only the declared closure and evaluates it in memory. Missing or ambiguous project declarations fail; there is no standalone fallback.

<gencode:copy id="generation-context-capture">
impl GenerationContext {
    /// Only an explicit root/entry pair can capture a context. Evaluation requires
    /// a declared project; missing/ambiguous declarations never fall back to weave.
    pub fn capture(root: &Path, entry: &str) -> Result<Self, String> {
        hick_lang::validate_project_path(entry)?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        if !root.is_dir() {
            return Err("Generation root must be a directory".into());
        }
        let sources = crate::pending::sources(&root, entry).map_err(|error| {
            format!(
                "Cannot load project entry {entry} under {}: {error}",
                root.display()
            )
        })?;
        let project = evaluate_project(entry, &sources, &ProjectTree::default())?;
        let revision = format!(
            "{:x}",
            Sha256::digest(
                serde_json::to_vec(&(&root, entry, &sources)).map_err(|e| e.to_string())?
            )
        );
        Ok(Self {
            root,
            entry: entry.into(),
            sources,
            tree: project.new,
            revision,
        })
    }

    pub fn plan(&self) -> GenerationPlan {
        GenerationPlan {
            root: self.root.clone(),
            entry: self.entry.clone(),
            revision: self.revision.clone(),
            outputs: self.tree.entries().keys().cloned().collect(),
        }
    }

</gencode:copy>

Publishing is separate from capture. Compare the disclosed revision and re-read sources before handing the complete tree to the existing external-output-edit journal.

<gencode:copy id="generation-context-publish">
    /// Publishing is a separate explicit operation. Refuse stale plans, then use
    /// the output-edit journal so unincorporated external edits are never erased.
    pub fn publish(self, expected_revision: &str) -> Result<Vec<PathBuf>, String> {
        if self.revision != expected_revision {
            return Err("Project sources changed; preview generation again".into());
        }
        if crate::pending::sources(&self.root, &self.entry)? != self.sources {
            return Err("Project sources changed; preview generation again".into());
        }
        crate::pending::materialize(&self.root, &self.entry, &self.sources, &self.tree)
    }
}

</gencode:copy>

The witness proves included output paths stay at the chosen root, capture writes nothing, stale plans fail, unbound pages fail, and ordinary output edits survive.

<gencode:copy id="generation-context-tests">
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn context_revision_binds_root_and_entry_and_ambiguous_declarations_fail() {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        let source = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project /><h:file path=\"app.txt\">same</h:file></h:doc>";
        for root in [first.path(), second.path()] {
            std::fs::write(root.join("project.md"), source).unwrap();
            std::fs::write(root.join("other.md"), source).unwrap();
        }
        let captured = GenerationContext::capture(first.path(), "project.md")
            .unwrap()
            .plan();
        let other_root = GenerationContext::capture(second.path(), "project.md").unwrap();
        assert_ne!(captured.revision, other_root.plan().revision);
        assert!(other_root.publish(&captured.revision).is_err());
        let other_entry = GenerationContext::capture(first.path(), "other.md").unwrap();
        assert_ne!(captured.revision, other_entry.plan().revision);
        assert!(other_entry.publish(&captured.revision).is_err());
        std::fs::write(
            first.path().join("project.md"),
            source.replace("<h:project />", "<h:project /><h:project />"),
        )
        .unwrap();
        assert!(GenerationContext::capture(first.path(), "project.md").is_err());
        assert!(!first.path().join("app.txt").exists());
        assert!(!second.path().join("app.txt").exists());
    }

    #[test]
    fn explicit_context_keeps_included_outputs_at_root_and_refuses_stale_or_unbound_pages() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir(root.join("source")).unwrap();
        let part = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:file path=\"app.txt\">one</h:file></h:doc>";
        std::fs::write(root.join("project.md"), "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\"><h:project sources=\"source/part.md\" /></h:doc>").unwrap();
        std::fs::write(root.join("source/part.md"), part).unwrap();
        let context = GenerationContext::capture(root, "project.md").unwrap();
        let plan = context.plan();
        assert_eq!(plan.outputs, vec!["app.txt"]);
        assert!(!root.join("app.txt").exists());
        assert!(GenerationContext::capture(&root.join("source"), "part.md").is_err());
        assert!(!root.join("source/app.txt").exists());
        std::fs::write(root.join("source/part.md"), part.replace("one", "two")).unwrap();
        assert!(
            context
                .publish(&plan.revision)
                .unwrap_err()
                .contains("changed")
        );
        let context = GenerationContext::capture(root, "project.md").unwrap();
        assert!(
            context
                .publish(&plan.revision)
                .unwrap_err()
                .contains("changed")
        );
        let context = GenerationContext::capture(root, "project.md").unwrap();
        let plan = context.plan();
        context.publish(&plan.revision).unwrap();
        assert_eq!(
            std::fs::read_to_string(root.join("app.txt")).unwrap(),
            "two"
        );
        assert!(!root.join("source/app.txt").exists());
        // An ordinary output edit is retained and blocks subsequent publication.
        std::fs::write(root.join("app.txt"), "external edit").unwrap();
        let context = GenerationContext::capture(root, "project.md").unwrap();
        let plan = context.plan();
        assert!(context.publish(&plan.revision).is_err());
        assert_eq!(
            std::fs::read_to_string(root.join("app.txt")).unwrap(),
            "external edit"
        );
    }
}
</gencode:copy>

<gencode:file path="crates/hick-project-env/src/generation.rs"><gencode:paste select="#generation-context-identity"/><gencode:paste select="#generation-context-capture"/><gencode:paste select="#generation-context-publish"/><gencode:paste select="#generation-context-tests"/></gencode:file>
</gencode:doc>
