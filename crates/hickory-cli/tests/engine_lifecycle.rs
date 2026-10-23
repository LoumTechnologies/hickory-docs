//! Guarantees: docs/guarantees/authoring/one-loop-owns-a-directory.md
//! docs/guarantees/authoring/the-app-and-the-cli-are-one-engine.md
//! Real executables, HTTP and WebSockets; no in-process routing harness.
use serde_json::{Value, json};
use std::{
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const DOC: &str = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
# A note
<hick:file path="hello.py">print("original")
</hick:file>
</hick:doc>
"#;

async fn wait_daemon_exit(project: &Project) {
    let endpoint = project.state.path().join("engine/endpoint.json");
    let deadline = Instant::now() + Duration::from_secs(14);
    while endpoint.exists() {
        assert!(Instant::now() < deadline, "daemon survived its last client");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_exits_after_last_detach_even_with_an_open_terminal() {
    let project = Project::new();
    let mut daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let a = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    let b = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    for owner in [&a, &b] {
        http.post(format!("{url}/attach"))
            .bearer_auth(token)
            .json(owner)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
    let terminal: Value = http
        .post(format!("{url}/clients/{}/api/terminals", b.client))
        .bearer_auth(token)
        .json(&json!({"title":"lifecycle witness", "argv": if cfg!(windows) { vec!["cmd.exe"] } else { vec!["/bin/sh"] }}))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(terminal["exit_code"].is_null());
    http.delete(format!("{url}/clients/{}/lease", a.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    tokio::time::sleep(Duration::from_secs(3)).await;
    assert!(
        daemon.child.try_wait().unwrap().is_none(),
        "another client still owns the daemon"
    );
    http.delete(format!("{url}/clients/{}/lease", b.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let detached = Instant::now();
    wait_daemon_exit(&project).await;
    assert!(
        detached.elapsed() < Duration::from_secs(6),
        "explicit detach waited for a stale lease"
    );
    let deadline = Instant::now() + Duration::from_secs(3);
    while daemon.child.try_wait().unwrap().is_none() {
        assert!(
            Instant::now() < deadline,
            "endpoint disappeared but daemon remained"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(project.state.path().join("engine/owner.lock").exists());
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_expires_crashed_clients_but_keeps_the_remaining_window() {
    let project = Project::new();
    let a = project.client(project.dir.path(), "A");
    let b = project.client(project.dir.path(), "B");
    let pid = project.endpoint()["pid"].clone();
    drop(a); // Kill without detaching: exercise lease expiry.
    tokio::time::sleep(Duration::from_secs(13)).await;
    assert_eq!(project.endpoint()["pid"], pid);
    assert!(docs(&b).await.is_array());
    drop(b);
    wait_daemon_exit(&project).await;
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[tokio::test(flavor = "multi_thread")]
async fn daemon_exits_after_a_one_shot_cli_command() {
    let project = Project::new();
    let status = project
        .hick()
        .arg("weave")
        .arg(project.dir.path().join("note.md"))
        .status()
        .unwrap();
    assert!(status.success());
    wait_daemon_exit(&project).await;
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_expires_an_abandoned_cli_even_while_its_worker_runs() {
    let project = Project::new();
    let window = project.client(project.dir.path(), "bootstrap");
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(40))
        .build()
        .unwrap();
    let lease = format!("{url}/commands/long-cli/lease");
    http.post(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let request = http
        .post(format!("{url}/command"))
        .bearer_auth(token)
        .json(&json!({
            "client":"long-cli", "executable":"/bin/sleep", "args":["60"],
            "cwd":project.dir.path(), "env":[], "input":[]
        }));
    let response = tokio::spawn(async move { request.send().await });
    drop(window);
    // Outlive the window's crashed lease while the foreground CLI owns work.
    for _ in 0..7 {
        tokio::time::sleep(Duration::from_secs(2)).await;
        http.post(&lease)
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap();
    }
    assert_eq!(project.endpoint()["pid"], endpoint["pid"]);
    assert!(
        !response.is_finished(),
        "long-running CLI worker unexpectedly finished"
    );
    // Simulate CLI process death: no detach and no more heartbeats.
    wait_daemon_exit(&project).await;
    let result = tokio::time::timeout(Duration::from_secs(3), response)
        .await
        .unwrap()
        .unwrap();
    assert!(result.is_err(), "orphaned worker prevented shutdown");
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_cancels_background_preparation_after_last_client_exits() {
    let project = Project::new();
    let signals = tempfile::tempdir().unwrap();
    let started = signals.path().join("started");
    let quoted = started.to_string_lossy().replace('\'', "'\\''");
    std::fs::write(project.dir.path().join("note.md"), format!(
        "<hick:container name=\"sh\" image=\"alpine:3.20\" />\n<hick:exec container=\"sh\">printf started > '{quoted}'; exec sleep 60</hick:exec>\n"
    )).unwrap();
    let _daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder().no_proxy().build().unwrap();
    let mut owner = hickory_cli::engine::Attach::new(
        project.dir.path().into(),
        hickory_cli::ExecutorChoice::Local,
    );
    owner.background = true;
    owner.run = true;
    http.post(format!("{url}/attach"))
        .bearer_auth(token)
        .json(&owner)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    wait_file(&started, "started").await;
    http.delete(format!("{url}/clients/{}/lease", owner.client))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    wait_daemon_exit(&project).await;
}

// Guarantee: docs/guarantees/authoring/the-daemon-exits-with-its-last-client.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh() {
    let project = Project::new();
    let _daemon = project.daemon().await;
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .no_proxy()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let lease = format!("{url}/commands/retiring-cli/lease");
    http.post(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let request = http
        .post(format!("{url}/command"))
        .bearer_auth(token)
        .json(&json!({
            "client":"retiring-cli", "executable":"/bin/sleep", "args":["60"],
            "cwd":project.dir.path(), "env":[], "input":[]
        }));
    let response = tokio::spawn(async move { request.send().await });
    tokio::time::sleep(Duration::from_millis(100)).await;
    http.delete(&lease)
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(6);
    while http
        .get(format!("{url}/health"))
        .bearer_auth(token)
        .send()
        .await
        .is_ok()
    {
        assert!(Instant::now() < deadline, "daemon did not begin retirement");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let reopened = project.client(project.dir.path(), "reopened");
    assert_ne!(project.endpoint()["pid"], endpoint["pid"]);
    assert!(docs(&reopened).await.is_array());
    assert!(response.await.unwrap().is_err());
    drop(reopened);
    wait_daemon_exit(&project).await;
}

struct Project {
    dir: tempfile::TempDir,
    state: tempfile::TempDir,
}
impl Project {
    async fn daemon(&self) -> Client {
        let child = self.hick().arg("__engine").spawn().unwrap();
        let daemon = Client {
            child,
            url: String::new(),
        };
        let deadline = Instant::now() + Duration::from_secs(10);
        while !self.state.path().join("engine/endpoint.json").exists() {
            assert!(Instant::now() < deadline, "daemon did not start");
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        daemon
    }
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.md"), DOC).unwrap();
        Self { dir, state }
    }
    fn client(&self, root: &Path, name: &str) -> Client {
        let bin = PathBuf::from(env!("CARGO_BIN_EXE_hick"))
            .parent()
            .unwrap()
            .join("examples")
            .join(format!("engine_client{}", std::env::consts::EXE_SUFFIX));
        let mut child = Command::new(bin)
            .arg(root)
            .arg(name)
            .env("HICKORY_STATE_DIR", self.state.path())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        assert!(
            line.starts_with("http://127.0.0.1:"),
            "client failed: {line}"
        );
        Client {
            child,
            url: line.trim().into(),
        }
    }
    fn endpoint(&self) -> Value {
        serde_json::from_slice(
            &std::fs::read(self.state.path().join("engine/endpoint.json")).unwrap(),
        )
        .unwrap()
    }
    fn hick(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_hick"));
        command
            .env("HICKORY_STATE_DIR", self.state.path())
            .env("HICKORY_EXECUTOR", "local");
        command
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        if let Ok(bytes) = std::fs::read(self.state.path().join("engine/endpoint.json"))
            && let Ok(endpoint) = serde_json::from_slice::<Value>(&bytes)
        {
            kill(endpoint["pid"].as_u64().unwrap() as u32);
        }
    }
}
struct Client {
    child: Child,
    url: String,
}
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
fn kill(pid: u32) {
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", &pid.to_string()])
        .status();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .status();
}
async fn docs(client: &Client) -> Value {
    reqwest::get(format!("{}/api/projects/local/docs", client.url))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
async fn wait_file(path: &Path, expected: &str) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if std::fs::read_to_string(path).is_ok_and(|s| s.contains(expected)) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "{} never contained {expected}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

// Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
// A real running cell holds initialization open until the test releases it.
// This tests ordering, rather than a launch-time threshold sensitive to CI load.
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn background_attach_serves_editor_storage_before_outputs_are_ready() {
    let project = Project::new();
    let _client = project.client(project.dir.path(), "bootstrap");
    let workspace = tempfile::tempdir().unwrap();
    let signals = tempfile::tempdir().unwrap();
    let started = signals.path().join("started");
    let release = signals.path().join("release");
    let quote = |path: &Path| format!("'{}'", path.display().to_string().replace('\'', "'\\''"));
    let source = format!(
        r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
<hick:container name="sh" image="alpine:3.20" />
<hick:exec container="sh">
printf started > {}
i=0; while [ ! -f {} ]; do i=$((i+1)); [ "$i" -lt 300 ] || exit 1; sleep 0.1; done
printf finished
</hick:exec>
<hick:file path="owned.txt">owned by the document</hick:file>
</hick:doc>
"#,
        quote(&started),
        quote(&release)
    );
    std::fs::write(workspace.path().join("slow.md"), &source).unwrap();
    std::fs::write(workspace.path().join("plain.txt"), "before").unwrap();
    let endpoint = project.endpoint();
    let url = endpoint["url"].as_str().unwrap();
    let token = endpoint["token"].as_str().unwrap();
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let mut opts = hickory_cli::engine::Attach::new(
        workspace.path().to_path_buf(),
        hickory_cli::ExecutorChoice::Local,
    );
    opts.background = true;
    opts.run = true;
    let response = http
        .post(format!("{url}/attach"))
        .bearer_auth(token)
        .json(&opts)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    wait_file(&started, "started").await;
    let api = format!("{url}/clients/{}/api", opts.client);
    let status: Value = http
        .get(format!("{api}/startup"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["ready"], false);
    let draft = json!({ "path": "untitled:startup", "contents": "Writing while the workspace loads", "base": "" });
    let response = http
        .put(format!("{api}/workspace/drafts"))
        .bearer_auth(token)
        .json(&draft)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let drafts: Value = http
        .get(format!("{api}/workspace/drafts"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(drafts["drafts"][0]["contents"], draft["contents"]);
    for path in ["settings/ui", "workspace/ui", "files"] {
        assert_eq!(
            http.get(format!("{api}/{path}"))
                .bearer_auth(token)
                .send()
                .await
                .unwrap()
                .status(),
            200
        );
    }
    let edit = json!({ "path": "plain.txt", "content": "after" });
    let response = http
        .put(format!("{api}/file"))
        .bearer_auth(token)
        .json(&edit)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 503);
    assert_eq!(response.headers()["x-hickory-startup"], "pending");
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("plain.txt")).unwrap(),
        "before"
    );
    let response = http
        .get(format!("{api}/file?path=owned.txt"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        503,
        "output reads must wait for ownership too"
    );
    std::fs::write(release, "go").unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let status: Value = http
            .get(format!("{api}/startup"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        if status["ready"] == true {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "workspace never finished: {status}"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("owned.txt")).unwrap(),
        "owned by the document"
    );
    let response = http
        .put(format!("{api}/file"))
        .bearer_auth(token)
        .json(&edit)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("plain.txt")).unwrap(),
        "after"
    );
    assert_eq!(
        std::fs::read_to_string(workspace.path().join("slow.md")).unwrap(),
        source
    );
}

// Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
#[tokio::test(flavor = "multi_thread")]
async fn desktop_proxy_routes_acp_connection_configuration_and_turns() {
    use futures::StreamExt;
    let project = Project::new();
    // Use the current Markdown document surface for this regression.
    std::fs::remove_file(project.dir.path().join("note.md")).unwrap();
    std::fs::write(
        project.dir.path().join("note.md"),
        DOC.replace("weave=\"note.md\"", "weave=\"reading.md\""),
    )
    .unwrap();
    let client = project.client(project.dir.path(), "ACP");
    let listing = docs(&client).await;
    assert!(
        listing.is_array(),
        "document listing through desktop proxy: {listing}"
    );
    let id = listing[0]["id"].as_str().unwrap().to_owned();
    let peer = PathBuf::from(env!("CARGO_BIN_EXE_hick"))
        .parent()
        .unwrap()
        .join("examples")
        .join(format!("acp_fixture{}", std::env::consts::EXE_SUFFIX));
    assert!(peer.is_file(), "build the ACP peer with just test-engine");
    let http = reqwest::Client::new();
    let response = http
        .put(format!("{}/api/agents", client.url))
        .json(&json!([{"id":"fixture","name":"ACP fixture","command":peer,"args":["--auth"]}]))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let base = format!("{}/api/docs/{id}/agent", client.url);
    let response = http
        .post(format!("{base}/acp"))
        .json(&json!({"backend":"fixture"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let connected: Value = response.json().await.unwrap();
    assert_eq!(connected["ready"], false, "{connected}");
    let response = http
        .post(format!("{base}/acp/authenticate"))
        .json(&json!({"method_id":"login"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let authenticated: Value = response.json().await.unwrap();
    assert_eq!(authenticated["ready"], true, "{authenticated}");
    let response = http.get(format!("{base}/acp")).send().await.unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let response = http
        .post(format!("{base}/acp/configure"))
        .json(&json!({"config_id":"model","value":"other"}))
        .send()
        .await
        .unwrap();
    assert!(
        response.status().is_success(),
        "{}",
        response.text().await.unwrap()
    );
    let (mut socket, _) = tokio_tungstenite::connect_async(format!(
        "{}/api/ws?doc=doc:{id}",
        client.url.replacen("http://", "ws://", 1)
    ))
    .await
    .unwrap();
    let response = http
        .post(&base)
        .json(&json!({"backend":"fixture","prompt":"hello"}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202, "{}", response.text().await.unwrap());
    let sent: Value = response.json().await.unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let response = http.get(format!("{base}/turns")).send().await.unwrap();
        assert!(
            response.status().is_success(),
            "{}",
            response.text().await.unwrap()
        );
        let turns: Value = response.json().await.unwrap();
        if let Some(turn) = turns["turns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == sent["session_id"] && t["status"] != "running")
        {
            assert_eq!(turn["status"], "ok", "{turn}");
            assert_eq!(turn["provider"], "acp:fixture");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "ACP turn never finished: {turns}"
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut streamed = false;
        while let Some(message) = socket.next().await {
            if let tokio_tungstenite::tungstenite::Message::Binary(bytes) = message.unwrap()
                && bytes.first() == Some(&0x01)
            {
                let event: Value = serde_json::from_slice(&bytes[1..]).unwrap();
                if event["run_id"] != sent["session_id"] {
                    continue;
                }
                streamed |= event["event"]["kind"] == "token";
                if event["status"] == "ok" {
                    assert!(streamed);
                    return;
                }
            }
        }
        panic!("agent socket ended before the turn completed");
    })
    .await
    .expect("agent turn never arrived through the desktop WebSocket");
    socket.close(None).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn windows_share_one_engine_and_one_live_document() {
    let project = Project::new();
    let a = project.client(project.dir.path(), "A");
    let pid = project.endpoint()["pid"].clone();
    let b = project.client(project.dir.path(), "B");
    assert_eq!(project.endpoint()["pid"], pid);
    assert_eq!(docs(&a).await, docs(&b).await);
    let id = docs(&a).await[0]["id"].as_str().unwrap().to_string();
    let http = reqwest::Client::new();
    let result = http
        .put(format!("{}/api/docs/{id}", a.url))
        .json(&json!({"source":DOC.replace("original", "changed")}))
        .send()
        .await
        .unwrap();
    assert!(
        result.status().is_success(),
        "{}",
        result.text().await.unwrap()
    );
    wait_file(&project.dir.path().join("hello.py"), "changed").await;
    let source: Value = http
        .get(format!("{}/api/docs/{id}", b.url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(source["source"].as_str().unwrap().contains("changed"));
    drop(a);
    assert!(
        reqwest::get(format!("{}/api/health", b.url))
            .await
            .unwrap()
            .status()
            .is_success()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_cli_and_a_window_attach_in_either_order() {
    for cli_first in [true, false] {
        let project = Project::new();
        let mut up = if cli_first {
            Some(
                project
                    .hick()
                    .arg("up")
                    .arg(project.dir.path())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };
        let a = project.client(project.dir.path(), "A");
        if up.is_none() {
            up = Some(
                project
                    .hick()
                    .arg("up")
                    .arg(project.dir.path())
                    .stderr(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        }
        let mut up = up.unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        assert!(
            up.try_wait().unwrap().is_none(),
            "second client refused to attach"
        );
        let output = project
            .hick()
            .arg("run")
            .arg(project.dir.path().join("note.md"))
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = up.kill();
        let _ = up.wait();
        assert!(
            reqwest::get(format!("{}/api/health", a.url))
                .await
                .unwrap()
                .status()
                .is_success()
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn parent_and_child_views_share_room_identity_and_reverse_edits() {
    let project = Project::new();
    let sub = project.dir.path().join("child");
    std::fs::create_dir(&sub).unwrap();
    std::fs::write(sub.join("child.md"), DOC.replace("note.md", "child.md")).unwrap();
    let child = project.client(&sub, "child"); // Narrow view first is the harder case.
    let parent = project.client(project.dir.path(), "parent");
    let child_id = docs(&child).await[0]["id"].clone();
    assert!(
        docs(&parent)
            .await
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d["id"] == child_id)
    );
    let path = sub.join("hello.py");
    wait_file(&path, "original").await;
    std::fs::write(&path, "print(\"reverse\")\n").unwrap();
    wait_file(&sub.join("child.md"), "reverse").await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "print(\"reverse\")\n"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn window_drafts_layout_and_native_actions_are_independent() {
    let project = Project::new();
    let a = project.client(project.dir.path(), "A");
    let b = project.client(project.dir.path(), "B");
    let http = reqwest::Client::new();
    for (client, text) in [(&a, "A"), (&b, "B")] {
        assert!(
            http.put(format!("{}/api/workspace/ui", client.url))
                .json(&json!({"state":{"tab":text}}))
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
        );
        assert!(
            http.put(format!("{}/api/workspace/drafts", client.url))
                .json(&json!({"path":"untitled","contents":text}))
                .send()
                .await
                .unwrap()
                .status()
                .is_success()
        );
        let picked: Value = http
            .post(format!("{}/api/pick-folder", client.url))
            .json(&json!({}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            picked["path"],
            project.dir.path().join(text).to_string_lossy().as_ref()
        );
    }
    for (client, text) in [(&a, "A"), (&b, "B")] {
        let layout: Value = http
            .get(format!("{}/api/workspace/ui", client.url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(layout["state"]["tab"], text);
        let drafts: Value = http
            .get(format!("{}/api/workspace/drafts", client.url))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(drafts["drafts"][0]["contents"], text);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn engine_crash_reconnects_existing_clients_without_deleting_a_lock() {
    let project = Project::new();
    let a = project.client(project.dir.path(), "A");
    let b = project.client(project.dir.path(), "B");
    let original = project.endpoint()["pid"].as_u64().unwrap() as u32;
    kill(original);
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        let recovered = reqwest::get(format!("{}/api/health", a.url))
            .await
            .is_ok_and(|r| r.status().is_success())
            && reqwest::get(format!("{}/api/health", b.url))
                .await
                .is_ok_and(|r| r.status().is_success());
        if recovered {
            break;
        }
        assert!(Instant::now() < deadline, "engine did not recover");
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    assert_ne!(project.endpoint()["pid"].as_u64().unwrap() as u32, original);
    assert!(project.state.path().join("engine/owner.lock").is_file());
}

#[tokio::test(flavor = "multi_thread")]
async fn engine_requires_authentication_and_recovers_stale_discovery() {
    let project = Project::new();
    std::fs::create_dir(project.state.path().join("engine")).unwrap();
    std::fs::write(
        project.state.path().join("engine/endpoint.json"),
        "corrupt discovery",
    )
    .unwrap();
    let a = project.client(project.dir.path(), "A");
    let endpoint = project.endpoint();
    assert_eq!(
        reqwest::get(format!("{}/health", endpoint["url"].as_str().unwrap()))
            .await
            .unwrap()
            .status(),
        reqwest::StatusCode::UNAUTHORIZED
    );
    let malicious = reqwest::Client::new()
        .post(format!("{}/api/window/close", a.url))
        .header("origin", "https://unrelated.example")
        .send()
        .await
        .unwrap();
    assert_eq!(malicious.status(), reqwest::StatusCode::FORBIDDEN);
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn symlink_aliases_attach_to_the_same_workspace() {
    let project = Project::new();
    let alias = project.state.path().join("alias");
    std::os::unix::fs::symlink(project.dir.path(), &alias).unwrap();
    let a = project.client(project.dir.path(), "A");
    let b = project.client(&alias, "B");
    assert_eq!(docs(&a).await, docs(&b).await);
}

#[tokio::test]
async fn file_and_folder_windows_keep_independent_files_visibility() {
    let project = Project::new();
    std::fs::remove_file(project.dir.path().join("note.md")).unwrap();
    let path = project.dir.path().join("note.md");
    std::fs::write(&path, "# A note\n").unwrap();
    let folder = project.client(project.dir.path(), "folder-view");
    let file = project.client(&path, "file-view");
    let another_folder = project.client(project.dir.path(), "another-folder-view");
    for (client, expected) in [(&folder, true), (&file, false), (&another_folder, true)] {
        let response: Value = reqwest::get(format!("{}/api/files", client.url))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(response["folder_open"], expected);
    }
    assert_eq!(docs(&folder).await, docs(&file).await);
}

// A nested filesystem helper must share the engine's existing write act.
#[tokio::test]
async fn representation_edits_through_the_real_engine_do_not_deadlock_or_dirty_prose() {
    let project = Project::new();
    std::fs::write(project.dir.path().join("plain.py"), "value = 1\n").unwrap();
    let client = project.client(project.dir.path(), "literate-window");
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let response = http
        .post(format!("{}/api/representations", client.url))
        .json(&json!({"backing":{"kind":"files","paths":["plain.py"]}}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let v = response.json::<Value>().await.unwrap();
    let response=http.put(format!("{}/api/representations/{}",client.url,v["id"].as_str().unwrap())).json(&json!({"revision":v["revision"],"source":v["source"].as_str().unwrap().replace("value = 1","value = 2")})).send().await.unwrap();
    assert_eq!(response.status(), 200, "{}", response.text().await.unwrap());
    assert_eq!(
        std::fs::read_to_string(project.dir.path().join("plain.py")).unwrap(),
        "value = 2\n"
    );
    assert!(
        !project
            .dir
            .path()
            .join(format!("__lens-{}.md", v["id"].as_str().unwrap()))
            .exists()
    );
}
