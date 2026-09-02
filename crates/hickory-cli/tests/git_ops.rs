//! The git pane's verbs, driven over HTTP against a real repository.
//!
//! Protects docs/guarantees/collaboration/the-git-pane-does-the-daily-loop.md
//!
//! System tests in the sense `.instructions/framework-agnostic-system-tests.md`
//! means: the router is bound on a real port and every call is a real HTTP
//! request, so the app could be rewritten without touching a line here.

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
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

async fn start(with_repo: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("notes.hick"),
        "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\" weave=\"notes.md\">\n# Notes\n</hick:doc>\n",
    )
    .unwrap();
    std::fs::write(root.join("a.txt"), "one\ntwo\n").unwrap();
    if with_repo {
        git(&root, &["init", "-q", "-b", "master"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        git(&root, &["add", "-A"]);
        git(&root, &["commit", "-q", "-m", "First"]);
    }
    let prepared = prepare(ServeOptions {
        target: root.join("notes.hick"),
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

#[tokio::test]
async fn a_folder_without_a_repository_says_so_rather_than_failing() {
    let session = start(false).await;
    let (status, body) = get(&session, "/api/git/changes").await;
    assert_eq!(status, 200);
    assert_eq!(body["repository"], false);
    let (status, body) = post(&session, "/api/git/stage", json!({ "all": true })).await;
    assert_eq!(status, 422, "{body}");
    assert!(body["error"].to_string().contains("git init"), "{body}");
}

#[tokio::test]
async fn the_daily_loop_stage_diff_commit_amend() {
    let session = start(true).await;
    std::fs::write(session.root.join("a.txt"), "one\ntwo\nthree\n").unwrap();
    std::fs::write(session.root.join("new.txt"), "hello\n").unwrap();

    // Changes: one modified, one untracked, each saying which side it is on.
    let (_, changes) = get(&session, "/api/git/changes").await;
    assert_eq!(changes["repository"], true);
    assert_eq!(changes["branch"], "master");
    let files = changes["files"].as_array().unwrap();
    let a = files
        .iter()
        .find(|f| f["path"] == "a.txt")
        .expect("a.txt listed");
    assert_eq!(a["tree"], "M");
    assert_eq!(a["index"], " ");
    let new = files
        .iter()
        .find(|f| f["path"] == "new.txt")
        .expect("new.txt listed");
    assert_eq!(new["index"], "?");

    // A diff for the tracked file, and for the untracked one as wholly added.
    let (_, diff) = get(&session, "/api/git/diff?path=a.txt").await;
    assert!(diff["diff"].as_str().unwrap().contains("+three"), "{diff}");
    let (_, diff) = get(&session, "/api/git/diff?path=new.txt").await;
    assert!(diff["diff"].as_str().unwrap().contains("+hello"), "{diff}");

    // Nothing staged: commit says so instead of making an empty commit.
    let (status, body) = post(&session, "/api/git/commit", json!({ "message": "x" })).await;
    assert_eq!(status, 422, "{body}");

    // Stage one, and it moves to the index side.
    let (status, _) = post(&session, "/api/git/stage", json!({ "paths": ["a.txt"] })).await;
    assert_eq!(status, 200);
    let (_, changes) = get(&session, "/api/git/changes").await;
    let a = changes["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "a.txt")
        .unwrap();
    assert_eq!(a["index"], "M");
    assert_eq!(a["tree"], " ");
    let (_, staged) = get(&session, "/api/git/diff?path=a.txt&staged=true").await;
    assert!(staged["diff"].as_str().unwrap().contains("+three"));

    // Unstage, stage everything, commit.
    post(&session, "/api/git/unstage", json!({ "paths": ["a.txt"] })).await;
    let (_, changes) = get(&session, "/api/git/changes").await;
    let a = changes["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "a.txt")
        .unwrap();
    assert_eq!(a["index"], " ");
    post(&session, "/api/git/stage", json!({ "all": true })).await;
    let (status, body) = post(&session, "/api/git/commit", json!({ "message": "Second" })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["subject"], "Second");
    let (_, changes) = get(&session, "/api/git/changes").await;
    assert!(changes["files"].as_array().unwrap().is_empty());

    // An empty message is refused with a reason.
    let (status, _) = post(&session, "/api/git/commit", json!({ "message": "  " })).await;
    assert_eq!(status, 400);

    // Amend rewrites the draft's message. No upstream, so nothing is
    // published and everything is a draft.
    let (status, body) = post(
        &session,
        "/api/git/commit",
        json!({ "message": "Second, reworded", "amend": true }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        git(&session.root, &["log", "-1", "--pretty=%s"]).trim(),
        "Second, reworded"
    );
    assert_eq!(
        git(&session.root, &["rev-list", "--count", "HEAD"]).trim(),
        "2"
    );
}

#[tokio::test]
async fn amending_a_published_commit_is_refused() {
    let session = start(true).await;
    // A bare "remote" the branch tracks, pushed to: HEAD is now below the
    // floor, a record somebody else may hold.
    let remote = tempfile::tempdir().unwrap();
    git(remote.path(), &["init", "-q", "--bare", "-b", "master"]);
    git(
        &session.root,
        &["remote", "add", "origin", remote.path().to_str().unwrap()],
    );
    git(&session.root, &["push", "-q", "-u", "origin", "master"]);

    let (status, body) = post(
        &session,
        "/api/git/commit",
        json!({ "message": "rewrite history", "amend": true }),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert!(body["error"].to_string().contains("floor"), "{body}");

    // A new commit above the floor CAN be amended.
    std::fs::write(session.root.join("a.txt"), "changed\n").unwrap();
    post(&session, "/api/git/stage", json!({ "all": true })).await;
    post(&session, "/api/git/commit", json!({ "message": "Draft" })).await;
    let (status, _) = post(
        &session,
        "/api/git/commit",
        json!({ "message": "Draft, reworded", "amend": true }),
    )
    .await;
    assert_eq!(status, 200);

    // And push carries it up; the pane reports ahead/behind meanwhile.
    let (_, changes) = get(&session, "/api/git/changes").await;
    assert_eq!(changes["ahead"], 1);
    assert_eq!(changes["upstream"], "origin/master");
    let (status, body) = post(&session, "/api/git/push", json!({})).await;
    assert_eq!(status, 200, "{body}");
    let (_, changes) = get(&session, "/api/git/changes").await;
    assert_eq!(changes["ahead"], 0);
    let (status, body) = post(&session, "/api/git/pull", json!({})).await;
    assert_eq!(status, 200, "{body}");
}

#[tokio::test]
async fn branches_stash_and_discard() {
    let session = start(true).await;
    let (status, body) = post(
        &session,
        "/api/git/checkout",
        json!({ "branch": "feature", "create": true }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let (_, branches) = get(&session, "/api/git/branches").await;
    let names: Vec<&str> = branches["branches"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["name"].as_str().unwrap())
        .collect();
    assert!(
        names.contains(&"feature") && names.contains(&"master"),
        "{branches}"
    );
    let current = branches["branches"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| b["current"] == true)
        .unwrap();
    assert_eq!(current["name"], "feature");

    // A switch with dirty work fails with git's own reason when it would
    // clobber; a clean one succeeds.
    let (status, _) = post(&session, "/api/git/checkout", json!({ "branch": "master" })).await;
    assert_eq!(status, 200);

    // Stash puts a change aside and pop brings it back.
    std::fs::write(session.root.join("a.txt"), "stashed\n").unwrap();
    let (status, body) = post(&session, "/api/git/stash", json!({ "action": "push" })).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("a.txt")).unwrap(),
        "one\ntwo\n"
    );
    let (status, _) = post(&session, "/api/git/stash", json!({ "action": "pop" })).await;
    assert_eq!(status, 200);
    assert_eq!(
        std::fs::read_to_string(session.root.join("a.txt")).unwrap(),
        "stashed\n"
    );

    // Discard: a tracked file goes back, an untracked one is deleted.
    std::fs::write(session.root.join("junk.txt"), "x\n").unwrap();
    let (status, body) = post(
        &session,
        "/api/git/discard",
        json!({ "paths": ["a.txt", "junk.txt"] }),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("a.txt")).unwrap(),
        "one\ntwo\n"
    );
    assert!(!session.root.join("junk.txt").exists());

    // A path outside the repository is refused before git ever sees it.
    let (status, _) = post(&session, "/api/git/discard", json!({ "paths": ["../x"] })).await;
    assert_eq!(status, 400);
}
