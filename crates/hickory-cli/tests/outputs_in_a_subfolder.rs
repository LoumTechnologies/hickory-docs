//! A document in a subfolder names its outputs the way the tree does.
//!
//! Protects docs/guarantees/authoring/an-output-is-named-from-the-open-folder.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

#[tokio::test]
async fn a_generated_file_in_a_subfolder_is_found_by_its_folder_relative_path() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::create_dir_all(root.join("tools")).unwrap();
    std::fs::write(
        root.join("tools/app.md"),
        "# A tool\n\n<hick:file path=\"app.py\">\nprint(1)\n</hick:file>\n",
    )
    .unwrap();
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };
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
    let doc_id = prepared.state.index.sole().expect("one document").0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let base = format!("http://127.0.0.1:{port}");
    let get = |path: String| {
        let base = base.clone();
        async move {
            let res = reqwest::get(format!("{base}{path}")).await.unwrap();
            (
                res.status().as_u16(),
                res.json::<Value>().await.unwrap_or(Value::Null),
            )
        }
    };

    // Listed the way the tree names it.
    let (_, listed) = get(format!("/api/docs/{doc_id}/outputs")).await;
    let paths: Vec<&str> = listed["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    assert!(paths.contains(&"tools/app.py"), "{listed}");
    assert!(!paths.contains(&"tools/app.md"), "{listed}");

    // Found by that name, and answered under it.
    let (status, file) = get(format!("/api/docs/{doc_id}/outputs/file?path=tools/app.py")).await;
    assert_eq!(status, 200, "{file}");
    assert_eq!(file["path"], "tools/app.py");
    assert!(file["content"].as_str().unwrap().contains("print(1)"));

    // The document's own spelling still works — a `<hick:file path>` is
    // relative to the document, and that is not wrong, only local.
    let (status, same) = get(format!("/api/docs/{doc_id}/outputs/file?path=app.py")).await;
    assert_eq!(status, 200, "{same}");
    assert_eq!(same["path"], "tools/app.py");

    // And a miss lists what exists in the tree's spelling.
    let (status, miss) = get(format!(
        "/api/docs/{doc_id}/outputs/file?path=tools/nope.py"
    ))
    .await;
    assert_eq!(status, 404);
    assert!(
        miss["error"].as_str().unwrap().contains("tools/app.py"),
        "{miss}"
    );
}
