//! The tree's dired verbs over real HTTP: `POST /api/files/op`.
//!
//! Protects docs/guarantees/authoring/the-tree-is-a-dired.md.

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;

static ENV_LOCK: std::sync::LazyLock<Arc<tokio::sync::Mutex<()>>> =
    std::sync::LazyLock::new(|| Arc::new(tokio::sync::Mutex::new(())));

struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
    _state: tempfile::TempDir,
    _env: tokio::sync::OwnedMutexGuard<()>,
}

async fn start() -> Session {
    let env = ENV_LOCK.clone().lock_owned().await;
    let dir = tempfile::tempdir().unwrap();
    let state_dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("note.hick"), "# note\n").unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("src/a.txt"), "a\n").unwrap();
    std::fs::write(root.join("src/b.txt"), "b\n").unwrap();
    std::fs::create_dir_all(root.join(".git")).unwrap();
    std::fs::write(root.join(".git/HEAD"), "ref: refs/heads/master\n").unwrap();
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    Session {
        base: format!("http://127.0.0.1:{port}"),
        root,
        _dir: dir,
        _state: state_dir,
        _env: env,
    }
}

async fn op(s: &Session, body: Value) -> (u16, Value) {
    let res = reqwest::Client::new()
        .post(format!("{}/api/files/op", s.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = res.status().as_u16();
    (status, res.json().await.unwrap_or(Value::Null))
}

#[tokio::test]
async fn every_verb_does_one_thing_on_disk() {
    let s = start().await;
    let (status, body) = op(
        &s,
        json!({"op": "rename", "path": "src/a.txt", "to": "c.txt"}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["to"], "src/c.txt");
    assert!(s.root.join("src/c.txt").exists() && !s.root.join("src/a.txt").exists());

    let (status, _) = op(&s, json!({"op": "mkdir", "path": "docs"})).await;
    assert_eq!(status, 200);
    assert!(s.root.join("docs").is_dir());

    let (status, body) = op(&s, json!({"op": "move", "path": "src/c.txt", "to": "docs"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["to"], "docs/c.txt");
    assert!(s.root.join("docs/c.txt").exists());

    let (status, body) = op(&s, json!({"op": "copy", "path": "src", "to": "docs"})).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        std::fs::read_to_string(s.root.join("docs/src/b.txt")).unwrap(),
        "b\n"
    );
    assert!(
        s.root.join("src/b.txt").exists(),
        "a copy leaves the source"
    );

    let (status, _) = op(&s, json!({"op": "create", "path": "docs/new.md"})).await;
    assert_eq!(status, 200);
    assert_eq!(
        std::fs::read_to_string(s.root.join("docs/new.md")).unwrap(),
        ""
    );

    let (status, _) = op(&s, json!({"op": "delete", "path": "docs/src"})).await;
    assert_eq!(status, 200);
    assert!(!s.root.join("docs/src").exists());
    let (status, _) = op(&s, json!({"op": "delete", "path": "docs/new.md"})).await;
    assert_eq!(status, 200);
    assert!(!s.root.join("docs/new.md").exists());
}

#[tokio::test]
async fn the_refusals_are_said_plainly() {
    let s = start().await;
    let cases = [
        (json!({"op": "delete", "path": "../etc"}), 400, "no `..`"),
        (
            json!({"op": "delete", "path": ".git/HEAD"}),
            400,
            "inside .git",
        ),
        (
            json!({"op": "delete", "path": ""}),
            400,
            "the folder itself",
        ),
        (
            json!({"op": "rename", "path": "src/a.txt", "to": "b.txt"}),
            409,
            "already exists",
        ),
        (
            json!({"op": "move", "path": "src", "to": "src"}),
            400,
            "into itself",
        ),
        (
            json!({"op": "create", "path": "x.hick"}),
            400,
            "New Document",
        ),
        (
            json!({"op": "delete", "path": "gone.txt"}),
            404,
            "not in this folder",
        ),
        (
            json!({"op": "shred", "path": "src/a.txt"}),
            400,
            "unknown file operation",
        ),
    ];
    for (body, want, says) in cases {
        let (status, answer) = op(&s, body.clone()).await;
        assert_eq!(status, want, "{body}: {answer}");
        assert!(
            answer["error"].as_str().unwrap_or("").contains(says),
            "{body}: expected {says:?} in {answer}"
        );
    }
    // Nothing above touched the disk.
    assert!(s.root.join("src/a.txt").exists() && s.root.join("src/b.txt").exists());
    assert!(s.root.join(".git/HEAD").exists());
}
