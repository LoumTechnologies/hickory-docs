use crate::{Action, Finding, Project, Provider, Readiness, host::Host};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct Uv;

fn manifest(root: &Path) -> Option<toml::Value> {
    std::fs::read_to_string(root.join("pyproject.toml"))
        .ok()?
        .parse()
        .ok()
}

pub fn environment(root: &Path) -> PathBuf {
    match &crate::config::config().uv_project_environment {
        Some(p) => {
            if p.is_absolute() {
                p.clone()
            } else {
                root.join(p)
            }
        }
        None => root.join(".venv"),
    }
}

fn member(root: &Path, candidate: &Path) -> bool {
    let Some(data) = manifest(root) else {
        return false;
    };
    let Some(workspace) = data
        .get("tool")
        .and_then(|t| t.get("uv"))
        .and_then(|u| u.get("workspace"))
    else {
        return false;
    };
    let Ok(relative) = candidate.strip_prefix(root) else {
        return false;
    };
    let matches = |key| {
        workspace
            .get(key)
            .and_then(toml::Value::as_array)
            .is_some_and(|patterns| {
                patterns.iter().filter_map(toml::Value::as_str).any(|p| {
                    globset::Glob::new(p)
                        .ok()
                        .is_some_and(|g| g.compile_matcher().is_match(relative))
                })
            })
    };
    matches("members") && !matches("exclude")
}

/// Nearest project, then its declared uv workspace (never an unrelated ancestor).
pub fn owner(directory: &Path, boundary: &Path) -> Option<PathBuf> {
    if !directory.starts_with(boundary) {
        return None;
    }
    let nearest = directory
        .ancestors()
        .take_while(|p| p.starts_with(boundary))
        .find(|p| p.join("pyproject.toml").is_file())?;
    for parent in nearest
        .ancestors()
        .skip(1)
        .take_while(|p| p.starts_with(boundary))
    {
        if parent.join("pyproject.toml").is_file() {
            if member(parent, nearest) {
                return Some(parent.to_path_buf());
            }
            break;
        }
    }
    Uv.detects(nearest)
        .filter(|p| {
            p.ambiguity.is_none() || crate::selected_manager(nearest, "uv").as_deref() == Some("uv")
        })
        .map(|p| p.root)
}

fn action(locked: bool) -> Action {
    Action {
        id: if locked { "sync" } else { "resolve-sync" }.into(),
        label: if locked { "Sync dependencies" } else { "Update lockfile and sync" }.into(),
        argv: if locked { vec!["uv", "sync", "--locked"] } else { vec!["uv", "sync"] }.into_iter().map(str::to_string).collect(),
        effects: if locked { "Installs the default dependency groups; removes undeclared packages. Keeps uv.lock unchanged." } else { "Creates or updates uv.lock, installs default dependency groups and removes undeclared packages." }.into(),
    }
}

#[async_trait]
impl Provider for Uv {
    fn detects(&self, directory: &Path) -> Option<Project> {
        let data = manifest(directory)?;
        let uv = data.get("tool").and_then(|t| t.get("uv"));
        if uv
            .and_then(|u| u.get("managed"))
            .and_then(toml::Value::as_bool)
            == Some(false)
        {
            return None;
        }
        if !directory.join("uv.lock").is_file()
            && uv.is_none()
            && !directory.join("uv.toml").is_file()
        {
            return None;
        }
        let conflict = ["poetry.lock", "pdm.lock", "Pipfile.lock"]
            .iter()
            .find(|f| directory.join(f).is_file());
        let mut evidence = vec![
            directory.join("pyproject.toml"),
            directory.join("uv.lock"),
            directory.join("uv.toml"),
            directory.join(".python-version"),
        ];
        if let Some(lock) = conflict {
            evidence.push(directory.join(lock));
        }
        // Members influence native workspace resolution and action revision.
        if let Some(patterns) = uv
            .and_then(|u| u.get("workspace"))
            .and_then(|w| w.get("members"))
            .and_then(toml::Value::as_array)
        {
            for entry in ignore::WalkBuilder::new(directory)
                .hidden(false)
                .follow_links(false)
                .filter_entry(|e| {
                    !matches!(
                        e.file_name().to_str(),
                        Some(".git" | ".venv" | "node_modules" | "target")
                    )
                })
                .build()
                .flatten()
            {
                if entry.file_name() == "pyproject.toml"
                    && entry.path() != directory.join("pyproject.toml")
                    && let Some(parent) = entry.path().parent()
                    && patterns.iter().filter_map(toml::Value::as_str).any(|p| {
                        globset::Glob::new(p).ok().is_some_and(|g| {
                            g.compile_matcher()
                                .is_match(parent.strip_prefix(directory).unwrap_or(parent))
                        })
                    })
                {
                    evidence.push(entry.path().to_path_buf());
                }
            }
        }
        Some(Project { root: directory.to_path_buf(), manager: "uv".into(), evidence,
            ambiguity: conflict.map(|lock| format!("Both uv configuration and {lock} are present. Choose the intended package manager before installing dependencies.")) })
    }

    fn supports(&self, manager: &str) -> bool {
        manager == "uv"
    }

    async fn inspect(&self, project: &Project, host: &Host) -> Finding {
        let root = &project.root;
        let Some(executable) = host.executable("uv") else {
            let mut f = Finding::new(
                project,
                Readiness::ManagerMissing,
                "This project uses uv, but uv is not available to Hickory.",
            );
            f.install_url = Some("https://docs.astral.sh/uv/getting-started/installation/".into());
            return f;
        };
        if !root.join("uv.lock").is_file() {
            let mut f = Finding::new(
                project,
                Readiness::LockMissing,
                "This uv project has no lockfile.",
            );
            let mut repair = action(false);
            repair.argv[0] = executable.to_string_lossy().into_owned();
            f.actions.push(repair);
            return f;
        }
        // Lock consistency is a separate native question. A sync check can
        // fail preparing wheel metadata before it reports a stale lock.
        if let Ok(out) = host
            .probe(
                &executable,
                root,
                &[
                    "lock",
                    "--check",
                    "--offline",
                    "--no-python-downloads",
                    "--no-build",
                ],
            )
            .await
            && out.code == Some(1)
            && out.stderr.contains("lockfile")
            && out.stderr.contains("needs to be updated")
            && out.stderr.contains("--check")
        {
            let mut f = Finding::new(
                project,
                Readiness::LockStale,
                "The Python manifest and uv.lock no longer agree.",
            );
            f.details = out.stderr;
            let mut repair = action(false);
            repair.argv[0] = executable.to_string_lossy().into_owned();
            f.actions.push(repair);
            return f;
        }
        let result = host
            .probe(
                &executable,
                root,
                &[
                    "sync",
                    "--check",
                    "--locked",
                    "--offline",
                    "--no-python-downloads",
                    "--no-build",
                    "--output-format",
                    "json",
                ],
            )
            .await;
        let mut f = match result {
            Err(reason) => {
                let mut f = Finding::new(
                    project,
                    Readiness::Unknown,
                    "Could not check this Python environment.",
                );
                f.details = reason;
                f
            }
            Ok(out) => {
                let data: Option<serde_json::Value> = serde_json::from_str(&out.stdout).ok();
                let reported_env = data
                    .as_ref()
                    .and_then(|v| v.pointer("/sync/environment/path"))
                    .and_then(|v| v.as_str())
                    .map(PathBuf::from)
                    .unwrap_or_else(|| environment(root));
                let known_lock_failure = out.code == Some(1)
                    && out.stderr.contains("lockfile")
                    && out.stderr.contains("needs to be updated")
                    && out.stderr.contains("--locked");
                let (state, message) = if known_lock_failure {
                    (
                        Readiness::LockStale,
                        "The Python manifest and uv.lock no longer agree.",
                    )
                } else if !reported_env.join("pyvenv.cfg").is_file() {
                    (
                        Readiness::EnvironmentMissing,
                        "Python dependencies are not installed in the selected project environment.",
                    )
                } else if let Some(data) = &data {
                    let changes = data.pointer("/sync/changes").and_then(|v| v.as_array());
                    if out.code == Some(0)
                        && changes.is_some_and(|c| c.is_empty())
                        && data.pointer("/sync/action").and_then(|v| v.as_str()) == Some("check")
                    {
                        (
                            Readiness::Ready,
                            "Python environment matches the selected dependency profile.",
                        )
                    } else if matches!(out.code, Some(0 | 1))
                        && changes.is_some_and(|c| !c.is_empty())
                    {
                        (
                            Readiness::EnvironmentStale,
                            "Python dependencies need synchronization.",
                        )
                    } else {
                        (
                            Readiness::Unknown,
                            "Uv could not establish whether this environment is synchronized.",
                        )
                    }
                } else {
                    (
                        Readiness::Unknown,
                        "This uv version did not return a supported environment check result.",
                    )
                };
                let mut f = Finding::new(project, state, message);
                // Stable output: uv timing lines vary on each inspection.
                f.details = out
                    .stderr
                    .lines()
                    .filter(|l| !l.starts_with("Resolved ") && !l.starts_with("Checked in "))
                    .collect::<Vec<_>>()
                    .join("\n");
                let python = reported_env.join(if cfg!(windows) {
                    "Scripts/python.exe"
                } else {
                    "bin/python"
                });
                f.interpreter = python.is_file().then_some(python);
                f
            }
        };
        match f.state {
            Readiness::LockStale => f.actions.push(action(false)),
            Readiness::EnvironmentMissing | Readiness::EnvironmentStale => {
                f.actions.push(action(true))
            }
            _ => {}
        }
        // Pin exactly the executable the engine found, not a shell expansion.
        for action in &mut f.actions {
            action.argv[0] = executable.to_string_lossy().into_owned();
        }
        f
    }
}
