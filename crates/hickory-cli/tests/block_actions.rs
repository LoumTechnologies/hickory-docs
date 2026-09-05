//! The generic routes of `docs/specs/freeform/the-minimal-core.md` step 3,
//! driven over real HTTP as a client would drive them:
//! `GET /api/elements` says what the app draws, and
//! `POST /api/docs/:id/blocks/:at/:action` asks one element to act.
//!
//! Protects docs/guarantees/language/an-element-is-declared-once.md and
//! docs/guarantees/language/an-action-is-asked-of-the-element.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::sync::Arc;

const DOC: &str = "# A cell\n\n<hick:exec container=\"c\">\necho hi\n</hick:exec>\n";

/// The byte at which the exec tag above starts.
const EXEC_AT: usize = 10;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

struct Session {
    base: String,
    doc_id: String,
    _dir: tempfile::TempDir,
    _state: tempfile::TempDir,
    _env: tokio::sync::OwnedMutexGuard<()>,
}

async fn start() -> Session {
    let env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("demo.hick"), DOC).unwrap();
    let root = dir.path().canonicalize().unwrap();
    // Per-user state goes to a throwaway directory, never the developer's.
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };

    let prepared = prepare(ServeOptions {
        target: root.join("demo.hick"),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .expect("session prepares");
    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    Session {
        base: format!("http://127.0.0.1:{port}"),
        doc_id,
        _dir: dir,
        _state: state_dir,
        _env: env,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let res = reqwest::get(format!("{}{path}", session.base))
        .await
        .unwrap();
    let status = res.status().as_u16();
    (status, res.json().await.unwrap_or(Value::Null))
}

async fn post(session: &Session, path: &str) -> (u16, Value) {
    let res = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = res.status().as_u16();
    (status, res.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn the_elements_route_lists_the_vocabulary_the_render_route_draws() {
    let s = start().await;
    let (status, body) = get(&s, "/api/elements").await;
    assert_eq!(status, 200);
    let elements = body["elements"].as_array().expect("an array");
    let names: Vec<&str> = elements
        .iter()
        .map(|e| e["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["diagram", "exec", "file", "when"]);
    let exec = &elements[1];
    assert_eq!(exec["kind"], "exec");
    assert_eq!(exec["actions"], serde_json::json!(["run"]));
    let attrs: Vec<&str> = exec["attributes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert!(attrs.contains(&"container"));

    // And the render route draws exactly those kinds.
    let (_, rendered) = get(&s, &format!("/api/docs/{}/render", s.doc_id)).await;
    let kinds: Vec<&str> = rendered["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["kind"].as_str().unwrap())
        .collect();
    assert_eq!(kinds, vec!["prose", "exec"]);
    assert_eq!(rendered["blocks"][1]["span"][0], EXEC_AT);
}

#[tokio::test]
async fn running_a_cell_is_an_action_the_element_asks_for_and_the_server_carries_out() {
    let s = start().await;
    let (status, body) = post(&s, &format!("/api/docs/{}/blocks/{EXEC_AT}/run", s.doc_id)).await;
    assert_eq!(status, 202, "{body}");
    assert_eq!(body["outcome"], "run");
    assert_eq!(body["cells"], serde_json::json!(["c:3"]));
    let run_id = body["run_id"].as_str().expect("a run id");

    // The run is the same kind of run the Run button starts: it has a
    // record, and it finishes.
    let mut last = Value::Null;
    for _ in 0..200 {
        let (_, run) = get(&s, &format!("/api/runs/{run_id}")).await;
        last = run.clone();
        if run["status"] != "running" && run["status"] != "queued" {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert_eq!(last["status"], "ok", "{last}");
}

#[tokio::test]
async fn an_action_the_element_does_not_know_is_refused_by_name() {
    let s = start().await;
    let (status, body) = post(
        &s,
        &format!("/api/docs/{}/blocks/{EXEC_AT}/whistle", s.doc_id),
    )
    .await;
    assert_eq!(status, 404, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("<hick:exec> has no action 'whistle'"),
        "{body}"
    );
}

#[tokio::test]
async fn an_action_on_a_byte_where_no_element_starts_is_refused() {
    let s = start().await;
    let (status, body) = post(&s, &format!("/api/docs/{}/blocks/0/run", s.doc_id)).await;
    assert_eq!(status, 422, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("no element starts at byte 0"),
        "{body}"
    );
}
