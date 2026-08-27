//! A terminal that writes the document, driven through the real HTTP API
//! with a real shell behind it.
//!
//! Protects docs/specs/freeform/a-terminal-that-writes-the-document.md
//!
//! The layers below this are pure and tested on their own — what a shell
//! reports (`hick_term::command`), whether a line may be written
//! (`hick_term::anchor`), and where it lands (`hickory_cli::anchor`). This
//! joins them to a PTY and a document, because the claim worth testing is
//! that **typing in a terminal writes a cell**, and that is not observable
//! from any one of them.
//!
//! Skipped loudly without bash. zsh's hook ships written and unverified —
//! it was not installed on the machine this was written on.

use std::path::PathBuf;
use std::time::Duration;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# Scaffolding

<hick:exec container="sdk">
echo already here
</hick:exec>

</hick:doc>
"##;

struct App {
    base: String,
    doc_id: String,
    root: PathBuf,
    client: reqwest::Client,
    _dir: tempfile::TempDir,
}

async fn open_app(shell: &str) -> App {
    // `TermConfig` is read once, when the server prepares — not per terminal
    // — so the shell has to be chosen before `prepare`, and these tests run
    // single-threaded because of it. Getting this wrong was worth finding:
    // an earlier draft set it per terminal and every assertion after the
    // first was about a different shell than it named.
    unsafe { std::env::set_var("HICKORY_SHELL", shell) };
    let dir = tempfile::tempdir().expect("a temp project");
    let root = dir.path().to_path_buf();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join("doc.hick"), DOC).unwrap();

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("the session prepares");
    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    App {
        base: format!("http://127.0.0.1:{port}/api"),
        doc_id,
        root,
        client: reqwest::Client::new(),
        _dir: dir,
    }
}

impl App {
    async fn post(&self, path: &str, body: Value) -> (reqwest::StatusCode, Value) {
        let response = self
            .client
            .post(format!("{}{path}", self.base))
            .json(&body)
            .send()
            .await
            .expect("the server answers");
        let status = response.status();
        let value = response.json().await.unwrap_or(Value::Null);
        (status, value)
    }

    /// Open a terminal running the session's configured shell.
    async fn open_terminal(&self) -> String {
        self.open_with(json!({ "title": "work", "cwd": self.root.to_string_lossy() }))
            .await
    }

    async fn open_with(&self, body: Value) -> String {
        let (status, body) = self.post("/terminals", body).await;
        assert!(status.is_success(), "opening a terminal: {body}");
        body["id"].as_str().expect("an id").to_string()
    }

    async fn type_line(&self, terminal: &str, line: &str) {
        let (status, body) = self
            .post(
                &format!("/terminals/{terminal}/input"),
                json!({ "data": line }),
            )
            .await;
        assert!(status.is_success(), "typing: {body}");
        tokio::time::sleep(Duration::from_millis(400)).await;
    }

    fn document(&self) -> String {
        std::fs::read_to_string(self.root.join("doc.hick")).expect("the document is readable")
    }

    async fn anchors(&self) -> Value {
        self.client
            .get(format!("{}/terminals/anchors", self.base))
            .send()
            .await
            .expect("answers")
            .json()
            .await
            .expect("json")
    }
}

fn have_bash() -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join("bash"))
        .find(|c| c.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_in_an_anchored_terminal_grows_the_cell() {
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    let app = open_app(&bash).await;
    let terminal = app.open_terminal().await;
    tokio::time::sleep(Duration::from_millis(700)).await;

    let (status, anchored) = app
        .post(
            &format!("/terminals/{terminal}/anchor"),
            json!({ "doc": app.doc_id, "container": "sdk" }),
        )
        .await;
    assert!(status.is_success(), "anchoring: {anchored}");
    assert_eq!(anchored["container"], "sdk");

    app.type_line(&terminal, "echo one\n").await;
    app.type_line(&terminal, "echo two | tr o 0\n").await;

    let document = app.document();
    // One cell that grew, not one cell per line.
    assert_eq!(
        document.matches("<hick:exec").count(),
        1,
        "a shell session became several cells:\n{document}"
    );
    assert!(
        document.contains("echo already here\necho one\necho two | tr o 0\n</hick:exec>"),
        "the typed lines are not in the cell, in order:\n{document}"
    );
    // And the document is still a document.
    hick_lang::parse(&document).expect("the anchored terminal broke its document");

    // Unanchoring stops the document receiving; the terminal keeps working.
    let response = app
        .client
        .delete(format!("{}/terminals/{terminal}/anchor", app.base))
        .send()
        .await
        .expect("answers");
    assert!(response.status().is_success());
    app.type_line(&terminal, "echo after\n").await;
    assert!(
        !app.document().contains("echo after"),
        "an unanchored terminal kept writing:\n{}",
        app.document()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_line_that_looks_like_a_secret_stops_the_recording() {
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    let app = open_app(&bash).await;
    let terminal = app.open_terminal().await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    app.post(
        &format!("/terminals/{terminal}/anchor"),
        json!({ "doc": app.doc_id, "container": "sdk" }),
    )
    .await;

    app.type_line(&terminal, "echo before\n").await;
    app.type_line(&terminal, "export DEMO_API_KEY=sk-ant-notreal-0123456789\n")
        .await;
    app.type_line(&terminal, "echo after\n").await;

    let document = app.document();
    // The gate held: the key never reached the document. The shell DID run
    // it — this scan gates the write, it does not undo one.
    assert!(
        !document.contains("sk-ant-notreal"),
        "a secret was written into the document:\n{document}"
    );
    assert!(document.contains("echo before"), "{document}");
    // Suspend, never filter: a cell with a hole in it claims a run that
    // cannot reproduce, so everything after the suspension is left out too.
    assert!(
        !document.contains("echo after"),
        "recording carried on past a suspension, leaving a hole:\n{document}"
    );

    // And a person can see why, without reading a diff.
    let anchors = app.anchors().await;
    let suspended = anchors["anchors"][&terminal]["suspended"]
        .as_str()
        .expect("the anchor says it is suspended");
    assert!(suspended.contains("looks like a secret"), "{suspended}");
    assert!(!suspended.to_lowercase().contains("safe"), "{suspended}");

    // Resuming starts a NEW cell: after a suspension the shell holds state
    // the document does not describe.
    let (status, resumed) = app
        .post(&format!("/terminals/{terminal}/anchor/resume"), json!({}))
        .await;
    assert!(status.is_success(), "{resumed}");
    assert!(resumed["suspended"].is_null(), "{resumed}");
    app.type_line(&terminal, "echo resumed\n").await;

    let document = app.document();
    assert_eq!(
        document.matches("<hick:exec").count(),
        2,
        "resuming continued the old cell instead of starting one:\n{document}"
    );
    assert!(document.contains("echo resumed"), "{document}");
    hick_lang::parse(&document).expect("still a document");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_shell_with_no_hook_is_refused_by_name_rather_than_recording_nothing() {
    // The failure this refusal exists to prevent: a fish session that looks
    // anchored, records nothing, and tells nobody.
    //
    // An explicit `argv` is the honest way to get an un-hooked session: the
    // integration is installed only for "your shell", because a named
    // program is not one and wrapping it would be changing what was asked
    // for.
    let app = open_app("/bin/bash").await;
    let terminal = app
        .open_with(json!({
            "title": "raw",
            "cwd": app.root.to_string_lossy(),
            "argv": ["/bin/sh", "-i"],
        }))
        .await;
    tokio::time::sleep(Duration::from_millis(400)).await;

    let (status, body) = app
        .post(
            &format!("/terminals/{terminal}/anchor"),
            json!({ "doc": app.doc_id, "container": "sdk" }),
        )
        .await;
    assert!(
        !status.is_success(),
        "an unhookable shell was anchored: {body}"
    );
    let message = body["error"]
        .as_str()
        .or_else(|| body["message"].as_str())
        .unwrap_or_default();
    assert!(
        message.contains("bash and zsh"),
        "the refusal does not say which shells work: {body}"
    );
    assert!(
        message.contains("still works"),
        "the refusal reads like the terminal is broken: {body}"
    );
}
