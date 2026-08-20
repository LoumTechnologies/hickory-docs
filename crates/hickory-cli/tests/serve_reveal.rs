//! `POST /api/reveal` and `POST /api/open-external` — the tree's door to the
//! rest of the machine, and the refusals that keep it a door rather than a
//! hole.
//!
//! Only the refusals are driven over the wire. A successful call ends in
//! `Finder`/`explorer`/`xdg-open` starting on whatever machine runs the suite,
//! which is not something a test may do — the accepting half is covered by the
//! resolver's unit tests in `src/serve/reveal.rs`, and by the fact that both
//! handlers do nothing else.
//!
//! Guards docs/guarantees/authoring/a-tree-row-opens-in-the-platform.md.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    _dir: tempfile::TempDir,
}

async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root: PathBuf = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("readme.md"), "hello").unwrap();

    let prepared = prepare(ServeOptions {
        target: root,
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

#[tokio::test]
async fn a_path_that_leaves_the_open_folder_is_refused_by_both_routes() {
    let session = start().await;
    for route in ["/api/reveal", "/api/open-external"] {
        for escape in ["../secret", "/etc/passwd", "sub/../../secret"] {
            let (status, body) = post(&session, route, json!({ "path": escape })).await;
            assert_eq!(status, 400, "{route} accepted {escape}: {body}");
            // The message names what was rejected, per user-facing-errors.
            let error = body["error"].as_str().unwrap_or_default();
            assert!(
                error.contains("relative to the open folder"),
                "{route} refused {escape} without saying why: {body}"
            );
        }
    }
}

#[tokio::test]
async fn a_path_that_is_not_there_says_so_rather_than_starting_anything() {
    let session = start().await;
    for route in ["/api/reveal", "/api/open-external"] {
        let (status, body) = post(&session, route, json!({ "path": "gone.md" })).await;
        assert_eq!(status, 404, "{route}: {body}");
        let error = body["error"].as_str().unwrap_or_default();
        assert!(error.contains("gone.md"), "{body}");
        // The most likely cause, named: the tree is a listing, and listings age.
        assert!(error.contains("refreshes"), "{body}");
    }
}
