//! File → New Project, over the real routes.
//!
//! Protects `docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md`.
//!
//! The parsing half is covered against checked-in fixtures in
//! `scaffold_templates.rs`. What is left is the half no fixture can show: that
//! the dialog's answer becomes a commit carrying its recipe, that the commit
//! holds exactly what the scaffolder wrote and nothing of the person's, and
//! that a scaffold which does not run commits nothing.
//!
//! Anything that needs a real `dotnet` says so and skips without one. The
//! machine that has an SDK gets the whole claim tested; the machine that does
//! not still tests everything up to the subprocess, which is where the
//! interesting mistakes are.
//!
//! `POST /api/scaffold` answers `202` with a terminal session, not `201` with
//! a commit: the scaffolder is watched rather than reported
//! (`docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md`),
//! and the commit lands when it exits. So every claim about a commit is made
//! after `settle()` has waited for the session to finish.

use std::path::PathBuf;

use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};

struct Session {
    base: String,
    root: PathBuf,
    _dir: tempfile::TempDir,
}

/// A folder with one document in it, a git repository, and a `.gitignore`
/// that ignores what a .NET restore writes.
///
/// The `.gitignore` is not scene-setting: it is the ingest's filter, and
/// without it `obj/` lands in the document. See
/// `docs/guarantees/execution/a-volume-carries-what-the-repository-carries.md`.
async fn start() -> Session {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.hick"), "# Notes\n").unwrap();
    std::fs::write(dir.path().join(".gitignore"), "obj/\nbin/\n").unwrap();
    let root = dir.path().canonicalize().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["add", "-A"],
        vec![
            "-c",
            "user.email=t@example.com",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "start",
        ],
    ] {
        let ok = std::process::Command::new("git")
            .args(&args)
            .current_dir(&root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        assert!(ok, "git {args:?} in the fixture repository");
    }

    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: Vec::new(),
        // `dotnet new` is the subject; the local executor keeps the test to
        // one process and no daemon.
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
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

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

/// Wait for a started scaffold to reach a verdict, then answer it.
///
/// The route hands back a terminal; the commit happens when that terminal's
/// command exits. Polling `GET /api/scaffold/result` is what the dialog does
/// too, so the test drives the app the way the app drives itself.
async fn settle(session: &Session, term: &str) -> Value {
    for _ in 0..600 {
        let (status, body) = get(session, &format!("/api/scaffold/result?session={term}")).await;
        assert_eq!(status, 200, "{body}");
        if body["state"] != json!("running") {
            return body;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("the scaffold never finished");
}

fn has_dotnet() -> bool {
    std::process::Command::new("dotnet")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn spec(name: &str, output: &str) -> Value {
    json!({
        "template": "console",
        "title": "Console App",
        "language": "C#",
        "name": name,
        "output": output,
        "image": "mcr.microsoft.com/dotnet/sdk:10.0",
        // Restore is a build step, not a scaffold step, and skipping it is
        // what keeps `obj/` out of the run entirely rather than relying on
        // the ignore filter to take it back out.
        "options": [{ "flag": "--no-restore" }],
    })
}

fn git_out(root: &std::path::Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[tokio::test]
async fn the_preview_is_the_commit_that_gets_made() {
    let session = start().await;
    let (status, preview) = post(
        &session,
        "/api/scaffold/preview",
        spec("Greeter", "greeter"),
    )
    .await;
    assert_eq!(status, 200, "{preview}");
    assert_eq!(
        preview["command"],
        "dotnet new console -o greeter -n Greeter --language 'C#' --no-restore"
    );
    let message = preview["message"].as_str().unwrap();
    assert!(
        message.starts_with("Scaffold Greeter with `dotnet new console`"),
        "{message}"
    );
    assert!(
        message.contains("Hick-Recipe: dotnet new console -o greeter"),
        "{message}"
    );
    assert!(
        message.contains("Hick-Image: mcr.microsoft.com/dotnet/sdk:10.0"),
        "{message}"
    );
    // The one thing a preview cannot know is said as such, not faked.
    assert!(
        message.contains("Hick-Output: <the tree hash, once it is committed> greeter"),
        "{message}"
    );
}

#[tokio::test]
async fn a_new_project_is_a_commit_holding_exactly_what_dotnet_wrote() {
    if !has_dotnet() {
        eprintln!("SKIPPED: no dotnet on this machine");
        return;
    }
    let session = start().await;
    // Work in flight, which the recipe commit must neither take nor lose.
    std::fs::write(session.root.join("notes.hick"), "# Notes\nmine\n").unwrap();
    let before = git_out(&session.root, &["rev-parse", "HEAD"]);

    let (status, started) = post(&session, "/api/scaffold", spec("Greeter", "greeter")).await;
    assert_eq!(status, 202, "{started}");
    // The recipe is recorded in the repository that holds the location, and
    // the answer says which one before anything has run.
    assert_eq!(
        started["repository"],
        session.root.to_string_lossy().as_ref()
    );
    assert_eq!(started["output"], "greeter");
    let term = started["session"]["id"].as_str().unwrap().to_string();
    let created = settle(&session, &term).await;
    assert_eq!(created["state"], "committed", "{created}");
    let sha = created["sha"].as_str().unwrap().to_string();
    assert_eq!(git_out(&session.root, &["rev-parse", "HEAD"]), sha);
    assert_eq!(git_out(&session.root, &["rev-parse", "HEAD^"]), before);
    let files: Vec<&str> = created["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert!(files.contains(&"greeter/Program.cs"), "{files:?}");
    assert!(files.iter().all(|f| f.starts_with("greeter/")), "{files:?}");
    assert!(
        !files.iter().any(|f| f.contains("/obj/")),
        "a restore's obj/ is not the scaffolder's: {files:?}"
    );
    // The person's edit is still theirs, uncommitted; the scaffold is clean.
    let status_lines = git_out(&session.root, &["status", "--porcelain"]);
    assert_eq!(status_lines.trim(), "M notes.hick", "{status_lines}");

    // The verdict is in the terminal that earned it, not only in an API
    // answer nobody reads: the commit and the command are one thing to read.
    // docs/guarantees/execution/a-command-the-app-runs-is-watched-in-a-terminal.md
    let (status, terminals) = get(&session, "/api/terminals").await;
    assert_eq!(status, 200, "{terminals}");
    let ours = terminals["sessions"]
        .as_array()
        .expect("sessions")
        .iter()
        .find(|s| s["id"] == json!(term))
        .expect("the scaffold's own session is still in the dock");
    assert_eq!(ours["title"], "New project: Greeter");
    assert!(
        ours["preview"]
            .as_str()
            .unwrap()
            .contains("replayed with a newer SDK"),
        "the session's last line is the verdict: {ours}"
    );

    // The log draws it as a recipe whose tree matches its trailer.
    let (status, log) = get(&session, "/api/git/log").await;
    assert_eq!(status, 200);
    let card = &log["commits"][0];
    assert_eq!(card["sha"], sha);
    assert_eq!(
        card["recipe"]["command"],
        created["message"]
            .as_str()
            .unwrap()
            .lines()
            .find_map(|l| l.strip_prefix("Hick-Recipe: "))
            .unwrap()
    );
    assert_eq!(card["recipe"]["output_path"], "greeter", "{card}");
    assert_eq!(card["recipe"]["output_matches"], true, "{card}");
    assert_eq!(card["recipe"]["output"], created["output_tree"]);

    // A second scaffold into the same folder is refused: it is occupied. And
    // refused *before* a terminal opens — a tab that exists only to say "that
    // folder is not empty" is one the person has to close for nothing.
    let (status, refused) = post(&session, "/api/scaffold", spec("Greeter", "greeter")).await;
    assert_eq!(status, 422, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("already exists"),
        "{refused}"
    );
    assert_eq!(
        git_out(&session.root, &["rev-parse", "HEAD"]),
        sha,
        "nothing was committed"
    );
}

#[tokio::test]
async fn a_scaffold_that_cannot_run_commits_nothing_and_leaves_no_folder() {
    let session = start().await;
    let before = git_out(&session.root, &["rev-parse", "HEAD"]);
    let mut body = spec("Ghost", "ghost");
    body["template"] = json!("no-such-template-exists");

    let (status, answer) = post(&session, "/api/scaffold", body).await;
    if !has_dotnet() {
        // No SDK is refused before a terminal is opened at all, and by a
        // field rather than a sentence.
        assert_eq!(status, 422, "{answer}");
        assert_eq!(answer["missing"], json!("dotnet"));
        return;
    }
    // With an SDK, an unknown template is `dotnet`'s refusal to make, and it
    // makes it in the terminal — where the person can read it.
    assert_eq!(status, 202, "{answer}");
    let term = answer["session"]["id"].as_str().unwrap().to_string();
    let ended = settle(&session, &term).await;
    assert_eq!(ended["state"], "failed", "{ended}");
    assert!(
        ended["error"].as_str().unwrap().contains("exited"),
        "{ended}"
    );
    assert_eq!(git_out(&session.root, &["rev-parse", "HEAD"]), before);
    assert!(
        !session.root.join("ghost").exists(),
        "a failed scaffold left a folder behind"
    );
    assert_eq!(git_out(&session.root, &["status", "--porcelain"]), "");
}

#[tokio::test]
async fn the_folders_a_scaffold_may_not_have_are_refused() {
    let session = start().await;
    for output in [".", "  ", "../escape", "/tmp/escape"] {
        let (status, error) = post(&session, "/api/scaffold", spec("Greeter", output)).await;
        assert_eq!(status, 422, "{output:?}: {error}");
        assert!(error["error"].is_string(), "{output:?}: {error}");
    }
    // An occupied folder, before any scaffolder runs.
    std::fs::create_dir_all(session.root.join("taken")).unwrap();
    std::fs::write(session.root.join("taken/x.txt"), "").unwrap();
    let (status, error) = post(&session, "/api/scaffold", spec("Greeter", "taken")).await;
    assert_eq!(status, 422, "{error}");
    assert!(
        error["error"].as_str().unwrap().contains("already exists"),
        "{error}"
    );
}

#[tokio::test]
async fn a_folder_without_a_repository_is_told_so_by_a_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("notes.hick"), "# Notes\n").unwrap();
    let root = dir.path().canonicalize().unwrap();
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
    let session = Session {
        base: format!("http://127.0.0.1:{port}"),
        root,
        _dir: dir,
    };
    let (status, error) = post(&session, "/api/scaffold", spec("Greeter", "greeter")).await;
    assert_eq!(status, 422, "{error}");
    // The dialog keys off the field, never the sentence — and the field says
    // *which* folder a `git init` would be run in, because the button that
    // answers this has to name one.
    assert_eq!(error["missing"], json!("repository"));
    assert_eq!(error["path"], session.root.to_string_lossy().as_ref());
    assert!(!session.root.join("greeter").exists());
}

#[tokio::test]
async fn a_project_is_made_where_the_person_said_and_recorded_by_that_repository() {
    // Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md:
    // the location is any folder on this machine, and the repository that
    // records the recipe is whichever one holds it — never the open one by
    // assumption.
    if !has_dotnet() {
        eprintln!("SKIPPED: no dotnet on this machine");
        return;
    }
    let session = start().await;
    // A second repository, somewhere else entirely, with the app still open
    // on the first.
    let elsewhere = tempfile::tempdir().unwrap();
    let other = elsewhere.path().canonicalize().unwrap();
    for args in [
        vec!["init", "-q"],
        vec![
            "-c",
            "user.email=t@example.com",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "start",
            "--allow-empty",
        ],
    ] {
        assert!(
            std::process::Command::new("git")
                .args(&args)
                .current_dir(&other)
                .status()
                .unwrap()
                .success()
        );
    }
    std::fs::create_dir_all(other.join("apps")).unwrap();

    let mut body = spec("Greeter", "greeter");
    body["location"] = json!(other.join("apps").to_string_lossy());
    let (status, started) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 202, "{started}");
    assert_eq!(started["repository"], other.to_string_lossy().as_ref());
    // `-o` is relative to that repository's root, not to the location: a
    // replay runs the recipe from the root.
    assert_eq!(started["output"], "apps/greeter");
    let term = started["session"]["id"].as_str().unwrap().to_string();
    let created = settle(&session, &term).await;
    assert_eq!(created["state"], "committed", "{created}");

    assert_eq!(
        git_out(&other, &["rev-parse", "HEAD"]),
        created["sha"].as_str().unwrap()
    );
    assert!(
        git_out(&other, &["show", "-s", "--format=%B", "HEAD"])
            .contains("Hick-Recipe: dotnet new console -o apps/greeter"),
        "the recipe is spelled from the repository root"
    );
    assert!(other.join("apps/greeter/Program.cs").is_file());
    // The open folder is untouched: a project made elsewhere is made
    // elsewhere.
    assert_eq!(git_out(&session.root, &["status", "--porcelain"]), "");
}

#[tokio::test]
async fn a_location_in_no_repository_is_offered_one_and_then_works() {
    // Protects docs/guarantees/authoring/a-new-project-is-a-recipe-commit.md:
    // a fixable failure is a button. `POST /api/git/init` is that button.
    let session = start().await;
    let plain = tempfile::tempdir().unwrap();
    let bare = plain.path().canonicalize().unwrap().join("fresh");

    let mut body = spec("Greeter", "greeter");
    body["location"] = json!(bare.to_string_lossy());
    let (status, refused) = post(&session, "/api/scaffold", body.clone()).await;
    assert_eq!(status, 422, "{refused}");
    assert_eq!(refused["missing"], json!("repository"));
    assert_eq!(refused["path"], bare.to_string_lossy().as_ref());

    // The button: it makes the folder as well as the repository, because a
    // location typed for a project need not exist yet.
    let (status, made) = post(
        &session,
        "/api/git/init",
        json!({ "path": bare.to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 200, "{made}");
    assert!(bare.join(".git").exists());

    if !has_dotnet() {
        return;
    }
    let (status, started) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 202, "{started}");
    assert_eq!(started["repository"], bare.to_string_lossy().as_ref());
    let term = started["session"]["id"].as_str().unwrap().to_string();
    let created = settle(&session, &term).await;
    assert_eq!(created["state"], "committed", "{created}");
    assert!(bare.join("greeter/Program.cs").is_file());
}

#[tokio::test]
async fn a_repository_inside_a_repository_is_refused() {
    let session = start().await;
    let (status, refused) = post(
        &session,
        "/api/git/init",
        json!({ "path": session.root.join("inner").to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 422, "{refused}");
    assert!(
        refused["error"]
            .as_str()
            .unwrap()
            .contains("already inside the repository"),
        "{refused}"
    );
}

#[tokio::test]
async fn a_preview_says_when_there_is_no_repository_rather_than_refusing() {
    // The person is still typing. A preview that blanks out teaches nothing;
    // it names the folder a repository would be made in instead.
    let session = start().await;
    let plain = tempfile::tempdir().unwrap();
    let bare = plain.path().canonicalize().unwrap().join("fresh");
    let mut body = spec("Greeter", "greeter");
    body["location"] = json!(bare.to_string_lossy());
    let (status, preview) = post(&session, "/api/scaffold/preview", body).await;
    assert_eq!(status, 200, "{preview}");
    assert_eq!(preview["repository"], Value::Null);
    assert_eq!(preview["needs_repository"], bare.to_string_lossy().as_ref());
    assert!(
        preview["message"]
            .as_str()
            .unwrap()
            .contains("Hick-Recipe: dotnet new console -o greeter"),
        "{preview}"
    );
}

#[tokio::test]
async fn the_catalogue_answers_or_says_there_is_no_sdk() {
    let session = start().await;
    let (status, body) = get(&session, "/api/scaffold/templates").await;

    if !has_dotnet() {
        // The one distinction the dialog draws its own screen from, and it is
        // a field rather than a sentence.
        assert_eq!(status, 422, "{body}");
        assert_eq!(body["missing"], json!("dotnet"));
        return;
    }

    assert_eq!(status, 200, "{body}");
    let templates = body["templates"].as_array().expect("templates");
    assert!(templates.len() > 5, "{body}");
    assert!(
        body["image"]
            .as_str()
            .unwrap()
            .starts_with("mcr.microsoft.com/dotnet/sdk:"),
        "{body}"
    );

    let (status, detail) = get(&session, "/api/scaffold/options?template=console").await;
    assert_eq!(status, 200, "{detail}");
    let flags: Vec<&str> = detail["options"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["flag"].as_str().unwrap())
        .collect();
    assert!(flags.contains(&"--framework"), "{flags:?}");

    // A template nobody has is a message about that template, not a 500.
    let (status, error) = get(&session, "/api/scaffold/options?template=nope-nope").await;
    assert_eq!(status, 422, "{error}");
}
