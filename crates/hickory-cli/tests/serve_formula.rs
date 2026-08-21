//! The formula route, driven the way the table's grid drives it.
//!
//! Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    _dir: tempfile::TempDir,
}

fn have(interpreters: &[&str]) -> bool {
    interpreters.iter().any(|name| {
        std::process::Command::new(name)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    })
}

async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.hick"), "# Notes\n").unwrap();
    let root = dir.path().canonicalize().unwrap();
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
    let text = resp.text().await.unwrap_or_default();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_grid_of_formulas_comes_back_computed() {
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let session = start().await;
    let (status, body) = post(
        &session,
        "/api/formula/evaluate",
        json!({
            "language": "python",
            "rows": [
                ["region", "units"],
                ["north", "120"],
                ["south", "90"],
                ["total", "=B2+B3"],
            ],
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["values"]["B4"], "210");
    assert_eq!(body["errors"].as_object().unwrap().len(), 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_formula_cells_come_back() {
    // Echoing literals would make the response the size of the table for no
    // reason.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let session = start().await;
    let (_, body) = post(
        &session,
        "/api/formula/evaluate",
        json!({ "language": "python", "rows": [["1", "2", "=A1+B1"]] }),
    )
    .await;
    let values = body["values"].as_object().unwrap();
    assert_eq!(values.len(), 1, "{values:?}");
    assert_eq!(values["C1"], "3");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_circular_reference_names_the_cells_in_the_circle() {
    // "Circular reference" without saying where is the single most useless
    // error a spreadsheet produces. No interpreter is needed to know this.
    let session = start().await;
    let (status, body) = post(
        &session,
        "/api/formula/evaluate",
        json!({ "language": "python", "rows": [["=B1", "=A1"]] }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let errors = body["errors"].as_object().unwrap();
    assert_eq!(errors.len(), 2);
    assert!(
        errors["A1"].as_str().unwrap().contains("circle"),
        "{errors:?}"
    );
    assert!(errors["A1"].as_str().unwrap().contains("B1"), "{errors:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_language_nothing_can_run_says_what_is_missing() {
    let session = start().await;
    let (status, body) = post(
        &session,
        "/api/formula/evaluate",
        json!({ "language": "cobol", "rows": [["=1+1"]] }),
    )
    .await;
    assert_eq!(status, 422);
    assert!(body["error"].as_str().unwrap().contains("cobol"), "{body}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_table_too_big_to_be_a_spreadsheet_is_refused_with_a_reason() {
    let session = start().await;
    let rows: Vec<Vec<String>> = (0..500)
        .map(|_| (0..50).map(|_| "1".to_string()).collect())
        .collect();
    let (status, body) = post(
        &session,
        "/api/formula/evaluate",
        json!({ "language": "python", "rows": rows }),
    )
    .await;
    assert_eq!(status, 422);
    assert!(
        body["error"].as_str().unwrap().contains("exec cell"),
        "the message must say what to do instead: {body}"
    );
}
