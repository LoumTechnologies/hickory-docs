//! `/api/terminals` — sessions, their bytes, and the attention queue.
//!
//! Driven over a real socket and a real WebSocket, serve_files.rs style: a
//! terminal is only a terminal if what comes back over the wire is what a
//! terminal emulator can render, so the claim has to be made at the wire.
//!
//! Protects:
//! - docs/guarantees/terminal/a-terminal-outlives-its-pane.md
//! - docs/guarantees/terminal/a-session-says-what-it-is-doing.md
//! - docs/guarantees/terminal/the-attention-queue-ranks-by-claim.md

use futures::StreamExt as _;
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

/// The argv of a session that prints `text` and exits.
///
/// A session's argv is handed straight to `portable_pty::CommandBuilder` with
/// no shell in front of it, so every program named here has to be a program.
/// `echo`, `true`, `false` and `cat` are not programs on Windows — they are
/// shell builtins or coreutils — and a session whose argv cannot be spawned
/// looks, from the outside, a lot like one that ran and failed. That is the
/// shape a test cannot tell apart from what it meant to assert, so the argv is
/// per-platform rather than the assertion being relaxed.
fn prints(text: &str) -> Vec<String> {
    argv(&["echo", text], &["cmd", "/C", "echo", text])
}

/// A session that exits 0 immediately.
fn exits_cleanly() -> Vec<String> {
    argv(&["true"], &["cmd", "/C", "exit", "0"])
}

/// A session that exits non-zero immediately.
fn exits_failing() -> Vec<String> {
    argv(&["false"], &["cmd", "/C", "exit", "1"])
}

/// A session that stays alive and prints back whatever is typed at it.
///
/// `findstr /n .` is the cmd-side `cat`: no file operand means it reads stdin,
/// and it keeps reading until the stream ends, which is what makes the "typing
/// reaches the program" half of the test a real question.
fn echoes_what_is_typed() -> Vec<String> {
    argv(&["cat"], &["findstr", "/n", "."])
}

fn argv(unix: &[&str], windows: &[&str]) -> Vec<String> {
    let chosen = if cfg!(windows) { windows } else { unix };
    chosen.iter().map(|part| part.to_string()).collect()
}

struct Session {
    base: String,
    _dir: tempfile::TempDir,
}

async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("README.md"), "hello").unwrap();

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        _dir: dir,
    }
}

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

/// Read from the session's socket until `needle` shows up, or give up.
async fn read_until(session: &Session, id: &str, needle: &str) -> String {
    let url = format!(
        "ws://{}/api/terminals/ws?session={id}",
        session.base.trim_start_matches("http://")
    );
    let (mut socket, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("the terminal socket accepts a client");

    let mut seen = String::new();
    for _ in 0..100 {
        let next = tokio::time::timeout(std::time::Duration::from_millis(200), socket.next()).await;
        match next {
            Ok(Some(Ok(Message::Binary(bytes)))) => {
                seen.push_str(&String::from_utf8_lossy(&bytes));
                if seen.contains(needle) {
                    break;
                }
            }
            Ok(Some(Ok(_))) => continue,
            Ok(Some(Err(_)) | None) => break,
            Err(_) => continue,
        }
    }
    drop(socket);
    seen
}

#[tokio::test(flavor = "multi_thread")]
async fn a_terminal_runs_a_command_and_the_socket_carries_what_it_printed() {
    let session = start().await;
    let (status, opened) = post(
        &session,
        "/api/terminals",
        json!({ "title": "greeting", "argv": prints("hello from the pty") }),
    )
    .await;
    assert_eq!(status, 200, "{opened}");
    let id = opened["id"].as_str().unwrap().to_string();
    assert_eq!(opened["title"], "greeting");

    let seen = read_until(&session, &id, "hello from the pty").await;
    assert!(seen.contains("hello from the pty"), "socket said: {seen:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_client_that_arrives_after_the_output_still_sees_it() {
    // The pane-close case: the bytes were produced while nobody was attached.
    let session = start().await;
    let (_, opened) = post(
        &session,
        "/api/terminals",
        json!({ "argv": prints("printed before you looked") }),
    )
    .await;
    let id = opened["id"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let seen = read_until(&session, &id, "printed before you looked").await;
    assert!(
        seen.contains("printed before you looked"),
        "replayed: {seen:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_command_that_fails_is_failed_and_leads_the_queue() {
    let session = start().await;
    let (_, ok) = post(
        &session,
        "/api/terminals",
        json!({ "argv": exits_cleanly() }),
    )
    .await;
    let (_, bad) = post(
        &session,
        "/api/terminals",
        json!({ "argv": exits_failing() }),
    )
    .await;
    let ok_id = ok["id"].as_str().unwrap().to_string();
    let bad_id = bad["id"].as_str().unwrap().to_string();

    // Both exit immediately; poll until the server has noticed.
    let mut listed = json!({});
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let (status, body) = get(&session, "/api/terminals").await;
        assert_eq!(status, 200, "{body}");
        let done = body["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["state"] == "finished" || s["state"] == "failed");
        if done {
            listed = body;
            break;
        }
    }

    let states: Vec<(String, String)> = listed["sessions"]
        .as_array()
        .expect("both sessions settled")
        .iter()
        .map(|s| {
            (
                s["id"].as_str().unwrap().to_string(),
                s["state"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert!(
        states.contains(&(ok_id.clone(), "finished".into())),
        "{states:?}"
    );
    assert!(
        states.contains(&(bad_id.clone(), "failed".into())),
        "{states:?}"
    );

    // A failure outranks a clean finish, so the queue leads with it.
    let queue: Vec<String> = listed["attention"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(queue.first(), Some(&bad_id), "queue was {queue:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_monitor_stays_out_of_the_queue() {
    let session = start().await;
    let (_, monitor) = post(
        &session,
        "/api/terminals",
        json!({ "title": "dev server", "argv": exits_failing(), "monitor": true }),
    )
    .await;
    let monitor_id = monitor["id"].as_str().unwrap().to_string();

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let (_, listed) = get(&session, "/api/terminals").await;
    let queue: Vec<&Value> = listed["attention"].as_array().unwrap().iter().collect();
    assert!(
        !queue.iter().any(|id| **id == json!(monitor_id)),
        "a failing dev server must not interrupt: {queue:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_reaches_the_program_and_closing_ends_the_session() {
    let session = start().await;
    let (_, opened) = post(
        &session,
        "/api/terminals",
        json!({ "argv": echoes_what_is_typed() }),
    )
    .await;
    let id = opened["id"].as_str().unwrap().to_string();

    // CR then LF, because the Enter key is a CR: a console reading a line on
    // Windows waits for one, and a Unix tty turns it into the newline `cat` is
    // waiting for (ICRNL). Sending only LF works on exactly one of the two.
    let (status, _) = post(
        &session,
        &format!("/api/terminals/{id}/input"),
        json!({ "data": "typed into cat\r\n" }),
    )
    .await;
    assert_eq!(status, 200);

    let seen = read_until(&session, &id, "typed into cat").await;
    assert!(seen.contains("typed into cat"), "echoed: {seen:?}");

    let closed = reqwest::Client::new()
        .delete(format!("{}/api/terminals/{id}", session.base))
        .send()
        .await
        .unwrap();
    assert_eq!(closed.status().as_u16(), 200);

    let (_, listed) = get(&session, "/api/terminals").await;
    assert!(
        listed["sessions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|s| s["id"] != json!(id)),
        "a closed session is gone: {listed}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn asking_for_a_terminal_that_is_gone_says_so_rather_than_hanging() {
    let session = start().await;
    let resp = reqwest::Client::new()
        .post(format!("{}/api/terminals/term-999/input", session.base))
        .json(&json!({ "data": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let body: Value = resp.json().await.unwrap();
    let message = body["error"].as_str().unwrap();
    assert!(message.contains("term-999"), "{message}");
    assert!(message.contains("closed"), "{message}");
}
