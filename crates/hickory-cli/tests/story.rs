//! The history lens's verbs, over the real routes.
//!
//! Protects docs/guarantees/lenses/a-recipe-commit-can-be-replayed.md,
//! docs/guarantees/lenses/the-tail-of-the-story-is-the-next-commit.md and
//! docs/guarantees/lenses/the-past-is-edited-by-rebase-above-the-floor.md.
//!
//! Real HTTP against a real repository, so the app could be rewritten
//! without touching a line here.

use std::path::{Path, PathBuf};
use std::process::Command;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

fn git(root: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("notes.md"), "# Notes\n").unwrap();
    std::fs::write(root.join(".gitignore"), "obj/\n").unwrap();
    git(&root, &["init", "-q", "-b", "master"]);
    git(&root, &["config", "user.email", "test@example.com"]);
    git(&root, &["config", "user.name", "Test"]);
    git(&root, &["add", "-A"]);
    git(&root, &["commit", "-qm", "start"]);
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
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let response = reqwest::get(format!("{}{path}", session.base))
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, response.json().await.unwrap_or(Value::Null))
}

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let response = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = response.status().as_u16();
    (status, response.json().await.unwrap_or(Value::Null))
}

fn subjects(root: &Path) -> String {
    git(root, &["log", "--reverse", "--format=%s"])
}

#[tokio::test]
async fn the_tail_runs_a_command_and_the_card_can_be_replayed() {
    let session = start().await;
    let version = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(version.path(), "v1\n").unwrap();
    let command = format!(
        "mkdir -p app app/obj && cp {} app/gen.txt && echo x > app/obj/cache",
        version.path().display()
    );

    // The tail: a command typed there runs in a clean worktree and its output
    // is committed as HEAD's child, with the recipe.
    let (status, run) = post(
        &session,
        "/api/git/recipe",
        json!({ "command": command, "output": "app" }),
    )
    .await;
    assert_eq!(status, 200, "{run}");
    let s1 = run["sha"].as_str().unwrap().to_string();
    assert_eq!(git(&session.root, &["rev-parse", "HEAD"]), s1);
    assert_eq!(
        std::fs::read_to_string(session.root.join("app/gen.txt")).unwrap(),
        "v1\n"
    );
    assert!(
        !session.root.join("app/obj").exists(),
        "ignored output is not materialised"
    );
    assert_eq!(git(&session.root, &["status", "--porcelain"]), "");

    // The log draws it as a recipe whose tree matches, and not yet replayed.
    let (_, log) = get(&session, "/api/git/log").await;
    let card = &log["commits"][0];
    assert_eq!(card["recipe"]["output_matches"], true, "{card}");
    assert!(card["recipe"].get("replay_of").is_none());

    // The person's edit, then the scaffolder "upgrades".
    std::fs::write(session.root.join("app/mine.txt"), "mine\n").unwrap();
    git(&session.root, &["add", "-A"]);
    git(&session.root, &["commit", "-qm", "Add mine"]);
    std::fs::write(version.path(), "v2\n").unwrap();

    // A dirty tree is refused in words, and nothing moves.
    std::fs::write(session.root.join("notes.md"), "# Notes\ndirty\n").unwrap();
    let (status, refused) = post(&session, "/api/git/replay", json!({ "sha": s1 })).await;
    assert_eq!(status, 422, "{refused}");
    assert!(refused["error"].as_str().unwrap().contains("uncommitted"));
    git(&session.root, &["checkout", "--", "notes.md"]);

    // Above the floor (nothing is published): a sibling, and a rebase.
    let (status, replay) = post(&session, "/api/git/replay", json!({ "sha": s1 })).await;
    assert_eq!(status, 200, "{replay}");
    assert_eq!(replay["moved"], "rebase");
    assert_eq!(replay["same"], false);
    assert_eq!(
        subjects(&session.root),
        "start\nReplay: Run `mkdir -p app app/obj…`\nAdd mine"
    );
    assert_eq!(
        std::fs::read_to_string(session.root.join("app/gen.txt")).unwrap(),
        "v2\n"
    );
    assert_eq!(
        std::fs::read_to_string(session.root.join("app/mine.txt")).unwrap(),
        "mine\n"
    );

    // The replay card carries the evidence: replayed, and it differed.
    let (_, log) = get(&session, "/api/git/log").await;
    let replayed = log["commits"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["sha"] == replay["sha"])
        .expect("the replay is in the log");
    assert_eq!(replayed["recipe"]["replay_of"], s1);
    assert_eq!(replayed["recipe"]["replay_same"], false);
    assert_eq!(replayed["recipe"]["output_matches"], true);
}

#[tokio::test]
async fn drafts_are_reworded_moved_and_dropped_and_records_are_refused() {
    let session = start().await;
    // Publish `start`, so it is a record; A, B, C are drafts.
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare"]);
    git(
        &session.root,
        &["remote", "add", "origin", &remote.path().to_string_lossy()],
    );
    git(&session.root, &["push", "-q", "-u", "origin", "master"]);
    for (name, subject) in [("a.txt", "A"), ("b.txt", "B"), ("c.txt", "C")] {
        std::fs::write(session.root.join(name), format!("{name}\n")).unwrap();
        git(&session.root, &["add", name]);
        git(&session.root, &["commit", "-qm", subject]);
    }
    let start_sha = git(&session.root, &["rev-parse", "HEAD~3"]);
    let b = git(&session.root, &["rev-parse", "HEAD~1"]);

    let (status, answer) = post(
        &session,
        "/api/git/reword",
        json!({ "sha": b, "message": "B, said better\n\nBecause." }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(subjects(&session.root), "start\nA\nB, said better\nC");

    let b = git(&session.root, &["rev-parse", "HEAD~1"]);
    let (status, answer) = post(
        &session,
        "/api/git/move",
        json!({ "sha": b, "direction": "earlier" }),
    )
    .await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(subjects(&session.root), "start\nB, said better\nA\nC");

    let a = git(&session.root, &["rev-parse", "HEAD~1"]);
    let (status, answer) = post(&session, "/api/git/drop", json!({ "sha": a })).await;
    assert_eq!(status, 200, "{answer}");
    assert_eq!(subjects(&session.root), "start\nB, said better\nC");
    assert!(!session.root.join("a.txt").exists());

    // A record is refused by name; the floor is the line.
    let (status, refused) = post(
        &session,
        "/api/git/reword",
        json!({ "sha": start_sha, "message": "nope" }),
    )
    .await;
    assert_eq!(status, 422, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("below the publication floor")
    );
    assert_eq!(subjects(&session.root), "start\nB, said better\nC");
    assert_eq!(git(&session.root, &["status", "--porcelain"]), "");
}

#[tokio::test]
async fn a_reorder_that_conflicts_is_answered_with_gits_words_and_left_where_git_left_it() {
    let session = start().await;
    // Two drafts that touch the same line: swapping them cannot apply cleanly.
    std::fs::write(session.root.join("notes.md"), "# Notes\none\n").unwrap();
    git(&session.root, &["commit", "-qam", "one"]);
    std::fs::write(session.root.join("notes.md"), "# Notes\ntwo\n").unwrap();
    git(&session.root, &["commit", "-qam", "two"]);
    let two = git(&session.root, &["rev-parse", "HEAD"]);

    let (status, stopped) = post(
        &session,
        "/api/git/move",
        json!({ "sha": two, "direction": "earlier" }),
    )
    .await;
    assert_eq!(status, 409, "{stopped}");
    let words = stopped["error"].as_str().unwrap();
    assert!(words.contains("rebase"), "{words}");
    assert!(words.contains("where git left it"), "{words}");
    // Git's state, not hidden: a rebase in progress.
    assert!(
        session.root.join(".git/rebase-merge").exists(),
        "the rebase was not left for the person to finish"
    );
    git(&session.root, &["rebase", "--abort"]);
    assert_eq!(subjects(&session.root), "start\none\ntwo");
}
