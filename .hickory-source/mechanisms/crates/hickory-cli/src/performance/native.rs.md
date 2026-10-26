<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/src/performance/native.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7c2a7d2033c5"></a>
## `crates/hickory-cli/src/performance/native.rs`

Related contracts: [Profiles keep their measured scope](../../../../../guarantees/execution/profiles-keep-their-measured-scope/contract.md).

<a id="f-7c2a7d2033c5-Profile"></a>
### `Profile`

<storycode:copy id="f-7c2a7d2033c5-Profile">
use super::*;
use std::sync::{Mutex, OnceLock};
use tokio::sync::oneshot;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Profile {
    pub argv: Vec<String>,
    pub format: String,
    pub report: String,
    pub collector: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub version_argv: Vec<String>,
    #[serde(default)]
    pub finish: Vec<String>,
}
</storycode:copy>
<a id="f-7c2a7d2033c5-Plan"></a>
### `Plan`

<storycode:copy id="f-7c2a7d2033c5-Plan">
pub(super) struct Plan {
    profile: Profile,
    report: PathBuf,
}
</storycode:copy>
<a id="f-7c2a7d2033c5-Sessions"></a>
### `Sessions`

<storycode:copy id="f-7c2a7d2033c5-Sessions">
type Sessions = Mutex<BTreeMap<String, Option<oneshot::Sender<()>>>>;
</storycode:copy>
<a id="f-7c2a7d2033c5-sessions"></a>
### `sessions`

<storycode:copy id="f-7c2a7d2033c5-sessions">
fn sessions() -> &'static Sessions {
    static S: OnceLock<Sessions> = OnceLock::new();
    S.get_or_init(|| Mutex::new(BTreeMap::new()))
}
</storycode:copy>
<a id="f-7c2a7d2033c5-key"></a>
### `key`

<storycode:copy id="f-7c2a7d2033c5-key">
fn key(root: &Path, id: &str) -> String {
    format!("{}:{id}", root.display())
}
</storycode:copy>
<a id="f-7c2a7d2033c5-active"></a>
### `active`

<storycode:copy id="f-7c2a7d2033c5-active">
pub(super) fn active(root: &Path, id: &str) -> bool {
    sessions().lock().unwrap().contains_key(&key(root, id))
}
</storycode:copy>
<a id="f-7c2a7d2033c5-stop"></a>
### `stop`

<storycode:copy id="f-7c2a7d2033c5-stop">
pub(super) fn stop(root: &Path, id: &str) -> bool {
    sessions()
        .lock()
        .unwrap()
        .get_mut(&key(root, id))
        .and_then(Option::take)
        .is_some_and(|s| s.send(()).is_ok())
}
</storycode:copy>
<a id="f-7c2a7d2033c5-config"></a>
### `config`

<storycode:copy id="f-7c2a7d2033c5-config">
pub(super) fn config(project: &Path) -> Result<BTreeMap<String, Profile>> {
    let p = project.join(".hick-profile.json");
    if !p.exists() {
        return Ok(BTreeMap::new());
    }
    Ok(serde_json::from_str(&std::fs::read_to_string(safe_path(
        project,
        ".hick-profile.json",
    )?)?)?)
}
</storycode:copy>
<a id="f-7c2a7d2033c5-plan"></a>
### `plan`

<storycode:copy id="f-7c2a7d2033c5-plan">
pub(super) fn plan(
    root: &Path,
    project: &Path,
    request: &Request,
    run: &mut Record,
) -> Result<Plan> {
    let profiles = config(project)?;
    let original = profiles
        .get(
            request
                .profile
                .as_deref()
                .context("Select a project profiling profile")?,
        )
        .context("Unknown profiling profile")?;
    ensure!(!original.argv.is_empty(), "Empty profiling command");
    ensure!(
        ["chrome-cpu", "speedscope", "cprofile-json", "firefox"]
            .contains(&original.format.as_str()),
        "Unsupported profiling format"
    );
    run.identity.collector = original.collector.clone();
    run.identity.version = if original.version.is_empty() {
        "unknown"
    } else {
        &original.version
    }
    .into();
    run.identity.configuration = serde_json::to_string(original)?;
    let base = |s: &str| {
        s.replace("{root}", &root.to_string_lossy())
            .replace("{project}", &project.to_string_lossy())
            .replace("{run}", &run.id)
    };
    let rel = base(&original.report);
    ensure!(
        Path::new(&rel)
            .components()
            .all(|c| matches!(c, std::path::Component::Normal(_))),
        "Report must be workspace-relative without parent traversal"
    );
    let mut ancestor = root.join(&rel);
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .context("No report ancestor")?
            .to_path_buf();
    }
    ensure!(
        ancestor.canonicalize()?.starts_with(root),
        "Report symlink escapes workspace"
    );
    let report = root.join(&rel);
    ensure!(
        !report.exists(),
        "Profile report already exists; use a unique {{run}} destination"
    );
    store::dir(root)?;
    std::fs::create_dir_all(report.parent().unwrap())?;
    let mut profile = original.clone();
    for argv in [
        &mut profile.argv,
        &mut profile.finish,
        &mut profile.version_argv,
    ] {
        for arg in argv {
            *arg = base(arg).replace("{report}", &report.to_string_lossy());
        }
    }
    Ok(Plan { profile, report })
}
</storycode:copy>
<a id="f-7c2a7d2033c5-start"></a>
### `start`

<storycode:copy id="f-7c2a7d2033c5-start">
pub(super) fn start(root: PathBuf, project: PathBuf, mut run: Record, plan: Plan) -> Result<()> {
    let (tx, rx) = oneshot::channel();
    sessions()
        .lock()
        .unwrap()
        .insert(key(&root, &run.id), Some(tx));
    tokio::spawn(async move {
        let result = collect(&root, &project, &mut run, plan, rx).await;
        store::finish(&root, &mut run, result);
        sessions().lock().unwrap().remove(&key(&root, &run.id));
    });
    Ok(())
}
</storycode:copy>
<a id="f-7c2a7d2033c5-collect"></a>
### `collect`

<storycode:copy id="f-7c2a7d2033c5-collect">
async fn collect(
    root: &Path,
    project: &Path,
    run: &mut Record,
    plan: Plan,
    stop: oneshot::Receiver<()>,
) -> Result<()> {
    if !plan.profile.version_argv.is_empty() {
        let output = crate::coverage::execute(&plan.profile.version_argv, project, None).await?;
        ensure!(
            output.code == Some(0),
            "Profiler version failed: {}",
            output.text
        );
        run.identity.version = output.text.trim().into();
    }
    let output = crate::coverage::execute(&plan.profile.argv, project, Some(stop)).await?;
    run.output = output.text;
    let code = output.code;
    if !plan.profile.finish.is_empty() {
        let finish = crate::coverage::execute(&plan.profile.finish, project, None).await?;
        run.output.push_str(&finish.text);
        ensure!(finish.code == Some(0), "Profile export failed");
    }
    let path = safe_path(root, &relative(root, &plan.report))?;
    ensure!(
        std::fs::metadata(&path)?.len() <= 32 * 1024 * 1024,
        "Profile too large"
    );
    let raw = std::fs::read_to_string(path)?;
    run.report = Some(hick_profile::parse(&plan.profile.format, &raw)?);
    resolve::bind(root, run, None)?;
    run.status = if code == Some(0) {
        "completed"
    } else {
        "partial"
    }
    .into();
    run.environment["process_exit_code"] = json!(code);
    Ok(())
}
</storycode:copy>

### Assemble `crates/hickory-cli/src/performance/native.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/src/performance/native.rs"><storycode:paste select="#f-7c2a7d2033c5-Profile"/><storycode:paste select="#f-7c2a7d2033c5-Plan"/><storycode:paste select="#f-7c2a7d2033c5-Sessions"/><storycode:paste select="#f-7c2a7d2033c5-sessions"/><storycode:paste select="#f-7c2a7d2033c5-key"/><storycode:paste select="#f-7c2a7d2033c5-active"/><storycode:paste select="#f-7c2a7d2033c5-stop"/><storycode:paste select="#f-7c2a7d2033c5-config"/><storycode:paste select="#f-7c2a7d2033c5-plan"/><storycode:paste select="#f-7c2a7d2033c5-start"/><storycode:paste select="#f-7c2a7d2033c5-collect"/></storycode:file>

</storycode:doc>
