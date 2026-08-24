//! The merged view over two real local worktrees, read-only.
//!
//! Protects `docs/guarantees/collaboration/the-merged-view-is-a-lens.md`.
//!
//! This is where the alignment risk lives: a bad alignment routes an edit
//! silently into the wrong file, so the first step is deliberately read-only
//! and deliberately conservative — fewer regions declared shared, more shown
//! as variants, because the failure of over-sharing is an edit in the wrong
//! place and the failure of under-sharing is a little redundant typing.

use std::path::{Path, PathBuf};

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::Value;

fn git(dir: &Path, args: &[&str]) -> std::process::Output {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="none">
<hick:file path="note.txt">shared head
</hick:file>
</hick:doc>
"##;

async fn start() -> Session {
    // One tempdir holding BOTH worktrees, so they are siblings that vanish
    // together and never collide with another test's.
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap().join("repo");
    std::fs::create_dir_all(&root).unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    git(&root, &["config", "user.email", "t@example.com"]);
    git(&root, &["config", "user.name", "T"]);
    git(&root, &["config", "commit.gpgsign", "false"]);
    std::fs::write(root.join("demo.hick"), DOC).unwrap();
    std::fs::write(root.join("shared.txt"), "head\nmaster only\ntail\n").unwrap();
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-q", "--no-verify", "-m", "base"]);

    // A second worktree on its own branch — "across branches" means across
    // worktrees, because a branch that is not checked out cannot be written
    // to without going behind the working tree.
    let side = root.parent().unwrap().join("side-tree");
    assert!(!side.exists());
    let side_str = side.to_string_lossy().to_string();
    git(&root, &["worktree", "add", "-q", "-b", "side", &side_str]);
    std::fs::write(side.join("shared.txt"), "head\nside only\ntail\n").unwrap();

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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        root,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> Value {
    reqwest::get(format!("{}{path}", session.base))
        .await
        .unwrap()
        .json()
        .await
        .unwrap_or(Value::Null)
}

#[tokio::test]
async fn both_worktrees_are_offered_as_sources() {
    let session = start().await;
    let answer = get(&session, "/api/worktrees").await;
    assert_eq!(answer["repository"], Value::Bool(true));
    let names: Vec<&str> = answer["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"side-tree"), "{answer}");
    assert_eq!(names.len(), 2, "{answer}");
    // Branches are named, because which branch a worktree is on is the thing
    // a person is choosing between.
    let branches: Vec<&str> = answer["worktrees"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|w| w["branch"].as_str())
        .collect();
    assert!(branches.contains(&"side"), "{answer}");
}

#[tokio::test]
async fn agreed_regions_appear_once_and_differences_appear_as_variants() {
    let session = start().await;
    let answer = get(&session, "/api/merged?path=shared.txt").await;
    let regions = answer["regions"].as_array().unwrap();

    let shared: String = regions
        .iter()
        .filter(|r| r["kind"] == "shared")
        .map(|r| r["text"].as_str().unwrap())
        .collect();
    assert_eq!(shared, "head\ntail\n", "{answer}");

    let variants: Vec<&Value> = regions.iter().filter(|r| r["kind"] == "variant").collect();
    assert_eq!(variants.len(), 1, "{answer}");
    let by = &variants[0]["by_source"];
    let sides: Vec<&str> = by
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert!(sides.contains(&"master only\n"), "{answer}");
    assert!(sides.contains(&"side only\n"), "{answer}");

    // It is a LENS, not a document: there is no save path, and the view says
    // so rather than leaving it to be discovered.
    assert_eq!(answer["read_only"], Value::Bool(true));
}

#[tokio::test]
async fn a_worktree_without_the_file_is_named_rather_than_dropped() {
    // "This branch does not have this file yet" is an answer somebody opened
    // the view to get.
    let session = start().await;
    std::fs::write(session.root.join("only-here.txt"), "a\nb\n").unwrap();
    let answer = get(&session, "/api/merged?path=only-here.txt").await;
    let missing: Vec<&str> = answer["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m.as_str().unwrap())
        .collect();
    assert_eq!(missing, vec!["side-tree"], "{answer}");
    // And the one source that has it is entirely shared with itself.
    assert_eq!(answer["variants"], Value::from(0), "{answer}");
}

#[tokio::test]
async fn one_target_can_be_asked_for_by_name() {
    let session = start().await;
    let answer = get(&session, "/api/merged?path=shared.txt&targets=side-tree").await;
    assert_eq!(answer["sources"].as_array().unwrap().len(), 1, "{answer}");
    assert_eq!(answer["variants"], Value::from(0), "{answer}");
}

#[tokio::test]
async fn a_path_that_escapes_the_repository_is_refused() {
    let session = start().await;
    let status = reqwest::get(format!("{}/api/merged?path=../etc/passwd", session.base))
        .await
        .unwrap()
        .status();
    assert_eq!(status.as_u16(), 400);
}

// ---------------------------------------------------------------------------
// Writing through the view
// ---------------------------------------------------------------------------

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

fn side_path(session: &Session) -> PathBuf {
    session.root.parent().unwrap().join("side-tree")
}

/// The index of the first shared / first variant region in the live view.
async fn region_indices(session: &Session) -> (usize, usize) {
    let view = get(session, "/api/merged?path=shared.txt").await;
    let regions = view["regions"].as_array().unwrap();
    let shared = regions.iter().position(|r| r["kind"] == "shared").unwrap();
    let variant = regions.iter().position(|r| r["kind"] == "variant").unwrap();
    (shared, variant)
}

#[tokio::test]
async fn a_shared_edit_writes_every_target_in_one_keystroke() {
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "HEAD\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["landed"], Value::from(2), "{answer}");
    assert_eq!(answer["partial"], Value::Bool(false), "{answer}");

    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "HEAD\nmaster only\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        "HEAD\nside only\ntail\n"
    );
}

#[tokio::test]
async fn just_here_writes_one_target_and_leaves_the_other_alone() {
    // The gesture no longer wraps text into a conditional in a file: it means
    // "write this to one target only", which is easier to implement and
    // easier to explain.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "HEAD\n",
            "route": "just-here",
            "source": "repo",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(answer["landed"], Value::from(1), "{answer}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "HEAD\nmaster only\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

#[tokio::test]
async fn a_target_not_opened_for_writing_is_never_written() {
    // Read-only is the DEFAULT for anything not explicitly opened for
    // writing: a long-lived release branch in the view must not silently
    // receive shared edits.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo"],
            "region": shared,
            "text": "HEAD\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    let named: Vec<&str> = answer["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["source"].as_str().unwrap())
        .collect();
    assert_eq!(named, vec!["repo"], "{answer}");
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

#[tokio::test]
async fn undo_puts_every_target_back_including_ones_the_write_did_not_reach() {
    // A half-applied edit that is then undone in three of four places is a
    // state somebody will reach on the first day.
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let before_main = std::fs::read_to_string(session.root.join("shared.txt")).unwrap();
    let before_side = std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap();

    let (_, written) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "CHANGED\n",
            "route": "shared",
        }),
    )
    .await;
    assert_eq!(written["landed"], Value::from(2));

    let (status, undone) = post(
        &session,
        "/api/merged/undo",
        serde_json::json!({ "path": "shared.txt", "undo": written["undo"] }),
    )
    .await;
    assert_eq!(status, 200, "{undone}");
    assert_eq!(undone["restored"], Value::from(2), "{undone}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        before_main
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        before_side
    );
}

#[tokio::test]
async fn editing_a_variant_touches_only_that_side() {
    let session = start().await;
    let (_, variant) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": variant,
            "text": "master changed\n",
            "route": "just-here",
            "source": "repo",
        }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("shared.txt")).unwrap(),
        "head\nmaster changed\ntail\n"
    );
    assert_eq!(
        std::fs::read_to_string(side_path(&session).join("shared.txt")).unwrap(),
        "head\nside only\ntail\n"
    );
}

#[tokio::test]
async fn a_write_routed_at_a_source_that_is_not_here_is_refused() {
    let session = start().await;
    let (shared, _) = region_indices(&session).await;
    let (status, answer) = post(
        &session,
        "/api/merged/write",
        serde_json::json!({
            "path": "shared.txt",
            "targets": ["repo", "side-tree"],
            "region": shared,
            "text": "x\n",
            "route": "just-here",
            "source": "ghost",
        }),
    )
    .await;
    assert_eq!(status, 422, "{answer}");
}
