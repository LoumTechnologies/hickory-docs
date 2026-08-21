//! Who last touched each line, for the editor's blame column.
//!
//! Protects docs/guarantees/lineage/the-blame-column-is-optional-and-honest.md
//!
//! The column is off by default, so every one of these cases is really the
//! same question: does turning it on ever make things worse? It must not fail
//! a file open, and it must not claim an author it does not have.

use std::process::Command;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;

struct Session {
    base: String,
    _dir: tempfile::TempDir,
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

async fn start(dir: tempfile::TempDir) -> Session {
    let root = dir.path().canonicalize().unwrap();
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

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

/// A repository with one commit touching `notes.txt`.
fn committed_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["config", "user.name", "Ada Lovelace"]);
    git(dir.path(), &["config", "user.email", "ada@example.com"]);
    std::fs::write(dir.path().join("notes.txt"), "one\ntwo\nthree\n").unwrap();
    git(dir.path(), &["add", "notes.txt"]);
    git(dir.path(), &["commit", "-q", "-m", "First three lines"]);
    dir
}

#[tokio::test(flavor = "multi_thread")]
async fn every_line_gets_its_author_from_one_blame() {
    let session = start(committed_repo()).await;
    let (status, body) = get(&session, "/api/blame?path=notes.txt").await;
    assert_eq!(status, 200);
    let lines = body["lines"].as_array().expect("a list of lines");
    assert_eq!(lines.len(), 3, "one entry per line: {body}");
    assert_eq!(lines[0]["line"], 1);
    assert_eq!(lines[2]["line"], 3);
    for line in lines {
        assert_eq!(line["author"], "Ada Lovelace");
        assert_eq!(line["email"], "ada@example.com");
        assert_eq!(line["summary"], "First three lines");
        assert_eq!(line["uncommitted"], false);
        assert!(
            line["commit"].as_str().unwrap().len() >= 7,
            "an abbreviated sha: {line}"
        );
        assert!(line["time"].as_i64().unwrap() > 0, "an author time");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_uncommitted_line_says_so_rather_than_borrowing_an_author() {
    // The working tree is the common case, not an edge case. Attributing a
    // line somebody just typed to whoever last touched the file would be a
    // lie in the direction that matters.
    let dir = committed_repo();
    std::fs::write(dir.path().join("notes.txt"), "one\ntwo\nthree\nfour\n").unwrap();
    let session = start(dir).await;

    let (_, body) = get(&session, "/api/blame?path=notes.txt").await;
    let lines = body["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 4);
    assert_eq!(lines[3]["uncommitted"], true);
    assert_eq!(lines[3]["commit"], "");
    assert_eq!(lines[3]["summary"], "Not committed yet");
    // ...and the committed lines are untouched by that.
    assert_eq!(lines[0]["author"], "Ada Lovelace");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_folder_that_is_not_a_repository_answers_nothing_rather_than_failing() {
    // The column is an optional annotation. Refusing to open a file because
    // its history is unavailable would be absurd.
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.txt"), "one\n").unwrap();
    let session = start(dir).await;

    let (status, body) = get(&session, "/api/blame?path=notes.txt").await;
    assert_eq!(status, 200);
    assert_eq!(body["lines"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn an_untracked_file_answers_nothing() {
    let dir = committed_repo();
    std::fs::write(dir.path().join("scratch.txt"), "new\n").unwrap();
    let session = start(dir).await;
    let (status, body) = get(&session, "/api/blame?path=scratch.txt").await;
    assert_eq!(status, 200);
    assert_eq!(body["lines"].as_array().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_path_outside_the_folder_is_refused() {
    let session = start(committed_repo()).await;
    for path in ["../secrets", "/etc/passwd", ""] {
        let (status, _) = get(&session, &format!("/api/blame?path={}", urlencoding(path))).await;
        assert_eq!(status, 400, "{path} must be refused");
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}
