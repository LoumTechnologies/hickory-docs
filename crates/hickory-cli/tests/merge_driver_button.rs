//! The banner's "Run hick init" button, driven over real HTTP:
//! `POST /api/git/merge-driver` runs `hick init` in the engine and the
//! status the banner asked about comes back fixed.
//!
//! Protects docs/guarantees/collaboration/a-missing-merge-driver-is-a-button.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;
use std::process::Command;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_AUTHOR_NAME", "t")
        .env("GIT_AUTHOR_EMAIL", "t@t")
        .env("GIT_COMMITTER_NAME", "t")
        .env("GIT_COMMITTER_EMAIL", "t@t")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[tokio::test]
async fn the_button_runs_hick_init_and_the_driver_is_then_defined() {
    let _env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note.hick"), "# A note\n").unwrap();
    // The committed half without the per-clone half: routing, no driver.
    std::fs::write(root.join(".gitattributes"), "*.hick merge=hick\n").unwrap();
    git(&root, &["init", "-q"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "routing only"]);
    unsafe { std::env::set_var("HICKORY_STATE_DIR", state_dir.path()) };

    let prepared = prepare(ServeOptions {
        target: root.join("note.hick"),
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
    let base = format!("http://127.0.0.1:{port}");

    let before: Value = reqwest::get(format!("{base}/api/git/merge-driver"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(before["ok"], false, "{before}");
    assert_eq!(before["status"]["configured"], false);

    let res = reqwest::Client::new()
        .post(format!("{base}/api/git/merge-driver"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().as_u16(), 200);
    let after: Value = res.json().await.unwrap();
    assert_eq!(after["ok"], true, "{after}");
    assert_eq!(after["changed"]["merge_driver"], true, "{after}");
    assert_eq!(after["status"]["configured"], true);

    // And the banner's own question now answers yes.
    let again: Value = reqwest::get(format!("{base}/api/git/merge-driver"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["ok"], true, "{again}");
}
