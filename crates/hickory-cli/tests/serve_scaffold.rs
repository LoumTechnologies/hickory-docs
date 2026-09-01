//! File → New Project, over the real routes.
//!
//! Protects `docs/guarantees/authoring/a-new-project-writes-the-command-it-ran.md`
//! and `docs/guarantees/authoring/a-new-project-that-cannot-run-still-leaves-a-document.md`.
//!
//! The parsing half is covered against checked-in fixtures in
//! `scaffold_templates.rs`. What is left is the half no fixture can show: that
//! the dialog's answer becomes a document on disk, that running it puts the
//! generator's bytes inside that document, and that a scaffold which does not
//! run leaves the document standing anyway.
//!
//! Anything that needs a real `dotnet` says so and skips without one. The
//! machine that has an SDK gets the whole claim tested; the machine that does
//! not still tests everything up to the subprocess, which is where the
//! interesting mistakes are.

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

#[tokio::test]
async fn the_preview_is_the_bytes_that_get_written() {
    let session = start().await;
    let mut body = spec("Greeter", "greeter");
    body["path"] = json!("greeter.hick");
    body["run"] = json!(false);

    let (status, preview) = post(&session, "/api/scaffold/preview", body.clone()).await;
    assert_eq!(status, 200, "{preview}");
    assert_eq!(
        preview["command"].as_str().unwrap(),
        "dotnet new console -o out -n Greeter --language 'C#' --no-restore"
    );

    let (status, created) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 201, "{created}");
    // The whole reason the preview is a route: what it showed and what was
    // written are the same function, so they cannot drift.
    assert_eq!(created["source"], preview["source"]);
    assert_eq!(
        std::fs::read_to_string(session.root.join("greeter.hick")).unwrap(),
        preview["source"].as_str().unwrap()
    );
}

#[tokio::test]
async fn a_new_project_owns_what_dotnet_wrote() {
    if !has_dotnet() {
        eprintln!("no dotnet on this machine — skipping the half that needs one");
        return;
    }
    let session = start().await;
    let mut body = spec("Greeter", "greeter");
    body["path"] = json!("greeter.hick");

    let (status, created) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 201, "{created}");
    assert_eq!(created["note"], Value::Null, "the scaffold ran: {created}");

    let files: Vec<&str> = created["ingested"]["files"]
        .as_array()
        .expect("an ingest happened")
        .iter()
        .map(|f| f.as_str().unwrap())
        .collect();
    assert!(
        files.contains(&"greeter/Program.cs"),
        "the generated program is in the document, under the volume's own \
         output path: {files:?}"
    );
    assert!(files.iter().any(|f| f.ends_with(".csproj")), "{files:?}");

    // The bytes really are in the document — that is the whole claim. Not a
    // reference to a cache, not a patch: the file on disk holds them.
    let source = std::fs::read_to_string(session.root.join("greeter.hick")).unwrap();
    assert!(source.contains("<hick:ingested"), "{source}");
    assert!(
        source.contains(r#"<hick:file path="greeter/Program.cs">"#),
        "{source}"
    );
    assert!(source.contains("Console.WriteLine"), "{source}");
    // And the run that wrote them is recorded on the block.
    assert!(
        source.contains(&format!(
            r#"sha256="{}""#,
            created["ingested"]["fingerprint"].as_str().unwrap()
        )),
        "{source}"
    );

    // The files are on disk when the route answers. An ingested volume is
    // deliberately no longer flushed as a pipeline output, so a weave is the
    // only thing that puts them there — and New Project doing it is what
    // stops the dialog closing on a folder with no project in it.
    assert!(session.root.join("greeter/Program.cs").exists());
    assert!(session.root.join("greeter.md").exists());

    // And a clone rebuilds the tree without running the generator at all.
    std::fs::remove_dir_all(session.root.join("greeter")).unwrap();
    let weave = std::process::Command::new(env!("CARGO_BIN_EXE_hick"))
        .args(["weave", "greeter.hick"])
        .current_dir(&session.root)
        .output()
        .expect("hick weave runs");
    assert!(
        weave.status.success(),
        "{}",
        String::from_utf8_lossy(&weave.stderr)
    );
    assert!(
        std::fs::read_to_string(session.root.join("greeter/Program.cs"))
            .unwrap()
            .contains("Console.WriteLine")
    );
}

#[tokio::test]
async fn a_scaffold_that_cannot_run_still_leaves_a_document() {
    let session = start().await;
    let mut body = spec("Ghost", "ghost");
    // A template no SDK has. The command is well-formed and the document is
    // fine; it is the generator that fails.
    body["template"] = json!("no-such-template-exists");
    body["path"] = json!("ghost.hick");

    let (status, created) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 201, "{created}");
    assert_eq!(created["ingested"], Value::Null);
    assert!(
        created["note"].as_str().is_some_and(|n| !n.is_empty()),
        "the failure is reported rather than swallowed: {created}"
    );

    // The part that worked is still there, and it is a complete document:
    // pressing Run is all that is left to try.
    let source = std::fs::read_to_string(session.root.join("ghost.hick")).unwrap();
    assert!(
        source.contains("dotnet new no-such-template-exists"),
        "{source}"
    );
    assert!(source.contains(r#"<hick:copy id="scaffold">"#), "{source}");
    assert!(!source.contains("<hick:ingested"), "{source}");
}

#[tokio::test]
async fn the_paths_a_document_may_not_have_are_refused() {
    let session = start().await;

    let mut body = spec("Greeter", "greeter");
    body["path"] = json!("greeter.txt");
    body["run"] = json!(false);
    let (status, error) = post(&session, "/api/scaffold", body.clone()).await;
    assert_eq!(status, 400, "{error}");
    assert!(error["error"].as_str().unwrap().contains(".hick"));

    body["path"] = json!("../escape.hick");
    let (status, error) = post(&session, "/api/scaffold", body.clone()).await;
    assert_eq!(status, 400, "{error}");

    // "New" is not a way to lose something.
    body["path"] = json!("notes.hick");
    let (status, error) = post(&session, "/api/scaffold", body.clone()).await;
    assert_eq!(status, 422, "{error}");
    assert_eq!(
        std::fs::read_to_string(session.root.join("notes.hick")).unwrap(),
        "# Notes\n"
    );

    // An empty output would scatter the scaffold across the notes folder.
    body["path"] = json!("greeter.hick");
    body["output"] = json!("  ");
    let (status, error) = post(&session, "/api/scaffold", body).await;
    assert_eq!(status, 400, "{error}");
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
