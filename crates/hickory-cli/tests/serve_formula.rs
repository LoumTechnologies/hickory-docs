//! The formula route, driven the way the table's grid drives it.
//!
//! Protects docs/guarantees/execution/a-formula-is-an-expression-in-a-real-language.md
//! and docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md

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

#[tokio::test(flavor = "multi_thread")]
async fn a_trace_walks_the_grid_cell_by_cell_in_the_order_it_ran() {
    // What the table's debugger steps through: which cell went when, what it
    // read, and what it came to.
    if !have(&["python3", "python"]) {
        eprintln!("skipped: no python on this machine");
        return;
    }
    let session = start().await;
    let (status, body) = post(
        &session,
        "/api/formula/trace",
        json!({
            "language": "python",
            "rows": [
                ["10", "=A1*2"],
                ["=B1+5", ""],
            ],
        }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let steps = body["steps"].as_array().unwrap();
    let order: Vec<&str> = steps.iter().map(|s| s["cell"].as_str().unwrap()).collect();
    assert_eq!(order, vec!["B1", "A2"], "{body}");

    assert_eq!(steps[0]["expression"], "A1*2");
    assert_eq!(steps[0]["level"], 0);
    assert_eq!(steps[0]["bindings"][0]["cell"], "A1");
    assert_eq!(steps[0]["bindings"][0]["text"], "10");
    // A blank cell is not the empty string, and the step is where the
    // difference is visible.
    assert_eq!(steps[0]["bindings"][0]["kind"], "number");
    assert_eq!(steps[0]["value"], "20");
    assert!(steps[0]["error"].is_null());

    assert_eq!(steps[1]["level"], 1, "the chain costs a second round trip");
    assert_eq!(steps[1]["bindings"][0]["text"], "20");
    assert_eq!(steps[1]["value"], "25");

    // The same answer the grid gets, from the same evaluation.
    assert_eq!(body["values"]["B1"], "20");
    assert_eq!(body["values"]["A2"], "25");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_trace_of_a_circle_has_nothing_to_step_through_and_says_why() {
    let session = start().await;
    let (status, body) = post(
        &session,
        "/api/formula/trace",
        json!({ "language": "python", "rows": [["=B1", "=A1"]] }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert!(body["steps"].as_array().unwrap().is_empty(), "{body}");
    assert!(
        body["errors"]["A1"].as_str().unwrap().contains("circle"),
        "{body}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_trace_refuses_a_table_that_is_really_a_dataset_too() {
    // The same limit as evaluating: stepping through a dataset would tie up
    // an interpreter to fill a panel nobody is reading.
    let session = start().await;
    let rows: Vec<Vec<String>> = (0..500)
        .map(|_| (0..50).map(|_| "1".to_string()).collect())
        .collect();
    let (status, body) = post(
        &session,
        "/api/formula/trace",
        json!({ "language": "python", "rows": rows }),
    )
    .await;
    assert_eq!(status, 422);
    assert!(
        body["error"].as_str().unwrap().contains("exec cell"),
        "{body}"
    );
}
