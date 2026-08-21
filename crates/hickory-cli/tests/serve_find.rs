//! Exhaustive find, and the replace that refuses to write a generated file.
//!
//! Protects docs/guarantees/search/find-and-replace-is-exhaustive.md
//!
//! Driven over real HTTP against the real router, the way the page drives it.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    dir: tempfile::TempDir,
}

async fn start(files: &[(&str, &str)]) -> Session {
    let dir = tempfile::tempdir().unwrap();
    for (path, body) in files {
        let full = dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&full, body).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();
    let target = files
        .iter()
        .find(|(p, _)| p.ends_with(".hick"))
        .map(|(p, _)| root.join(p))
        .unwrap_or_else(|| root.clone());

    let prepared = prepare(ServeOptions {
        target,
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
        dir,
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

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="notes.md">
# Notes

<hick:file path="orders.py">total = 1
</hick:file>
</hick:doc>
"##;

#[tokio::test(flavor = "multi_thread")]
async fn find_reports_every_match_in_path_order() {
    // Exhaustive, unlike `/api/search`, which is ranked top-k. A ranked
    // answer is a SAMPLE, and replacing across a sample silently changes some
    // of the occurrences.
    let session = start(&[("b.txt", "alpha\nbeta alpha\n"), ("a.txt", "alpha\n")]).await;

    let (status, body) = get(&session, "/api/find?q=alpha").await;
    assert_eq!(status, 200);
    let files = body["files"].as_array().unwrap();
    let paths: Vec<&str> = files.iter().map(|f| f["path"].as_str().unwrap()).collect();
    assert_eq!(paths, vec!["a.txt", "b.txt"], "path order");
    // Two lines in b.txt, and the second line has one match.
    let b = &files[1];
    assert_eq!(b["matches"].as_array().unwrap().len(), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_literal_search_is_not_a_regex() {
    // Someone typing `a.b` into a box means `a.b`.
    let session = start(&[("a.txt", "a.b\naxb\n")]).await;
    let (_, body) = get(&session, "/api/find?q=a.b").await;
    let matches = body["files"][0]["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "only the literal line matches");
    assert_eq!(matches[0]["text"], "a.b");
}

#[tokio::test(flavor = "multi_thread")]
async fn case_is_ignored_unless_asked_for() {
    let session = start(&[("a.txt", "Alpha\nalpha\n")]).await;
    let (_, insensitive) = get(&session, "/api/find?q=alpha").await;
    assert_eq!(
        insensitive["files"][0]["matches"].as_array().unwrap().len(),
        2
    );
    let (_, sensitive) = get(&session, "/api/find?q=alpha&case=true").await;
    assert_eq!(
        sensitive["files"][0]["matches"].as_array().unwrap().len(),
        1
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_broken_regex_says_how_to_search_for_it_literally() {
    let session = start(&[("a.txt", "x\n")]).await;
    let (status, body) = get(&session, "/api/find?q=%5B&regex=true").await;
    assert_eq!(status, 400);
    assert!(
        body["error"].as_str().unwrap().contains("regex option off"),
        "{body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn replace_rewrites_matches_and_counts_them() {
    let session = start(&[("a.txt", "old\nold and old\n")]).await;
    let (status, body) = post(
        &session,
        "/api/find/replace",
        json!({ "q": "old", "replacement": "new" }),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body["replacements"], 3);
    let on_disk = std::fs::read_to_string(session.dir.path().join("a.txt")).unwrap();
    assert_eq!(on_disk, "new\nnew and new\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn replace_refuses_a_generated_file_and_names_its_document() {
    // Writing here either loses the edit at the next weave or fights the
    // up-loop for it. Naming the document points at the change that survives
    // — and that fixes every other copy at the same time.
    let session = start(&[("app.hick", DOC), ("orders.py", "total = 1\n")]).await;

    let (status, body) = post(
        &session,
        "/api/find/replace",
        json!({ "q": "total", "replacement": "sum" }),
    )
    .await;
    assert_eq!(status, 200);

    let skipped = body["skipped"].as_array().unwrap();
    let orders = skipped
        .iter()
        .find(|s| s["path"] == "orders.py")
        .expect("orders.py was skipped");
    assert_eq!(orders["reason"], "generated");
    assert!(orders["document"].is_string(), "the document is named");

    // The file is untouched...
    let on_disk = std::fs::read_to_string(session.dir.path().join("orders.py")).unwrap();
    assert_eq!(on_disk, "total = 1\n");
    // ...and the document, which IS source, was rewritten.
    let doc = std::fs::read_to_string(session.dir.path().join("app.hick")).unwrap();
    assert!(doc.contains("sum = 1"), "the document is source:\n{doc}");
}

#[tokio::test(flavor = "multi_thread")]
async fn find_marks_a_generated_file_before_anyone_presses_replace() {
    let session = start(&[("app.hick", DOC), ("orders.py", "total = 1\n")]).await;
    let (_, body) = get(&session, "/api/find?q=total").await;
    let files = body["files"].as_array().unwrap();
    let orders = files.iter().find(|f| f["path"] == "orders.py").unwrap();
    assert!(
        orders["generated_by"].is_string(),
        "the UI can grey it out before the button is pressed: {orders}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn replace_touches_only_the_paths_it_was_given() {
    let session = start(&[("a.txt", "x\n"), ("b.txt", "x\n")]).await;
    let (_, body) = post(
        &session,
        "/api/find/replace",
        json!({ "q": "x", "replacement": "y", "paths": ["a.txt"] }),
    )
    .await;
    assert_eq!(body["replacements"], 1);
    assert_eq!(
        std::fs::read_to_string(session.dir.path().join("b.txt")).unwrap(),
        "x\n"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_empty_pattern_is_refused_rather_than_matching_everywhere() {
    let session = start(&[("a.txt", "x\n")]).await;
    assert_eq!(get(&session, "/api/find?q=").await.0, 400);
}
