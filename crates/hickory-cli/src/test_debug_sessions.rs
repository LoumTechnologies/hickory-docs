use super::*;
use std::collections::BTreeMap;

/// Saved compiler inputs and target, selected by engine admission rather than UI argv.
pub(crate) struct TestLaunch {
    pub workspace: PathBuf,
    pub workspace_id: String,
    pub rust_manifest: Option<(String, String)>,
    pub target_line: u32,
    pub target_character: u32,
    pub target_hash: String,
    pub open_sources: BTreeMap<String, String>,
    pub input: PathBuf,
    pub input_source: String,
    pub expected_sources: BTreeMap<String, String>,
    pub source_texts: BTreeMap<String, String>,
    pub construction: Option<(hick_lsp::document::HickDocumentState, PathBuf)>,
    pub target: String,
    pub language: String,
    pub filter: String,
}
impl Registry {
    pub(crate) async fn start_test_with_sources(
        &self,
        launch: TestLaunch,
        sources: &[hick_dap::SourceBreakpoints],
        on_build: &mut (dyn FnMut(BuildOutput) + Send),
    ) -> Result<(String, Arc<Live>, Vec<BreakpointStatus>)> {
        self.ensure_session_namespace()?;
        require_saved_test_workspace(&launch.workspace, &launch.workspace_id)?;
        let root_identity = launch.workspace_id.clone();
        let (root, mut scratch, program, mapping) =
            if let Some((state, entry)) = &launch.construction {
                let scratch = tempfile::tempdir()?;
                let tree = if let Some(tree) = &state.selected_project {
                    tree.clone()
                } else {
                    let mut tree = hick_lang::ProjectTree::default();
                    for file in &state.virtual_files {
                        tree.insert(
                            &file.path,
                            hick_lang::ProjectEntry::file(file.written_content().into_bytes()),
                        )
                        .map_err(anyhow::Error::msg)?;
                    }
                    tree
                };
                let base = entry.parent().unwrap_or(&launch.workspace);
                let target = launch
                    .workspace
                    .join(&launch.target)
                    .strip_prefix(base)?
                    .to_path_buf();
                anyhow::ensure!(
                    tree.get(target.to_str().context("Test target is not UTF-8")?)
                        .is_some_and(|entry| entry.kind == hick_lang::EntryKind::File),
                    "Selected test target is absent or is not a regular constructed file"
                );
                hick_lsp::write_tree(scratch.path(), &tree).map_err(anyhow::Error::msg)?;
                let program = scratch.path().join(target);
                let mapping = Mapping::for_state(entry, &state, scratch.path())?
                    .with_primary_source(&launch.input, &launch.input_source)?;
                (
                    scratch.path().to_path_buf(),
                    Some(scratch),
                    program,
                    mapping,
                )
            } else {
                let program = launch.workspace.join(&launch.target).canonicalize()?;
                anyhow::ensure!(
                    program.starts_with(&launch.workspace),
                    "Test target escaped the open workspace"
                );
                let mapping = Mapping::identity(&launch.input).with_plain_sources(
                    &launch.workspace,
                    launch
                        .source_texts
                        .iter()
                        .map(|(path, text)| (path.as_str(), text.as_bytes())),
                );
                (launch.workspace.clone(), None, program, mapping)
            };
        if let Some((relative, expected)) = &launch.rust_manifest {
            let manifest = root.join(relative);
            let bytes = std::fs::read(&manifest)?;
            anyhow::ensure!(
                crate::serve::source_tests::digest(&bytes) == *expected,
                "Constructed or saved Rust manifest changed before preparation"
            );
        }
        let mut plan = hick_plugins::testing::test_debug_command(
            &root,
            &program,
            &launch.filter,
            &launch.language,
        )
        .map_err(anyhow::Error::msg)?;
        // Discover against the source-side project environment, including a
        // generated target whose host path does not exist. Language selection
        // remains based on the selected runtime target, not Markdown input.
        let host_target = launch.workspace.join(&launch.target);
        let adapter = hick_dap::adapter_for(&host_target, &launch.workspace)?;
        if launch.language == "python"
            && let Some(python) = hick_project_env::python_interpreter(
                launch.input.parent().unwrap_or(&launch.workspace),
                &launch.workspace,
            )
        {
            plan.launch["python"] = serde_json::json!(python);
        }
        let prepared_artifact =
            if let Some(hick_plugins::testing::TestDebugPreparation::RustCargo { filter }) =
                plan.preparation.take()
            {
                if scratch.is_none() {
                    scratch = Some(tempfile::tempdir()?)
                }
                let target = scratch
                    .as_ref()
                    .unwrap()
                    .path()
                    .join(".hick-rust-test-target");
                std::fs::create_dir_all(&target)?;
                let selection = rust_test_source_proof::Selection::capture(
                    &plan.program,
                    launch.target_line,
                    launch.target_character,
                    &launch.target_hash,
                    &filter,
                    &plan.cwd,
                )?;
                let metadata_tool = rust_test_source_proof::MetadataTool::from_adapter(
                    &adapter.adapter,
                    &adapter.command,
                )?;
                Some(
                    prepare_rust_selected_test_proven(
                        &mut plan,
                        &filter,
                        &target,
                        selection,
                        metadata_tool,
                        if launch.construction.is_none() {
                            Some((&launch.workspace, &launch.open_sources))
                        } else {
                            None
                        },
                        on_build,
                    )
                    .await?,
                )
            } else {
                None
            };
        // Adapter provisioning/build preparation can be slow. Recheck every
        // admitted author immediately before creating the runtime session.
        crate::serve::source_tests::check_saved_sources(
            &launch.workspace,
            &launch.expected_sources,
        )
        .map_err(|error| anyhow::anyhow!("{}", error.detail()))?;
        require_saved_test_workspace(&launch.workspace, &root_identity)?;
        if let Some((relative, expected)) = &launch.rust_manifest {
            anyhow::ensure!(
                crate::serve::source_tests::digest(&std::fs::read(root.join(relative))?)
                    == *expected,
                "Rust manifest changed during compiler preparation"
            );
        }
        if let Some(prepared) = &prepared_artifact {
            prepared.current()?
        }
        on_build(BuildOutput::Note(format!(
            "{} · {} provider · saved contributors checked; process outcome only",
            plan.title, plan.framework
        )));
        let (session, statuses) = Session::start_with_sources(
            Launch {
                adapter: adapter.command,
                transport: adapter.transport,
                adapter_extra: adapter.launch_extra,
                multi_session: adapter.multi_session,
                program: plan.program,
                cwd: plan.cwd,
                extra: plan.launch,
            },
            Arc::new(mapping),
            &[],
            sources,
        )
        .await?;
        self.register(session, scratch, statuses).await
    }
}
fn require_saved_test_workspace(path: &Path, expected: &str) -> Result<()> {
    let actual = crate::serve::resource_inputs::RootIdentity::fingerprint(path)
        .map_err(anyhow::Error::msg)?;
    anyhow::ensure!(
        actual == expected,
        "Saved test workspace identity changed during preparation"
    );
    Ok(())
}
#[cfg(test)]
mod saved_test_root_witness {
    use super::*;
    #[test]
    fn same_path_and_same_bytes_never_substitute_a_new_physical_root() {
        let parent = tempfile::tempdir().unwrap();
        let path = parent.path().join("project");
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("lib.rs"), "#[test]fn chosen(){}").unwrap();
        let original = crate::serve::resource_inputs::RootIdentity::fingerprint(&path).unwrap();
        require_saved_test_workspace(&path, &original).unwrap();
        std::fs::rename(&path, parent.path().join("old-project")).unwrap();
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("lib.rs"), "#[test]fn chosen(){}").unwrap();
        assert!(require_saved_test_workspace(&path, &original).is_err());
    }
}

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
#[derive(Debug, Clone, PartialEq, Eq)]
struct RustTestArtifact {
    executable: PathBuf,
    source: PathBuf,
    identity: RustExecutableIdentity,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct RustExecutableIdentity {
    len: u64,
    modified: Option<std::time::SystemTime>,
    #[cfg(unix)]
    dev: u64,
    #[cfg(unix)]
    ino: u64,
}
fn rust_executable_identity(path: &Path) -> Result<RustExecutableIdentity> {
    let metadata = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "Rust executable must be an ordinary file"
    );
    #[cfg(unix)]
    use std::os::unix::fs::MetadataExt;
    Ok(RustExecutableIdentity {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        dev: metadata.dev(),
        #[cfg(unix)]
        ino: metadata.ino(),
    })
}
fn rust_artifact_current(artifact: &RustTestArtifact) -> Result<()> {
    anyhow::ensure!(
        artifact.executable.canonicalize()? == artifact.executable
            && rust_executable_identity(&artifact.executable)? == artifact.identity,
        "Rust test artifact changed after compiler admission"
    );
    Ok(())
}
fn rust_test_artifacts(
    bytes: &[u8],
    runtime: &Path,
    target: &Path,
) -> Result<Vec<RustTestArtifact>> {
    anyhow::ensure!(
        bytes.len() <= 8 * 1024 * 1024,
        "Rust test compiler output budget exceeded"
    );
    let runtime = runtime.canonicalize()?;
    let target = target.canonicalize()?;
    let mut artifacts = Vec::new();
    for line in bytes.split(|b| *b == b'\n').filter(|line| !line.is_empty()) {
        anyhow::ensure!(
            line.len() <= 1024 * 1024,
            "Rust compiler message is too large"
        );
        let value: serde_json::Value = serde_json::from_slice(line)
            .context("Rust compiler emitted a malformed JSON message")?;
        if value.get("reason").and_then(|v| v.as_str()) != Some("compiler-artifact") {
            continue;
        }
        if value.pointer("/profile/test").and_then(|v| v.as_bool()) != Some(true) {
            continue;
        }
        let Some(path) = value.get("executable").and_then(|v| v.as_str()) else {
            continue;
        };
        let executable = Path::new(path);
        anyhow::ensure!(
            executable.is_absolute(),
            "Rust test executable is not absolute"
        );
        let metadata = std::fs::symlink_metadata(executable)?;
        anyhow::ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "Rust test artifact is not a regular owned file"
        );
        let executable = executable.canonicalize()?;
        anyhow::ensure!(
            executable.starts_with(&target),
            "Rust test executable escaped its owned build target"
        );
        let path = value
            .pointer("/target/src_path")
            .and_then(|v| v.as_str())
            .context("Rust test artifact has no target source")?;
        let source = Path::new(path).canonicalize()?;
        anyhow::ensure!(
            source.starts_with(&runtime),
            "Rust test artifact belongs to another runtime root"
        );
        let identity = rust_executable_identity(&executable)?;
        rust_builtin_harness(&value, &runtime, &source)?;
        let artifact = RustTestArtifact {
            executable,
            source,
            identity,
        };
        if !artifacts.contains(&artifact) {
            artifacts.push(artifact)
        };
        anyhow::ensure!(
            artifacts.len() <= 128,
            "Rust test target count budget exceeded"
        );
    }
    anyhow::ensure!(
        !artifacts.is_empty(),
        "Cargo produced no test executable for the saved selection"
    );
    Ok(artifacts)
}
#[cfg(test)]
fn rust_harness_names(bytes: &[u8], filter: &str) -> Result<Vec<String>> {
    anyhow::ensure!(
        bytes.len() <= 1024 * 1024,
        "Rust test listing output budget exceeded"
    );
    let text = std::str::from_utf8(bytes).context("Rust test listing was not UTF-8")?;
    let mut names = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_suffix(": test") {
            anyhow::ensure!(
                name.len() <= 8192 && !name.contains('\0'),
                "Malformed Rust test identity"
            );
            if name == filter
                || name
                    .strip_suffix(filter)
                    .is_some_and(|prefix| prefix.ends_with("::"))
            {
                names.push(name.to_owned())
            }
        }
    }
    anyhow::ensure!(
        names.len() <= 1,
        "The selected Rust name matches multiple tests. Choose a qualified collected identity."
    );
    Ok(names)
}
fn rust_harness_candidates(bytes: &[u8], filter: &str) -> Result<Vec<String>> {
    anyhow::ensure!(
        bytes.len() <= 1024 * 1024,
        "Rust test listing output budget exceeded"
    );
    let text = std::str::from_utf8(bytes).context("Rust test listing was not UTF-8")?;
    let mut names = Vec::new();
    for line in text.lines() {
        if let Some(name) = line.strip_suffix(": test") {
            anyhow::ensure!(
                name.len() <= 8192 && !name.contains('\0'),
                "Malformed Rust test identity"
            );
            if name == filter
                || name
                    .strip_suffix(filter)
                    .is_some_and(|prefix| prefix.ends_with("::"))
            {
                names.push(name.to_owned())
            }
        }
    }
    Ok(names)
}
#[cfg(test)]
mod rust_test_artifact_tests {
    use super::*;
    #[test]
    fn exact_qualified_listing_never_runs_or_substitutes_a_sibling() {
        assert_eq!(
            rust_harness_names(b"module::chosen: test\nmodule::other: test\n", "chosen").unwrap(),
            vec!["module::chosen"]
        );
        assert!(rust_harness_names(b"one::chosen: test\ntwo::chosen: test\n", "chosen").is_err());
        assert!(
            rust_harness_names(b"module::unchosen: test\n", "chosen")
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            rust_harness_names(b"one::chosen: test\ntwo::chosen: test\n", "one::chosen").unwrap(),
            vec!["one::chosen"]
        );
    }
    #[test]
    fn compiler_artifacts_are_pinned_to_the_owned_runtime_and_target() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let exe = target.path().join("selected");
        std::fs::write(&exe, b"binary").unwrap();
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname=\"selected\"\nversion=\"0.1.0\"\n[lib]\npath=\"lib.rs\"\n",
        )
        .unwrap();
        let source = root.path().join("lib.rs");
        std::fs::write(&source, b"#[test]fn selected(){}").unwrap();
        let value = serde_json::json!({"reason":"compiler-artifact","profile":{"test":true},"manifest_path":root.path().join("Cargo.toml"),"executable":exe,"target":{"name":"selected","kind":["lib"],"src_path":source}});
        assert_eq!(
            rust_test_artifacts(
                &serde_json::to_vec(&value).unwrap(),
                root.path(),
                target.path()
            )
            .unwrap()
            .len(),
            1
        );
        let foreign = tempfile::tempdir().unwrap();
        assert!(
            rust_test_artifacts(
                &serde_json::to_vec(&value).unwrap(),
                foreign.path(),
                target.path()
            )
            .is_err()
        );
        assert!(
            rust_test_artifacts(
                &serde_json::to_vec(&value).unwrap(),
                root.path(),
                foreign.path()
            )
            .is_err()
        );
        std::fs::remove_file(exe).unwrap();
        assert!(
            rust_test_artifacts(
                &serde_json::to_vec(&value).unwrap(),
                root.path(),
                target.path()
            )
            .is_err()
        );
    }
    #[test]
    fn compiler_diagnostics_cannot_impersonate_a_harness() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        for bytes in [
            b"not json".as_slice(),
            br#"{"reason":"compiler-message","executable":"/tmp/fake","profile":{"test":true}}"#
                .as_slice(),
        ] {
            assert!(rust_test_artifacts(bytes, root.path(), target.path()).is_err())
        }
        assert!(rust_harness_names(&vec![b'x'; 1024 * 1024 + 1], "chosen").is_err());
    }
}
fn rust_cargo_tool() -> Result<(PathBuf, PathBuf, RustExecutableIdentity)> {
    let path = std::env::var_os("PATH").context("Cargo is not installed on the engine PATH")?;
    for dir in std::env::split_paths(&path) {
        if !dir.is_absolute() {
            continue;
        }
        let executable = dir.join(if cfg!(windows) { "cargo.exe" } else { "cargo" });
        if executable.is_file() {
            let canonical = executable.canonicalize()?;
            let identity = rust_executable_identity(&canonical)?;
            return Ok((executable, canonical, identity));
        }
    }
    anyhow::bail!("Cargo is unavailable. Install the Rust toolchain before Debug test.")
}
fn rust_cargo_current(tool: &(PathBuf, PathBuf, RustExecutableIdentity)) -> Result<()> {
    anyhow::ensure!(
        tool.0.canonicalize()? == tool.1 && rust_executable_identity(&tool.1)? == tool.2,
        "The Cargo executable changed during Rust test preparation"
    );
    Ok(())
}
fn rust_builtin_harness(value: &serde_json::Value, runtime: &Path, source: &Path) -> Result<()> {
    let path = value
        .get("manifest_path")
        .and_then(|v| v.as_str())
        .context("Rust compiler artifact has no exact package manifest")?;
    let original = std::fs::symlink_metadata(path)?;
    anyhow::ensure!(
        original.is_file() && !original.file_type().is_symlink(),
        "Rust package manifest is not an ordinary file"
    );
    let manifest = Path::new(path).canonicalize()?;
    anyhow::ensure!(
        manifest == runtime.join("Cargo.toml").canonicalize()?,
        "Rust compiler artifact belongs to another package manifest"
    );
    let meta = std::fs::symlink_metadata(&manifest)?;
    anyhow::ensure!(
        meta.is_file() && meta.len() <= 1048576,
        "Rust package manifest is not a bounded ordinary file"
    );
    let text = std::fs::read_to_string(&manifest)?;
    let body: toml::Value = text
        .parse()
        .context("Cannot prove the built-in Rust test harness from this package manifest")?;
    let name = value
        .pointer("/target/name")
        .and_then(|v| v.as_str())
        .context("Rust test target has no name")?;
    let kinds = value
        .pointer("/target/kind")
        .and_then(|v| v.as_array())
        .context("Rust test target has no kind")?;
    anyhow::ensure!(kinds.len() == 1, "Rust test target kind is ambiguous");
    let kind = kinds[0]
        .as_str()
        .context("Malformed Rust test target kind")?;
    let package = body
        .get("package")
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .context("Rust package identity is unavailable")?;
    let same = |path: &str| runtime.join(path).canonicalize().ok().as_deref() == Some(source);
    let default_source = |family: &str| {
        if family == "bin" {
            same(&format!("src/bin/{name}.rs"))
                || same(&format!("src/bin/{name}/main.rs"))
                || (name == package && same("src/main.rs"))
        } else {
            same(&format!("tests/{name}.rs")) || same(&format!("tests/{name}/main.rs"))
        }
    };
    let target = if matches!(kind, "lib" | "proc-macro") {
        let target = body.get("lib");
        if let Some(target) = target {
            anyhow::ensure!(target.is_table(), "Malformed Rust library target")
        };
        let expected = target
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(str::to_owned)
            .unwrap_or_else(|| package.replace('-', "_"));
        anyhow::ensure!(
            name == expected
                && same(
                    target
                        .and_then(|v| v.get("path"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("src/lib.rs")
                ),
            "Rust test library does not match the admitted package source"
        );
        target
    } else if matches!(kind, "bin" | "test") {
        let declared = body
            .get(kind)
            .map(|v| {
                v.as_array()
                    .context("Malformed Rust test target declarations")
            })
            .transpose()?;
        let matches = declared
            .into_iter()
            .flatten()
            .filter(|v| {
                v.get("name").and_then(|v| v.as_str()) == Some(name)
                    || v.get("path").and_then(|v| v.as_str()).is_some_and(same)
            })
            .collect::<Vec<_>>();
        anyhow::ensure!(
            matches.len() <= 1,
            "Rust test target declarations are ambiguous"
        );
        let target = matches.first().copied();
        if let Some(target) = target {
            anyhow::ensure!(
                target.is_table()
                    && target
                        .get("name")
                        .and_then(|v| v.as_str())
                        .is_none_or(|n| n == name),
                "Rust test target identity changed"
            );
            if let Some(path) = target.get("path").and_then(|v| v.as_str()) {
                anyhow::ensure!(same(path), "Rust test target source changed")
            } else {
                anyhow::ensure!(
                    default_source(kind),
                    "Rust test target has no proven default source"
                )
            }
        } else {
            let auto = if kind == "bin" {
                "autobins"
            } else {
                "autotests"
            };
            anyhow::ensure!(
                body.get("package")
                    .and_then(|v| v.get(auto))
                    .and_then(|v| v.as_bool())
                    != Some(false)
                    && default_source(kind),
                "Rust test target is not a proven built-in default"
            )
        };
        target
    } else {
        anyhow::bail!("This Rust target kind does not provide a proven built-in test harness")
    };
    anyhow::ensure!(
        target
            .and_then(|v| v.get("harness"))
            .is_none_or(|v| v.as_bool() == Some(true)),
        "Rust target uses a custom or unproven harness; Debug refused before listing or running it"
    );
    Ok(())
}
struct RustTestProcess {
    child: tokio::process::Child,
    pid: u32,
}
impl Drop for RustTestProcess {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGKILL);
        }
        let _ = self.child.start_kill();
    }
}
async fn rust_test_command_output(
    command: tokio::process::Command,
) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    rust_test_command_output_with_limit(command, std::time::Duration::from_secs(120)).await
}
async fn rust_test_command_output_with_limit(
    mut command: tokio::process::Command,
    limit: std::time::Duration,
) -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
    use tokio::io::AsyncReadExt;
    async fn bounded<R: tokio::io::AsyncRead + Unpin>(stream: R) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        stream
            .take(8 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .await?;
        anyhow::ensure!(
            bytes.len() <= 8 * 1024 * 1024,
            "Rust test preparation output budget exceeded"
        );
        Ok(bytes)
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command
        .spawn()
        .context("Could not start the Rust test toolchain")?;
    let stdout = child.stdout.take().context("Rust stdout unavailable")?;
    let stderr = child.stderr.take().context("Rust stderr unavailable")?;
    let mut running = RustTestProcess {
        pid: child.id().context("Rust preparation process id missing")?,
        child,
    };
    let read = async {
        let (out, err, status) = tokio::try_join!(bounded(stdout), bounded(stderr), async {
            Ok::<_, anyhow::Error>(running.child.wait().await?)
        })?;
        anyhow::ensure!(
            status.success(),
            "Rust test preparation failed: {}",
            String::from_utf8_lossy(&err[..err.len().min(16384)])
        );
        Ok((out, err))
    };
    tokio::time::timeout(limit, read)
        .await
        .context("Rust test preparation exceeded its time budget")?
}
#[cfg(test)]
async fn prepare_rust_selected_test_inner(
    plan: &mut hick_plugins::testing::TestDebugCommand,
    filter: &str,
    target: &Path,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<RustTestArtifact> {
    anyhow::ensure!(
        !filter.is_empty() && filter.len() <= 8192 && !filter.contains('\0'),
        "Rust test debug requires one literal saved test identity"
    );
    let runtime_path = plan.cwd.canonicalize()?;
    let runtime = same_file::Handle::from_path(&runtime_path)?;
    let manifest_before = std::fs::read(runtime_path.join("Cargo.toml"))?;
    anyhow::ensure!(
        manifest_before.len() <= 1048576,
        "Rust manifest preparation budget exceeded"
    );
    let runtime_current = || -> Result<()> {
        anyhow::ensure!(
            plan.cwd.canonicalize()? == runtime_path
                && same_file::Handle::from_path(&plan.cwd)? == runtime,
            "Rust runtime directory was replaced during preparation"
        );
        anyhow::ensure!(
            std::fs::read(runtime_path.join("Cargo.toml"))? == manifest_before,
            "Rust package manifest changed during preparation"
        );
        Ok(())
    };
    let cargo = rust_cargo_tool()?;
    rust_cargo_current(&cargo)?;
    let mut command = tokio::process::Command::new(&cargo.0);
    command
        .current_dir(&plan.cwd)
        .args(["test", "--no-run", "--message-format=json", "--target-dir"])
        .arg(target)
        .env("CARGO_INCREMENTAL", "0");
    on_build(BuildOutput::Note(
        "Preparing saved Rust test harnesses; no test bodies run during compiler discovery".into(),
    ));
    let (output, stderr) = rust_test_command_output(command).await?;
    rust_cargo_current(&cargo)?;
    runtime_current()?;
    if !stderr.is_empty() {
        on_build(BuildOutput::Note(
            String::from_utf8_lossy(&stderr[..stderr.len().min(16384)]).into_owned(),
        ))
    }
    let artifacts = rust_test_artifacts(&output, &plan.cwd, target)?;
    let mut selected = None;
    for artifact in artifacts {
        runtime_current()?;
        rust_artifact_current(&artifact)?;
        let mut list = tokio::process::Command::new(&artifact.executable);
        list.current_dir(&plan.cwd)
            .args(["--list", "--format", "terse"]);
        let (names, _) = rust_test_command_output(list).await?;
        rust_artifact_current(&artifact)?;
        runtime_current()?;
        for name in rust_harness_names(&names, filter)? {
            anyhow::ensure!(
                selected.is_none(),
                "The selected Rust test matches more than one compiled harness. Choose a qualified collected identity."
            );
            selected = Some((artifact.clone(), name));
        }
    }
    let (artifact, name) = selected.context(
        "The saved Rust test identity is absent from the freshly prepared compiler artifacts",
    )?;
    plan.program = artifact.executable.clone();
    plan.launch = serde_json::json!({"args":[name,"--exact","--nocapture"]});
    Ok(artifact)
}

#[cfg(test)]
async fn prepare_rust_selected_test(
    plan: &mut hick_plugins::testing::TestDebugCommand,
    filter: &str,
    target: &Path,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<RustTestArtifact> {
    tokio::time::timeout(
        std::time::Duration::from_secs(120),
        prepare_rust_selected_test_inner(plan, filter, target, on_build),
    )
    .await
    .context("Rust test preparation exceeded its aggregate time budget")?
}
#[cfg(test)]
mod rust_test_prepare_live {
    use super::*;
    #[tokio::test]
    async fn genuine_cargo_artifact_lists_without_bodies_and_loads_only_exact_case() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::write(
            root.path().join("Cargo.toml"),
            "[package]\nname=\"owned-debug-test\"\nversion=\"0.1.0\"\nedition=\"2024\"\n",
        )
        .unwrap();
        let source = "#[cfg(test)]mod tests{#[test]fn chosen(){std::fs::write(\"chosen-ran\",b\"ok\").unwrap();}#[test]fn sibling(){std::fs::write(\"sibling-ran\",b\"wrong\").unwrap();}}";
        std::fs::write(root.path().join("src/lib.rs"), source).unwrap();
        let mut plan = hick_plugins::testing::TestDebugCommand {
            title: "owned test fixture".into(),
            framework: "libtest",
            preparation: None,
            cwd: root.path().to_owned(),
            program: root.path().join("src/lib.rs"),
            launch: serde_json::json!({}),
        };
        let mut notes = Vec::new();
        let artifact =
            prepare_rust_selected_test(&mut plan, "chosen", target.path(), &mut |output| {
                notes.push(output)
            })
            .await
            .unwrap();
        assert_eq!(
            plan.launch["args"],
            serde_json::json!(["tests::chosen", "--exact", "--nocapture"])
        );
        assert!(!root.path().join("chosen-ran").exists());
        assert!(!root.path().join("sibling-ran").exists());
        rust_artifact_current(&artifact).unwrap();
        let mut run = tokio::process::Command::new(&plan.program);
        run.current_dir(&plan.cwd)
            .args(["tests::chosen", "--exact", "--nocapture"]);
        rust_test_command_output(run).await.unwrap();
        assert_eq!(
            std::fs::read(root.path().join("chosen-ran")).unwrap(),
            b"ok"
        );
        assert!(!root.path().join("sibling-ran").exists());
        assert_eq!(
            std::fs::read_to_string(root.path().join("src/lib.rs")).unwrap(),
            source
        );
    }
}
#[cfg(test)]
mod rust_test_custom_harness_witness {
    use super::*;
    #[tokio::test]
    async fn genuine_custom_harness_is_refused_before_any_list_execution() {
        let root = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::write(root.path().join("Cargo.toml"),"[workspace]\n[package]\nname=\"custom-harness\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[lib]\nharness=false\n").unwrap();
        std::fs::write(
            root.path().join("src/lib.rs"),
            "fn main(){std::fs::write(\"custom-main-ran\",b\"wrong\").unwrap();}",
        )
        .unwrap();
        let mut plan = hick_plugins::testing::TestDebugCommand {
            title: "owned custom fixture".into(),
            framework: "libtest",
            preparation: None,
            cwd: root.path().to_owned(),
            program: root.path().join("src/lib.rs"),
            launch: serde_json::json!({}),
        };
        let error = prepare_rust_selected_test(&mut plan, "chosen", target.path(), &mut |_| {})
            .await
            .unwrap_err();
        assert!(
            error.to_string().contains("custom or unproven harness"),
            "{error:#}"
        );
        assert!(!root.path().join("custom-main-ran").exists());
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn timed_preparation_kills_its_owned_descendant_group() {
        let root = tempfile::tempdir().unwrap();
        let pidfile = root.path().join("child.pid");
        let mut command = tokio::process::Command::new("/bin/sh");
        command
            .args([
                "-c",
                "sleep 30 & echo $! > \"$1\"; wait",
                "owned-cleanup-fixture",
            ])
            .arg(&pidfile);
        assert!(
            rust_test_command_output_with_limit(command, std::time::Duration::from_millis(300))
                .await
                .is_err()
        );
        let pid: libc::pid_t = std::fs::read_to_string(pidfile)
            .unwrap()
            .trim()
            .parse()
            .unwrap();
        for _ in 0..100 {
            if unsafe { libc::kill(pid, 0) } != 0 {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        panic!("Owned Rust preparation descendant survived timeout");
    }
}

#[path = "rust_test_source_proof.rs"]
mod rust_test_source_proof;
struct PreparedRustTest {
    artifact: RustTestArtifact,
    source: rust_test_source_proof::Preparation,
}
impl PreparedRustTest {
    fn current(&self) -> Result<()> {
        rust_artifact_current(&self.artifact)?;
        self.source.current()
    }
}
async fn prepare_rust_selected_test_proven_inner(
    plan: &mut hick_plugins::testing::TestDebugCommand,
    filter: &str,
    target: &Path,
    selection: rust_test_source_proof::Selection,
    metadata_tool: rust_test_source_proof::MetadataTool,
    physical: Option<(&Path, &BTreeMap<String, String>)>,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PreparedRustTest> {
    anyhow::ensure!(
        !filter.is_empty() && filter.len() <= 8192 && !filter.contains('\0'),
        "Rust test debug requires one literal saved test identity"
    );
    let runtime_path = plan.cwd.canonicalize()?;
    let runtime = same_file::Handle::from_path(&runtime_path)?;
    let manifest_before = std::fs::read(runtime_path.join("Cargo.toml"))?;
    anyhow::ensure!(
        manifest_before.len() <= 1048576,
        "Rust manifest preparation budget exceeded"
    );
    let runtime_current = || -> Result<()> {
        anyhow::ensure!(
            plan.cwd.canonicalize()? == runtime_path
                && same_file::Handle::from_path(&plan.cwd)? == runtime,
            "Rust runtime directory was replaced during preparation"
        );
        anyhow::ensure!(
            std::fs::read(runtime_path.join("Cargo.toml"))? == manifest_before,
            "Rust package manifest changed during preparation"
        );
        Ok(())
    };
    let cargo = rust_cargo_tool()?;
    rust_cargo_current(&cargo)?;
    let mut metadata = tokio::process::Command::new(&cargo.0);
    metadata
        .current_dir(&plan.cwd)
        .args(["metadata", "--no-deps", "--format-version=1"]);
    rust_held_cwd(&mut metadata, &runtime)?;
    let (reading, _) = rust_test_command_output(metadata).await?;
    rust_cargo_current(&cargo)?;
    runtime_current()?;
    selection.current()?;
    let source = rust_test_source_proof::Preparation::capture(
        selection,
        &serde_json::from_slice(&reading)?,
        &runtime_path,
        physical,
    )?;
    let mut command = tokio::process::Command::new(&cargo.0);
    command
        .current_dir(&plan.cwd)
        .args(["test", "--no-run", "--message-format=json", "--target-dir"])
        .arg(target)
        .env("CARGO_INCREMENTAL", "0");
    rust_held_cwd(&mut command, &runtime)?;
    on_build(BuildOutput::Note(
        "Preparing saved Rust test harnesses; no test bodies run during compiler discovery".into(),
    ));
    let (output, stderr) = rust_test_command_output(command).await?;
    rust_cargo_current(&cargo)?;
    runtime_current()?;
    if !stderr.is_empty() {
        on_build(BuildOutput::Note(
            String::from_utf8_lossy(&stderr[..stderr.len().min(16384)]).into_owned(),
        ))
    }
    source.current()?;
    let artifacts = rust_test_artifacts(&output, &plan.cwd, target)?;
    let mut selected = None;
    for artifact in artifacts {
        source.admit_artifact(&artifact)?;
        runtime_current()?;
        rust_artifact_current(&artifact)?;
        let mut list = tokio::process::Command::new(&artifact.executable);
        list.current_dir(&plan.cwd)
            .args(["--list", "--format", "terse"]);
        rust_held_cwd(&mut list, &runtime)?;
        source.current()?;
        let (names, _) = rust_test_command_output(list).await?;
        source.current()?;
        rust_artifact_current(&artifact)?;
        runtime_current()?;
        if let Some(name) =
            source.listed_identity(&artifact, &rust_harness_candidates(&names, filter)?)?
        {
            anyhow::ensure!(
                selected.is_none(),
                "The selected Rust test matches more than one compiled harness. Choose a qualified collected identity."
            );
            selected = Some((artifact.clone(), name));
        }
    }
    let (artifact, name) = selected.context(
        "The saved Rust test identity is absent from the freshly prepared compiler artifacts",
    )?;
    let crate_name = rust_artifact_crate_name(&output, &artifact.executable)?;
    source
        .verify_compiled(&metadata_tool, &artifact, &name, &crate_name)
        .await?;
    source.current()?;
    runtime_current()?;
    let mut args = vec![name, "--exact".into(), "--nocapture".into()];
    if source.ignored() {
        args.push("--include-ignored".into())
    }
    plan.program = artifact.executable.clone();
    plan.launch = serde_json::json!({"args":args});
    Ok(PreparedRustTest { artifact, source })
}

async fn prepare_rust_selected_test_proven(
    plan: &mut hick_plugins::testing::TestDebugCommand,
    filter: &str,
    target: &Path,
    selection: rust_test_source_proof::Selection,
    metadata_tool: rust_test_source_proof::MetadataTool,
    physical: Option<(&Path, &BTreeMap<String, String>)>,
    on_build: &mut (dyn FnMut(BuildOutput) + Send),
) -> Result<PreparedRustTest> {
    tokio::time::timeout(
        std::time::Duration::from_secs(120),
        prepare_rust_selected_test_proven_inner(
            plan,
            filter,
            target,
            selection,
            metadata_tool,
            physical,
            on_build,
        ),
    )
    .await
    .context("Rust test preparation exceeded its aggregate time budget")?
}
fn rust_held_cwd(command: &mut tokio::process::Command, root: &same_file::Handle) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        let directory = root.as_file().try_clone()?;
        unsafe {
            command.pre_exec(move || {
                if libc::fchdir(directory.as_raw_fd()) == 0 {
                    Ok(())
                } else {
                    Err(std::io::Error::last_os_error())
                }
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (command, root);
        anyhow::bail!("Pinned Rust compiler working directories are unavailable on this host")
    };
    Ok(())
}
fn rust_artifact_crate_name(output: &[u8], executable: &Path) -> Result<String> {
    let mut names = std::collections::BTreeSet::new();
    for line in output.split(|v| *v == b'\n') {
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_slice(line)?;
        if value.get("reason").and_then(|v| v.as_str()) == Some("compiler-artifact")
            && value
                .get("executable")
                .and_then(|v| v.as_str())
                .and_then(|p| Path::new(p).canonicalize().ok())
                .as_deref()
                == Some(executable)
        {
            names.insert(
                value
                    .pointer("/target/name")
                    .and_then(|v| v.as_str())
                    .context("Compiled Rust target name missing")?
                    .to_owned(),
            );
        }
    }
    anyhow::ensure!(names.len() == 1, "Compiled Rust target name is ambiguous");
    Ok(names.into_iter().next().unwrap())
}
