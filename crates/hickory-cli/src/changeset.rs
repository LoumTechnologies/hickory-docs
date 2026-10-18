//! A frozen input and a repairable, byte-exact final artifact. No writes or execution.
use anyhow::{Context, Result, bail, ensure};
use hick_lang::{ProjectInventory, ProjectSnapshot, ProjectTree};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

pub const CAPABILITY: &str = "hickorydocs.com/changesets";
pub const METHOD: &str = "_hickorydocs.com/changeset";
pub const INSTRUCTIONS: &str = "Hickory changeset contract v1: call changeset_status to obtain the frozen starting project, current output fingerprint and a response.md template. Submit a concise literate explanation of the entire successful A-to-B change, without abandoned attempts, through submit_changeset. Pass its boundary and expected fingerprint plus entry and sources (a map of .md paths to complete document text). Repair diagnostics and resubmit until accepted. Any later project writes require resubmission. Acceptance checks source-output equality only, not runtime behavior, explanatory accuracy, authorship or approval. Do not claim every native write was reviewed. Workspace/editor status context identifies reserved editor entries: reproduce them in memory, never save them as paths. Use the returned entry name for the final document.";

#[derive(Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub entry: String,
    pub sources: BTreeMap<String, String>,
    pub expected: String,
}
#[derive(Clone)]
pub struct Contract {
    pub root: PathBuf,
    pub inventory: ProjectInventory,
    pub starting: ProjectTree,
    pub boundary: String,
    pub accepted: Option<Artifact>,
    pub workspace: Option<(
        crate::workspace_capture::WorkspaceCapture,
        std::sync::Arc<crate::serve::acp::edits::Edits>,
    )>,
}

pub fn fingerprint(tree: &ProjectTree) -> String {
    // Provenance is evidence about bytes, not part of code equality.
    let entries: BTreeMap<_, _> = tree
        .entries()
        .iter()
        .map(|(p, e)| (p, (&e.bytes, e.kind, e.executable)))
        .collect();
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&entries).expect("project serializes"))
    )
}
pub fn response_entry(starting: &ProjectTree, current: &ProjectTree) -> String {
    let mut entry = "response.md".to_owned();
    while starting.get(&entry).is_some() || current.get(&entry).is_some() {
        entry.insert(0, '_');
    }
    entry
}
impl Contract {
    pub fn new(
        root: PathBuf,
        inventory: ProjectInventory,
        starting: ProjectTree,
        boundary: String,
    ) -> Self {
        Self {
            root,
            inventory,
            starting,
            boundary,
            accepted: None,
            workspace: None,
        }
    }
    pub fn capture(doc: &Path, boundary: String) -> Result<Self> {
        let doc = doc.canonicalize()?;
        let root = doc
            .parent()
            .context("Project needs a directory")?
            .to_path_buf();
        let entry = doc
            .file_name()
            .and_then(|p| p.to_str())
            .context("Project needs a UTF-8 name")?;
        let sources = hick_lang::load_project_sources(entry, &|name| {
            let file = root.join(name);
            if !std::fs::symlink_metadata(&file)
                .map_err(|e| e.to_string())?
                .is_file()
                || !file
                    .canonicalize()
                    .map_err(|e| e.to_string())?
                    .starts_with(&root)
            {
                return Err(format!("Unconfined Markdown source: {name}"));
            }
            std::fs::read_to_string(file).map_err(|e| e.to_string())
        })
        .map_err(anyhow::Error::msg)?;
        let project = hick_lang::evaluate_project(entry, &sources, &ProjectTree::default())
            .map_err(anyhow::Error::msg)?;
        let starting = crate::project_revision::directory(&root, Some(&project.inventory))?;
        Ok(Self::new(root, project.inventory, starting, boundary))
    }
    pub fn current(&self) -> Result<ProjectTree> {
        if let Some((scope, edits)) = &self.workspace {
            return scope.current(&edits.buffers.lock().unwrap());
        }
        crate::project_revision::directory(&self.root, Some(&self.inventory))
    }
    pub fn status(&mut self) -> Result<Value> {
        let current = self.current()?;
        let expected = fingerprint(&current);
        if self
            .accepted
            .as_ref()
            .is_some_and(|a| a.expected != expected)
        {
            self.accepted = None;
        }
        let entry = response_entry(&self.starting, &current);
        let template = hick_lang::transition_response(
            &self.starting,
            &current,
            "Explain the successful change here.",
        );
        Ok(
            json!({"version":1,"boundary":self.boundary,"expected":expected,
            "context":self.workspace.as_ref().map(|(scope,_)| json!({"kind":"workspace-editors","capture":scope,"editorNamespace":hick_lang::EDITOR_BUFFER_PREFIX,"exclusions":self.inventory.exclude})),
            "startingFingerprint":fingerprint(&self.starting),"differences":self.starting.compare(&current),
            "accepted":self.accepted.is_some(),"starting":ProjectSnapshot::capture(&self.starting),
            "current":ProjectSnapshot::capture(&current),"entry":entry,
            "template":template.as_ref().ok(),"templateError":template.as_ref().err(),
            "evidence":"source-output-equality-only","writeControl":"not-enforced"}),
        )
    }
    pub fn submit(&mut self, args: &Value) -> Result<Value> {
        ensure!(
            args["boundary"].as_str() == Some(&self.boundary),
            "Wrong or expired changeset boundary; call changeset_status."
        );
        // A repair in this boundary replaces acceptance. A stale turn cannot revoke it.
        self.accepted = None;
        let current = self.current()?;
        let expected = fingerprint(&current);
        ensure!(
            args["expected"].as_str() == Some(&expected),
            "Project changed since the status read; call changeset_status and regenerate the artifact."
        );
        let entry = args["entry"]
            .as_str()
            .context("Pass entry, the .md entry path")?;
        let sources: BTreeMap<String, String> = serde_json::from_value(args["sources"].clone())
            .context("Pass sources as a map of .md paths to complete document text")?;
        for name in sources.keys() {
            hick_lang::validate_project_path(name).map_err(anyhow::Error::msg)?;
            ensure!(
                name.ends_with(".md"),
                "Changeset source must be a .md document: {name}"
            );
        }
        let differences = verify_sources(&self.starting, &current, entry, &sources)?;
        if !differences.is_empty() {
            return Ok(
                json!({"accepted":false,"code":"output-mismatch","differences":differences,
                "repair":"Replay the complete changeset over starting, include every current output and no abandoned outputs, then resubmit.","expected":expected}),
            );
        }
        ensure!(
            fingerprint(&self.current()?) == expected,
            "Project changed during verification; read status and resubmit."
        );
        self.accepted = Some(Artifact {
            entry: entry.into(),
            sources,
            expected: expected.clone(),
        });
        Ok(
            json!({"accepted":true,"boundary":self.boundary,"expected":expected,"evidence":"source-output-equality-only","writeControl":"not-enforced"}),
        )
    }
    pub fn completion(&mut self) -> Result<Artifact> {
        let expected = fingerprint(&self.current()?);
        if self
            .accepted
            .as_ref()
            .is_some_and(|a| a.expected != expected)
        {
            self.accepted = None;
        }
        let Some(artifact) = &self.accepted else {
            bail!(
                "No current accepted changeset. Call changeset_status, repair and submit_changeset before completing."
            );
        };
        Ok(artifact.clone())
    }
    pub fn call(&mut self, name: &str, args: &Value) -> Result<Value> {
        match name {
            "changeset_status" => {
                let mut value = self.status()?;
                if args["include_snapshots"] != true {
                    let object = value.as_object_mut().expect("status object");
                    object.remove("starting");
                    object.remove("current");
                }
                Ok(value)
            }
            "submit_changeset" => self.submit(args),
            _ => bail!("Unknown changeset operation"),
        }
    }
}

/// The same exact replay comparison is used at submission and attribution finalization.
pub fn verify_sources(
    starting: &ProjectTree,
    completed: &ProjectTree,
    entry: &str,
    sources: &BTreeMap<String, String>,
) -> Result<Vec<hick_lang::ProjectDifference>> {
    let project =
        hick_lang::evaluate_project(entry, sources, starting).map_err(anyhow::Error::msg)?;
    ensure!(
        sources
            .keys()
            .all(|path| project.inventory.documents.contains(path)),
        "The artifact contains unused Markdown sources; remove them or declare them in the project closure."
    );
    Ok(project.new.compare(completed))
}

pub fn catalogue(begin: bool) -> Vec<Value> {
    let mut tools = vec![
        json!({"name":"changeset_status","description":"Read the frozen A, actual current B, fingerprint and literate final-response template. No execution or writes; acceptance is source equality, not authorship or runtime verification.","inputSchema":{"type":"object","properties":{"include_snapshots":{"type":"boolean","description":"Include full A/B byte snapshots. Default false avoids duplicating unchanged code in model context."}}}}),
        json!({"name":"submit_changeset","description":"Verify a final literate .md changeset over the frozen starting tree against the complete current output tree. Repair diagnostics and resubmit. Later writes invalidate acceptance.","inputSchema":{"type":"object","properties":{"boundary":{"type":"string"},"expected":{"type":"string"},"entry":{"type":"string"},"sources":{"type":"object","additionalProperties":{"type":"string"}}},"required":["boundary","expected","entry","sources"]}}),
    ];
    if begin {
        tools.insert(0,json!({"name":"begin_changeset","description":"Freeze a declared project (doc) or explicit open workspace directory (workspace) BEFORE making changes. One boundary per MCP server; an existing boundary cannot be reset. Use changeset_status and submit_changeset to deliver the final artifact.","inputSchema":{"type":"object","properties":{"doc":{"type":"string"},"workspace":{"type":"string"},"include_snapshots":{"type":"boolean"}}}}));
    }
    tools
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir, Contract) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("project.md"), "<hick:project/>").unwrap();
        std::fs::write(dir.path().join("app.ts"), "dirty A\n").unwrap();
        let contract = Contract::capture(&dir.path().join("project.md"), "turn-1".into()).unwrap();
        (dir, contract)
    }
    fn submission(contract: &mut Contract) -> Value {
        let status = contract.status().unwrap();
        json!({"boundary":status["boundary"],"expected":status["expected"],"entry":"response.md","sources":{"response.md":status["template"]}})
    }
    // Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
    #[test]
    fn failed_repair_complete_inventory_and_later_write_invalidation() {
        let (dir, mut contract) = setup();
        std::fs::write(dir.path().join("app.ts"), "B\n").unwrap();
        std::fs::write(dir.path().join("new.ts"), "new\n").unwrap();
        let good = submission(&mut contract);
        let mut bad = good.clone();
        bad["sources"]["response.md"] = json!("<hick:project/>");
        let refused = contract.submit(&bad).unwrap();
        assert_eq!(refused["accepted"], false);
        assert!(
            refused["differences"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["path"] == "new.ts")
        );
        assert!(contract.completion().is_err());
        assert_eq!(contract.submit(&good).unwrap()["accepted"], true);
        assert!(contract.completion().is_ok());
        std::fs::write(dir.path().join("app.ts"), "C\n").unwrap();
        assert!(contract.completion().is_err());
        assert!(
            contract
                .submit(&good)
                .unwrap_err()
                .to_string()
                .contains("changed")
        );
        let repaired = submission(&mut contract);
        assert_eq!(contract.submit(&repaired).unwrap()["accepted"], true);
    }
    #[test]
    fn boundaries_sources_and_metadata_are_checked_without_writes() {
        let (_dir, mut contract) = setup();
        assert!(
            contract
                .call("changeset_status", &json!({}))
                .unwrap()
                .get("starting")
                .is_none()
        );
        assert!(
            contract
                .call("changeset_status", &json!({"include_snapshots":true}))
                .unwrap()
                .get("starting")
                .is_some()
        );
        let good = submission(&mut contract);
        let mut wrong = good.clone();
        wrong["boundary"] = json!("turn-0");
        assert!(contract.submit(&wrong).is_err());
        let mut outside = good.clone();
        outside["sources"] = json!({"../response.md":"<hick:project/>"});
        assert!(contract.submit(&outside).is_err());
        let mut missing = good.clone();
        missing["sources"] =
            json!({"response.md":"<hick:project/><hick:include src=\"missing.md\"/>"});
        assert!(contract.submit(&missing).is_err());
        assert_eq!(contract.submit(&good).unwrap()["accepted"], true);
        assert!(contract.submit(&wrong).is_err());
        assert!(
            contract.completion().is_ok(),
            "A stale boundary must not revoke a current artifact"
        );
        let mut unused = good.clone();
        unused["sources"]["unused.md"] = json!("abandoned material");
        assert!(
            contract
                .submit(&unused)
                .unwrap_err()
                .to_string()
                .contains("unused Markdown")
        );
        assert_eq!(
            std::fs::read_to_string(contract.root.join("app.ts")).unwrap(),
            "dirty A\n"
        );
        assert!(!contract.root.join("response.md").exists());
    }
    #[cfg(unix)]
    #[test]
    fn changed_executable_mode_invalidates_acceptance() {
        use std::os::unix::fs::PermissionsExt;
        let (_dir, mut contract) = setup();
        let good = submission(&mut contract);
        assert_eq!(contract.submit(&good).unwrap()["accepted"], true);
        std::fs::set_permissions(
            contract.root.join("app.ts"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_eq!(contract.status().unwrap()["accepted"], false);
        assert!(contract.completion().is_err());
    }
}
