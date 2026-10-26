<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/agent_changesets.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-35273396ec57"></a>
## `crates/hickory-cli/tests/agent_changesets.rs`

Related contracts: [Finish an agent turn against its captured project boundary](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md), [A final changeset revision applies B to C](../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md).

<a id="f-35273396ec57-Host"></a>
### `Host`

Real HTTP + ACP subprocess + private MCP bridge. No live model required.

<storycode:copy id="f-35273396ec57-Host">
//! Real HTTP + ACP subprocess + private MCP bridge. No live model required.
// Guarantee: docs/guarantees/agent/changeset-completion-is-negotiated-and-checked.md
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::time::Duration;

struct Host {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
    base: String,
    doc: String,
}
</storycode:copy>
<a id="f-35273396ec57-host"></a>
### `host`

<storycode:copy id="f-35273396ec57-host">
async fn host(negotiated: bool) -> Host {
    host_scope(negotiated, true, true).await
}
</storycode:copy>
<a id="f-35273396ec57-host_scope"></a>
### `host_scope`

<storycode:copy id="f-35273396ec57-host_scope">
async fn host_scope(negotiated: bool, folder: bool, declared: bool) -> Host {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("project.md"),
        if declared {
            "<hick:project/><hick:file path=\"app.ts\">declared\n</hick:file>"
        } else {
            "# Ordinary workspace note\n"
        },
    )
    .unwrap();
    std::fs::write(root.join("app.ts"), "dirty A\n").unwrap();
    let options = ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join(".hick-cache/ui.json")),
    };
    let prepared = if folder {
        prepare(options).await
    } else {
        hickory_cli::serve::prepare_without_folder(options).await
    }
    .unwrap();
    let doc = if declared {
        prepared.state.index.id_for_path("project.md")
    } else {
        "workspace".into()
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let host = Host {
        _dir: dir,
        root,
        base,
        doc,
    };
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!(
            "examples/acp_fixture{}",
            std::env::consts::EXE_SUFFIX
        ));
    let response = reqwest::Client::new().put(format!("{}/api/agents",host.base)).json(&json!([{"id":"fixture","name":"Fixture","command":binary,"args":if negotiated {vec!["--changesets"]} else {vec![]},"env":{}}])).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let (_, value) = request(
        &host,
        &format!("/api/docs/{}/agent/acp", host.doc),
        Some(json!({"backend":"fixture"})),
    )
    .await;
    assert_eq!(value["ready"], true, "{value}");
    host
}
</storycode:copy>
<a id="f-35273396ec57-request"></a>
### `request`

<storycode:copy id="f-35273396ec57-request">
async fn request(host: &Host, path: &str, body: Option<Value>) -> (u16, Value) {
    let client = reqwest::Client::new();
    let req = if let Some(body) = body {
        client.post(format!("{}{path}", host.base)).json(&body)
    } else {
        client.get(format!("{}{path}", host.base))
    };
    let response = req.send().await.unwrap();
    (response.status().as_u16(), response.json().await.unwrap())
}
</storycode:copy>
<a id="f-35273396ec57-run"></a>
### `run`

<storycode:copy id="f-35273396ec57-run">
async fn run(host: &Host, prompt: &str) -> String {
    let (status, value) = request(
        host,
        &format!("/api/docs/{}/agent", host.doc),
        Some(json!({"backend":"fixture","prompt":prompt})),
    )
    .await;
    assert_eq!(status, 202, "{value}");
    value["session_id"].as_str().unwrap().into()
}
</storycode:copy>
<a id="f-35273396ec57-finished"></a>
### `finished`

<storycode:copy id="f-35273396ec57-finished">
async fn finished(host: &Host, turn: &str) -> Value {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let (_, value) =
                request(host, &format!("/api/docs/{}/agent/turns", host.doc), None).await;
            if let Some(turn) = value["turns"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == turn && t["status"] != "running")
            {
                return turn.clone();
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("turn did not finish")
}
</storycode:copy>
<a id="f-35273396ec57-negotiated_agent_repairs_artifact_via_mcp_and_acp_before_completing"></a>
### `negotiated_agent_repairs_artifact_via_mcp_and_acp_before_completing`

<storycode:copy id="f-35273396ec57-negotiated_agent_repairs_artifact_via_mcp_and_acp_before_completing">
#[tokio::test]
async fn negotiated_agent_repairs_artifact_via_mcp_and_acp_before_completing() {
    let host = host(true).await;
    let turn = run(&host, "contract-repair").await;
    let done = finished(&host, &turn).await;
    assert_eq!(done["status"], "ok", "{done}");
    let (status, capture) = request(
        &host,
        &format!("/api/docs/{}/agent/projects/{turn}", host.doc),
        None,
    )
    .await;
    assert_eq!(status, 200, "{capture}");
    let source = capture["artifact"]["sources"]["response.md"]
        .as_str()
        .unwrap();
    assert!(source.contains("contract B"));
    assert!(!source.contains("abandoned draft"));
    let a: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["starting"].clone()).unwrap();
    let b: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["completed"].clone()).unwrap();
    let evaluated = hick_lang::evaluate_project(
        "response.md",
        &std::collections::BTreeMap::from([("response.md".into(), source.into())]),
        &a.restore().unwrap(),
    )
    .unwrap();
    assert!(evaluated.new.compare(&b.restore().unwrap()).is_empty());
    assert!(
        capture["ambiguous"]
            .as_array()
            .unwrap()
            .contains(&json!("app.ts")),
        "Equality must not manufacture exclusive authorship"
    );
    let record =
        std::fs::read_to_string(host.root.join(done["session"].as_str().unwrap())).unwrap();
    assert!(record.contains("changeset-repair"));
    assert!(record.contains("output-mismatch"));
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "contract B\n"
    );
}
</storycode:copy>
<a id="f-35273396ec57-legacy_adapter_keeps_ordinary_completion_without_claiming_artifact_acceptance"></a>
### `legacy_adapter_keeps_ordinary_completion_without_claiming_artifact_acceptance`

<storycode:copy id="f-35273396ec57-legacy_adapter_keeps_ordinary_completion_without_claiming_artifact_acceptance">
#[tokio::test]
async fn legacy_adapter_keeps_ordinary_completion_without_claiming_artifact_acceptance() {
    let host = host(false).await;
    let turn = run(&host, "literate-native").await;
    let done = finished(&host, &turn).await;
    assert_eq!(done["status"], "ok", "{done}");
    let (_, capture) = request(
        &host,
        &format!("/api/docs/{}/agent/projects/{turn}", host.doc),
        None,
    )
    .await;
    assert!(capture["artifact"].is_null());
    assert!(capture["response"].is_null());
}
</storycode:copy>
<a id="f-35273396ec57-negotiated_completion_without_artifact_stops_after_bounded_repairs"></a>
### `negotiated_completion_without_artifact_stops_after_bounded_repairs`

<storycode:copy id="f-35273396ec57-negotiated_completion_without_artifact_stops_after_bounded_repairs">
#[tokio::test]
async fn negotiated_completion_without_artifact_stops_after_bounded_repairs() {
    let host = host(true).await;
    let turn = run(&host, "contract-never").await;
    let done = finished(&host, &turn).await;
    assert_eq!(done["status"], "error", "{done}");
    assert!(done["error"].as_str().unwrap().contains("three repairs"));
    let (_, capture) = request(
        &host,
        &format!("/api/docs/{}/agent/projects/{turn}", host.doc),
        None,
    )
    .await;
    assert!(capture["artifact"].is_null());
    assert!(
        capture["error"]
            .as_str()
            .unwrap()
            .contains("did not complete")
    );
}
</storycode:copy>
<a id="f-35273396ec57-cancellation_is_not_a_failed_artifact_and_does_not_trigger_repairs"></a>
### `cancellation_is_not_a_failed_artifact_and_does_not_trigger_repairs`

<storycode:copy id="f-35273396ec57-cancellation_is_not_a_failed_artifact_and_does_not_trigger_repairs">
#[tokio::test]
async fn cancellation_is_not_a_failed_artifact_and_does_not_trigger_repairs() {
    let host = host(true).await;
    let turn = run(&host, "contract-cancel").await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    let (status, _) = request(
        &host,
        &format!("/api/docs/{}/agent/stop", host.doc),
        Some(json!({})),
    )
    .await;
    assert_eq!(status, 200);
    let done = finished(&host, &turn).await;
    assert_eq!(done["status"], "stopped", "{done}");
}

</storycode:copy>
<a id="f-35273396ec57-a_native_write_after_acceptance_requires_a_new_artifact"></a>
### `a_native_write_after_acceptance_requires_a_new_artifact`

<storycode:copy id="f-35273396ec57-a_native_write_after_acceptance_requires_a_new_artifact">
#[tokio::test]
async fn a_native_write_after_acceptance_requires_a_new_artifact() {
    let host = host(true).await;
    let turn = run(&host, "contract-late-write").await;
    let done = finished(&host, &turn).await;
    assert_eq!(done["status"], "ok", "{done}");
    let (_, capture) = request(
        &host,
        &format!("/api/docs/{}/agent/projects/{turn}", host.doc),
        None,
    )
    .await;
    let source = capture["artifact"]["sources"]["response.md"]
        .as_str()
        .unwrap();
    assert!(source.contains("late C"));
    assert!(!source.contains("contract B"));
    let record =
        std::fs::read_to_string(host.root.join(done["session"].as_str().unwrap())).unwrap();
    assert!(record.contains("changeset-repair"));
}

</storycode:copy>
<a id="f-35273396ec57-standalone_mcp_freezes_one_task_across_engine_forwarding"></a>
### `standalone_mcp_freezes_one_task_across_engine_forwarding`

<storycode:copy id="f-35273396ec57-standalone_mcp_freezes_one_task_across_engine_forwarding">
#[tokio::test]
async fn standalone_mcp_freezes_one_task_across_engine_forwarding() {
    let engine_state = tempfile::tempdir().unwrap();
    for workspace in [false, true] {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("project.md"),
            if workspace {
                "# Ordinary note\n"
            } else {
                "<hick:project/>"
            },
        )
        .unwrap();
        std::fs::write(dir.path().join("app.ts"), "dirty A\n").unwrap();
        let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_hick"))
            .args(["mcp", "--doc", "project.md"])
            .env("HICKORY_STATE_DIR", engine_state.path())
            .current_dir(dir.path())
            .env("HICKORY_EXECUTOR", "local")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        async fn call(
            stdin: &mut tokio::process::ChildStdin,
            lines: &mut tokio::io::Lines<BufReader<tokio::process::ChildStdout>>,
            name: &str,
            args: Value,
        ) -> Value {
            let message = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":args}});
            stdin
                .write_all(format!("{message}\n").as_bytes())
                .await
                .unwrap();
            stdin.flush().await.unwrap();
            let line = tokio::time::timeout(Duration::from_secs(30), lines.next_line())
                .await
                .unwrap()
                .unwrap()
                .expect("MCP closed unexpectedly");
            serde_json::from_str(&line).unwrap()
        }
        let before = call(&mut stdin, &mut lines, "changeset_status", json!({})).await;
        assert_eq!(before["result"]["isError"], true);
        let begin = call(
            &mut stdin,
            &mut lines,
            "begin_changeset",
            if workspace {
                json!({"workspace":".","include_snapshots":true})
            } else {
                json!({"include_snapshots":true})
            },
        )
        .await;
        assert_eq!(
            begin["result"]["isError"], false,
            "workspace={workspace}: {begin}"
        );
        let start: Value =
            serde_json::from_str(begin["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            start["starting"]["entries"]["app.ts"]["bytes"],
            json!(b"dirty A\n".to_vec())
        );
        std::fs::write(dir.path().join("app.ts"), "final B\n").unwrap();
        let reset = call(&mut stdin, &mut lines, "begin_changeset", json!({})).await;
        assert_eq!(reset["result"]["isError"], true);
        let status = call(&mut stdin, &mut lines, "changeset_status", json!({})).await;
        let status: Value =
            serde_json::from_str(status["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        let args = json!({"boundary":status["boundary"],"expected":status["expected"],"entry":"response.md","sources":{"response.md":status["template"]}});
        let submitted = call(&mut stdin, &mut lines, "submit_changeset", args).await;
        assert_eq!(submitted["result"]["isError"], false, "{submitted}");
        let accepted: Value =
            serde_json::from_str(submitted["result"]["content"][0]["text"].as_str().unwrap())
                .unwrap();
        assert_eq!(accepted["accepted"], true);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("app.ts")).unwrap(),
            "final B\n"
        );
        assert!(!dir.path().join("response.md").exists());
        drop(stdin);
        assert!(
            tokio::time::timeout(Duration::from_secs(15), child.wait())
                .await
                .unwrap()
                .unwrap()
                .success()
        );
    }
}

</storycode:copy>
<a id="f-35273396ec57-ordinary_workspace_and_folderless_editors_share_repair_lifecycle"></a>
### `ordinary_workspace_and_folderless_editors_share_repair_lifecycle`

A real controls client supplies authoritative snapshots while the subprocess
exercises both ACP and MCP repair. No application-internal request harness.

<storycode:copy id="f-35273396ec57-ordinary_workspace_and_folderless_editors_share_repair_lifecycle">
// A real controls client supplies authoritative snapshots while the subprocess
// exercises both ACP and MCP repair. No application-internal request harness.
#[tokio::test]
async fn ordinary_workspace_and_folderless_editors_share_repair_lifecycle() {
    for folder in [true, false] {
        let host = host_scope(true, folder, false).await;
        let mut buffers = json!([
            {"id":"one","name":"Untitled","path":null,"content":"editor A\n","focused":true},
            {"id":"two","name":"Untitled","path":null,"content":"second A\n"}
        ]);
        if folder {
            buffers.as_array_mut().unwrap().push(
                json!({"id":"saved","name":"app.ts","path":"app.ts","content":"dirty editor A\n"}),
            );
        }
        let mut completed = buffers.clone();
        if folder {
            completed[2]["content"] = json!("dirty editor B\n");
        }
        completed[0]["content"] = json!("editor B\n");
        completed[1]["content"] = json!("second B\n");
        let base = host.base.clone();
        let editors = tokio::spawn(async move {
            let client = reqwest::Client::new();
            loop {
                let state: Value = client
                    .get(format!("{base}/api/docs/workspace/agent/acp"))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                if let Some(id) = state["edits"]["captureRequest"].as_str() {
                    let response = client.post(format!("{base}/api/docs/workspace/agent/acp/edits")).json(&json!({"capture_request":id,"context":{"live":true,"buffers":completed}})).send().await.unwrap();
                    assert_eq!(response.status(), 200);
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        let (status, started) = request(&host, "/api/docs/workspace/agent", Some(json!({"backend":"fixture","prompt":"contract-repair","context":{"live":true,"buffers":buffers}}))).await;
        assert_eq!(status, 202, "{started}");
        let turn = started["session_id"].as_str().unwrap();
        let done = finished(&host, turn).await;
        editors.abort();
        assert_eq!(done["status"], "ok", "{done}");
        let (_, capture) = request(
            &host,
            &format!("/api/docs/workspace/agent/projects/{turn}"),
            None,
        )
        .await;
        assert!(capture["artifact"].is_object(), "{capture}");
        assert_eq!(capture["workspace"]["root"].is_string(), folder);
        let a: hick_lang::ProjectSnapshot =
            serde_json::from_value(capture["starting"].clone()).unwrap();
        let b: hick_lang::ProjectSnapshot =
            serde_json::from_value(capture["completed"].clone()).unwrap();
        let a = a.restore().unwrap();
        let b = b.restore().unwrap();
        assert_eq!(a.get("app.ts").is_some(), folder);
        assert_eq!(
            a.entries()
                .keys()
                .filter(|p| p.starts_with(hick_lang::EDITOR_BUFFER_PREFIX))
                .count(),
            if folder { 3 } else { 2 }
        );
        let artifact = &capture["artifact"];
        let sources = serde_json::from_value(artifact["sources"].clone()).unwrap();
        let project =
            hick_lang::evaluate_project(artifact["entry"].as_str().unwrap(), &sources, &a).unwrap();
        assert!(project.new.compare(&b).is_empty());
        assert!(!host.root.join("response.md").exists());
        assert!(!host.root.join(".git").exists());
    }
}

</storycode:copy>
<a id="f-35273396ec57-browser_workspace_changesets"></a>
### `browser_workspace_changesets`

<storycode:copy id="f-35273396ec57-browser_workspace_changesets">
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "Starts Vite and Playwright; run just test-workspace-changesets-browser"]
async fn browser_workspace_changesets() {
    use std::{
        io::{BufRead, BufReader},
        process::{Command, Stdio},
    };
    struct Child(std::process::Child);
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let web = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/web");
    for folder in [true, false] {
        let host = host_scope(true, folder, false).await;
        let mut vite = Child(
            Command::new("node")
                .args(["node_modules/vite/bin/vite.js", "--clearScreen", "false"])
                .current_dir(&web)
                .env("HICKORY_API_ORIGIN", &host.base)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .unwrap(),
        );
        let mut url = String::new();
        let reader = BufReader::new(vite.0.stdout.take().unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            if let Some(start) = line.find("http://127.0.0.1:") {
                url = line[start..].split_whitespace().next().unwrap().to_owned();
                break;
            }
        }
        assert!(!url.is_empty(), "Vite did not start");
        let status = Command::new("npx")
            .args(["playwright", "test", "e2e/workspace-changesets.spec.ts"])
            .current_dir(&web)
            .env("HICKORY_E2E_CHANNEL", "chromium")
            .env("HICKORY_E2E_URL", url)
            .env("HICKORY_CHANGESET_BROWSER", "1")
            .env("HICKORY_CHANGESET_FOLDER", folder.to_string())
            .status()
            .unwrap();
        assert!(
            status.success(),
            "Workspace changeset browser flow failed (folder={folder})"
        );
        assert!(!host.root.join("__hickory_editor_buffers__").exists());
    }
}

</storycode:copy>
[Read the test `accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b` beside its requirement](../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md#f-35273396ec57-accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b).

### Assemble `crates/hickory-cli/tests/agent_changesets.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/agent_changesets.rs"><storycode:paste select="#f-35273396ec57-Host"/><storycode:paste select="#f-35273396ec57-host"/><storycode:paste select="#f-35273396ec57-host_scope"/><storycode:paste select="#f-35273396ec57-request"/><storycode:paste select="#f-35273396ec57-run"/><storycode:paste select="#f-35273396ec57-finished"/><storycode:paste select="#f-35273396ec57-negotiated_agent_repairs_artifact_via_mcp_and_acp_before_completing"/><storycode:paste select="#f-35273396ec57-legacy_adapter_keeps_ordinary_completion_without_claiming_artifact_acceptance"/><storycode:paste select="#f-35273396ec57-negotiated_completion_without_artifact_stops_after_bounded_repairs"/><storycode:paste select="#f-35273396ec57-cancellation_is_not_a_failed_artifact_and_does_not_trigger_repairs"/><storycode:paste select="#f-35273396ec57-a_native_write_after_acceptance_requires_a_new_artifact"/><storycode:paste select="#f-35273396ec57-standalone_mcp_freezes_one_task_across_engine_forwarding"/><storycode:paste select="#f-35273396ec57-ordinary_workspace_and_folderless_editors_share_repair_lifecycle"/><storycode:paste select="#f-35273396ec57-browser_workspace_changesets"/><storycode:paste select="#f-35273396ec57-accepted_process_result_b_can_be_revised_to_c_without_replaying_a_over_b"/></storycode:file>

</storycode:doc>
