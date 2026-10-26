<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/serve/environments.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-532a6a243ee2"></a>
## `crates/hickory-cli/src/serve/environments.rs`

<a id="f-532a6a243ee2-Environments"></a>
### `Environments`

Thin workspace adapter for hick-project-env. Repairs are real terminal sessions.

<storycode:copy id="f-532a6a243ee2-Environments">
//! Thin workspace adapter for hick-project-env. Repairs are real terminal sessions.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Query, State},
};
use hick_project_env::{Finding, host::Host};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct Environments {
    cache: Mutex<Cache>,
    host: Host,
}

</storycode:copy>
<a id="f-532a6a243ee2-impl-Environments"></a>
### `impl-Environments`

<storycode:copy id="f-532a6a243ee2-impl-Environments">
impl Environments {
    pub fn with_host(host: Host) -> Self {
        Self {
            cache: Mutex::new(Cache::default()),
            host,
        }
    }
}

</storycode:copy>
<a id="f-532a6a243ee2-Cache"></a>
### `Cache`

<storycode:copy id="f-532a6a243ee2-Cache">
#[derive(Default)]
struct Cache {
    checked: Option<Instant>,
    findings: Vec<Finding>,
    running: HashMap<String, Arc<hick_term::Session>>,
}

</storycode:copy>
<a id="f-532a6a243ee2-choices"></a>
### `choices`

<storycode:copy id="f-532a6a243ee2-choices">
fn choices(root: &Path) -> ApiResult<(std::path::PathBuf, HashMap<String, String>)> {
    let store = hickory_workspace::WorkspaceStore::for_project(root)
        .map_err(|e| ApiError::unavailable(e.to_string()))?;
    let path = store.dir().join("package-managers.json");
    let values = match std::fs::read(&path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| {
            ApiError::unprocessable(format!("Unreadable package-manager choices: {e}"))
        })?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
        Err(e) => return Err(ApiError::unavailable(e.to_string())),
    };
    Ok((path, values))
}

</storycode:copy>
<a id="f-532a6a243ee2-InspectQuery"></a>
### `InspectQuery`

<storycode:copy id="f-532a6a243ee2-InspectQuery">
#[derive(Deserialize, Default)]
pub struct InspectQuery {
    #[serde(default)]
    refresh: bool,
}

</storycode:copy>
<a id="f-532a6a243ee2-Status"></a>
### `Status`

<storycode:copy id="f-532a6a243ee2-Status">
#[derive(Serialize)]
pub struct Status {
    findings: Vec<Finding>,
    running: HashMap<String, String>,
}

</storycode:copy>
<a id="f-532a6a243ee2-scan"></a>
### `scan`

<storycode:copy id="f-532a6a243ee2-scan">
async fn scan(root: &Path, host: &Host) -> ApiResult<Vec<Finding>> {
    let root = root.to_path_buf();
    let projects = tokio::task::spawn_blocking(move || {
        hick_plugins::registry()
            .environment_registry()
            .discover(&root)
    })
    .await
    .map_err(|e| ApiError::internal(e.to_string()))?;
    let registry = hick_plugins::registry().environment_registry();
    let mut findings = Vec::new();
    for mut project in projects {
        if let Some(manager) = hick_project_env::selected_manager(&project.root, &project.manager)
            && project.ambiguity.is_some()
            && (project.manager == "uv" && manager == "uv"
                || project.manager != "uv" && matches!(manager.as_str(), "npm" | "pnpm"))
        {
            project.manager = manager;
            project.ambiguity = None;
        }
        findings.push(registry.inspect(&project, host).await);
    }
    Ok(findings)
}

</storycode:copy>
<a id="f-532a6a243ee2-inspect"></a>
### `inspect`

<storycode:copy id="f-532a6a243ee2-inspect">
pub async fn inspect(
    State(state): State<LocalState>,
    Query(query): Query<InspectQuery>,
) -> ApiResult<Json<Status>> {
    let mut cache = state.environments.cache.lock().await;
    cache
        .running
        .retain(|_, session| session.exit_code().is_none());
    if cache.running.is_empty()
        && (query.refresh
            || cache
                .checked
                .is_none_or(|t| t.elapsed() > Duration::from_secs(15)))
    {
        let findings = scan(state.index.root(), &state.environments.host).await?;
        if findings.iter().any(|f| {
            f.state == hick_project_env::Readiness::Ready
                && cache
                    .findings
                    .iter()
                    .any(|old| old.project == f.project && old.state != f.state)
        }) {
            state.lsp.refresh_environments();
        }
        cache.findings = findings;
        cache.checked = Some(Instant::now());
    }
    Ok(Json(Status {
        findings: cache.findings.clone(),
        running: cache
            .running
            .iter()
            .map(|(project, session)| (project.clone(), session.id.clone()))
            .collect(),
    }))
}

</storycode:copy>
<a id="f-532a6a243ee2-RunAction"></a>
### `RunAction`

<storycode:copy id="f-532a6a243ee2-RunAction">
#[derive(Deserialize)]
pub struct RunAction {
    project: String,
    manager: String,
    revision: String,
    action: String,
}

</storycode:copy>
<a id="f-532a6a243ee2-act"></a>
### `act`

<storycode:copy id="f-532a6a243ee2-act">
pub async fn act(
    State(state): State<LocalState>,
    Json(body): Json<RunAction>,
) -> ApiResult<Json<hick_term::SessionSummary>> {
    let mut cache = state.environments.cache.lock().await;
    cache
        .running
        .retain(|_, session| session.exit_code().is_none());
    if cache.running.contains_key(&body.project) {
        return Err(ApiError::conflict(
            "Dependency installation is already running for this project.",
        ));
    }
    // Rescan and inspect, including manifests changed since the notice was drawn.
    // Submitted paths and commands are never trusted: only discovered projects act.
    let findings = scan(state.index.root(), &state.environments.host).await?;
    let finding = findings
        .iter()
        .find(|f| f.project == body.project && f.manager == body.manager)
        .ok_or_else(|| {
            ApiError::not_found("This project is no longer part of the open workspace.")
        })?;
    if finding.revision != body.revision {
        return Err(ApiError::conflict(
            "The project environment changed. Recheck before installing dependencies.",
        ));
    }
    let action = finding
        .actions
        .iter()
        .find(|a| a.id == body.action)
        .ok_or_else(|| {
            ApiError::bad_request("This environment action is not offered for the current finding.")
        })?;
    let session = state
        .terminals
        .open(hick_term::SessionSpec {
            title: format!("{}: {}", finding.manager, action.label),
            cwd: std::path::PathBuf::from(&finding.project),
            argv: action.argv.clone(),
            monitor: false,
        })
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    let summary = session.summary();
    cache.findings = findings;
    cache.running.insert(body.project.clone(), session.clone());
    let environments = state.environments.clone();
    tokio::spawn(async move {
        while session.exit_code().is_none() {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        let mut cache = environments.cache.lock().await;
        cache.running.remove(&body.project);
        cache.checked = None;
        state.lsp.refresh_environments();
    });
    Ok(Json(summary))
}

</storycode:copy>
<a id="f-532a6a243ee2-ChooseManager"></a>
### `ChooseManager`

<storycode:copy id="f-532a6a243ee2-ChooseManager">
#[derive(Deserialize)]
pub struct ChooseManager {
    project: String,
    manager: String,
    revision: String,
    choice: String,
}

</storycode:copy>
<a id="f-532a6a243ee2-choose"></a>
### `choose`

<storycode:copy id="f-532a6a243ee2-choose">
pub async fn choose(
    State(state): State<LocalState>,
    Json(body): Json<ChooseManager>,
) -> ApiResult<Json<serde_json::Value>> {
    let mut cache = state.environments.cache.lock().await;
    let findings = scan(state.index.root(), &state.environments.host).await?;
    let f = findings
        .iter()
        .find(|f| f.project == body.project && f.manager == body.manager)
        .ok_or_else(|| ApiError::not_found("This project was not discovered in the workspace."))?;
    if f.revision != body.revision || !f.manager_choices.contains(&body.choice) {
        return Err(ApiError::conflict(
            "This package-manager choice is no longer offered. Recheck the project.",
        ));
    }
    let (path, mut values) = choices(Path::new(&body.project))?;
    values.insert(body.manager, body.choice);
    super::store::write_atomic(&path, &serde_json::to_vec(&values).unwrap())
        .map_err(|e| ApiError::unavailable(e.to_string()))?;
    cache.checked = None;
    Ok(Json(serde_json::json!({"ok":true})))
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/serve/environments.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/serve/environments.rs"><storycode:paste select="#f-532a6a243ee2-Environments"/><storycode:paste select="#f-532a6a243ee2-impl-Environments"/><storycode:paste select="#f-532a6a243ee2-Cache"/><storycode:paste select="#f-532a6a243ee2-choices"/><storycode:paste select="#f-532a6a243ee2-InspectQuery"/><storycode:paste select="#f-532a6a243ee2-Status"/><storycode:paste select="#f-532a6a243ee2-scan"/><storycode:paste select="#f-532a6a243ee2-inspect"/><storycode:paste select="#f-532a6a243ee2-RunAction"/><storycode:paste select="#f-532a6a243ee2-act"/><storycode:paste select="#f-532a6a243ee2-ChooseManager"/><storycode:paste select="#f-532a6a243ee2-choose"/></storycode:file>

</storycode:doc>
