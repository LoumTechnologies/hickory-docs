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
//! Run against **both** hooked shells, because they are not the same
//! mechanism underneath — bash reports through `PS0` and `history 1`, zsh
//! through `preexec` — and the zsh path additionally exercises the
//! `ZDOTDIR` forwarding the integration needs to keep a person's own startup
//! files working. Skipped loudly for a shell that is not installed.

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
    }

    /// Type a line and wait until the document has taken it.
    ///
    /// Waiting on the EFFECT rather than on a duration, because a fixed sleep
    /// is a guess about how long a shell takes to source its startup files
    /// and run a command — and a guess that is right on an idle machine and
    /// wrong on a busy one is a flaky test, which this repository treats as a
    /// defect rather than something to retry.
    async fn type_and_record(&self, terminal: &str, line: &str, expect_total: usize) {
        self.type_line(terminal, line).await;
        self.wait_for(terminal, |anchor| {
            anchor["recorded"].as_u64().unwrap_or(0) as usize >= expect_total
        })
        .await;
    }

    /// Poll the session list until this terminal reaches `state`.
    ///
    /// `working` is what a foreground child looks like from outside — which
    /// is how the test knows a REPL has actually taken the terminal, rather
    /// than guessing how long python takes to start.
    async fn wait_for_state(&self, terminal: &str, state: &str) {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        let mut last = String::new();
        while std::time::Instant::now() < deadline {
            let list: Value = self
                .client
                .get(format!("{}/terminals", self.base))
                .send()
                .await
                .expect("answers")
                .json()
                .await
                .expect("json");
            last = list["sessions"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .find(|s| s["id"] == terminal)
                .and_then(|s| s["state"].as_str())
                .unwrap_or_default()
                .to_string();
            if last == state {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the terminal never reached `{state}`; it is `{last}`");
    }

    /// Poll this terminal's anchor until `done`, or fail saying what it held.
    async fn wait_for(&self, terminal: &str, done: impl Fn(&Value) -> bool) -> Value {
        let deadline = std::time::Instant::now() + Duration::from_secs(20);
        let mut last = Value::Null;
        while std::time::Instant::now() < deadline {
            let anchors = self.anchors().await;
            last = anchors["anchors"][terminal].clone();
            if done(&last) {
                return last;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        panic!("the anchor never reached the expected state; it holds {last}");
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

fn have(shell: &str) -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(shell))
        .find(|c| c.is_file())
        .map(|p| p.to_string_lossy().into_owned())
}

/// Every shell hick installs a command hook for.
const HOOKED: &[&str] = &["bash", "zsh"];

/// The installed hooked shells, saying which are missing rather than
/// quietly testing one and reporting a pass for both.
fn hooked_shells() -> Vec<(&'static str, String)> {
    let found: Vec<(&'static str, String)> = HOOKED
        .iter()
        .filter_map(|name| have(name).map(|path| (*name, path)))
        .collect();
    for name in HOOKED {
        if !found.iter().any(|(n, _)| n == name) {
            eprintln!("SKIPPED {name}: not installed on this machine");
        }
    }
    assert!(!found.is_empty(), "no hooked shell is installed");
    found
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_in_an_anchored_terminal_grows_the_cell() {
    for (name, shell) in hooked_shells() {
        eprintln!("--- {name} ---");
        grows_the_cell(&shell).await;
    }
}

async fn grows_the_cell(shell: &str) {
    let app = open_app(shell).await;
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

    app.type_and_record(&terminal, "echo one\n", 1).await;
    app.type_and_record(&terminal, "echo two | tr o 0\n", 2)
        .await;

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
    // Nothing to wait FOR — the claim is that nothing happens — so this one
    // needs a real pause, and it is the only one.
    tokio::time::sleep(Duration::from_millis(800)).await;
    assert!(
        !app.document().contains("echo after"),
        "an unanchored terminal kept writing:\n{}",
        app.document()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_line_that_looks_like_a_secret_stops_the_recording() {
    for (name, shell) in hooked_shells() {
        eprintln!("--- {name} ---");
        secret_stops_recording(&shell).await;
    }
}

async fn secret_stops_recording(shell: &str) {
    let app = open_app(shell).await;
    let terminal = app.open_terminal().await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    app.post(
        &format!("/terminals/{terminal}/anchor"),
        json!({ "doc": app.doc_id, "container": "sdk" }),
    )
    .await;

    app.type_and_record(&terminal, "echo before\n", 1).await;
    app.type_line(&terminal, "export DEMO_API_KEY=sk-ant-notreal-0123456789\n")
        .await;
    app.wait_for(&terminal, |anchor| !anchor["suspended"].is_null())
        .await;
    app.type_line(&terminal, "echo after\n").await;
    tokio::time::sleep(Duration::from_millis(600)).await;

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
    app.wait_for(&terminal, |_| app.document().contains("echo resumed"))
        .await;

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
async fn a_line_typed_with_a_leading_space_never_reaches_the_document() {
    for (name, shell) in hooked_shells() {
        eprintln!("--- {name} ---");
        leading_space_is_not_recorded(&shell).await;
    }
}

/// The convention every shell with a history has, honoured by hick rather
/// than left to the shell — because the two shells disagree about it
/// completely at the wire, and a person's "do not record this" must not
/// depend on which one they run.
async fn leading_space_is_not_recorded(shell: &str) {
    let app = open_app(shell).await;
    let terminal = app.open_terminal().await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    app.post(
        &format!("/terminals/{terminal}/anchor"),
        json!({ "doc": app.doc_id, "container": "sdk" }),
    )
    .await;

    app.type_and_record(&terminal, "echo recorded\n", 1).await;
    app.type_line(&terminal, " echo hidden\n").await;
    app.wait_for(&terminal, |anchor| !anchor["suspended"].is_null())
        .await;
    app.type_line(&terminal, "echo after\n").await;
    tokio::time::sleep(Duration::from_millis(600)).await;

    let document = app.document();
    assert!(document.contains("echo recorded"), "{document}");
    assert!(
        !document.contains("echo hidden"),
        "a line typed with a leading space was written down:\n{document}"
    );
    // Suspend, never filter — so what follows it is left out too.
    assert!(
        !document.contains("echo after"),
        "recording carried on past the hidden line, leaving a hole:\n{document}"
    );

    let anchors = app.anchors().await;
    let why = anchors["anchors"][&terminal]["suspended"]
        .as_str()
        .expect("the anchor says it is suspended");
    assert!(
        why.contains("space") || why.contains("history"),
        "the reason does not mention why: {why}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_into_a_repl_is_said_out_loud_and_recording_survives_it() {
    // The case the process-group test exists for, and it is a MESSAGE rather
    // than a state: inside `python3` the shell never sees the keystrokes, so
    // nothing is reported and nothing needs refusing. What a person needs is
    // to be told why their lines stopped appearing — and then to have the
    // cell carry on when they quit.
    for (name, shell) in hooked_shells() {
        eprintln!("--- {name} ---");
        repl_is_announced(&shell).await;
    }
}

async fn repl_is_announced(shell: &str) {
    if !std::path::Path::new("/proc/self/comm").exists() {
        eprintln!("SKIPPED: naming the foreground program is Linux-only so far");
        return;
    }
    let app = open_app(shell).await;
    let terminal = app.open_terminal().await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    app.post(
        &format!("/terminals/{terminal}/anchor"),
        json!({ "doc": app.doc_id, "container": "sdk" }),
    )
    .await;

    app.type_and_record(&terminal, "echo before\n", 1).await;
    app.type_and_record(&terminal, "python3 -q\n", 2).await;
    // The REPL has to actually hold the terminal before anything is typed
    // into it, or the keys reach the shell and are recorded as commands.
    // `working` is what a foreground child looks like from outside.
    app.wait_for_state(&terminal, "working").await;

    // Typing INTO the REPL. These keys never reach the shell — and the
    // notice is raised on the input path, so it appears because somebody
    // typed, not because a child exists.
    app.type_line(&terminal, "print('inside')\n").await;
    app.wait_for(&terminal, |anchor| !anchor["foreign"].is_null())
        .await;
    app.type_line(&terminal, "quit()\n").await;
    // Back at the prompt before typing again, or the next keystroke lands
    // while python is still exiting and the note is raised a second time
    // with nothing left to type that would clear it.
    app.wait_for_state(&terminal, "idle").await;
    app.type_and_record(&terminal, "echo after\n", 3).await;

    let document = app.document();
    // The REPL's own lines are not commands and are not recorded — by
    // construction, not by refusal.
    assert!(
        !document.contains("print('inside')"),
        "a REPL's input was recorded as a shell command:\n{document}"
    );
    // And the anchor was NOT suspended: when the REPL exits the shell reports
    // again, and the cell carries on. That is the whole difference between
    // this and a secret.
    assert!(
        document.contains("echo before") && document.contains("echo after"),
        "recording did not survive the REPL:\n{document}"
    );
    let anchors = app.anchors().await;
    assert!(
        anchors["anchors"][&terminal]["suspended"].is_null(),
        "a program reading the keys suspended the anchor, which nothing has to resume: {}",
        anchors["anchors"][&terminal]
    );

    // The standing note cleared itself when the shell got the terminal back:
    // there was nothing to resume, which is the point.
    assert!(
        anchors["anchors"][&terminal]["foreign"].is_null(),
        "the note outlived the program it was about: {}",
        anchors["anchors"][&terminal]
    );
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
    let app = open_app(&have("bash").expect("bash")).await;
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
    // Both shells by name, and for bash the VERSION — "bash" alone is what
    // the message used to say, and it was true by name and false in fact on
    // every Mac, where `/bin/bash` is 3.2 and has no `PS0` to hook.
    assert!(
        message.contains("zsh") && message.contains("bash"),
        "the refusal does not say which shells work: {body}"
    );
    assert!(
        message.contains("4.4"),
        "the refusal names bash without naming the version that works: {body}"
    );
    assert!(
        message.contains("still works"),
        "the refusal reads like the terminal is broken: {body}"
    );
}
