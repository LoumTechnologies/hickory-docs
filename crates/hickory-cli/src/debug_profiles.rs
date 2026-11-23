//! Resolve a profile independently from the source whose breakpoints are primary.
use super::*;
use crate::launch_profiles::{self, LaunchInputs, PhysicalSources, Profile};
use hick_lang::ProjectEntry;
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
    pub inputs: LaunchInputs,
    pub root: PathBuf,
    pub viewed: String,
    identity: Identity,
}
enum Identity {
    Captured(String),
    Document(PathBuf, Box<hick_lsp::document::HickDocumentState>, String),
    Fence(PathBuf, String),
    Plain(PathBuf),
}
impl LaunchContext {
    pub fn notices(&self) -> &[String] {
        self.inputs.inventory().notices()
    }
    pub fn fingerprint(&self) -> String {
        let mut h = Sha256::new();
        h.update(match &self.inputs {
            LaunchInputs::Workspace(_) => b"physical".as_slice(),
            LaunchInputs::Constructed(_) => b"constructed".as_slice(),
        });
        for (path, entry) in self.inputs.inventory().entries() {
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
        if let Identity::Document(_, _, raw_source) | Identity::Fence(_, raw_source) = &self.identity {
            h.update(raw_source.as_bytes());
        }
        format!("{:x}", h.finalize())
    }

    pub fn check_expected(&self, code: Option<&str>, document: Option<&str>) -> Result<()> {
        if let Some(code) = code {
            anyhow::ensure!(
                self.inputs
                    .inventory()
                    .get(&self.viewed)
                    .is_some_and(|e| e.bytes.as_ref() == code.as_bytes()),
                "The saved file differs from the displayed code. Save or reload it before choosing a launch profile."
            );
        }
        if let (Some(expected), Identity::Document(_, _, raw_source) | Identity::Fence(_, raw_source)) = (document, &self.identity) {
            anyhow::ensure!(
                raw_source == expected,
                "The document changed. Save or reload it before choosing a launch profile."
            );
        }
        Ok(())
    }
    pub fn profiles(&self) -> Result<Vec<Profile>> {
        if matches!(self.identity, Identity::Fence(_, _)) {
            return Ok(launch_profiles::fence_profiles(&self.viewed));
        }
        launch_profiles::discover(self.inputs.inventory()).map_err(anyhow::Error::msg)
    }
    pub fn selected(&self, selection: &ProfileSelection) -> Result<Profile> {
        anyhow::ensure!(
            self.fingerprint() == selection.fingerprint,
            "The project changed after choosing a launch profile. Open Launch profiles and choose again."
        );
        if matches!(self.identity, Identity::Fence(_, _)) {
            return self.profiles()?.into_iter().find(|profile| profile.definition.id == selection.id)
                .context("Choose a current Run or Debug profile for this fence.");
        }
        launch_profiles::select(self.inputs.inventory(), Some(&selection.id)).map_err(anyhow::Error::msg)
    }
    pub fn prepare(&self) -> Result<(PathBuf, Option<tempfile::TempDir>)> {
        if matches!(self.identity, Identity::Plain(_)) {
            return Ok((self.root.clone(), None));
        }
        let scratch = tempfile::tempdir()?;
        let tree = self
            .inputs
            .project()
            .context("Launch inputs are not a constructed project")?;
        hick_lsp::write_tree(scratch.path(), tree).map_err(anyhow::Error::msg)?;
        Ok((scratch.path().to_path_buf(), Some(scratch)))
    }
    fn mapping(&self, root: &Path) -> Result<Mapping> {
        match &self.identity {
            Identity::Captured(id) => Mapping::captured(
                self.inputs
                    .project()
                    .context("Captured launch has no constructed project")?,
                root,
                id,
                &self.viewed,
            ),
            Identity::Document(path, state, raw_source) => {
                Mapping::for_contributor(path, &self.root, state, root, raw_source)
            }
            Identity::Fence(path, source) => Mapping::authored_fence(
                path, source, self.inputs.project().context("Missing fence program")?, root, &self.viewed,
            ),
            Identity::Plain(path) => Ok(Mapping::identity(path).with_plain_sources(
                root,
                self.inputs
                    .inventory()
                    .entries()
                    .iter()
                    .filter(|(_, entry)| entry.kind == hick_lang::EntryKind::File)
                    .map(|(path, entry)| (path.as_str(), entry.bytes.as_ref())),
            )),
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
            anyhow::ensure!(capture.recorded_fence.is_none(),
                "A recorded message uses its own pinned fence Debug action. Return to the message to launch it.");
            return Ok(LaunchContext {
                inputs: LaunchInputs::Constructed(capture.tree.clone()),
                root: root.into(),
                viewed: path,
                identity: Identity::Captured(id),
            });
        }
        let path = source_document(root, uri)?;
        if is_document(&path) {
            let source = std::fs::read_to_string(&path)?;
            let endpoint =
                hick_lsp::ProjectEndpoint::parse(selection).map_err(anyhow::Error::msg)?;
            if let Some(line) = viewed.and_then(|target| target.strip_prefix("fence:")) {
                anyhow::ensure!(selection == "new" && source.len() <= 1024 * 1024,
                    "Choose a fence in a current saved document of at most 1 MiB.");
                let line: usize = line.parse().context("Choose a code fence's opening line again.")?;
                let start = source.match_indices('\n').nth(line).map(|(at, _)| at + 1)
                    .context("Choose a complete code fence's opening line.")?;
                let region = hick_lang::source_context::source_code_regions(&source)
                    .map_err(anyhow::Error::msg)?.into_iter()
                    .find(|region| region.kind == "fence" && region.from == start)
                    .context("Choose a complete ordinary code fence's opening line.")?;
                let extension = match region.language.as_deref().unwrap_or("").to_ascii_lowercase().as_str() {
                    "py" | "python" | "python3" => "py",
                    "js" | "javascript" | "node" => "js",
                    "ts" | "typescript" => "ts",
                    "rs" | "rust" => "rs",
                    "c" => "c",
                    "cpp" | "c++" | "cxx" => "cpp",
                    "go" | "golang" => "go",
                    "cs" | "csharp" | "c#" => "cs",
                    "rb" | "ruby" => "rb",
                    _ => anyhow::bail!("This fence has no standalone runner. Choose a supported language."),
                };
                let (code, origins) = region.literal_fence(&source, "fence.md", 20_000).map_err(anyhow::Error::msg)?;
                anyhow::ensure!(!code.trim().is_empty(), "Choose a fence containing a program.");
                let program = format!("fence.{extension}");
                let mut entry = ProjectEntry::file(code.into_bytes());
                entry.origins = origins;
                let mut tree = hick_lang::ProjectTree::default();
                tree.insert(&program, entry).map_err(anyhow::Error::msg)?;
                return Ok(LaunchContext {
                    inputs: LaunchInputs::Constructed(tree),
                    root: path.parent().unwrap_or(root).into(),
                    viewed: program,
                    identity: Identity::Fence(path, source),
                });
            }
            let (project_root, state) = crate::project_documents::debug_launch_state(root, &path, &source, &endpoint)?;
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
                inputs: LaunchInputs::Constructed(tree),
                root: project_root,
                viewed: target,
                identity: Identity::Document(path, Box::new(state), source),
            });
        }
        // Workspace discovery respects gitignore, never follows directory symlinks,
        // and reads source/manifests only. It does not materialize a second workspace.
        let mut tree = PhysicalSources::default();
        let lockfiles: std::collections::HashSet<_> = hick_project_env::providers::node::managers()
            .iter().flat_map(|manager| manager.lockfiles.iter().map(String::as_str)).collect();
        for item in ignore::WalkBuilder::new(root)
            .hidden(false)
            .follow_links(false)
            .build()
        {
            let item = match item {
                Ok(item) => item,
                Err(error) => {
                    tree.skip(format!("Some launch inputs could not be observed: {error}"));
                    continue;
                }
            };
            if !item.file_type().is_some_and(|t| t.is_file()) {
                continue;
            }
            let relative_path = item.path().strip_prefix(root)?;
            let Some(relative) = relative_path.to_str() else {
                tree.skip(format!(
                    "Skipped a non-UTF-8 physical source name: {:?}",
                    relative_path
                ));
                continue;
            };
            #[cfg(windows)]
            let relative = relative.replace('\\', "/");
            if let Err(error) = launch_profiles::inventory::validate_workspace_path(&relative) {
                tree.skip(error);
                continue;
            }
            let relevant = hick_dap::language_of(std::path::Path::new(&relative)).is_some()
                || [".py", ".js", ".mjs", ".cjs", ".rs", ".go", ".cs", ".csproj"]
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
            if !relevant && item.file_name().to_str().is_some_and(|name| lockfiles.contains(name)) {
                // Launch discovery needs only the lockfile's presence. Include
                // the exact name in the capture without reading lockfile bytes.
                tree.insert(&relative, ProjectEntry::file(Vec::new()))
                    .map_err(anyhow::Error::msg)?;
            } else if relevant {
                match std::fs::read(item.path()) {
                    Ok(bytes) => {
                        tree.insert(&relative, ProjectEntry::file(bytes))
                            .map_err(anyhow::Error::msg)?;
                    }
                    Err(error) => {
                        tree.skip(format!(
                            "Source {relative:?} was not captured for launch discovery: {error}"
                        ));
                    }
                }
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
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("The viewed source name is not UTF-8"))?
            .to_owned();
        #[cfg(windows)]
        let viewed = viewed.replace('\\', "/");
        tree.insert(&viewed, ProjectEntry::file(std::fs::read(&path)?))
            .map_err(anyhow::Error::msg)?;
        Ok(LaunchContext {
            inputs: LaunchInputs::Workspace(tree),
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
        self.ensure_session_namespace()?;
        let profile = context.selected(selection)?;
        anyhow::ensure!(
            profile.can_debug,
            "{} is a Run-only profile. Choose Run, or an application profile with a debug adapter.",
            profile.definition.name
        );
        if let Some(endpoint) = &profile.definition.attach {
            // Attach owns no program or scratch directory and grants no source
            // revision proof. Existing frame/source readers remain read-only.
            let mapping = context.mapping(&context.root)?;
            on_build(BuildOutput::Note(format!(
                "Attach {} · loopback port {} · external source revision unverified",
                profile.definition.name, endpoint.port
            )));
            let session = Session::attach_to_endpoint(endpoint.port,
                endpoint.arguments.clone(), Arc::new(mapping)).await?;
            return self.register(session, None, Vec::new()).await;
        }
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
        let install_root = self.host_root.as_deref().unwrap_or(&context.root);
        let adapter = hick_dap::adapter_for(&entry, install_root)?;
        let program = if adapter.hosted {
            anyhow::ensure!(profile.definition.artifact.is_none(),
                "Java Debug resolves the main class from the selected source. Remove the binary artifact from this profile.");
            entry.clone()
        } else if let Some(artifact) = &profile.definition.artifact {
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
        if adapter.hosted {
            extra["args"] = serde_json::json!(crate::java_debug::launch_arguments(&profile.definition.args));
        }
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
        let (session, statuses) = Self::start_discovered_session(
            Launch {
                transport: adapter.transport,
                multi_session: adapter.multi_session,
                adapter_extra: adapter.launch_extra.clone(),
                adapter: adapter.command,
                program,
                cwd,
                extra,
            },
            adapter.hosted.then_some((root.as_path(), install_root)),
            mapping,
            breakpoints,
            sources,
            on_build,
        )
        .await?;
        self.register(session, scratch, statuses).await
    }
}
pub(crate) fn source_document(root: &Path, source: &str) -> Result<PathBuf> {
    if source.starts_with("hick-project:///") {
        return Ok(PathBuf::from(source));
    }
    let relative = if let Some(encoded) = source.strip_prefix("hick:///") {
        anyhow::ensure!(
            !encoded.contains(['?', '#']),
            "Encode literal query/fragment characters in the source URI, then request Debug again."
        );
        let input = encoded.as_bytes();
        let mut bytes = Vec::with_capacity(input.len());
        let mut at = 0;
        while at < input.len() {
            if input[at] == b'%' {
                let digits = input.get(at + 1..at + 3).context(
                    "Malformed source URI escape. Encode its literal percent signs and try again.",
                )?;
                let hex = |b: u8| (b as char).to_digit(16).map(|n| n as u8);
                let byte = hex(digits[0]).zip(hex(digits[1])).map(|(a,b)|a*16+b).context("Malformed source URI escape. Encode its literal percent signs and try again.")?;
                bytes.push(byte);
                at += 3;
            } else {
                bytes.push(input[at]);
                at += 1;
            }
        }
        String::from_utf8(bytes)
            .context("Source URI is not valid UTF-8. Choose a workspace source again.")?
    } else {
        source.to_owned()
    };
    launch_profiles::inventory::validate_workspace_path(&relative).map_err(anyhow::Error::msg)?;
    let path = root.join(&relative).canonicalize().with_context(|| {
        format!("Cannot resolve source {relative}. Save or reopen it before Debug.")
    })?;
    anyhow::ensure!(
        path.to_str().is_some(),
        "The adapter cannot represent this exact source path as UTF-8. Choose a supported source path."
    );
    anyhow::ensure!(
        path.starts_with(root.canonicalize()?),
        "Debug source is outside the open folder. Choose a workspace source."
    );
    anyhow::ensure!(
        std::fs::symlink_metadata(&path)?.is_file(),
        "Debug source must be a regular file. Choose a source file."
    );
    Ok(path)
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
