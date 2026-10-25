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
pub(super) struct Plan {
    profile: Profile,
    report: PathBuf,
}
type Sessions = Mutex<BTreeMap<String, Option<oneshot::Sender<()>>>>;
fn sessions() -> &'static Sessions {
    static S: OnceLock<Sessions> = OnceLock::new();
    S.get_or_init(|| Mutex::new(BTreeMap::new()))
}
fn key(root: &Path, id: &str) -> String {
    format!("{}:{id}", root.display())
}
pub(super) fn active(root: &Path, id: &str) -> bool {
    sessions().lock().unwrap().contains_key(&key(root, id))
}
pub(super) fn stop(root: &Path, id: &str) -> bool {
    sessions()
        .lock()
        .unwrap()
        .get_mut(&key(root, id))
        .and_then(Option::take)
        .is_some_and(|s| s.send(()).is_ok())
}
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
        ["chrome-cpu", "speedscope", "cprofile-json"].contains(&original.format.as_str()),
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
