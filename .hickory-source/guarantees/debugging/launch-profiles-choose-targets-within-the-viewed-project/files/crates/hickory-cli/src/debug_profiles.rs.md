<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# crates/hickory-cli/src/debug_profiles.rs

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Launch profiles choose targets within the viewed project](../../../../contract.md)

Captured SHA-256: `41b284e0e33f68f57f859e3144ec9965c6ef167ddf8413488dbe1b3735cf092b`.

<book:entry path="crates/hickory-cli/src/debug_profiles.rs">
//! Resolve a profile independently from the source whose breakpoints are primary.
use super::*;
use crate::launch_profiles::{self, Profile};
use hick_lang::{ProjectEntry, ProjectTree};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSelection {
    pub id: String,
    pub fingerprint: String,
}

pub struct LaunchContext {
    pub tree: ProjectTree,
    pub root: PathBuf,
    pub viewed: String,
    identity: Identity,
}
enum Identity {
    Captured(String),
    Document(PathBuf, Box<hick_lsp::document::HickDocumentState>, String),
    Plain(PathBuf),
}
impl LaunchContext {
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        for (path, entry) in self.tree.entries() {
            h.update((path.len() as u64).to_le_bytes());
            h.update(path.as_bytes());
            h.update((entry.bytes.len() as u64).to_le_bytes());
            h.update(&entry.bytes);
            h.update([
                entry.executable as u8,
                matches!(entry.kind, hick_lang::EntryKind::File) as u8,
            ]);
            for origin in &entry.origins {
                h.update(serde_json::to_vec(origin).expect("byte origins are serializable"));
            }
        }
        if let Identity::Document(_, _, raw_source) = &self.identity {
            h.update(raw_source.as_bytes());
        }
        format!("{:x}", h.finalize())
    }

    pub fn check_expected(&self, code: Option<&str>, document: Option<&str>) -> Result<()> {
        if let Some(code) = code {
            anyhow::ensure!(
                self.tree
                    .get(&self.viewed)
                    .is_some_and(|e| e.bytes.as_ref() == code.as_bytes()),
                "The saved file differs from the displayed code. Save or reload it before choosing a launch profile."
            );
        }
        if let (Some(expected), Identity::Document(_, _, raw_source)) = (document, &self.identity) {
            anyhow::ensure!(
                raw_source == expected,
                "The document changed. Save or reload it before choosing a launch profile."
            );
        }
        Ok(())
    }
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        launch_profiles::discover(&self.tree).map_err(anyhow::Error::msg)
    }
    pub fn selected(&self, selection: &ProfileSelection) -> Result<Profile> {
        anyhow::ensure!(
            self.fingerprint() == selection.fingerprint,
            "The project changed after choosing a launch profile. Open Launch profiles and choose again."
        );
        launch_profiles::select(&self.tree, Some(&selection.id)).map_err(anyhow::Error::msg)
    }
    pub fn prepare(&self) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
        if matches!(self.identity, Identity::Plain(_)) {
            return Ok((self.root.clone(), None));
        }
        let scratch = tempfile::tempdir()?;
        hick_lsp::write_tree(scratch.path(), &self.tree).map_err(anyhow::Error::msg)?;
        Ok((scratch.path().to_path_buf(), Some(scratch)))
    }
    fn mapping(&self, root: &Path) -> Result<Mapping> {
        match &self.identity {
            Identity::Captured(id) => Mapping::captured(&self.tree, root, id, &self.viewed),
            Identity::Document(path, state, raw_source) => {
                Ok(Mapping::for_state(path, state, root)?.with_observation_source(raw_source))
            }
            Identity::Plain(path) => Ok(Mapping::identity(path)),
        }
    }
}
impl Registry {
    pub fn launch_context(
        &self,
        root: &Path,
        uri: &str,
        selection: &str,
        viewed: Option<&str>,
    ) -> Result<LaunchContext> {
        if uri.starts_with("hick-project:///") {
            let (id, path, capture) = self
                .projects
                .get(&tower_lsp::lsp_types::Url::parse(uri)?)
                .map_err(anyhow::Error::msg)?;
            return Ok(LaunchContext {
                tree: capture.tree.clone(),
                root: root.into(),
                viewed: path,
                identity: Identity::Captured(id),
            });
        }
        let relative = uri.strip_prefix("hick:///").unwrap_or(uri);
        hick_lang::validate_project_path(relative).map_err(anyhow::Error::msg)?;
        let path = root.join(relative).canonicalize()?;
        anyhow::ensure!(
            path.starts_with(root.canonicalize()?),
            "Launch source is outside the open folder. Choose a file inside this workspace."
        );
        if is_document(&path) {
            let source = std::fs::read_to_string(&path)?;
            let endpoint =
                hick_lsp::ProjectEndpoint::parse(selection).map_err(anyhow::Error::msg)?;
            let state = crate::project_documents::state_at(&path, &source, &endpoint)?;
            let mut tree = state.selected_project.clone().unwrap_or_default();
            if state.selected_project.is_none() {
                for file in &state.virtual_files {
                    tree.insert(
                        &file.path,
                        ProjectEntry::file(file.written_content().into_bytes()),
                    )
                    .map_err(anyhow::Error::msg)?;
                }
            }
            let target = if matches!(
                endpoint,
                hick_lsp::ProjectEndpoint::LatestFile { .. }
                    | hick_lsp::ProjectEndpoint::LatestCode { .. }
            ) {
                hick_lsp::debug_selection(
                    state
                        .evaluated_project
                        .as_ref()
                        .context("Missing project")?,
                    &endpoint,
                )
                .map_err(anyhow::Error::msg)?
                .1
            } else {
                viewed
                    .context("Choose a file block before selecting a launch profile")?
                    .into()
            };
            return Ok(LaunchContext {
                tree,
                root: path.parent().unwrap_or(root).into(),
                viewed: target,
                identity: Identity::Document(path, Box::new(state), source),
            });
        }
        // Workspace discovery respects gitignore, never follows directory symlinks,
        // and reads source/manifests only. It does not materialize a second workspace.
        let mut tree = ProjectTree::default();
        for item in ignore::WalkBuilder::new(root)
            .hidden(false)
            .follow_links(false)
            .build()
        {
            let item = item?;
            if !item.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let relative = item
                .path()
                .strip_prefix(root)?
                .to_string_lossy()
                .replace('\\', "/");
            let relevant = [".py", ".js", ".mjs", ".cjs", ".rs", ".go", ".cs", ".csproj"]
                .iter()
                .any(|e| relative.ends_with(e))
                || [
                    launch_profiles::CONFIG,
                    "package.json",
                    "Cargo.toml",
                    "go.mod",
                    "pyproject.toml",
                    "pytest.ini",
                ]
                .contains(&item.file_name().to_string_lossy().as_ref());
            if relevant {
                tree.insert(&relative, ProjectEntry::file(std::fs::read(item.path())?))
                    .map_err(anyhow::Error::msg)?;
            }
        }
        // Hidden config remains available even if the user's ignore rules hide it.
        let config = root.join(launch_profiles::CONFIG);
        if config.exists() {
            anyhow::ensure!(
                std::fs::symlink_metadata(&config)?.is_file(),
                "{} must be a regular file",
                launch_profiles::CONFIG
            );
            tree.insert(
                launch_profiles::CONFIG,
                ProjectEntry::file(std::fs::read(config)?),
            )
            .map_err(anyhow::Error::msg)?;
        }
        let viewed = path
            .strip_prefix(root.canonicalize()?)?
            .to_string_lossy()
            .replace('\\', "/");
        tree.insert(&viewed, ProjectEntry::file(std::fs::read(&path)?))
            .map_err(anyhow::Error::msg)?;
        Ok(LaunchContext {
            tree,
            root: root.into(),
            viewed,
            identity: Identity::Plain(path),
        })
    }

    pub async fn start_profile(
        &self,
        context: LaunchContext,
        selection: &ProfileSelection,
        breakpoints: &[Breakpoint],
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        let profile = context.selected(selection)?;
        anyhow::ensure!(
            profile.can_debug,
            "{} is a Run-only profile. Choose Run, or an application profile with a debug adapter.",
            profile.definition.name
        );
        let (root, scratch) = context.prepare()?;
        let cwd = safe_cwd(&root, &profile.definition.cwd)?;
        run_build(&profile, &cwd, on_build).await?;
        let entry = root.join(
            profile
                .definition
                .program
                .as_ref()
                .context("Profile has no program")?,
        );
        let adapter =
            hick_dap::adapter_for(&entry, self.host_root.as_deref().unwrap_or(&context.root))?;
        let program = if let Some(artifact) = &profile.definition.artifact {
            let artifact = root.join(artifact).canonicalize().context(
                "The configured debug artifact does not exist. Fix the build or artifact path.",
            )?;
            anyhow::ensure!(
                artifact.starts_with(root.canonicalize()?),
                "The debug artifact escapes the project"
            );
            artifact
        } else if scratch.is_some() {
            hick_dap::build(&entry, &root, &context.root, on_build).await?
        } else {
            hick_dap::build_plain(&entry, &root, on_build).await?
        };
        let mut extra =
            serde_json::json!({"args":profile.definition.args,"env":profile.definition.env});
        if hick_dap::language_of(&entry) == Some("python")
            && let Some(python) = hick_project_env::python_interpreter(&context.root, &context.root)
        {
            extra["python"] = serde_json::json!(python);
        }
        on_build(BuildOutput::Note(format!(
            "Debug {} · source {} · project {}",
            profile.definition.name,
            context.viewed,
            context.fingerprint()
        )));
        let mapping = context.mapping(&root)?;
        let (session, statuses) = Session::start_with_sources(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd,
                extra,
            },
            Arc::new(mapping),
            breakpoints,
            sources,
        )
        .await?;
        Ok(self.register(session, scratch, statuses).await)
    }
}

pub(crate) fn safe_cwd(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = root.join(relative).canonicalize().with_context(|| {
        format!(
            "Working directory {relative} does not exist in this project. Fix the launch profile."
        )
    })?;
    anyhow::ensure!(
        path.starts_with(root.canonicalize()?),
        "Launch working directory escapes this project. Fix the launch profile."
    );
    Ok(path)
}
async fn run_build(
    profile: &Profile,
    cwd: &Path,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<()> {
    let argv = &profile.definition.build;
    if argv.is_empty() {
        return Ok(());
    }
    on_build(BuildOutput::Cmd(format!("{argv:?}")));
    let mut child = tokio::process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .envs(&profile.definition.env)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut out = BufReader::new(child.stdout.take().context("Missing build stdout")?).lines();
    let mut err = BufReader::new(child.stderr.take().context("Missing build stderr")?).lines();
    let (mut stdout, mut stderr) = (true, true);
    while stdout || stderr {
        tokio::select! {
            line = out.next_line(), if stdout => match line? {Some(line)=>on_build(BuildOutput::Out(line)),None=>stdout=false},
            line = err.next_line(), if stderr => match line? {Some(line)=>on_build(BuildOutput::Err(line)),None=>stderr=false},
        }
    }
    let status = child.wait().await?;
    on_build(BuildOutput::Exit(status.code().unwrap_or(-1)));
    anyhow::ensure!(
        status.success(),
        "Build for {} failed; see its output and fix the profile or project.",
        profile.definition.name
    );
    Ok(())
}
</book:entry>

</book:doc>
