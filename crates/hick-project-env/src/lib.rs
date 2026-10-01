//! Workspace environment providers. No parser, element, UI or installer lives here.
mod config;
pub mod host;
pub mod providers;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Project {
    pub root: PathBuf,
    pub manager: String,
    pub evidence: Vec<PathBuf>,
    pub ambiguity: Option<String>,
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub label: String,
    pub argv: Vec<String>,
    pub effects: String,
}

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

#[async_trait]
pub trait Provider: Send + Sync {
    fn detects(&self, directory: &Path) -> Option<Project>;
    fn supports(&self, manager: &str) -> bool;
    async fn inspect(&self, project: &Project, host: &host::Host) -> Finding;
}

pub struct Registry {
    providers: Vec<Box<dyn Provider>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new(vec![
            Box::new(providers::uv::Uv),
            Box::new(providers::node::Node),
        ])
    }
}

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

/// An explicit per-user choice is local state, never a repository edit.
pub fn selected_manager(root: &Path, detected: &str) -> Option<String> {
    let store = hickory_workspace::WorkspaceStore::for_project(root).ok()?;
    let data: std::collections::HashMap<String, String> =
        serde_json::from_slice(&std::fs::read(store.dir().join("package-managers.json")).ok()?)
            .ok()?;
    data.get(detected).cloned()
}
