//! Conservative package-manager coverage: absence is known; a directory alone cannot prove freshness.
use crate::{Action, Finding, Project, Provider, Readiness, host::Host};
use async_trait::async_trait;
use std::path::Path;

pub struct Node;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manager {
    pub name: String,
    pub lockfiles: Vec<String>,
    workspace_yaml: Option<String>,
    #[serde(default)]
    install: Vec<String>,
    #[serde(default)]
    locked_install: Vec<String>,
    install_url: Option<String>,
}

// Catalog entries share the package.json project model. Names are host PATH
// commands and lockfiles are exact basenames, including for local additions.
pub fn managers() -> &'static [Manager] {
    static CATALOG: std::sync::OnceLock<Vec<Manager>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        let mut values: Vec<Manager> = serde_json::from_str(r#"[
            {"name":"npm","lockfiles":["package-lock.json","npm-shrinkwrap.json"],"install":["install"],"locked_install":["ci"],"install_url":"https://docs.npmjs.com/downloading-and-installing-node-js-and-npm"},
            {"name":"pnpm","lockfiles":["pnpm-lock.yaml"],"workspace_yaml":"pnpm-workspace.yaml","install":["install"],"locked_install":["install","--frozen-lockfile"],"install_url":"https://pnpm.io/installation"},
            {"name":"yarn","lockfiles":["yarn.lock"]},
            {"name":"bun","lockfiles":["bun.lock","bun.lockb"],"install":["install"],"locked_install":["install","--frozen-lockfile"],"install_url":"https://bun.com/docs/installation"}
        ]"#).expect("built-in package-manager catalog");
        if let Some(path) = std::env::var_os("HICKORY_PACKAGE_MANAGERS") {
            match read_catalog(std::path::Path::new(&path), &values) {
                Ok(additions) => values.extend(additions),
                Err(error) => eprintln!("Could not load HICKORY_PACKAGE_MANAGERS: {error}"),
            }
        }
        values
    })
}

fn read_catalog(path: &Path, builtins: &[Manager]) -> Result<Vec<Manager>, String> {
    use std::io::Read;
    if !std::fs::metadata(path).map_err(|e| e.to_string())?.is_file() {
        return Err("Expected a regular catalog file".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path).map_err(|e| e.to_string())?.take(32_769).read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    if bytes.len() > 32_768 {
        return Err("Package-manager catalog exceeds 32 KiB".into());
    }
    let additions: Vec<Manager> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let mut names: std::collections::HashSet<&str> =
        builtins.iter().map(|manager| manager.name.as_str()).collect();
    names.extend(["uv", "ambiguous"]);
    let mut locks: std::collections::HashSet<&str> = builtins
        .iter().flat_map(|manager| manager.lockfiles.iter().map(String::as_str)).collect();
    let basename = |name: &str| {
        !name.is_empty() && name.len() <= 96 && !name.starts_with('.')
            && name.bytes().all(|ch| ch.is_ascii_alphanumeric() || b"._-".contains(&ch))
    };
    for manager in &additions {
        if !basename(&manager.name)
            || manager.name.starts_with('-')
            || !names.insert(&manager.name)
            || manager.lockfiles.is_empty()
            || manager.lockfiles.iter().any(|lock| !basename(lock) || !locks.insert(lock))
            || manager.workspace_yaml.as_deref().is_some_and(|file| !basename(file))
            || manager.install_url.as_deref().is_some_and(|url| !url.starts_with("https://"))
        {
            return Err(format!("Invalid or duplicate package-manager entry {:?}", manager.name));
        }
    }
    Ok(additions)
}

fn manager(name: &str) -> Option<&'static Manager> {
    managers().iter().find(|entry| entry.name == name)
}

/// Both inventory-backed launches and filesystem-backed inspection use the
/// same precedence. Conflicting declarations and lockfiles require a choice.
pub fn resolve<'a>(
    manifest: &'a serde_json::Value,
    has_lock: impl Fn(&str) -> bool,
) -> Result<Option<&'a str>, ()> {
    let mut selected = manifest["packageManager"].as_str()
        .and_then(|declaration| declaration.split('@').next())
        .filter(|name| !name.is_empty());
    for candidate in managers() {
        if candidate.lockfiles.iter().any(|lock| has_lock(lock)) {
            match selected {
                Some(name) if name != candidate.name => return Err(()),
                Some(_) => {}
                None => selected = Some(&candidate.name),
            }
        }
    }
    Ok(selected)
}

fn package(root: &Path) -> Option<serde_json::Value> {
    serde_json::from_str(&std::fs::read_to_string(root.join("package.json")).ok()?).ok()
}

fn patterns(root: &Path, manager: &str) -> Vec<String> {
    let values = if let Some(workspace) = self::manager(manager).and_then(|entry| entry.workspace_yaml.as_deref()) {
        std::fs::read_to_string(root.join(workspace))
            .ok()
            .and_then(|text| serde_yaml::from_str::<serde_json::Value>(&text).ok())
            .and_then(|v| v.get("packages").and_then(|v| v.as_array()).cloned())
    } else {
        package(root)
            .and_then(|p| p.get("workspaces").cloned())
            .and_then(|v| {
                v.as_array()
                    .cloned()
                    .or_else(|| v.get("packages").and_then(|p| p.as_array()).cloned())
            })
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
            .any(|p| p.is_file() && p.file_name().and_then(|n| n.to_str()).is_some_and(is_lockfile));
    if !unselected && manager(&project.manager).is_none() {
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

pub fn is_lockfile(name: &str) -> bool {
    managers().iter().any(|entry| entry.lockfiles.iter().any(|lock| lock == name))
}

#[async_trait]
impl Provider for Node {
    fn detects(&self, root: &Path) -> Option<Project> {
        let data = package(root)?;
        let resolved = resolve(&data, |lock| root.join(lock).is_file());
        let manager = resolved.ok().flatten().unwrap_or("ambiguous");
        let ambiguity = (manager == "ambiguous").then(|| {
            "JavaScript package-manager evidence is ambiguous. Choose the intended manager before installing dependencies.".into()
        });
        Some(Project {
            root: root.to_path_buf(),
            manager: manager.into(),
            manager_choices: if ambiguity.is_some() {
                managers().iter().filter(|entry| !entry.install.is_empty())
                    .map(|entry| entry.name.clone()).collect()
            } else {
                Vec::new()
            },
            ambiguity,
            evidence: std::iter::once(root.join("package.json"))
                .chain(managers().iter().flat_map(|entry| {
                    entry.lockfiles.iter().chain(entry.workspace_yaml.iter())
                }).map(|file| root.join(file)))
                .collect(),
        })
    }
    fn markers(&self) -> &'static [&'static str] {
        &["package.json"]
    }
    fn owns(&self, project: &Project, boundary: &Path) -> bool {
        !is_member(project, boundary)
    }
    fn supports(&self, manager: &str) -> bool {
        self::manager(manager).is_some_and(|entry| !entry.install.is_empty())
    }
    fn additional_evidence(&self, project: &Project) -> Vec<std::path::PathBuf> {
        member_manifests(&project.root, &project.manager)
    }

    async fn inspect(&self, project: &Project, host: &Host) -> Finding {
        let manager = self::manager(&project.manager).expect("selected catalog manager");
        let name = &manager.name;
        let Some(executable) = host.executable(name) else {
            let mut f = Finding::new(
                project,
                Readiness::ManagerMissing,
                format!("This project uses {name}, but it is not available to Hickory."),
            );
            f.install_url = manager.install_url.clone();
            return f;
        };
        let dependencies = project.evidence.iter()
            .filter(|path| path.file_name().is_some_and(|name| name == "package.json"))
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
            let locked = !manager.locked_install.is_empty()
                && manager.lockfiles.iter().any(|file| project.root.join(file).is_file());
            let args = if locked { &manager.locked_install } else { &manager.install };
            f.actions.push(Action {
                id: "install".into(),
                label: "Install dependencies".into(),
                argv: std::iter::once(executable.to_string_lossy().into_owned())
                    .chain(args.iter().cloned())
                    .collect(),
                effects: if name == "npm" && locked {
                    "Runs a clean install; replaces node_modules and preserves the lockfile."
                } else if locked {
                    "Installs workspace dependencies using the lockfile."
                } else {
                    "Installs dependencies and may create or update the lockfile."
                }
                .into(),
            });
        }
        f
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn catalog_loads_an_additional_manager_and_rejects_conflicting_entries() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let existing = managers();
        std::fs::write(temp.path(), r#"[
            {"name":"fixture-pm","lockfiles":["fixture.lock"],"install":["restore"]}
        ]"#).unwrap();
        let loaded = read_catalog(temp.path(), existing).unwrap();
        assert_eq!(loaded[0].name, "fixture-pm");
        assert_eq!(loaded[0].install, ["restore"]);
        std::fs::write(temp.path(), r#"[{"name":"fixture-pm","lockfiles":["bun.lock"]}]"#).unwrap();
        assert!(read_catalog(temp.path(), existing).is_err());
    }

    #[test]
    fn locked_manager_selection_has_a_fixed_metadata_budget() {
        let checks = AtomicUsize::new(0);
        let manifest = serde_json::json!({"packageManager":"bun@1.3.0"});
        let result = resolve(&manifest, |file| {
            checks.fetch_add(1, Ordering::Relaxed);
            file == "bun.lock"
        });
        assert_eq!(result, Ok(Some("bun")));
        assert!(checks.load(Ordering::Relaxed) <= managers().iter().map(|entry| entry.lockfiles.len()).sum());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn bun_workspace_inspection_plans_a_locked_install_without_starting_bun() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(root.join("packages/member")).unwrap();
        std::fs::create_dir(root.join("tools")).unwrap();
        std::fs::write(root.join("tools/bun"), "#!/bin/sh\ntouch unexpectedly-started\n").unwrap();
        std::fs::set_permissions(root.join("tools/bun"), std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(root.join("bun.lock"), "lock marker").unwrap();
        std::fs::write(root.join("package.json"), r#"{"packageManager":"bun@1.3.0","workspaces":["packages/*"]}"#).unwrap();
        let member = root.join("packages/member/package.json");
        std::fs::write(&member, r#"{"dependencies":{"fixture":"1.0.0"}}"#).unwrap();

        let registry = crate::Registry::default();
        let projects = registry.discover(root);
        assert_eq!(projects.len(), 1);
        let host = Host { path: root.join("tools").into_os_string(), ..Host::default() };
        let first = registry.inspect(&projects[0], &host).await;
        assert_eq!(first.state, Readiness::EnvironmentMissing);
        assert_eq!(first.actions[0].argv[1..], ["install", "--frozen-lockfile"]);
        assert!(!root.join("unexpectedly-started").exists());

        std::fs::write(member, r#"{"dependencies":{"fixture":"2.0.0"}}"#).unwrap();
        let changed = registry.inspect(&projects[0], &host).await;
        assert_ne!(first.revision, changed.revision);
    }
}
