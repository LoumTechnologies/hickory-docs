<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/desktop/src-tauri/tests/serves_one_origin.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-e7ec95ab75f6"></a>
## `apps/desktop/src-tauri/tests/serves_one_origin.rs`

Wrap the shared engine and web interface in the native desktop host; keep host lifecycle behavior at this boundary.

Related contracts: [Select model context without saving open buffers](../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md), [A Desktop Build Carries The UI](../../../../../guarantees/authoring/a-desktop-build-carries-the-ui/contract.md), [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [One loop owns a directory and never reacts to itself](../../../../../guarantees/authoring/one-loop-owns-a-directory/contract.md), [The Desktop App And The CLI Are One Engine](../../../../../guarantees/authoring/the-app-and-the-cli-are-one-engine/contract.md), [The App Starts As A Lightweight Editor](../../../../../guarantees/authoring/the-app-starts-as-a-lightweight-editor/contract.md).

<a id="f-e7ec95ab75f6-DOC"></a>
### `DOC`

The desktop app serves its UI and its API from the same origin.
This is the property the whole wiring exists for. The frontend uses
relative `fetch` paths and derives its WebSocket URL from `location.host`,
so if the page and the API were ever on different origins, every request
the editor makes would fail — and it would fail only in the packaged app,
where it is hardest to notice.

<storycode:copy id="f-e7ec95ab75f6-DOC">
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

</storycode:copy>
<a id="f-e7ec95ab75f6-start_in"></a>
### `start_in`

<storycode:copy id="f-e7ec95ab75f6-start_in">
async fn start_in(dir: &Path) -> (hickory_desktop_lib::server::Session, reqwest::Client) {
    std::fs::write(dir.join("demo.md"), DOC).expect("write doc");
    let session = hickory_desktop_lib::server::start(dir, None, None)
        .await
        .expect("the engine starts");
    (session, reqwest::Client::new())
}

</storycode:copy>
<a id="f-e7ec95ab75f6-the_ui_and_the_api_answer_on_one_origin"></a>
### `the_ui_and_the_api_answer_on_one_origin`

<storycode:copy id="f-e7ec95ab75f6-the_ui_and_the_api_answer_on_one_origin">
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

</storycode:copy>
<a id="f-e7ec95ab75f6-an_unknown_path_returns_the_shell"></a>
### `an_unknown_path_returns_the_shell`

An unknown path returns the shell, not a 404: client-side routing depends
on it, and a deep link that 404s is a broken app rather than a broken URL.

<storycode:copy id="f-e7ec95ab75f6-an_unknown_path_returns_the_shell">
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

</storycode:copy>
[Read the test `a_blank_window_has_editor_apis_but_no_open_folder` beside its requirement](../../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md#f-e7ec95ab75f6-a_blank_window_has_editor_apis_but_no_open_folder).
[Read the test `the_page_serves_its_built_scripts_and_styles` beside its requirement](../../../../../guarantees/authoring/a-desktop-build-carries-the-ui/contract.md#f-e7ec95ab75f6-the_page_serves_its_built_scripts_and_styles).
[Read the test `a_second_session_on_the_same_folder_attaches` beside its requirement](../../../../../guarantees/authoring/one-loop-owns-a-directory/contract.md#f-e7ec95ab75f6-a_second_session_on_the_same_folder_attaches).
<a id="f-e7ec95ab75f6-a_remembered_folder_is_reopened"></a>
### `a_remembered_folder_is_reopened`

Which folder gets opened
A remembered folder survives a restart, so the app reopens where the user
left off instead of asking every launch.

<storycode:copy id="f-e7ec95ab75f6-a_remembered_folder_is_reopened">
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

</storycode:copy>
<a id="f-e7ec95ab75f6-a_remembered_folder_that_is_gone_is_forgotten"></a>
### `a_remembered_folder_that_is_gone_is_forgotten`

A remembered folder that has since been moved, renamed, or unmounted must
not be reopened. The user gets the picker, not an error about a folder they
have forgotten they ever opened.

<storycode:copy id="f-e7ec95ab75f6-a_remembered_folder_that_is_gone_is_forgotten">
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

</storycode:copy>
<a id="f-e7ec95ab75f6-remembering_into_an_unwritable_place_does_not_panic"></a>
### `remembering_into_an_unwritable_place_does_not_panic`

Remembering must never be able to fail a launch: an unwritable config
directory costs one trip through the picker next time, nothing more.

<storycode:copy id="f-e7ec95ab75f6-remembering_into_an_unwritable_place_does_not_panic">
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

</storycode:copy>
[Read the test `folder_layout_follows_the_first_folder_attached_to_a_blank_window` beside its requirement](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md#f-e7ec95ab75f6-folder_layout_follows_the_first_folder_attached_to_a_blank_window).

### Assemble `apps/desktop/src-tauri/tests/serves_one_origin.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/desktop/src-tauri/tests/serves_one_origin.rs"><storycode:paste select="#f-e7ec95ab75f6-DOC"/><storycode:paste select="#f-e7ec95ab75f6-start_in"/><storycode:paste select="#f-e7ec95ab75f6-the_ui_and_the_api_answer_on_one_origin"/><storycode:paste select="#f-e7ec95ab75f6-an_unknown_path_returns_the_shell"/><storycode:paste select="#f-e7ec95ab75f6-a_blank_window_has_editor_apis_but_no_open_folder"/><storycode:paste select="#f-e7ec95ab75f6-the_page_serves_its_built_scripts_and_styles"/><storycode:paste select="#f-e7ec95ab75f6-a_second_session_on_the_same_folder_attaches"/><storycode:paste select="#f-e7ec95ab75f6-a_remembered_folder_is_reopened"/><storycode:paste select="#f-e7ec95ab75f6-a_remembered_folder_that_is_gone_is_forgotten"/><storycode:paste select="#f-e7ec95ab75f6-remembering_into_an_unwritable_place_does_not_panic"/><storycode:paste select="#f-e7ec95ab75f6-folder_layout_follows_the_first_folder_attached_to_a_blank_window"/></storycode:file>

</storycode:doc>
