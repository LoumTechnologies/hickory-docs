<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Select model context without saving open buffers

For: Authors using an agent to change documents while retaining review control.

Keep unsaved text open but leave it out of Next context projection. Its edit identity may be captured, but opening the tab alone must not send its full text to the model.

## What must happen

Open buffers can supply exact edit snapshots without becoming model-visible text. The current workspace projection explicitly selects expanded or pinned text for the next capture. Unselected open files contribute identity and edit snapshots, not their complete contents. Captured unsaved bytes remain unsaved. Named buffers retain file identity; identifiable untitled buffers use editor-only addresses rather than invented paths. Historical receipts describe what was actually delivered and do not certify today's source or claim support.

## Limits of the evidence

Opening an editor is not evidence of provider delivery. Opaque ACP filesystem tools retain their independent access. The historical HTTP witnesses do not establish the current Python projection.

Current delivery authority: [selected reading projection](../expanded-reading-projection/contract.md).


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/the-agent-sees-the-open-editors.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: The October 18 selected-reading requirement supersedes the earlier send-all-open-editors contract. Keep legacy delivery behavior as historical evidence and make expanded-reading-projection the current delivery authority.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.

## Implementation

## Test the behavior: `a_blank_window_has_editor_apis_but_no_open_folder`

This is the maintained test fragment from `apps/desktop/src-tauri/tests/serves_one_origin.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-e7ec95ab75f6-a_blank_window_has_editor_apis_but_no_open_folder"></a>
### `a_blank_window_has_editor_apis_but_no_open_folder`

File → New Window supports editor sessions without selecting a folder.

<storycode:copy id="f-e7ec95ab75f6-a_blank_window_has_editor_apis_but_no_open_folder">
/// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
/// File → New Window supports editor sessions without selecting a folder.
#[tokio::test(flavor = "multi_thread")]
async fn a_blank_window_has_editor_apis_but_no_open_folder() {
    // Ordinary startup ignores remembered folders, even when they are git repos.
    // Guarantee: docs/guarantees/authoring/the-app-starts-as-a-lightweight-editor.md
    let config = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for args in [
        vec!["init", "-q"],
        vec!["config", "merge.hick.driver", "unused-driver"],
    ] {
        assert!(
            std::process::Command::new("git")
                .arg("-C")
                .arg(project.path())
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    hickory_desktop_lib::server::remember(config.path(), project.path());
    let session = hickory_desktop_lib::server::start_blank_with_config(Some(config.path()), None)
        .await
        .expect("the blank page starts");
    let http = reqwest::Client::new();

    let page = http.get(&session.url).send().await.expect("GET /");
    assert!(page.status().is_success());
    assert!(
        page.text()
            .await
            .expect("page body")
            .contains("<div id=\"root\">")
    );

    let projects = http
        .get(format!("{}/api/projects", session.url))
        .send()
        .await
        .expect("GET /api/projects");
    assert!(projects.status().is_success());
    let files: serde_json::Value = http
        .get(format!("{}/api/files", session.url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(files["folder_open"], false);
    assert_ne!(
        files["root_path"],
        project.path().to_string_lossy().as_ref()
    );
    let merge: serde_json::Value = http
        .get(format!("{}/api/git/merge-driver", session.url))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(merge["ok"], true);
    assert_eq!(merge["status"]["repository"], false);
    let turns = http
        .get(format!("{}/api/docs/workspace/agent/turns", session.url))
        .send()
        .await
        .unwrap();
    assert!(turns.status().is_success());
}

</storycode:copy>

## Test the behavior: `acp_connects_to_workspace_with_untitled_context_and_resumes`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_acp.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-2fce65a349e6-acp_connects_to_workspace_with_untitled_context_and_resumes"></a>
### `acp_connects_to_workspace_with_untitled_context_and_resumes`

<storycode:copy id="f-2fce65a349e6-acp_connects_to_workspace_with_untitled_context_and_resumes">
// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
#[tokio::test(flavor = "multi_thread")]
async fn acp_connects_to_workspace_with_untitled_context_and_resumes() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    let ready = app.connect(None).await;
    assert_eq!(ready["ready"], true, "{ready}");
    let (status, response) = app.request("POST", &app.route(""), json!({
        "backend":"fixture", "prompt":"Discuss this draft", "context":{"buffers":[
            {"name":"Untitled 1", "path":null, "content":"Unsaved <hick:exec> draft", "focused":true}
        ]}
    })).await;
    assert_eq!(status, 202, "{response}");
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    let session = turn["session"].as_str().unwrap().to_string();
    let source = std::fs::read_to_string(app.root.join(&session)).unwrap();
    assert!(source.contains("editor-buffers"), "{source}");
    assert!(source.contains("Unsaved"));
    assert!(!app.root.join(".hick-workspace-agent").exists());
    assert!(!app.root.join("Untitled 1").exists());
    let view = hickory_agent::session_view::session_view(&source);
    assert_eq!(view.turns[0].prompt, "Discuss this draft");
    let root = app.root.clone();
    drop(app);
    let mut restarted = start(&root).await;
    restarted.doc = "workspace".into();
    let ready = restarted.connect(Some(&session)).await;
    assert_eq!(ready["ready"], true, "{ready}");
}

</storycode:copy>

## Implementation review

### Implementation

- [apps/web/src/lib/contextProjection.ts](../../../mechanisms/apps/web/src/lib/contextProjection.ts.md#output-5194716bada7)
- [crates/hick-literate/src/session_receipts.rs](../../../mechanisms/crates/hick-literate/src/session_receipts.rs.md#output-4e6c982edf60)
- [crates/hickory-agent/src/tools/read_file.rs](../../../mechanisms/crates/hickory-agent/src/tools/read_file.rs.md#output-7845fffcfd2e)

### Tests and fixtures

- [apps/desktop/src-tauri/tests/serves_one_origin.rs](../../../mechanisms/apps/desktop/src-tauri/tests/serves_one_origin.rs.md#output-e7ec95ab75f6)
- [apps/web/src/lib/contextProjection.test.ts](../expanded-reading-projection/contract.md#output-cd54a8e364bd)
- [apps/web/src/views/WorkspaceChat.test.tsx](../../../mechanisms/apps/web/src/views/WorkspaceChat.test.tsx.md#output-41ee32c19327)
- [crates/hickory-cli/tests/serve_acp.rs](../../../mechanisms/crates/hickory-cli/tests/serve_acp.rs.md#output-2fce65a349e6)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.
- Open evidence gaps: no-existing-code-or-test-reference-resolved.

</storycode:doc>
