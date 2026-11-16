use super::*;
use std::collections::BTreeMap;

/// Saved compiler inputs and target, selected by engine admission rather than UI argv.
pub(crate) struct TestLaunch {
    pub workspace: PathBuf,
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
        let (root, scratch, program, mapping) = if let Some((state, entry)) = &launch.construction {
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
        // Adapter provisioning/build preparation can be slow. Recheck every
        // admitted author immediately before creating the runtime session.
        crate::serve::source_tests::check_saved_sources(
            &launch.workspace,
            &launch.expected_sources,
        )
        .map_err(|error| anyhow::anyhow!("{}", error.detail()))?;
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
