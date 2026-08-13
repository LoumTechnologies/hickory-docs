//! The desktop app serves its UI and its API from the same origin.
//!
//! Guarantee: `docs/guarantees/authoring/the-app-and-the-cli-are-one-engine.md`
//!
//! This is the property the whole wiring exists for. The frontend uses
//! relative `fetch` paths and derives its WebSocket URL from `location.host`,
//! so if the page and the API were ever on different origins, every request
//! the editor makes would fail — and it would fail only in the packaged app,
//! where it is hardest to notice.

use std::path::Path;

const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="demo.md">
# Demo

<hick:file path="greet.py">
print("hello")
</hick:file>
</hick:doc>
"#;

async fn start_in(dir: &Path) -> (hickory_desktop_lib::server::Session, reqwest::Client) {
    std::fs::write(dir.join("demo.hick"), DOC).expect("write doc");
    let session = hickory_desktop_lib::server::start(dir)
        .await
        .expect("the engine starts");
    (session, reqwest::Client::new())
}

#[tokio::test(flavor = "multi_thread")]
async fn the_ui_and_the_api_answer_on_one_origin() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (session, http) = start_in(dir.path()).await;

    // The page.
    let page = http.get(&session.url).send().await.expect("GET /");
    assert!(page.status().is_success(), "status {}", page.status());
    let body = page.text().await.expect("body");
    assert!(
        body.contains("<div id=\"root\">"),
        "the root element should be served: {}",
        &body[..body.len().min(200)]
    );

    // The API, same origin, no token and no CORS preflight in sight.
    let health = http
        .get(format!("{}/api/health", session.url))
        .send()
        .await
        .expect("GET /api/health");
    assert!(health.status().is_success(), "status {}", health.status());

    // And the document the folder actually contains.
    let projects = http
        .get(format!("{}/api/projects", session.url))
        .send()
        .await
        .expect("GET /api/projects");
    assert!(projects.status().is_success());
}

/// An unknown path returns the shell, not a 404: client-side routing depends
/// on it, and a deep link that 404s is a broken app rather than a broken URL.
#[tokio::test(flavor = "multi_thread")]
async fn an_unknown_path_returns_the_shell() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (session, http) = start_in(dir.path()).await;

    let resp = http
        .get(format!("{}/docs/anything", session.url))
        .send()
        .await
        .expect("GET a client route");
    assert!(resp.status().is_success());
    assert!(
        resp.text()
            .await
            .expect("body")
            .contains("<div id=\"root\">")
    );
}

/// The app takes the same directory lock `hick up` takes, so one folder is
/// never open in two writers at once.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_session_on_the_same_folder_is_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_session, _http) = start_in(dir.path()).await;

    let second = hickory_desktop_lib::server::start(dir.path()).await;
    let err = second.err().expect("the second session must be refused");
    assert!(
        err.to_string().contains("already open"),
        "the refusal should say why: {err}"
    );
}

// ---------------------------------------------------------------------------
// Which folder gets opened
// ---------------------------------------------------------------------------

/// A remembered folder survives a restart, so the app reopens where the user
/// left off instead of asking every launch.
#[test]
fn a_remembered_folder_is_reopened() {
    let config = tempfile::tempdir().expect("config dir");
    let project = tempfile::tempdir().expect("project dir");

    assert!(
        hickory_desktop_lib::server::last_opened(config.path()).is_none(),
        "a fresh install remembers nothing"
    );

    hickory_desktop_lib::server::remember(config.path(), project.path());
    assert_eq!(
        hickory_desktop_lib::server::last_opened(config.path()).as_deref(),
        Some(project.path()),
    );
}

/// A remembered folder that has since been moved, renamed, or unmounted must
/// not be reopened. The user gets the picker, not an error about a folder they
/// have forgotten they ever opened.
#[test]
fn a_remembered_folder_that_is_gone_is_forgotten() {
    let config = tempfile::tempdir().expect("config dir");
    let project = tempfile::tempdir().expect("project dir");
    let path = project.path().to_path_buf();

    hickory_desktop_lib::server::remember(config.path(), &path);
    drop(project);

    assert!(
        hickory_desktop_lib::server::last_opened(config.path()).is_none(),
        "a path that no longer exists is not a folder to reopen"
    );
}

/// Remembering must never be able to fail a launch: an unwritable config
/// directory costs one trip through the picker next time, nothing more.
#[test]
fn remembering_into_an_unwritable_place_does_not_panic() {
    let project = tempfile::tempdir().expect("project dir");
    // A path under a regular file can never be created as a directory.
    let file = tempfile::NamedTempFile::new().expect("temp file");
    let impossible = file.path().join("config");

    hickory_desktop_lib::server::remember(&impossible, project.path());
    assert!(hickory_desktop_lib::server::last_opened(&impossible).is_none());
}
