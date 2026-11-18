use super::{
    LocalState,
    api::{ApiError, ApiResult},
    source_tests::{self, RunRequest, Selection},
};
use axum::{Json, extract::State};
use serde::Serialize;
use std::{collections::BTreeMap, path::Path};

pub(crate) const PROTOCOL: u32 = 1;
#[derive(Debug, Clone, Serialize)]
pub struct Binding {
    pub version: u32,
    pub workspace_id: String,
    pub input_path: String,
    pub input_hash: String,
    pub analysis_fingerprint: String,
    pub candidate_id: String,
    pub candidate_name: String,
    pub selection: Selection,
    pub checked_sources: BTreeMap<String, String>,
    pub language: String,
    pub framework: String,
}
#[derive(Serialize)]
pub struct Plan {
    pub version: u32,
    pub test_debug_protocol: u32,
    pub available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub workspace_id: String,
    pub input_path: String,
    pub input_hash: String,
    pub analysis_fingerprint: String,
    pub candidate_id: String,
    pub candidate_name: String,
    pub selection: Selection,
    pub source_uri: String,
    pub source_hash: String,
    pub language: String,
    pub framework: String,
    pub name: String,
    pub notices: Vec<String>,
}
pub(crate) struct Admission {
    pub plan: Plan,
    pub binding: Binding,
    pub launch: crate::debug_sessions::TestLaunch,
}
fn constructed_rust_test_manifest(root:&Path,target:&str,state:&hick_lsp::document::HickDocumentState,entry:&Path)->Result<(String,String),String>{
 let host=root.join(target);let runtime=host.strip_prefix(entry.parent().unwrap_or(root)).map_err(|_|"Constructed Rust target escaped its compiler root")?;let mut dir=runtime.parent().ok_or("Rust test target has no parent")?;
 loop{let name=dir.join("Cargo.toml");let key=name.to_str().ok_or("Rust manifest path is not UTF-8")?;
  let bytes=if let Some(tree)=&state.selected_project{if let Some(file)=tree.get(key){if file.kind!=hick_lang::EntryKind::File{return Err("Constructed Rust manifest is not an ordinary file".into())}Some(file.bytes.to_vec())}else{None}}else{let files=state.virtual_files.iter().filter(|file|file.path==key).collect::<Vec<_>>();if files.len()>1{return Err("Constructed Rust manifest is ambiguous".into())}files.first().map(|file|file.written_content().into_bytes())};
  if let Some(bytes)=bytes{if bytes.len()>1048576{return Err("Rust manifest exceeds the debug preparation budget".into())}return Ok((key.to_owned(),source_tests::digest(&bytes)))}
  match dir.parent(){Some(parent)=>dir=parent,None=>return Err("The constructed Rust test has no admitted Cargo.toml".into())}
 }
}

/// Read compiler facts and saved authors again; a previous plan grants no execution.
pub(crate) fn admit(root: &Path, request: &RunRequest) -> ApiResult<Admission> {
    if request.version != 1 {
        return Err(ApiError::conflict(
            "Unsupported source-test protocol. Update the app and engine together.",
        ));
    }
    let root = root
        .canonicalize()
        .map_err(|error| ApiError::unprocessable(format!("Workspace unavailable: {error}")))?;
    let mut scan = source_tests::scan_with_sources(&root, &request.analysis, None)
        .map_err(ApiError::unprocessable)?;
    let (candidate, filter) = source_tests::select_saved_candidate(&scan, request)?;
    let candidate = candidate.clone();
    let checked_sources = source_tests::check_saved_sources(&root, &scan.source_hashes)?;
    let source_uri = crate::launch_profiles::inventory::workspace_source_uri(&scan.input_path)
        .map_err(ApiError::bad_request)?;
    let constructed_manifest=if candidate.language=="rust"{scan.debug_state.as_ref().map(|(state,entry)|constructed_rust_test_manifest(&root,&candidate.target.path,state,entry))}else{None};
    let preview_manifest=constructed_manifest.as_ref().and_then(|value|value.as_ref().ok()).map(|(path,_)|root.join(path));
    let provider=if let Some(Err(reason))=&constructed_manifest{Err(reason.clone())}else{hick_plugins::testing::test_debug_command_with_manifest(&root,&root.join(&candidate.target.path),&filter,&candidate.language,preview_manifest.as_deref())};
    let rust_manifest=if candidate.language=="rust" {if let Ok(plan)=&provider{if let Some(Ok(descriptor))=&constructed_manifest{Some(descriptor.clone())}else{let path=plan.cwd.join("Cargo.toml");let relative=path.strip_prefix(&root).map_err(|_|ApiError::unprocessable("Rust manifest escaped workspace"))?.to_str().ok_or_else(||ApiError::unprocessable("Rust manifest is not UTF-8"))?.to_owned();let metadata=std::fs::symlink_metadata(&path).map_err(|error|ApiError::unprocessable(error.to_string()))?;if !metadata.is_file()||metadata.file_type().is_symlink()||metadata.len()>1048576{return Err(ApiError::unprocessable("Rust manifest must be a bounded ordinary saved file"))}let bytes=std::fs::read(&path).map_err(|error|ApiError::unprocessable(format!("Rust manifest unavailable: {error}")))?;if request.analysis.sources.get(&relative).is_some_and(|draft|draft.as_bytes()!=bytes){return Err(ApiError::unprocessable("Save the selected Cargo.toml draft before Debug test"))}if bytes.len()>1048576{return Err(ApiError::unprocessable("Rust manifest debug budget exceeded"))}Some((relative,source_tests::digest(&bytes)))}}else{None}}else{None};
    let (available, framework, name, reason) = match provider {
        Ok(value) => (true, value.framework.to_owned(), value.title, None),
        Err(message) => (
            false,
            "unavailable".into(),
            format!("Debug {}", candidate.name),
            Some(message),
        ),
    };
    let binding = Binding {
        version: PROTOCOL,
        workspace_id: scan.workspace_id.clone(),
        input_path: scan.input_path.clone(),
        input_hash: scan.input_hash.clone(),
        analysis_fingerprint: scan.fingerprint.clone(),
        candidate_id: candidate.id.clone(),
        candidate_name: candidate.name.clone(),
        selection: request.selection,
        checked_sources,
        language: candidate.language.clone(),
        framework: framework.clone(),
    };
    let mut notices = scan.notices;
    notices.push("Saved contributors are checked at admission, not proof of every runtime dependency. Debugger exit is a process outcome, not a collected test verdict.".into());
    let plan = Plan {
        version: 1,
        test_debug_protocol: PROTOCOL,
        available,
        reason,
        workspace_id: binding.workspace_id.clone(),
        input_path: binding.input_path.clone(),
        input_hash: binding.input_hash.clone(),
        analysis_fingerprint: binding.analysis_fingerprint.clone(),
        candidate_id: binding.candidate_id.clone(),
        candidate_name: binding.candidate_name.clone(),
        selection: binding.selection,
        source_uri,
        source_hash: binding.input_hash.clone(),
        language: binding.language.clone(),
        framework,
        name,
        notices,
    };
    let launch = crate::debug_sessions::TestLaunch {
        workspace_id:scan.workspace_id.clone(),
        rust_manifest,
        target_line:candidate.target.line,
        target_character:candidate.target.character,
        target_hash:candidate.target.hash.clone(),
        open_sources:request.analysis.sources.clone(),
        workspace: root.clone(),
        input: root.join(&scan.input_path),
        input_source: request.analysis.source.clone(),
        expected_sources: binding.checked_sources.clone(),
        source_texts: scan.source_texts,
        construction: scan.debug_state.take(),
        target: candidate.target.path,
        language: candidate.language,
        filter,
    };
    Ok(Admission {
        plan,
        binding,
        launch,
    })
}
pub async fn plan(
    State(state): State<LocalState>,
    Json(request): Json<RunRequest>,
) -> ApiResult<Json<Plan>> {
    let root = state.index.root().to_path_buf();
    tokio::task::spawn_blocking(move || admit(&root, &request).map(|value| Json(value.plan)))
        .await
        .map_err(|error| {
            ApiError::internal(format!(
                "Test debug analysis failed: {error}. Refresh the source tests."
            ))
        })?
}
#[cfg(test)]
mod tests {
    use super::*;
    fn request(root: &Path, path: &str, text: &str, selection: Selection) -> RunRequest {
        std::fs::write(root.join(path), text).unwrap();
        let analysis = source_tests::Request {
            path: path.into(),
            source: text.into(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        let scan = source_tests::scan(root, &analysis).unwrap();
        RunRequest {
            version: 1,
            input: source_tests::RunInput::Saved {},
            analysis,
            fingerprint: scan.fingerprint,
            candidate_id: scan.candidates[0].id.clone(),
            selection,
        }
    }
    #[test]
    fn dedicated_wire_refuses_ordinary_profile_fields_before_effects() {
        let value = serde_json::json!({"op":"start_test","doc":"hick:///test.py","request_id":"test-wire","source_hash":"a".repeat(64),"source_test":{"version":1,"input":{"kind":"saved"},"analysis":{"path":"test.py","source":"def test_x(): pass\n"},"fingerprint":"unexecuted","candidate_id":"unexecuted","selection":{"kind":"name_filter"}}});
        assert!(matches!(
            serde_json::from_value::<crate::serve::debug_bridge::Request>(value.clone()).unwrap(),
            crate::serve::debug_bridge::Request::StartTest(_)
        ));
        for name in ["program", "project", "profile"] {
            let mut combined = value.clone();
            combined[name] = serde_json::Value::Null;
            assert!(
                serde_json::from_value::<crate::serve::debug_bridge::Request>(combined).is_err(),
                "{name}"
            );
        }
    }
    #[test]
    fn plain_plan_preserves_literal_source_and_qualified_family_without_effects() {
        let directory = tempfile::tempdir().unwrap();
        let path = "test %2F λ?#.py";
        let request = request(
            directory.path(),
            path,
            "class TestChoice:\n    def test_selected(self):\n        value=41\n        assert value==41\n",
            Selection::NameFilter {},
        );
        let admitted = admit(directory.path(), &request).unwrap();
        assert!(admitted.plan.available);
        assert_eq!(admitted.launch.filter, "TestChoice::test_selected");
        assert_eq!(
            admitted.plan.source_uri,
            "hick:///test%20%252F%20%CE%BB%3F%23.py"
        );
        assert_eq!(admitted.plan.input_hash, admitted.plan.source_hash);
        assert_eq!(admitted.binding.checked_sources.len(), 1);
        assert_eq!(
            std::fs::read_dir(directory.path()).unwrap().count(),
            1,
            "Planning must not materialize or run a process"
        );
    }
    #[test]
    fn changed_saved_source_and_stale_candidate_never_prepare_a_launch() {
        let directory = tempfile::tempdir().unwrap();
        let mut request = request(
            directory.path(),
            "test.py",
            "def test_selected(): pass\n",
            Selection::NameFilter {},
        );
        std::fs::write(
            directory.path().join("test.py"),
            "def test_changed(): pass\n",
        )
        .unwrap();
        assert!(admit(directory.path(), &request).is_err());
        std::fs::write(directory.path().join("test.py"), &request.analysis.source).unwrap();
        request.candidate_id = "unobserved".into();
        assert!(admit(directory.path(), &request).is_err());
    }
    #[test]
    fn missing_provider_is_an_explicit_plan_not_an_ordinary_debug_fallback() {
        let directory = tempfile::tempdir().unwrap();
        let request = request(
            directory.path(),
            "test.js",
            "test('test_selected',()=>{});\n",
            Selection::ContainingScope {},
        );
        let admitted = admit(directory.path(), &request).unwrap();
        assert!(!admitted.plan.available);
        assert!(
            admitted
                .plan
                .reason
                .as_ref()
                .unwrap()
                .contains("does not supply")
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires debugpy and pytest in HICKORY_THREAD_TEST_VENV or target/thread-debug-env"]
    async fn actual_pytest_selection_preserves_authored_input_and_excludes_sibling_through_completion()
     {
        use serde_json::{Value, json};
        use std::sync::Arc;
        async fn ask(
            registry: &Arc<crate::debug_sessions::Registry>,
            root: &Path,
            value: Value,
        ) -> Vec<Value> {
            super::super::debug_bridge::handle(
                registry,
                root,
                serde_json::from_value(value).unwrap(),
            )
            .await
            .into_iter()
            .map(|value| serde_json::to_value(value).unwrap())
            .collect()
        }
        fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
            rows.iter()
                .find(|row| row["event"] == kind)
                .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
        }
        for authored in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            std::fs::create_dir(root.join(".git")).unwrap();
            // An explicit project declaration owns interpreter selection; a loose
            // .venv directory alone does not grant project-environment authority.
            std::fs::write(root.join("pyproject.toml"), "[tool.uv]\n").unwrap();
            let venv = std::env::var_os("HICKORY_THREAD_TEST_VENV")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/thread-debug-env")
                });
            std::os::unix::fs::symlink(venv.canonicalize().unwrap(), root.join(".venv")).unwrap();
            let sentinel = root.join("sibling-ran.txt");
            let code = format!(
                "from pathlib import Path\nclass TestChoice:\n    def test_selected(self):\n        selected_value = 41\n        selected_value += 0  # source-test-breakpoint\n        assert selected_value == 41\n    def test_sibling(self):\n        Path({:?}).write_text('wrong family')\n",
                sentinel.to_str().unwrap()
            );
            let (path, source) = if authored {
                (
                    "test_doc.md",
                    format!(
                        "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"test_choice.py\">\n{code}</h:file>\n</h:doc>\n"
                    ),
                )
            } else {
                ("test_choice.py", code)
            };
            let request = request(&root, path, &source, Selection::NameFilter {});
            let admitted = admit(&root, &request).unwrap();
            assert!(admitted.plan.available);
            let line = source
                .lines()
                .position(|line| line.contains("# source-test-breakpoint"))
                .unwrap();
            let registry = Arc::new(crate::debug_sessions::Registry::with_projects(
                Default::default(),
                root.clone(),
            ));
            let rows=ask(&registry,&root,json!({"op":"start_test","doc":admitted.plan.source_uri,"request_id":"actual-test-start","source_hash":admitted.plan.source_hash,"source_test":request,"breakpoints":[{"line":line}]})).await;
            let started = event(&rows, "started");
            let stopped = event(&rows, "stopped");
            let session = started["session"].as_str().unwrap();
            assert_eq!(
                started["source_test"]["candidate_id"],
                admitted.binding.candidate_id
            );
            assert_eq!(
                started["source_test"]["checked_sources"],
                json!(admitted.binding.checked_sources)
            );
            assert_eq!(stopped["line"], line, "{stopped:?}");
            let frame = stopped["frames"][0]["id"].clone();
            let result=ask(&registry,&root,json!({"op":"eval","session":session,"expression":"selected_value","stop":stopped["stop"],"thread":stopped["thread"],"frame":frame,"request_id":"selected-value"})).await;
            assert_eq!(event(&result, "value")["value"], "41");
            let cleared=ask(&registry,&root,json!({"op":"breakpoints","session":session,"breakpoints":[],"source_hash":admitted.plan.source_hash,"request_id":"clear-test-mark"})).await;
            assert!(
                cleared.iter().all(|row| row["event"] != "failed"),
                "{cleared:?}"
            );
            let finished=ask(&registry,&root,json!({"op":"step","session":session,"how":"continue","stop":stopped["stop"],"thread":stopped["thread"],"request_id":"complete-selected-family"})).await;
            assert_eq!(event(&finished, "finished")["exit_code"], 0, "{finished:?}");
            assert!(!sentinel.exists(), "The sibling test was executed");
            assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), source);
            if authored {
                assert!(
                    !root.join("test_choice.py").exists(),
                    "Authored runtime output leaked into the workspace"
                );
            }
            assert!(registry.get(session).await.is_err());
        }
    }
}
#[cfg(test)]
mod rust_selected_test_live {
    use super::*;
    use serde_json::{Value, json};
    use std::sync::Arc;
    async fn ask(
        registry: &Arc<crate::debug_sessions::Registry>,
        root: &Path,
        request: Value,
    ) -> Vec<Value> {
        crate::serve::debug_bridge::handle(registry, root, serde_json::from_value(request).unwrap())
            .await
            .into_iter()
            .map(|row| serde_json::to_value(row).unwrap())
            .collect()
    }
    fn event<'a>(rows: &'a [Value], kind: &str) -> &'a Value {
        rows.iter()
            .find(|row| row["event"] == kind)
            .unwrap_or_else(|| panic!("Missing {kind}: {rows:?}"))
    }
    fn saved_request(root: &Path, path: &str, text: &str) -> RunRequest {
        std::fs::write(root.join(path), text).unwrap();
        let analysis = source_tests::Request {
            path: path.into(),
            source: text.into(),
            project_entry: None,
            sources: BTreeMap::new(),
            capture: false,
        };
        let scan = source_tests::scan(root, &analysis).unwrap();
        let candidate = scan
            .candidates
            .iter()
            .find(|c| c.name == "test_selected")
            .unwrap();
        RunRequest {
            version: 1,
            input: source_tests::RunInput::Saved {},
            analysis,
            fingerprint: scan.fingerprint,
            candidate_id: candidate.id.clone(),
            selection: Selection::NameFilter {},
        }
    }
    #[test]
    fn constructed_rust_plan_reads_the_compiler_manifest_without_materializing_it() {
        let dir = tempfile::tempdir().unwrap();
        let text = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"Cargo.toml\">\n[package]\nname=\"owned\"\nversion=\"0.1.0\"\n</h:file>\n<h:file path=\"src/lib.rs\">\n#[test]fn test_selected(){}\n</h:file>\n</h:doc>\n";
        let request = saved_request(dir.path(), "cases.md", text);
        let value = admit(dir.path(), &request).unwrap();
        assert!(value.plan.available, "{:?}", value.plan.reason);
        assert_eq!(value.launch.rust_manifest.as_ref().unwrap().0, "Cargo.toml");
        assert!(!dir.path().join("Cargo.toml").exists());
        assert!(!dir.path().join("src/lib.rs").exists());
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[test]
    fn constructed_rust_missing_manifest_stays_unavailable_without_output() {
        let dir = tempfile::tempdir().unwrap();
        let text = "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"src/lib.rs\">\n#[test]fn test_selected(){}\n</h:file>\n</h:doc>\n";
        let request = saved_request(dir.path(), "cases.md", text);
        let value = admit(dir.path(), &request).unwrap();
        assert!(!value.plan.available);
        assert!(
            value
                .plan
                .reason
                .unwrap()
                .contains("no admitted Cargo.toml")
        );
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires installed Cargo and LLDB DAP; owned physical and compiler-author fixture"]
    async fn genuine_rust_selected_test_uses_exact_cargo_harness_and_preserves_authored_sources() {
        for authored in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            std::fs::create_dir(root.join(".git")).unwrap();
            let sibling = root.join("sibling-ran");
            let code = format!(
                "#[cfg(test)]\nmod cases {{\n #[test]\n fn test_selected() {{\n  let selected_value=41;\n  assert_eq!(selected_value,41); // owned-rust-stop\n }}\n #[test]\n fn test_sibling() {{std::fs::write({:?},b\"wrong\").unwrap();}}\n}}\n",
                sibling.to_str().unwrap()
            );
            let manifest = "[workspace]\n[package]\nname=\"owned-selected-debug\"\nversion=\"0.1.0\"\nedition=\"2024\"\n";
            let (path, text) = if authored {
                (
                    "tests.md",
                    format!(
                        "<h:doc xmlns:h=\"http://www.hickorydocs.com/1.0\">\n<h:file path=\"Cargo.toml\">\n{manifest}</h:file>\n<h:file path=\"src/lib.rs\">\n{code}</h:file>\n</h:doc>\n"
                    ),
                )
            } else {
                std::fs::write(root.join("Cargo.toml"), manifest).unwrap();
                std::fs::create_dir(root.join("src")).unwrap();
                ("src/lib.rs", code)
            };
            let request = saved_request(&root, path, &text);
            let admitted = admit(&root, &request).unwrap();
            assert!(admitted.plan.available, "{:?}", admitted.plan.reason);
            assert_eq!(admitted.plan.framework, "libtest");
            let line = text
                .lines()
                .position(|line| line.contains("owned-rust-stop"))
                .unwrap();
            let registry = Arc::new(crate::debug_sessions::Registry::with_projects(
                Default::default(),
                root.clone(),
            ));
            let rows=ask(&registry,&root,json!({"op":"start_test","doc":admitted.plan.source_uri,"source_hash":admitted.plan.source_hash,"request_id":"rust-selected","source_test":request,"breakpoints":[{"line":line}]})).await;
            let started = event(&rows, "started");
            let stopped = event(&rows, "stopped");
            assert_eq!(stopped["line"], line);
            let session = started["session"].as_str().unwrap();
            assert_eq!(
                started["source_test"]["candidate_id"],
                admitted.binding.candidate_id
            );
            assert!(!sibling.exists());
            let cleared=ask(&registry,&root,json!({"op":"breakpoints","session":session,"source_hash":admitted.plan.source_hash,"request_id":"clear-rust-mark","breakpoints":[]})).await;
            assert!(
                cleared.iter().all(|row| row["event"] != "failed"),
                "{cleared:?}"
            );
            let done=ask(&registry,&root,json!({"op":"step","session":session,"how":"continue","stop":stopped["stop"],"thread":stopped["thread"],"request_id":"finish-rust-selected"})).await;
            assert_eq!(event(&done, "finished")["exit_code"], 0);
            assert!(!sibling.exists());
            assert_eq!(std::fs::read_to_string(root.join(path)).unwrap(), text);
            if authored {
                assert!(!root.join("Cargo.toml").exists());
                assert!(!root.join("src/lib.rs").exists())
            }
            assert!(registry.get(session).await.is_err());
        }
    }
    #[cfg(unix)]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "requires installed Cargo and LLDB DAP; exact ordinary external module and cfg-disabled sibling fixtures"]
    async fn genuine_rust_external_module_and_disabled_sibling_have_proven_source_owners() {
        for disabled in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let root = directory.path().canonicalize().unwrap();
            std::fs::create_dir(root.join(".git")).unwrap();
            std::fs::create_dir(root.join("src")).unwrap();
            std::fs::write(root.join("Cargo.toml"),"[workspace]\n[package]\nname='owned-source-proof'\nversion='0.1.0'\nedition='2024'\n").unwrap();
            let sentinel = root.join("wrong-sibling-ran");
            let code = "#[test]\nfn test_selected(){\n let selected_value=41;\n assert_eq!(selected_value,41); // owned-external-stop\n}\n";
            let declarations = if disabled {
                "#[cfg(any())]mod selected;\nmod sibling;\n"
            } else {
                "mod selected;\nmod sibling;\n"
            };
            std::fs::write(root.join("src/lib.rs"), declarations).unwrap();
            std::fs::write(
                root.join("src/sibling.rs"),
                format!(
                    "#[test]fn test_selected(){{std::fs::write({:?},b\"wrong\").unwrap();}}\n",
                    sentinel.to_str().unwrap()
                ),
            )
            .unwrap();
            let request = saved_request(&root, "src/selected.rs", code);
            let admitted = admit(&root, &request).unwrap();
            let registry = Arc::new(crate::debug_sessions::Registry::with_projects(
                Default::default(),
                root.clone(),
            ));
            let line = code
                .lines()
                .position(|line| line.contains("owned-external-stop"))
                .unwrap();
            let rows=ask(&registry,&root,json!({"op":"start_test","doc":admitted.plan.source_uri,"source_hash":admitted.plan.source_hash,"request_id":"rust-source-proof","source_test":request,"breakpoints":[{"line":line}]})).await;
            if disabled {
                assert!(!rows.iter().any(|r| r["event"] == "started"), "{rows:?}");
                assert!(rows.iter().any(|r| r["event"] == "failed"), "{rows:?}");
                assert!(
                    serde_json::to_string(&rows)
                        .unwrap()
                        .contains("no proven active module/target identity"),
                    "{rows:?}"
                );
            } else {
                let started = event(&rows, "started");
                let stopped = event(&rows, "stopped");
                assert_eq!(stopped["line"], line);
                let session = started["session"].as_str().unwrap();
                let cleared=ask(&registry,&root,json!({"op":"breakpoints","session":session,"source_hash":admitted.plan.source_hash,"request_id":"clear-external-mark","breakpoints":[]})).await;
                assert!(cleared.iter().all(|r| r["event"] != "failed"));
                let done=ask(&registry,&root,json!({"op":"step","session":session,"how":"continue","stop":stopped["stop"],"thread":stopped["thread"],"request_id":"finish-external"})).await;
                assert_eq!(event(&done, "finished")["exit_code"], 0);
            }
            assert!(!sentinel.exists());
            assert_eq!(
                std::fs::read_to_string(root.join("src/selected.rs")).unwrap(),
                code
            );
        }
    }
}
