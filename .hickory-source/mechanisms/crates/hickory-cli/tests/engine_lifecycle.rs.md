<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/engine_lifecycle.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6ea35164aaa4"></a>
## `crates/hickory-cli/tests/engine_lifecycle.rs`

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../guarantees/agent/acp-agents-are-first-class/contract.md), [A literate view writes through to ordinary source](../../../../guarantees/authoring/a-literate-view-writes-through-to-ordinary-source/contract.md), [One loop owns a directory and never reacts to itself](../../../../guarantees/authoring/one-loop-owns-a-directory/contract.md), [The Desktop App And The CLI Are One Engine](../../../../guarantees/authoring/the-app-and-the-cli-are-one-engine/contract.md), [The daemon exits with its last client](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md), [The Editor Does Not Wait For Workspace Preparation](../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md).

<a id="f-6ea35164aaa4-DOC"></a>
### `DOC`

Real executables, HTTP and WebSockets; no in-process routing harness.

<storycode:copy id="f-6ea35164aaa4-DOC">
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

</storycode:copy>
<a id="f-6ea35164aaa4-wait_daemon_exit"></a>
### `wait_daemon_exit`

<storycode:copy id="f-6ea35164aaa4-wait_daemon_exit">
async fn wait_daemon_exit(project: &Project) {
    let endpoint = project.state.path().join("engine/endpoint.json");
    let deadline = Instant::now() + Duration::from_secs(14);
    while endpoint.exists() {
        assert!(Instant::now() < deadline, "daemon survived its last client");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

</storycode:copy>
[Read the test `daemon_exits_after_last_detach_even_with_an_open_terminal` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_exits_after_last_detach_even_with_an_open_terminal).
[Read the test `daemon_expires_crashed_clients_but_keeps_the_remaining_window` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_expires_crashed_clients_but_keeps_the_remaining_window).
[Read the test `daemon_exits_after_a_one_shot_cli_command` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_exits_after_a_one_shot_cli_command).
[Read the test `daemon_expires_an_abandoned_cli_even_while_its_worker_runs` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_expires_an_abandoned_cli_even_while_its_worker_runs).
[Read the test `daemon_cancels_background_preparation_after_last_client_exits` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_cancels_background_preparation_after_last_client_exits).
[Read the test `daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh` beside its requirement](../../../../guarantees/authoring/the-daemon-exits-with-its-last-client/contract.md#f-6ea35164aaa4-daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh).
<a id="f-6ea35164aaa4-Project"></a>
### `Project`

<storycode:copy id="f-6ea35164aaa4-Project">
struct Project {
    dir: tempfile::TempDir,
    state: tempfile::TempDir,
}
</storycode:copy>
<a id="f-6ea35164aaa4-impl-Project"></a>
### `impl-Project`

<storycode:copy id="f-6ea35164aaa4-impl-Project">
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
</storycode:copy>
<a id="f-6ea35164aaa4-impl-Drop-for-Project"></a>
### `impl-Drop for Project`

<storycode:copy id="f-6ea35164aaa4-impl-Drop-for-Project">
impl Drop for Project {
    fn drop(&mut self) {
        if let Ok(bytes) = std::fs::read(self.state.path().join("engine/endpoint.json"))
            && let Ok(endpoint) = serde_json::from_slice::<Value>(&bytes)
        {
            kill(endpoint["pid"].as_u64().unwrap() as u32);
        }
    }
}
</storycode:copy>
<a id="f-6ea35164aaa4-Client"></a>
### `Client`

<storycode:copy id="f-6ea35164aaa4-Client">
struct Client {
    child: Child,
    url: String,
}
</storycode:copy>
<a id="f-6ea35164aaa4-impl-Drop-for-Client"></a>
### `impl-Drop for Client`

<storycode:copy id="f-6ea35164aaa4-impl-Drop-for-Client">
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
</storycode:copy>
<a id="f-6ea35164aaa4-kill"></a>
### `kill`

<storycode:copy id="f-6ea35164aaa4-kill">
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
</storycode:copy>
<a id="f-6ea35164aaa4-docs"></a>
### `docs`

<storycode:copy id="f-6ea35164aaa4-docs">
async fn docs(client: &Client) -> Value {
    reqwest::get(format!("{}/api/projects/local/docs", client.url))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}
</storycode:copy>
<a id="f-6ea35164aaa4-wait_file"></a>
### `wait_file`

<storycode:copy id="f-6ea35164aaa4-wait_file">
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

</storycode:copy>
[Read the test `background_attach_serves_editor_storage_before_outputs_are_ready` beside its requirement](../../../../guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation/contract.md#f-6ea35164aaa4-background_attach_serves_editor_storage_before_outputs_are_ready).
[Read the test `desktop_proxy_routes_acp_connection_configuration_and_turns` beside its requirement](../../../../guarantees/agent/acp-agents-are-first-class/contract.md#f-6ea35164aaa4-desktop_proxy_routes_acp_connection_configuration_and_turns).
<a id="f-6ea35164aaa4-windows_share_one_engine_and_one_live_document"></a>
### `windows_share_one_engine_and_one_live_document`

<storycode:copy id="f-6ea35164aaa4-windows_share_one_engine_and_one_live_document">
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

</storycode:copy>
<a id="f-6ea35164aaa4-a_cli_and_a_window_attach_in_either_order"></a>
### `a_cli_and_a_window_attach_in_either_order`

<storycode:copy id="f-6ea35164aaa4-a_cli_and_a_window_attach_in_either_order">
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

</storycode:copy>
<a id="f-6ea35164aaa4-parent_and_child_views_share_room_identity_and_reverse_edits"></a>
### `parent_and_child_views_share_room_identity_and_reverse_edits`

<storycode:copy id="f-6ea35164aaa4-parent_and_child_views_share_room_identity_and_reverse_edits">
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

</storycode:copy>
<a id="f-6ea35164aaa4-window_drafts_layout_and_native_actions_are_independent"></a>
### `window_drafts_layout_and_native_actions_are_independent`

<storycode:copy id="f-6ea35164aaa4-window_drafts_layout_and_native_actions_are_independent">
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

</storycode:copy>
<a id="f-6ea35164aaa4-engine_crash_reconnects_existing_clients_without_deleting_a_lock"></a>
### `engine_crash_reconnects_existing_clients_without_deleting_a_lock`

<storycode:copy id="f-6ea35164aaa4-engine_crash_reconnects_existing_clients_without_deleting_a_lock">
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

</storycode:copy>
<a id="f-6ea35164aaa4-engine_requires_authentication_and_recovers_stale_discovery"></a>
### `engine_requires_authentication_and_recovers_stale_discovery`

<storycode:copy id="f-6ea35164aaa4-engine_requires_authentication_and_recovers_stale_discovery">
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

</storycode:copy>
<a id="f-6ea35164aaa4-symlink_aliases_attach_to_the_same_workspace"></a>
### `symlink_aliases_attach_to_the_same_workspace`

<storycode:copy id="f-6ea35164aaa4-symlink_aliases_attach_to_the_same_workspace">
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

</storycode:copy>
<a id="f-6ea35164aaa4-file_and_folder_windows_keep_independent_files_visibility"></a>
### `file_and_folder_windows_keep_independent_files_visibility`

<storycode:copy id="f-6ea35164aaa4-file_and_folder_windows_keep_independent_files_visibility">
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

</storycode:copy>
<a id="f-6ea35164aaa4-representation_edits_through_the_real_engine_do_not_deadlock_or_dirty_prose"></a>
### `representation_edits_through_the_real_engine_do_not_deadlock_or_dirty_prose`

A nested filesystem helper must share the engine's existing write act.

<storycode:copy id="f-6ea35164aaa4-representation_edits_through_the_real_engine_do_not_deadlock_or_dirty_prose">
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
</storycode:copy>

### Assemble `crates/hickory-cli/tests/engine_lifecycle.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/engine_lifecycle.rs"><storycode:paste select="#f-6ea35164aaa4-DOC"/><storycode:paste select="#f-6ea35164aaa4-wait_daemon_exit"/><storycode:paste select="#f-6ea35164aaa4-daemon_exits_after_last_detach_even_with_an_open_terminal"/><storycode:paste select="#f-6ea35164aaa4-daemon_expires_crashed_clients_but_keeps_the_remaining_window"/><storycode:paste select="#f-6ea35164aaa4-daemon_exits_after_a_one_shot_cli_command"/><storycode:paste select="#f-6ea35164aaa4-daemon_expires_an_abandoned_cli_even_while_its_worker_runs"/><storycode:paste select="#f-6ea35164aaa4-daemon_cancels_background_preparation_after_last_client_exits"/><storycode:paste select="#f-6ea35164aaa4-daemon_reopen_waits_for_the_retiring_owner_and_starts_fresh"/><storycode:paste select="#f-6ea35164aaa4-Project"/><storycode:paste select="#f-6ea35164aaa4-impl-Project"/><storycode:paste select="#f-6ea35164aaa4-impl-Drop-for-Project"/><storycode:paste select="#f-6ea35164aaa4-Client"/><storycode:paste select="#f-6ea35164aaa4-impl-Drop-for-Client"/><storycode:paste select="#f-6ea35164aaa4-kill"/><storycode:paste select="#f-6ea35164aaa4-docs"/><storycode:paste select="#f-6ea35164aaa4-wait_file"/><storycode:paste select="#f-6ea35164aaa4-background_attach_serves_editor_storage_before_outputs_are_ready"/><storycode:paste select="#f-6ea35164aaa4-desktop_proxy_routes_acp_connection_configuration_and_turns"/><storycode:paste select="#f-6ea35164aaa4-windows_share_one_engine_and_one_live_document"/><storycode:paste select="#f-6ea35164aaa4-a_cli_and_a_window_attach_in_either_order"/><storycode:paste select="#f-6ea35164aaa4-parent_and_child_views_share_room_identity_and_reverse_edits"/><storycode:paste select="#f-6ea35164aaa4-window_drafts_layout_and_native_actions_are_independent"/><storycode:paste select="#f-6ea35164aaa4-engine_crash_reconnects_existing_clients_without_deleting_a_lock"/><storycode:paste select="#f-6ea35164aaa4-engine_requires_authentication_and_recovers_stale_discovery"/><storycode:paste select="#f-6ea35164aaa4-symlink_aliases_attach_to_the_same_workspace"/><storycode:paste select="#f-6ea35164aaa4-file_and_folder_windows_keep_independent_files_visibility"/><storycode:paste select="#f-6ea35164aaa4-representation_edits_through_the_real_engine_do_not_deadlock_or_dirty_prose"/></storycode:file>

</storycode:doc>
