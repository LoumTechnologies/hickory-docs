//! Discovery starts no process. Run uses the same resolved profile as Debug.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use crate::debug_sessions::{ProfileSelection, Registry};
use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
pub struct Query {
    pub uri: String,
    #[serde(default = "current")]
    pub project: String,
    pub program: Option<String>,
    pub expected_source: Option<String>,
    pub expected_document: Option<String>,
}
fn current() -> String {
    "new".into()
}
fn context(
    state: &LocalState,
    query: &Query,
) -> anyhow::Result<crate::debug_sessions::LaunchContext> {
    let context = Registry::with_projects(state.lsp.projects(), state.index.root().into())
        .launch_context(
            state.index.root(),
            &query.uri,
            &query.project,
            query.program.as_deref(),
        )?;
    context.check_expected(
        query.expected_source.as_deref(),
        query.expected_document.as_deref(),
    )?;
    Ok(context)
}
pub async fn list(
    State(state): State<LocalState>,
    Json(query): Json<Query>,
) -> ApiResult<Json<Value>> {
    let context = context(&state, &query).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    let profiles = context
        .profiles()
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(
        json!({"profiles":profiles,"fingerprint":context.fingerprint(),"viewed":context.viewed,"workspace":state.index.root(),"configuration":crate::launch_profiles::CONFIG}),
    ))
}
#[derive(Deserialize)]
pub struct Run {
    #[serde(flatten)]
    pub context: Query,
    pub profile: ProfileSelection,
}
pub async fn run(State(state): State<LocalState>, Json(body): Json<Run>) -> ApiResult<Json<Value>> {
    let result = (|| -> anyhow::Result<_> {
        let context = context(&state, &body.context)?;
        let profile = context.selected(&body.profile)?;
        let (root, scratch) = context.prepare()?;
        open_run(&state, &profile, &context.root, &root, scratch, None)
    })();
    result
        .map(|s| Json(json!(s)))
        .map_err(|e| ApiError::unprocessable(format!("{e:#}")))
}

/// A terminal is the single Run mechanism for both profiles and test gutters.
pub(super) fn open_run(
    state: &LocalState,
    profile: &crate::launch_profiles::Profile,
    workspace: &std::path::Path,
    root: &std::path::Path,
    scratch: Option<tempfile::TempDir>,
    title: Option<String>,
) -> anyhow::Result<hick_term::SessionSummary> {
    let cwd = crate::debug_sessions::safe_cwd(root, &profile.definition.cwd)?;
    let mut argv = profile.run_argv(root).map_err(anyhow::Error::msg)?;
    if argv.first().is_some_and(|s| s == "python3")
        && let Some(python) = hick_project_env::python_interpreter(workspace, workspace)
    {
        argv[0] = python.to_string_lossy().into_owned();
    }
    let command = terminal_command(profile, &argv);
    let session = state.terminals.open(hick_term::SessionSpec {
        title: title.unwrap_or_else(|| format!("Run {}", profile.definition.name)),
        cwd,
        argv: command,
        monitor: false,
    })?;
    let summary = session.summary();
    // The captured tree lives until the actual terminal child exits, even
    // if the pane or its originating comparison is closed.
    tokio::spawn(async move {
        let _scratch = scratch;
        loop {
            if session.exit_code().is_some() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    Ok(summary)
}

fn terminal_command(profile: &crate::launch_profiles::Profile, argv: &[String]) -> Vec<String> {
    let d = &profile.definition;
    #[cfg(not(windows))]
    {
        let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        let command = |args: &[String]| args.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ");
        let mut script = "set -e\n".to_owned();
        for (key, value) in &d.env {
            script.push_str(&format!("export {key}={}\n", quote(value)));
        }
        if !d.build.is_empty() {
            script.push_str(&format!("{}\n", command(&d.build)));
        }
        script.push_str(&format!("exec {}", command(argv)));
        vec!["sh".into(), "-c".into(), script]
    }
    #[cfg(windows)]
    {
        let quote = |s: &str| format!("'{}'", s.replace('\'', "''"));
        let command = |args: &[String]| {
            format!(
                "& {}",
                args.iter().map(|s| quote(s)).collect::<Vec<_>>().join(" ")
            )
        };
        let mut script = "$ErrorActionPreference='Stop';\n".to_owned();
        for (key, value) in &d.env {
            script.push_str(&format!("$env:{key}={};\n", quote(value)));
        }
        if !d.build.is_empty() {
            script.push_str(&format!(
                "{}; if ($LASTEXITCODE -ne 0) {{ exit $LASTEXITCODE }};\n",
                command(&d.build)
            ));
        }
        script.push_str(&format!("{}; exit $LASTEXITCODE", command(argv)));
        vec![
            "powershell.exe".into(),
            "-NoProfile".into(),
            "-Command".into(),
            script,
        ]
    }
}
