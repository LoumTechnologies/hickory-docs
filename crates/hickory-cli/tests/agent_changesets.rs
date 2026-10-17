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
async fn host(negotiated: bool) -> Host {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("project.md"),
        "<hick:project/><hick:file path=\"app.ts\">declared\n</hick:file>",
    )
    .unwrap();
    std::fs::write(root.join("app.ts"), "dirty A\n").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join(".hick-cache/ui.json")),
    })
    .await
    .unwrap();
    let doc = prepared.state.index.id_for_path("project.md");
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

#[tokio::test]
async fn standalone_mcp_freezes_one_task_across_engine_forwarding() {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("project.md"), "<hick:project/>").unwrap();
    std::fs::write(dir.path().join("app.ts"), "dirty A\n").unwrap();
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_hick"))
        .args(["mcp", "--doc", "project.md"])
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
        json!({"include_snapshots":true}),
    )
    .await;
    assert_eq!(begin["result"]["isError"], false, "{begin}");
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
        serde_json::from_str(submitted["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
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
