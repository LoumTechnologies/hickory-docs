<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_blame.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-39e5027e9d27"></a>
## `crates/hickory-cli/tests/serve_blame.rs`

Related contracts: [Review threads preserve the questioned target](../../../../guarantees/authoring/review-threads-preserve-the-questioned-target/contract.md), [A viewer snapshot keeps recorded results historical](../../../../guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical/contract.md), [Authorship display accepts non-Git providers](../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md), [The Blame Column Is Optional And Honest](../../../../guarantees/lineage/the-blame-column-is-optional-and-honest/contract.md).

<a id="f-39e5027e9d27-Session"></a>
### `Session`

Who last touched each line, for the editor's blame column.
Protects docs/guarantees/lineage/the-blame-column-is-optional-and-honest.md
The column is off by default, so every one of these cases is really the
same question: does turning it on ever make things worse? It must not fail
a file open, and it must not claim an author it does not have.

<storycode:copy id="f-39e5027e9d27-Session">
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

</storycode:copy>
<a id="f-39e5027e9d27-opening_a_csharp_file_exposes_its_debuggable_language"></a>
### `opening_a_csharp_file_exposes_its_debuggable_language`

<storycode:copy id="f-39e5027e9d27-opening_a_csharp_file_exposes_its_debuggable_language">
#[tokio::test]
async fn opening_a_csharp_file_exposes_its_debuggable_language() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("Program.cs"), "Console.WriteLine(7);\n").unwrap();
    let session = start(dir).await;
    let (status, file) = get(&session, "/api/file?path=Program.cs").await;
    assert_eq!(status, 200, "{file}");
    assert_eq!(file["language"], "csharp");
    assert_eq!(file["content"], "Console.WriteLine(7);\n");
}

</storycode:copy>
<a id="f-39e5027e9d27-git"></a>
### `git`

<storycode:copy id="f-39e5027e9d27-git">
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

</storycode:copy>
<a id="f-39e5027e9d27-start"></a>
### `start`

<storycode:copy id="f-39e5027e9d27-start">
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

</storycode:copy>
<a id="f-39e5027e9d27-get"></a>
### `get`

<storycode:copy id="f-39e5027e9d27-get">
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

</storycode:copy>
<a id="f-39e5027e9d27-committed_repo"></a>
### `committed_repo`

A repository with one commit touching `notes.txt`.

<storycode:copy id="f-39e5027e9d27-committed_repo">
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

</storycode:copy>
<a id="f-39e5027e9d27-every_line_gets_its_author_from_one_blame"></a>
### `every_line_gets_its_author_from_one_blame`

<storycode:copy id="f-39e5027e9d27-every_line_gets_its_author_from_one_blame">
#[tokio::test(flavor = "multi_thread")]
async fn every_line_gets_its_author_from_one_blame() {
    let session = start(committed_repo()).await;
    let (status, body) = get(&session, "/api/blame?path=notes.txt").await;
    assert_eq!(status, 200);
    let lines = body["lines"].as_array().expect("a list of lines");
    assert_eq!(lines.len(), 3, "one entry per line: {body}");
    assert_eq!(
        lines[0]["text"], "one",
        "attribution must identify the actual saved bytes"
    );
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

</storycode:copy>
<a id="f-39e5027e9d27-an_uncommitted_line_says_so_rather_than_borrowing_an_author"></a>
### `an_uncommitted_line_says_so_rather_than_borrowing_an_author`

<storycode:copy id="f-39e5027e9d27-an_uncommitted_line_says_so_rather_than_borrowing_an_author">
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

</storycode:copy>
<a id="f-39e5027e9d27-a_folder_that_is_not_a_repository_answers_nothing_rather_than_failing"></a>
### `a_folder_that_is_not_a_repository_answers_nothing_rather_than_failing`

<storycode:copy id="f-39e5027e9d27-a_folder_that_is_not_a_repository_answers_nothing_rather_than_failing">
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

</storycode:copy>
<a id="f-39e5027e9d27-an_untracked_file_answers_nothing"></a>
### `an_untracked_file_answers_nothing`

<storycode:copy id="f-39e5027e9d27-an_untracked_file_answers_nothing">
#[tokio::test(flavor = "multi_thread")]
async fn an_untracked_file_answers_nothing() {
    let dir = committed_repo();
    std::fs::write(dir.path().join("scratch.txt"), "new\n").unwrap();
    let session = start(dir).await;
    let (status, body) = get(&session, "/api/blame?path=scratch.txt").await;
    assert_eq!(status, 200);
    assert_eq!(body["lines"].as_array().unwrap().len(), 0);
}

</storycode:copy>
<a id="f-39e5027e9d27-a_path_outside_the_folder_is_refused"></a>
### `a_path_outside_the_folder_is_refused`

<storycode:copy id="f-39e5027e9d27-a_path_outside_the_folder_is_refused">
#[tokio::test(flavor = "multi_thread")]
async fn a_path_outside_the_folder_is_refused() {
    let session = start(committed_repo()).await;
    for path in ["../secrets", "/etc/passwd", ""] {
        let (status, _) = get(&session, &format!("/api/blame?path={}", urlencoding(path))).await;
        assert_eq!(status, 400, "{path} must be refused");
    }
}

</storycode:copy>
<a id="f-39e5027e9d27-urlencoding"></a>
### `urlencoding`

<storycode:copy id="f-39e5027e9d27-urlencoding">
fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            other => format!("%{:02X}", other as u32),
        })
        .collect()
}

</storycode:copy>
<a id="f-39e5027e9d27-reviews_and_snapshots_preserve_captured_source_without_running_it"></a>
### `reviews_and_snapshots_preserve_captured_source_without_running_it`

- docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
- docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md

<storycode:copy id="f-39e5027e9d27-reviews_and_snapshots_preserve_captured_source_without_running_it">
// Guarantees:
// - docs/guarantees/authoring/review-threads-preserve-the-questioned-target.md
// - docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
#[tokio::test(flavor = "multi_thread")]
async fn reviews_and_snapshots_preserve_captured_source_without_running_it() {
    use sha2::{Digest, Sha256};
    let dir = tempfile::tempdir().unwrap();
    let source = "# Requirement\n\n<hick:container name=\"shell\" />\n<hick:exec container=\"shell\">\nprintf should-not-run\n</hick:exec>\n";
    std::fs::write(dir.path().join("spec.md"), source).unwrap();
    let session = start(dir).await;
    let client = reqwest::Client::new();
    let (_, history) = get(&session, "/api/reviews").await;
    let target = serde_json::json!({"kind":"claim","path":"spec.md","quote":"Requirement","source_hash":format!("{:x}",Sha256::digest(source)),"lines":[1,1]});
    let response = client.post(format!("{}/api/reviews",session.base)).json(&serde_json::json!({"scope":history["scope"],"target":target,"action":"comment","body":"Does this cover retries?"})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let event: Value = response.json().await.unwrap();
    std::fs::write(
        session._dir.path().join("spec.md"),
        format!("{source}\nChanged\n"),
    )
    .unwrap();
    let (_, history) = get(&session, "/api/reviews").await;
    assert_eq!(history["targets"][0]["current_state"], "changed");
    let response=client.post(format!("{}/api/reviews",session.base)).json(&serde_json::json!({"scope":history["scope"],"thread":event["thread"],"action":"resolve","body":"Reviewed the correction"})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let response=client.post(format!("{}/api/reviews",session.base)).json(&serde_json::json!({"scope":"wrong workspace","thread":event["thread"],"action":"resolve","body":"Wrong workspace"})).send().await.unwrap();
    assert_eq!(response.status(), 409);
    let (_, history) = get(&session, "/api/reviews").await;
    assert_eq!(history["events"].as_array().unwrap().len(), 2);
    assert_eq!(history["events"][0]["target"], target);
    let (_, docs) = get(&session, "/api/projects/local/docs").await;
    let id = docs
        .as_array()
        .unwrap()
        .iter()
        .find(|doc| doc["path"] == "spec.md")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let (status, snapshot) = get(&session, &format!("/api/docs/{id}/snapshot")).await;
    assert_eq!(status, 200, "{snapshot}");
    assert_eq!(snapshot["format"], "hickory-viewer-snapshot");
    assert_eq!(snapshot["cells"][0]["status"], "unrecorded");
    assert!(
        snapshot["cells"][0]["transcript"]
            .as_array()
            .is_none_or(|events| events.is_empty())
    );
    assert!(
        snapshot["execution"]
            .as_str()
            .unwrap()
            .contains("does not execute")
    );
}

</storycode:copy>
[Read the test `conversation_roles_supply_origin_without_git` beside its requirement](../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md#f-39e5027e9d27-conversation_roles_supply_origin_without_git).
[Read the test `unicode_and_mixed_roles_do_not_fabricate_a_single_origin` beside its requirement](../../../../guarantees/lineage/authorship-display-accepts-non-git-providers/contract.md#f-39e5027e9d27-unicode_and_mixed_roles_do_not_fabricate_a_single_origin).
<a id="f-39e5027e9d27-snapshots_keep_saved_notebook_output_without_running_the_cell"></a>
### `snapshots_keep_saved_notebook_output_without_running_the_cell`

<storycode:copy id="f-39e5027e9d27-snapshots_keep_saved_notebook_output_without_running_the_cell">
#[tokio::test(flavor = "multi_thread")]
async fn snapshots_keep_saved_notebook_output_without_running_the_cell() {
    let dir = tempfile::tempdir().unwrap();
    let source = "```python exec=\"true\" id=\"rich\"\nopen('must-not-exist', 'w').write('executed')\n```\n\n```output-for=\"#rich\" hash=\"saved\" input-hash=\"historical\" exit=\"0\"\nrecorded text\n\u{1e}HICKORY_DISPLAY {\"kind\":\"image\",\"mime\":\"image/svg+xml\",\"data\":\"PHN2Zz48L3N2Zz4=\"}\n```\n";
    std::fs::write(dir.path().join("rich.md"), source).unwrap();
    let session = start(dir).await;
    let (_, docs) = get(&session, "/api/projects/local/docs").await;
    let id = docs
        .as_array()
        .unwrap()
        .iter()
        .find(|doc| doc["path"] == "rich.md")
        .unwrap()["id"]
        .as_str()
        .unwrap();
    let (status, snapshot) = get(&session, &format!("/api/docs/{id}/snapshot")).await;
    assert_eq!(status, 200, "{snapshot}");
    let cell = &snapshot["cells"][0];
    assert_eq!(cell["status"], "unrecorded");
    assert!(
        cell["saved_output"]
            .as_str()
            .unwrap()
            .contains("recorded text\n\u{1e}HICKORY_DISPLAY")
    );
    assert!(cell["transcript"].is_null());
    assert!(!session._dir.path().join("must-not-exist").exists());
    assert_eq!(snapshot["source"], source);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_blame.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_blame.rs"><storycode:paste select="#f-39e5027e9d27-Session"/><storycode:paste select="#f-39e5027e9d27-opening_a_csharp_file_exposes_its_debuggable_language"/><storycode:paste select="#f-39e5027e9d27-git"/><storycode:paste select="#f-39e5027e9d27-start"/><storycode:paste select="#f-39e5027e9d27-get"/><storycode:paste select="#f-39e5027e9d27-committed_repo"/><storycode:paste select="#f-39e5027e9d27-every_line_gets_its_author_from_one_blame"/><storycode:paste select="#f-39e5027e9d27-an_uncommitted_line_says_so_rather_than_borrowing_an_author"/><storycode:paste select="#f-39e5027e9d27-a_folder_that_is_not_a_repository_answers_nothing_rather_than_failing"/><storycode:paste select="#f-39e5027e9d27-an_untracked_file_answers_nothing"/><storycode:paste select="#f-39e5027e9d27-a_path_outside_the_folder_is_refused"/><storycode:paste select="#f-39e5027e9d27-urlencoding"/><storycode:paste select="#f-39e5027e9d27-reviews_and_snapshots_preserve_captured_source_without_running_it"/><storycode:paste select="#f-39e5027e9d27-conversation_roles_supply_origin_without_git"/><storycode:paste select="#f-39e5027e9d27-unicode_and_mixed_roles_do_not_fabricate_a_single_origin"/><storycode:paste select="#f-39e5027e9d27-snapshots_keep_saved_notebook_output_without_running_the_cell"/></storycode:file>

</storycode:doc>
