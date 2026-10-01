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

const DOC: &str = r#"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="note.md">
# A note
<hick:file path="hello.py">print("original")
</hick:file>
</hick:doc>
"#;

struct Project {
    dir: tempfile::TempDir,
    state: tempfile::TempDir,
}
impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("note.hick"), DOC).unwrap();
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
        if let Ok(bytes) = std::fs::read(self.state.path().join("engine/endpoint.json")) {
            if let Ok(endpoint) = serde_json::from_slice::<Value>(&bytes) {
                kill(endpoint["pid"].as_u64().unwrap() as u32);
            }
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

// Guarantee: docs/guarantees/agent/acp-agents-are-first-class.md
#[tokio::test(flavor = "multi_thread")]
async fn desktop_proxy_routes_acp_connection_configuration_and_turns() {
    use futures::StreamExt;
    let project = Project::new();
    // Use the current Markdown document surface for this regression.
    std::fs::remove_file(project.dir.path().join("note.hick")).unwrap();
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
            .arg(project.dir.path().join("note.hick"))
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
    std::fs::write(
        sub.join("child.hick"),
        DOC.replace("note.md", "child.md"),
    )
    .unwrap();
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
    wait_file(&sub.join("child.hick"), "reverse").await;
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
