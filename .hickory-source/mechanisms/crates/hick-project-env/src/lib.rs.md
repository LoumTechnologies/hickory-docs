<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-project-env/src/lib.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0a1bae4be7ba"></a>
## `crates/hick-project-env/src/lib.rs`

Discover workspace environments and publish evaluated project entries through the host filesystem boundary.

<a id="f-0a1bae4be7ba-config"></a>
### `config`

Workspace environment providers. No parser, element, UI or installer lives here.

<storycode:copy id="f-0a1bae4be7ba-config">
//! Workspace environment providers. No parser, element, UI or installer lives here.
mod config;
</storycode:copy>
<a id="f-0a1bae4be7ba-host"></a>
### `host`

<storycode:copy id="f-0a1bae4be7ba-host">
pub mod host;
</storycode:copy>
<a id="f-0a1bae4be7ba-providers"></a>
### `providers`

<storycode:copy id="f-0a1bae4be7ba-providers">
pub mod providers;

</storycode:copy>
<a id="f-0a1bae4be7ba-use_declaration"></a>
### `use_declaration`

<storycode:copy id="f-0a1bae4be7ba-use_declaration">
use async_trait::async_trait;
</storycode:copy>
<a id="f-0a1bae4be7ba-use_declaration-2"></a>
### `use_declaration`

<storycode:copy id="f-0a1bae4be7ba-use_declaration-2">
use serde::{Deserialize, Serialize};
</storycode:copy>
<a id="f-0a1bae4be7ba-use_declaration-3"></a>
### `use_declaration`

<storycode:copy id="f-0a1bae4be7ba-use_declaration-3">
use sha2::{Digest, Sha256};
</storycode:copy>
<a id="f-0a1bae4be7ba-use_declaration-4"></a>
### `use_declaration`

<storycode:copy id="f-0a1bae4be7ba-use_declaration-4">
use std::path::{Path, PathBuf};

</storycode:copy>
<a id="f-0a1bae4be7ba-Project"></a>
### `Project`

<storycode:copy id="f-0a1bae4be7ba-Project">
#[derive(Clone, Debug)]
pub struct Project {
    pub root: PathBuf,
    pub manager: String,
    pub evidence: Vec<PathBuf>,
    pub ambiguity: Option<String>,
}

</storycode:copy>
<a id="f-0a1bae4be7ba-Readiness"></a>
### `Readiness`

<storycode:copy id="f-0a1bae4be7ba-Readiness">
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Readiness {
    ManagerMissing,
    EnvironmentMissing,
    EnvironmentStale,
    LockMissing,
    LockStale,
    Ambiguous,
    Unknown,
    Ready,
}

</storycode:copy>
<a id="f-0a1bae4be7ba-Action"></a>
### `Action`

<storycode:copy id="f-0a1bae4be7ba-Action">
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub argv: Vec<String>,
    pub effects: String,
}

</storycode:copy>
<a id="f-0a1bae4be7ba-Finding"></a>
### `Finding`

<storycode:copy id="f-0a1bae4be7ba-Finding">
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Finding {
    pub project: String,
    pub manager: String,
    pub target: String,
    pub profile: String,
    pub revision: String,
    pub state: Readiness,
    pub message: String,
    pub details: String,
    pub actions: Vec<Action>,
    pub install_url: Option<String>,
    pub interpreter: Option<PathBuf>,
    pub manager_choices: Vec<String>,
}

</storycode:copy>
<a id="f-0a1bae4be7ba-impl-Finding"></a>
### `impl-Finding`

<storycode:copy id="f-0a1bae4be7ba-impl-Finding">
impl Finding {
    pub fn new(project: &Project, state: Readiness, message: impl Into<String>) -> Self {
        Self {
            project: project.root.to_string_lossy().into_owned(),
            manager: project.manager.clone(),
            target: "host".into(),
            profile: "manager defaults".into(),
            revision: String::new(),
            state,
            message: message.into(),
            details: String::new(),
            actions: Vec::new(),
            install_url: None,
            interpreter: None,
            manager_choices: if project.ambiguity.is_some() {
                if project.manager == "uv" {
                    vec!["uv".into()]
                } else {
                    vec!["npm".into(), "pnpm".into()]
                }
            } else {
                Vec::new()
            },
        }
    }
}

</storycode:copy>
<a id="f-0a1bae4be7ba-Provider"></a>
### `Provider`

<storycode:copy id="f-0a1bae4be7ba-Provider">
#[async_trait]
pub trait Provider: Send + Sync {
    fn detects(&self, directory: &Path) -> Option<Project>;
    fn supports(&self, manager: &str) -> bool;
    async fn inspect(&self, project: &Project, host: &host::Host) -> Finding;
}

</storycode:copy>
<a id="f-0a1bae4be7ba-Registry"></a>
### `Registry`

<storycode:copy id="f-0a1bae4be7ba-Registry">
pub struct Registry {
    providers: Vec<Box<dyn Provider>>,
}

</storycode:copy>
<a id="f-0a1bae4be7ba-impl-Default-for-Registry"></a>
### `impl-Default for Registry`

<storycode:copy id="f-0a1bae4be7ba-impl-Default-for-Registry">
impl Default for Registry {
    fn default() -> Self {
        Self::new(vec![
            Box::new(providers::uv::Uv),
            Box::new(providers::node::Node),
        ])
    }
}

</storycode:copy>
<a id="f-0a1bae4be7ba-impl-Registry"></a>
### `impl-Registry`

<storycode:copy id="f-0a1bae4be7ba-impl-Registry">
impl Registry {
    pub fn new(providers: Vec<Box<dyn Provider>>) -> Self {
        Self { providers }
    }

    /// Bounded, gitignore-aware discovery; never follow directory symlinks.
    pub fn discover(&self, root: &Path) -> Vec<Project> {
        let mut directories = std::collections::BTreeSet::new();
        directories.insert(root.to_path_buf());
        for entry in ignore::WalkBuilder::new(root)
            .hidden(false)
            .follow_links(false)
            .filter_entry(|e| {
                !matches!(
                    e.file_name().to_str(),
                    Some(
                        ".git"
                            | ".venv"
                            | "venv"
                            | "node_modules"
                            | "target"
                            | "dist"
                            | "build"
                            | ".hick-cache"
                    )
                )
            })
            .build()
            .flatten()
        {
            if entry.file_type().is_some_and(|t| t.is_file())
                && matches!(
                    entry.file_name().to_str(),
                    Some("pyproject.toml" | "package.json")
                )
                && let Some(parent) = entry.path().parent()
            {
                directories.insert(parent.to_path_buf());
            }
        }
        let mut projects = Vec::new();
        for directory in directories {
            for provider in &self.providers {
                if let Some(project) = provider.detects(&directory) {
                    if project.manager == "uv"
                        && providers::uv::owner(&directory, root).is_some_and(|p| p != directory)
                    {
                        continue;
                    }
                    if providers::node::is_member(&project, root) {
                        continue;
                    }
                    projects.push(project);
                }
            }
        }
        projects.sort_by(|a, b| a.root.cmp(&b.root).then(a.manager.cmp(&b.manager)));
        projects
    }

    pub async fn inspect(&self, project: &Project, host: &host::Host) -> Finding {
        let mut finding = if let Some(reason) = &project.ambiguity {
            Finding::new(project, Readiness::Ambiguous, reason)
        } else if let Some(provider) = self.providers.iter().find(|p| p.supports(&project.manager))
        {
            provider.inspect(project, host).await
        } else {
            Finding::new(
                project,
                Readiness::Unknown,
                "This package manager has no environment provider yet.",
            )
        };
        // Evidence content, result and selected environment all participate.
        // A repeated identical check produces the same dismissal/action revision.
        let mut hash = Sha256::new();
        for file in &project.evidence {
            hash.update(file.to_string_lossy().as_bytes());
            hash.update(std::fs::read(file).unwrap_or_default());
        }
        let mut stable = finding.clone();
        stable.details.clear(); // Native diagnostic timings are not project revisions.
        hash.update(serde_json::to_vec(&stable).unwrap());
        finding.revision = format!("{:x}", hash.finalize());
        finding
    }
}

</storycode:copy>
<a id="f-0a1bae4be7ba-python_interpreter"></a>
### `python_interpreter`

<storycode:copy id="f-0a1bae4be7ba-python_interpreter">
pub fn python_interpreter(directory: &Path, boundary: &Path) -> Option<PathBuf> {
    let owner = providers::uv::owner(directory, boundary)?;
    let env = providers::uv::environment(&owner);
    let python = env.join(if cfg!(windows) {
        "Scripts/python.exe"
    } else {
        "bin/python"
    });
    python.is_file().then_some(python)
}

</storycode:copy>
<a id="f-0a1bae4be7ba-selected_manager"></a>
### `selected_manager`

An explicit per-user choice is local state, never a repository edit.

<storycode:copy id="f-0a1bae4be7ba-selected_manager">
/// An explicit per-user choice is local state, never a repository edit.
pub fn selected_manager(root: &Path, detected: &str) -> Option<String> {
    let store = hickory_workspace::WorkspaceStore::for_project(root).ok()?;
    let data: std::collections::HashMap<String, String> =
        serde_json::from_slice(&std::fs::read(store.dir().join("package-managers.json")).ok()?)
            .ok()?;
    data.get(detected).cloned()
}

</storycode:copy>
<a id="f-0a1bae4be7ba-tree"></a>
### `tree`

<storycode:copy id="f-0a1bae4be7ba-tree">
pub mod pending;
mod tree;
</storycode:copy>
<a id="f-0a1bae4be7ba-use_declaration-5"></a>
### `use_declaration`

<storycode:copy id="f-0a1bae4be7ba-use_declaration-5">
pub use tree::write_tree;
</storycode:copy>

### Assemble `crates/hick-project-env/src/lib.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-project-env/src/lib.rs"><storycode:paste select="#f-0a1bae4be7ba-config"/><storycode:paste select="#f-0a1bae4be7ba-host"/><storycode:paste select="#f-0a1bae4be7ba-providers"/><storycode:paste select="#f-0a1bae4be7ba-use_declaration"/><storycode:paste select="#f-0a1bae4be7ba-use_declaration-2"/><storycode:paste select="#f-0a1bae4be7ba-use_declaration-3"/><storycode:paste select="#f-0a1bae4be7ba-use_declaration-4"/><storycode:paste select="#f-0a1bae4be7ba-Project"/><storycode:paste select="#f-0a1bae4be7ba-Readiness"/><storycode:paste select="#f-0a1bae4be7ba-Action"/><storycode:paste select="#f-0a1bae4be7ba-Finding"/><storycode:paste select="#f-0a1bae4be7ba-impl-Finding"/><storycode:paste select="#f-0a1bae4be7ba-Provider"/><storycode:paste select="#f-0a1bae4be7ba-Registry"/><storycode:paste select="#f-0a1bae4be7ba-impl-Default-for-Registry"/><storycode:paste select="#f-0a1bae4be7ba-impl-Registry"/><storycode:paste select="#f-0a1bae4be7ba-python_interpreter"/><storycode:paste select="#f-0a1bae4be7ba-selected_manager"/><storycode:paste select="#f-0a1bae4be7ba-tree"/><storycode:paste select="#f-0a1bae4be7ba-use_declaration-5"/></storycode:file>

</storycode:doc>
