//! Conservative npm/pnpm coverage: absence is known; a directory alone cannot prove freshness.
use crate::{Action, Finding, Project, Provider, Readiness, host::Host};
use async_trait::async_trait;
use std::path::Path;

pub struct Node;

fn package(root: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).ok()?).ok()
}

fn patterns(root: &Path, manager: &str) -> Vec<String> {
    let values = if manager == "npm" {
        package(root)
            .and_then(|p| p.get("workspaces").cloned())
            .and_then(|v| {
                v.as_array()
                    .cloned()
                    .or_else(|| v.get("packages").and_then(|p| p.as_array()).cloned())
            })
    } else {
        std::fs::read_to_string(root.join("pnpm-workspace.yaml"))
            .ok()
            .and_then(|text| serde_yaml::from_str::<serde_json::Value>(&text).ok())
            .and_then(|v| v.get("packages").and_then(|v| v.as_array()).cloned())
    };
    values
        .unwrap_or_default()
        .iter()
        .filter_map(|p| p.as_str().map(str::to_string))
        .collect()
}

fn matches_member(patterns: &[String], relative: &Path) -> bool {
    let matches = |p: &str| {
        globset::Glob::new(p)
            .ok()
            .is_some_and(|g| g.compile_matcher().is_match(relative))
    };
    patterns
        .iter()
        .filter(|p| !p.starts_with('!'))
        .any(|p| matches(p))
        && !patterns
            .iter()
            .filter_map(|p| p.strip_prefix('!'))
            .any(matches)
}

fn member_manifests(root: &Path, manager: &str) -> Vec<std::path::PathBuf> {
    let patterns = patterns(root, manager);
    if patterns.is_empty() {
        return Vec::new();
    }
    ignore::WalkBuilder::new(root)
        .hidden(false)
        .follow_links(false)
        .filter_entry(|e| {
            !matches!(
                e.file_name().to_str(),
                Some(".git" | "node_modules" | ".venv" | "target" | "dist")
            )
        })
        .build()
        .flatten()
        .filter(|e| e.file_name() == "package.json" && e.file_type().is_some_and(|t| t.is_file()))
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.strip_prefix(root).ok())
                .is_some_and(|p| matches_member(&patterns, p))
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

pub fn is_member(project: &Project, boundary: &Path) -> bool {
    let unselected = project.manager == "ambiguous"
        && !project
            .evidence
            .iter()
            .skip(1)
            .any(|p| p.is_file() && p.file_name().is_some_and(|n| n != "pnpm-workspace.yaml"));
    if !unselected && !matches!(project.manager.as_str(), "npm" | "pnpm") {
        return false;
    }
    for parent in project
        .root
        .ancestors()
        .skip(1)
        .take_while(|p| p.starts_with(boundary))
    {
        if let Some(owner) = Node.detects(parent)
            && owner.ambiguity.is_none()
            && (unselected || owner.manager == project.manager)
            && matches_member(
                &patterns(parent, &owner.manager),
                project.root.strip_prefix(parent).unwrap(),
            )
        {
            return true;
        }
        if parent.join("package.json").is_file() {
            break;
        }
    }
    false
}

#[async_trait]
impl Provider for Node {
    fn detects(&self, root: &Path) -> Option<Project> {
        let data = package(root)?;
        let declaration = data
            .get("packageManager")
            .and_then(|v| v.as_str())
            .map(|s| s.split('@').next().unwrap_or(s));
        let locks = [
            ("npm", "package-lock.json"),
            ("npm", "npm-shrinkwrap.json"),
            ("pnpm", "pnpm-lock.yaml"),
            ("yarn", "yarn.lock"),
            ("bun", "bun.lock"),
            ("bun", "bun.lockb"),
        ];
        let found: std::collections::BTreeSet<_> = locks
            .iter()
            .filter(|(_, lock)| root.join(lock).is_file())
            .map(|(manager, _)| *manager)
            .collect();
        let manager = declaration
            .or_else(|| {
                if found.len() == 1 {
                    found.first().copied()
                } else {
                    None
                }
            })
            .unwrap_or("ambiguous");
        let ambiguity = if manager == "ambiguous" || found.iter().any(|m| *m != manager) {
            Some("JavaScript package-manager evidence is ambiguous. Choose the intended manager before installing dependencies.".into())
        } else {
            None
        };
        Some(Project {
            root: root.to_path_buf(),
            manager: manager.into(),
            ambiguity,
            evidence: std::iter::once(root.join("package.json"))
                .chain(locks.iter().map(|(_, lock)| root.join(lock)))
                .chain(std::iter::once(root.join("pnpm-workspace.yaml")))
                .chain(member_manifests(root, manager))
                .collect(),
        })
    }
    fn supports(&self, manager: &str) -> bool {
        matches!(manager, "npm" | "pnpm")
    }

    async fn inspect(&self, project: &Project, host: &Host) -> Finding {
        let manager = &project.manager;
        let Some(executable) = host.executable(manager) else {
            let mut f = Finding::new(
                project,
                Readiness::ManagerMissing,
                format!("This project uses {manager}, but it is not available to Hickory."),
            );
            f.install_url = Some(
                if manager == "npm" {
                    "https://docs.npmjs.com/downloading-and-installing-node-js-and-npm"
                } else {
                    "https://pnpm.io/installation"
                }
                .into(),
            );
            return f;
        };
        let dependencies = std::iter::once(project.root.join("package.json"))
            .chain(member_manifests(&project.root, manager))
            .any(|manifest| {
                let data = package(manifest.parent().unwrap_or(&project.root)).unwrap_or_default();
                ["dependencies", "devDependencies", "optionalDependencies"]
                    .iter()
                    .any(|k| {
                        data.get(k)
                            .and_then(|v| v.as_object())
                            .is_some_and(|o| !o.is_empty())
                    })
            });
        let missing = dependencies && !project.root.join("node_modules").is_dir();
        let mut f = Finding::new(
            project,
            if missing {
                Readiness::EnvironmentMissing
            } else {
                Readiness::Unknown
            },
            if missing {
                "JavaScript dependencies are not installed."
            } else {
                "JavaScript dependency freshness has not been verified."
            },
        );
        f.details = "This provider detects a missing installation. An existing node_modules directory does not establish that all dependencies match the lockfile. Recheck after installing externally.".into();
        if missing {
            let locked = if manager == "npm" {
                project.root.join("package-lock.json").is_file()
                    || project.root.join("npm-shrinkwrap.json").is_file()
            } else {
                project.root.join("pnpm-lock.yaml").is_file()
            };
            let args = match (manager.as_str(), locked) {
                ("npm", true) => vec!["ci"],
                ("pnpm", true) => vec!["install", "--frozen-lockfile"],
                _ => vec!["install"],
            };
            f.actions.push(Action {
                id: "install".into(),
                label: "Install dependencies".into(),
                argv: std::iter::once(executable.to_string_lossy().into_owned())
                    .chain(args.into_iter().map(str::to_string))
                    .collect(),
                effects: match (manager.as_str(), locked) {
                    ("npm", true) => {
                        "Runs a clean install; replaces node_modules and preserves the lockfile."
                    }
                    ("pnpm", true) => "Installs workspace dependencies; refuses a stale lockfile.",
                    _ => "Installs dependencies and may create or update the lockfile.",
                }
                .into(),
            });
        }
        f
    }
}
