<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Explicit UI action dispatch checks scope, state, policy and retries

For: Authors using an agent to change documents while retaining review control.

Given a command explicitly registered with `UiActions`, observing its parent
returns a bounded descriptor with subject, schema, current state and an opaque
action handle. A structured invocation through that registry checks workspace,
window, current actor policy, current revision, enabled state and exact typed
arguments before calling its registered semantic handler. Quoted text, source
location handles and forged handles do not execute commands. Observation identity
is distinct from executable identity and does not grant an edit baseline.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/agent/ui-action-dispatch.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

## Test the behavior: `held`

This is the maintained test fragment from `crates/hickory-cli/src/up/state/resolution_tests.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-3b41d39d831f-held"></a>
### `held`

<storycode:copy id="f-3b41d39d831f-held">
use super::*;

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
fn held(root: &Path) -> (WovenState, OutputResolution) {
    let path = root.join("out.txt");
    std::fs::write(&path, "disk side").unwrap();
    let mut state = WovenState::default();
    state.outputs.insert(
        path.clone(),
        OutputState {
            doc: root.join("doc.md"),
            rel_path: "out.txt".into(),
            content: "produced side".into(),
            provenance: vec![],
        },
    );
    state.hold_output(&path, "Cannot carry back".into());
    let request = OutputResolution {
        root: root.to_path_buf(),
        path,
        content: "merged".into(),
        expected_current: "disk side".into(),
        expected_produced: "produced side".into(),
        mode: ResolutionMode::Merge,
    };
    (state, request)
}

</storycode:copy>

## Test the behavior: `expected_root_source_rejects_changed_bytes_before_parsing_or_execution`

This is the maintained test fragment from `crates/hickory-cli/tests/engineer_tour.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-ae34e687590d-expected_root_source_rejects_changed_bytes_before_parsing_or_execution"></a>
### `expected_root_source_rejects_changed_bytes_before_parsing_or_execution`

The downloaded tour is an executable specification, not just browser code.

<storycode:copy id="f-ae34e687590d-expected_root_source_rejects_changed_bytes_before_parsing_or_execution">
//! The downloaded tour is an executable specification, not just browser code.
use hickory_cli::{ExecutorChoice, RunMode, run_doc};

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
#[tokio::test]
async fn expected_root_source_rejects_changed_bytes_before_parsing_or_execution() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("guarded.md");
    let original = "# Captured requirement\n";
    let expected = hick_context_protocol::digest(original.as_bytes());
    std::fs::write(&path, "<hick:exec>this must never execute</hick:exec>\n").unwrap();
    let error = hickory_cli::run_doc_expected(
        &path,
        &[],
        RunMode::Execute,
        ExecutorChoice::Local,
        &expected,
    )
    .await
    .err()
    .expect("Changed source must be refused");
    assert!(
        error
            .to_string()
            .contains("Source revision changed before execution"),
        "{error:#}"
    );
    std::fs::write(&path, original).unwrap();
    let result = hickory_cli::run_doc_expected(
        &path,
        &[],
        RunMode::Verify,
        ExecutorChoice::Local,
        &expected,
    )
    .await
    .unwrap();
    assert_eq!(result.source, original);
    assert_eq!(
        hick_context_protocol::digest(result.source.as_bytes()),
        expected
    );
}
</storycode:copy>

## Test the behavior: `output_resolution_http_waits_for_the_applied_or_refused_loop_reply`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_output_resolution.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-074b9e71856c-output_resolution_http_waits_for_the_applied_or_refused_loop_reply"></a>
### `output_resolution_http_waits_for_the_applied_or_refused_loop_reply`

<storycode:copy id="f-074b9e71856c-output_resolution_http_waits_for_the_applied_or_refused_loop_reply">
//! Guarantee: docs/guarantees/agent/ui-action-dispatch.md
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
    up::{
        UpCommand,
        state::{OutputState, WovenState, set_read_only},
    },
};
use serde_json::{Value, json};

#[tokio::test]
async fn output_resolution_http_waits_for_the_applied_or_refused_loop_reply() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let path = root.join("out.txt");
    let mut woven = WovenState::default();
    woven
        .write_output(
            &root,
            &path,
            OutputState {
                doc: root.join("doc.md"),
                rel_path: "out.txt".into(),
                content: "produced".into(),
                provenance: vec![],
            },
        )
        .unwrap();
    set_read_only(&path, false).unwrap();
    std::fs::write(&path, "disk").unwrap();
    woven.hold_output(&path, "Cannot carry back".into());
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let (sender, mut commands) = tokio::sync::mpsc::unbounded_channel();
    *prepared.state.up_commands.lock().unwrap() = Some(sender);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!(
        "http://{}/api/v1/outputs/resolve",
        listener.local_addr().unwrap()
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let client = reqwest::Client::new();
    for (expected, status) in [("stale disk", 409), ("disk", 200)] {
        let body = json!({ "path": "out.txt", "content": "merged", "expected_current": expected, "expected_produced": "produced" });
        let client = client.clone();
        let endpoint = url.clone();
        let response =
            tokio::spawn(async move { client.post(endpoint).json(&body).send().await.unwrap() });
        let command = tokio::time::timeout(std::time::Duration::from_secs(5), commands.recv())
            .await
            .unwrap()
            .unwrap();
        assert!(
            !response.is_finished(),
            "Queued work must not be acknowledged as applied"
        );
        match command {
            UpCommand::Resolve { request, reply } => {
                assert_eq!(request.path, path);
                let outcome = woven.resolve_output(&request).map_err(|e| e.to_string());
                reply.send(outcome).unwrap();
            }
            _ => panic!("Wrong host command"),
        }
        let response = response.await.unwrap();
        assert_eq!(response.status().as_u16(), status);
        let body: Value = response.json().await.unwrap();
        if status == 409 {
            assert!(body["error"].as_str().unwrap().contains("changed on disk"));
            assert!(woven.held().contains_key(&path));
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "disk");
        } else {
            assert_eq!(
                body,
                json!({ "ok": true, "path": "out.txt", "applied": true, "carried_back": false })
            );
            assert_eq!(std::fs::read_to_string(&path).unwrap(), "merged");
        }
    }
    assert_eq!(client.post(&url).json(&json!({ "path": "../escape", "content": "x", "expected_current": "", "expected_produced": "" })).send().await.unwrap().status(), 400);
    assert!(commands.try_recv().is_err());
    woven.hold_output(&path, "Merged bytes are still different".into());
    let endpoint = url.replace("/resolve", "/regenerate");
    let regeneration = tokio::spawn(async move {
        reqwest::Client::new().post(endpoint).json(&json!({ "path": "out.txt", "expected_current": "merged", "expected_produced": "produced" })).send().await.unwrap()
    });
    let command = tokio::time::timeout(std::time::Duration::from_secs(5), commands.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(!regeneration.is_finished());
    match command {
        UpCommand::Resolve { request, reply } => {
            assert_eq!(
                request.mode,
                hickory_cli::up::state::ResolutionMode::Regenerate
            );
            let outcome = woven.resolve_output(&request).map_err(|e| e.to_string());
            reply.send(outcome).unwrap();
        }
        _ => panic!("Wrong regeneration command"),
    }
    let response = regeneration.await.unwrap();
    assert_eq!(response.status(), 200);
    let body: Value = response.json().await.unwrap();
    assert_eq!(body["applied"], true);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "produced");
    assert!(std::fs::metadata(&path).unwrap().permissions().readonly());
    set_read_only(&path, false).unwrap();
    let windows = url.replace("/outputs/resolve", "/ui/windows");
    assert_eq!(
        client
            .post(&windows.replace("/api/v1/", "/api/"))
            .json(&json!({}))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );
    let response = client.post(&windows).json(&json!({})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let scope: Value = response.json().await.unwrap();
    assert_eq!(scope["window"].as_str().unwrap().len(), 32);
    assert_eq!(scope["workspace"].as_str().unwrap().len(), 64);
    assert_eq!(
        client
            .delete(format!("{windows}/{}", scope["window"].as_str().unwrap()))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
    server.abort();
}
</storycode:copy>

## Test the behavior: `python_agent_uses_originating_window_ui_tools_with_retained_context`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_python_agent.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-08f0cd3133cc-python_agent_uses_originating_window_ui_tools_with_retained_context"></a>
### `python_agent_uses_originating_window_ui_tools_with_retained_context`

Real confined Python + HTTP broker, scripted provider and UI endpoint.
The separate component tests witness the actual shared UI controller.

<storycode:copy id="f-08f0cd3133cc-python_agent_uses_originating_window_ui_tools_with_retained_context">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
// Real confined Python + HTTP broker, scripted provider and UI endpoint.
// The separate component tests witness the actual shared UI controller.
#[tokio::test]
async fn python_agent_uses_originating_window_ui_tools_with_retained_context() {
    ui_transport_case(false, false).await;
}

</storycode:copy>

## Test the behavior: `agent_input_checks_the_current_host_prompt_before_sending_bytes`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_terminals.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-5d8946e53299-agent_input_checks_the_current_host_prompt_before_sending_bytes"></a>
### `agent_input_checks_the_current_host_prompt_before_sending_bytes`

<storycode:copy id="f-5d8946e53299-agent_input_checks_the_current_host_prompt_before_sending_bytes">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn agent_input_checks_the_current_host_prompt_before_sending_bytes() {
    let session = start().await;
    let (status, opened) = post(&session, "/api/terminals", json!({"title":"decision","argv":["/bin/sh","-c","printf 'Proceed? [y/N]'; read response; printf '\\nANSWER:%s\\n' \"$response\""]})).await;
    assert_eq!(status, 200, "{opened}");
    let id = opened["id"].as_str().unwrap();
    read_until(&session, id, "Proceed? [y/N]").await;
    let (status, error) = post(
        &session,
        &format!("/api/terminals/{id}/input"),
        json!({"agent":true,"data":"yes\n"}),
    )
    .await;
    assert_eq!(status, 409, "{error}");
    let (status, error) = post(
        &session,
        &format!("/api/terminals/{id}/input"),
        json!({"agent":true,"data":"x".repeat(16385)}),
    )
    .await;
    assert_eq!(status, 400, "{error}");
    let (status, result) = post(
        &session,
        &format!("/api/terminals/{id}/input"),
        json!({"data":"yes\n"}),
    )
    .await;
    assert_eq!(status, 200, "{result}");
    assert!(
        read_until(&session, id, "ANSWER:yes")
            .await
            .contains("ANSWER:yes")
    );
}

</storycode:copy>

## Test the behavior: `terminal_output_reads_are_bounded_and_revision_bound`

This is the maintained test fragment from `crates/hickory-cli/tests/serve_terminals.rs`; its original guarantee annotation selects this narrative owner. Inspect the assertions below before treating the annotation as coverage.

<a id="f-5d8946e53299-terminal_output_reads_are_bounded_and_revision_bound"></a>
### `terminal_output_reads_are_bounded_and_revision_bound`

<storycode:copy id="f-5d8946e53299-terminal_output_reads_are_bounded_and_revision_bound">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
#[tokio::test(flavor = "multi_thread")]
async fn terminal_output_reads_are_bounded_and_revision_bound() {
    let session = start().await;
    let (status, opened) = post(
        &session,
        "/api/terminals",
        json!({"argv":prints("bounded terminal witness")}),
    )
    .await;
    assert_eq!(status, 200, "{opened}");
    let id = opened["id"].as_str().unwrap();
    let _ = read_until(&session, id, "bounded terminal witness").await;
    let route = format!("/api/terminals/{id}/output?view=screen&offset=0&limit=16384");
    let (status, captured) = get(&session, &route).await;
    assert_eq!(status, 200, "{captured}");
    let bytes: Vec<u8> = serde_json::from_value(captured["bytes"].clone()).unwrap();
    assert!(
        String::from_utf8(bytes)
            .unwrap()
            .contains("bounded terminal witness")
    );
    assert_eq!(captured["span"]["start"], 0);
    let (status, _) = get(&session, &format!("{route}&revision=forged")).await;
    assert_eq!(status, 409);
    let (status, _) = get(
        &session,
        &format!("/api/terminals/{id}/output?view=screen&offset=0&limit=16385"),
    )
    .await;
    assert_eq!(status, 400);
    let offset = captured["total"].as_u64().unwrap() + 1;
    let (status, _) = get(
        &session,
        &format!("/api/terminals/{id}/output?view=screen&offset={offset}&limit=1"),
    )
    .await;
    assert_eq!(status, 400);
}

</storycode:copy>

## Implementation chapters

These chapters keep this behavior's larger implementation readable without duplicating shared code.

- [`apps/web/e2e/uiPicture.spec.ts`](implementation/apps/web/e2e/uiPicture.spec.ts.md#output-bc3d0c24b142)
- [`apps/web/src/components/AcpRecord.test.tsx`](implementation/apps/web/src/components/AcpRecord.test.tsx.md#output-4fad025b1410)
- [`apps/web/src/components/AgentProject.test.tsx`](implementation/apps/web/src/components/AgentProject.test.tsx.md#output-5a918691d171)
- [`apps/web/src/components/AnchoredCoach.test.tsx`](implementation/apps/web/src/components/AnchoredCoach.test.tsx.md#output-1457fcd4dbfb)
- [`apps/web/src/components/CapturedReferences.test.tsx`](implementation/apps/web/src/components/CapturedReferences.test.tsx.md#output-81bc4ab2895e)
- [`apps/web/src/components/GithubReviewThreads.test.tsx`](implementation/apps/web/src/components/GithubReviewThreads.test.tsx.md#output-605f0df8137c)
- [`apps/web/src/components/MathPanel.test.tsx`](implementation/apps/web/src/components/MathPanel.test.tsx.md#output-a7d8fefd10d5)
- [`apps/web/src/components/MeetingImport.test.tsx`](implementation/apps/web/src/components/MeetingImport.test.tsx.md#output-4adb88414524)
- [`apps/web/src/components/MergeView.test.tsx`](implementation/apps/web/src/components/MergeView.test.tsx.md#output-8d7ee66b0e18)
- [`apps/web/src/components/MoreActions.test.tsx`](implementation/apps/web/src/components/MoreActions.test.tsx.md#output-590ab520e87f)
- [`apps/web/src/components/OutputEditorPane.test.tsx`](implementation/apps/web/src/components/OutputEditorPane.test.tsx.md#output-0c1c6071248d)
- [`apps/web/src/components/PlainFileActions.test.tsx`](implementation/apps/web/src/components/PlainFileActions.test.tsx.md#output-ce14f95f9a34)
- [`apps/web/src/components/SnapshotExport.test.tsx`](implementation/apps/web/src/components/SnapshotExport.test.tsx.md#output-b566afcc22ee)
- [`apps/web/src/components/TableGestureActions.test.tsx`](implementation/apps/web/src/components/TableGestureActions.test.tsx.md#output-e3d9d7d798b0)
- [`apps/web/src/components/TooltipLayer.test.tsx`](implementation/apps/web/src/components/TooltipLayer.test.tsx.md#output-4920bdec172e)
- [`apps/web/src/components/UiCheckbox.test.tsx`](implementation/apps/web/src/components/UiCheckbox.test.tsx.md#output-4557771256d8)
- [`apps/web/src/components/UiCheckbox.tsx`](implementation/apps/web/src/components/UiCheckbox.tsx.md#output-01e1dcea4a76)
- [`apps/web/src/components/UiLink.test.tsx`](implementation/apps/web/src/components/UiLink.test.tsx.md#output-9503a258f59f)
- [`apps/web/src/components/lineageUiActions.test.tsx`](implementation/apps/web/src/components/lineageUiActions.test.tsx.md#output-b841ae76cd46)
- [`apps/web/src/components/uiConversationActions.test.tsx`](implementation/apps/web/src/components/uiConversationActions.test.tsx.md#output-f7ffa11bc8cc)
- [`apps/web/src/components/uiDatasetActions.test.tsx`](implementation/apps/web/src/components/uiDatasetActions.test.tsx.md#output-e98fa629eb67)
- [`apps/web/src/components/uiResultLists.test.tsx`](implementation/apps/web/src/components/uiResultLists.test.tsx.md#output-7b4789da89ae)
- [`apps/web/src/components/uiTableActions.test.tsx`](implementation/apps/web/src/components/uiTableActions.test.tsx.md#output-75db5720569d)
- [`apps/web/src/components/useMediaActions.test.tsx`](implementation/apps/web/src/components/useMediaActions.test.tsx.md#output-fb9d2ad5052a)
- [`apps/web/src/debug/ProviderReading.test.tsx`](implementation/apps/web/src/debug/ProviderReading.test.tsx.md#output-9b03d99a7bfc)
- [`apps/web/src/debug/uiDebugActions.test.tsx`](implementation/apps/web/src/debug/uiDebugActions.test.tsx.md#output-4e4cda819949)
- [`apps/web/src/debug/uiRuntimeActions.test.tsx`](implementation/apps/web/src/debug/uiRuntimeActions.test.tsx.md#output-1790b7a5242a)
- [`apps/web/src/editor/codeActionChoices.test.tsx`](implementation/apps/web/src/editor/codeActionChoices.test.tsx.md#output-7d80c08f42fb)
- [`apps/web/src/editor/completionActions.test.tsx`](implementation/apps/web/src/editor/completionActions.test.tsx.md#output-d0dde764b787)
- [`apps/web/src/editor/languageAnnotations.test.tsx`](implementation/apps/web/src/editor/languageAnnotations.test.tsx.md#output-54c8647e6438)
- [`apps/web/src/editor/languageEdits.test.tsx`](implementation/apps/web/src/editor/languageEdits.test.tsx.md#output-241995db076e)
- [`apps/web/src/editor/taskList.controls.test.tsx`](implementation/apps/web/src/editor/taskList.controls.test.tsx.md#output-9f04cfb579bb)
- [`apps/web/src/editor/uiEditorActions.test.tsx`](implementation/apps/web/src/editor/uiEditorActions.test.tsx.md#output-20ef02f76b56)
- [`apps/web/src/editor/useEditorEditing.test.tsx`](implementation/apps/web/src/editor/useEditorEditing.test.tsx.md#output-efe4b52de8b3)
- [`apps/web/src/editor/useEditorLanguageActions.test.tsx`](implementation/apps/web/src/editor/useEditorLanguageActions.test.tsx.md#output-a7aead7d56cd)
- [`apps/web/src/embed/AssetPreviewImage.test.tsx`](implementation/apps/web/src/embed/AssetPreviewImage.test.tsx.md#output-d6a4681e6566)
- [`apps/web/src/embed/BrowserWorkspace.controls.test.tsx`](implementation/apps/web/src/embed/BrowserWorkspace.controls.test.tsx.md#output-4bb8f636db00)
- [`apps/web/src/embed/EmbeddingFixture.test.tsx`](implementation/apps/web/src/embed/EmbeddingFixture.test.tsx.md#output-8cbb75d36cd7)
- [`apps/web/src/embed/StaticUiSurface.test.tsx`](implementation/apps/web/src/embed/StaticUiSurface.test.tsx.md#output-b72f7c25d76c)
- [`apps/web/src/embed/publicationAsset.test.ts`](implementation/apps/web/src/embed/publicationAsset.test.ts.md#output-f80f5c932bce)
- [`apps/web/src/environments/environmentActions.test.tsx`](implementation/apps/web/src/environments/environmentActions.test.tsx.md#output-562f20f75499)
- [`apps/web/src/lib/uiActionConnection.test.tsx`](implementation/apps/web/src/lib/uiActionConnection.test.tsx.md#output-2217191ba72c)
- [`apps/web/src/shell/TreePrompt.controls.test.tsx`](implementation/apps/web/src/shell/TreePrompt.controls.test.tsx.md#output-130fb158c588)
- [`apps/web/src/shell/treeMenu.clipboard.test.ts`](implementation/apps/web/src/shell/treeMenu.clipboard.test.ts.md#output-249d5b90c8f3)
- [`apps/web/src/shell/uiDiredActions.test.tsx`](implementation/apps/web/src/shell/uiDiredActions.test.tsx.md#output-e5212391d228)
- [`apps/web/src/shell/useRibbonActions.test.tsx`](implementation/apps/web/src/shell/useRibbonActions.test.tsx.md#output-2089413b681b)
- [`apps/web/src/shell/workspaceStatusControls.test.tsx`](implementation/apps/web/src/shell/workspaceStatusControls.test.tsx.md#output-79955af4100c)
- [`apps/web/src/terminal/uiTerminalActions.test.tsx`](implementation/apps/web/src/terminal/uiTerminalActions.test.tsx.md#output-91310c95ec4c)
- [`apps/web/src/views/BisectPane.test.tsx`](implementation/apps/web/src/views/BisectPane.test.tsx.md#output-8c59f0b8becf)
- [`apps/web/src/views/DocumentReading.controls.test.tsx`](implementation/apps/web/src/views/DocumentReading.controls.test.tsx.md#output-c7ae8a3155cc)
- [`apps/web/src/views/RepresentationActions.test.tsx`](implementation/apps/web/src/views/RepresentationActions.test.tsx.md#output-92d47e8f9d7f)
- [`apps/web/src/views/RepresentationLibrary.test.tsx`](implementation/apps/web/src/views/RepresentationLibrary.test.tsx.md#output-b21809301432)
- [`apps/web/src/views/RepresentationVersions.test.tsx`](implementation/apps/web/src/views/RepresentationVersions.test.tsx.md#output-6b2ad7fe6164)
- [`apps/web/src/views/WelcomePane.controls.test.tsx`](implementation/apps/web/src/views/WelcomePane.controls.test.tsx.md#output-a6400780f2db)
- [`apps/web/src/views/uiContextProjectionActions.test.tsx`](implementation/apps/web/src/views/uiContextProjectionActions.test.tsx.md#output-75b9b8324e45)
- [`apps/web/src/views/uiDocumentRunActions.test.tsx`](implementation/apps/web/src/views/uiDocumentRunActions.test.tsx.md#output-14d5d529a63c)
- [`apps/web/src/views/uiGitHistoryActions.test.tsx`](implementation/apps/web/src/views/uiGitHistoryActions.test.tsx.md#output-a3c9920d0b05)
- [`crates/hickory-cli/src/serve/ui_actions.rs`](implementation/crates/hickory-cli/src/serve/ui_actions.rs.md#output-f9fb61439562)
- [`crates/hickory-cli/src/up/state/resolution_tests.rs`](implementation/crates/hickory-cli/src/up/state/resolution_tests.rs.md#output-3b41d39d831f)
- [`crates/hickory-cli/tests/serve_output_resolution.rs`](implementation/crates/hickory-cli/tests/serve_output_resolution.rs.md#output-074b9e71856c)
- [`scripts/inventory-ui-actions.mjs`](implementation/scripts/inventory-ui-actions.mjs.md#output-209db9f9ea87)
- [`scripts/ui-reachability.test.mjs`](implementation/scripts/ui-reachability.test.mjs.md#output-fe2b8d41f747)
<storycode:include file="implementation/apps/web/e2e/uiPicture.spec.ts.md" />
<storycode:include file="implementation/apps/web/src/components/AcpRecord.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/AgentProject.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/AnchoredCoach.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/CapturedReferences.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/GithubReviewThreads.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/MathPanel.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/MeetingImport.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/MergeView.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/MoreActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/OutputEditorPane.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/PlainFileActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/SnapshotExport.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/TableGestureActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/TooltipLayer.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/UiCheckbox.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/UiCheckbox.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/UiLink.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/lineageUiActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/uiConversationActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/uiDatasetActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/uiResultLists.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/uiTableActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/components/useMediaActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/ProviderReading.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/uiDebugActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/debug/uiRuntimeActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/codeActionChoices.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/completionActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/languageAnnotations.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/languageEdits.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/taskList.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/uiEditorActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/useEditorEditing.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/editor/useEditorLanguageActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/embed/AssetPreviewImage.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/embed/BrowserWorkspace.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/embed/EmbeddingFixture.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/embed/StaticUiSurface.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/embed/publicationAsset.test.ts.md" />
<storycode:include file="implementation/apps/web/src/environments/environmentActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/lib/uiActionConnection.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/shell/TreePrompt.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/shell/treeMenu.clipboard.test.ts.md" />
<storycode:include file="implementation/apps/web/src/shell/uiDiredActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/shell/useRibbonActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/shell/workspaceStatusControls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/terminal/uiTerminalActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/BisectPane.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/DocumentReading.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/RepresentationActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/RepresentationLibrary.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/RepresentationVersions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/WelcomePane.controls.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/uiContextProjectionActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/uiDocumentRunActions.test.tsx.md" />
<storycode:include file="implementation/apps/web/src/views/uiGitHistoryActions.test.tsx.md" />
<storycode:include file="implementation/crates/hickory-cli/src/serve/ui_actions.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/src/up/state/resolution_tests.rs.md" />
<storycode:include file="implementation/crates/hickory-cli/tests/serve_output_resolution.rs.md" />
<storycode:include file="implementation/scripts/inventory-ui-actions.mjs.md" />
<storycode:include file="implementation/scripts/ui-reachability.test.mjs.md" />

## Implementation review

### Implementation

- [apps/desktop/src-tauri/src/lib.rs](../../../mechanisms/apps/desktop/src-tauri/src/lib.rs.md#output-42caba993099)
- [apps/desktop/src-tauri/src/server.rs](../../../mechanisms/apps/desktop/src-tauri/src/server.rs.md#output-821f8d0a36be)
- [apps/web/src/components/MoreActions.tsx](../../editor-intelligence/actions-belong-to-their-subject/contract.md#output-63699185668b)
- [apps/web/src/components/PrivatePredictionCoach.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-f32ded7fd60b)
- [apps/web/src/components/TooltipLayer.tsx](../../../mechanisms/apps/web/src/components/TooltipLayer.tsx.md#output-7e5de871748f)
- [apps/web/src/components/UiCheckbox.tsx](implementation/apps/web/src/components/UiCheckbox.tsx.md#output-01e1dcea4a76)
- [apps/web/src/editor/CardRail.tsx](../../../mechanisms/apps/web/src/editor/CardRail.tsx.md#output-f6019c4774f8)
- [apps/web/src/environments/EnvironmentPanel.tsx](../../../mechanisms/apps/web/src/environments/EnvironmentPanel.tsx.md#output-a428d5761b36)
- [apps/web/src/environments/useEnvironments.ts](../../../mechanisms/apps/web/src/environments/useEnvironments.ts.md#output-fc03d5c53bd5)
- [apps/web/src/lib/nativeWindowActions.ts](../../../mechanisms/apps/web/src/lib/nativeWindowActions.ts.md#output-14de373a5fd7)
- [apps/web/src/lib/tooltip.ts](../../../mechanisms/apps/web/src/lib/tooltip.ts.md#output-d5b928a2f7eb)
- [apps/web/src/lib/uiActionBridge.tsx](../../../mechanisms/apps/web/src/lib/uiActionBridge.tsx.md#output-5571d9f178fc)
- [apps/web/src/lib/uiActions.ts](../../../mechanisms/apps/web/src/lib/uiActions.ts.md#output-0cc430d9eb03)
- [apps/web/src/lib/uiReading.ts](../../../mechanisms/apps/web/src/lib/uiReading.ts.md#output-fb2fe531fbd6)
- [crates/hickory-cli/examples/engine_client.rs](../../../mechanisms/crates/hickory-cli/examples/engine_client.rs.md#output-2ffdd2bc63e0)
- [crates/hickory-cli/src/engine/client.rs](../../../mechanisms/crates/hickory-cli/src/engine/client.rs.md#output-5c9c7cadce49)
- [crates/hickory-cli/src/engine/proxy.rs](../../../mechanisms/crates/hickory-cli/src/engine/proxy.rs.md#output-b24aba9d1334)
- [crates/hickory-cli/src/serve/mod.rs](../../../mechanisms/crates/hickory-cli/src/serve/mod.rs.md#output-09ea558208cd)
- [crates/hickory-cli/src/serve/shell.rs](../../../mechanisms/crates/hickory-cli/src/serve/shell.rs.md#output-16984801199b)
- [crates/hickory-cli/src/serve/ui_actions.rs](implementation/crates/hickory-cli/src/serve/ui_actions.rs.md#output-f9fb61439562)
- [scripts/inventory-ui-actions.mjs](implementation/scripts/inventory-ui-actions.mjs.md#output-209db9f9ea87)

### Tests and fixtures

- [apps/agent/tests/test_worker.py](../../../mechanisms/apps/agent/tests/test_worker.py.md#output-9f33e52791d8)
- [apps/web/e2e/staticUiParity.spec.ts](../../../mechanisms/apps/web/e2e/staticUiParity.spec.ts.md#output-b95b6714a2a7)
- [apps/web/e2e/uiPicture.spec.ts](implementation/apps/web/e2e/uiPicture.spec.ts.md#output-bc3d0c24b142)
- [apps/web/src/components/AcpControls.test.tsx](../../../mechanisms/apps/web/src/components/AcpControls.test.tsx.md#output-616d79251e2d)
- [apps/web/src/components/AcpRecord.test.tsx](implementation/apps/web/src/components/AcpRecord.test.tsx.md#output-4fad025b1410)
- [apps/web/src/components/AgentChanges.test.tsx](../../../mechanisms/apps/web/src/components/AgentChanges.tsx.md#output-7ab5cdb69360)
- [apps/web/src/components/AgentProject.test.tsx](implementation/apps/web/src/components/AgentProject.test.tsx.md#output-5a918691d171)
- [apps/web/src/components/AnchoredCoach.test.tsx](implementation/apps/web/src/components/AnchoredCoach.test.tsx.md#output-1457fcd4dbfb)
- [apps/web/src/components/CapturedReferences.test.tsx](implementation/apps/web/src/components/CapturedReferences.test.tsx.md#output-81bc4ab2895e)
- [apps/web/src/components/CellPanel.test.tsx](../../../mechanisms/apps/web/src/components/CellPanel.tsx.md#output-85e341faa089)
- [apps/web/src/components/ChatDock.test.tsx](../../../mechanisms/apps/web/src/components/ChatDock.tsx.md#output-e7bd6d898a44)
- [apps/web/src/components/ComparisonEditor.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonEditor.test.tsx.md#output-3e23ac42e4a1)
- [apps/web/src/components/ComparisonTools.test.tsx](../../../mechanisms/apps/web/src/components/ComparisonTools.test.tsx.md#output-d1cfc2ed25eb)
- [apps/web/src/components/DivergedBanner.test.tsx](../../../mechanisms/apps/web/src/components/DivergedBanner.test.tsx.md#output-c3c47782ddb8)
- [apps/web/src/components/FeatureSettings.test.tsx](../../../mechanisms/apps/web/src/components/FeatureSettings.test.tsx.md#output-637fcce1408f)
- [apps/web/src/components/FormulaDebugger.test.tsx](../../../mechanisms/apps/web/src/components/FormulaDebugger.tsx.md#output-1acac206a244)
- [apps/web/src/components/GithubReviewThreads.test.tsx](implementation/apps/web/src/components/GithubReviewThreads.test.tsx.md#output-605f0df8137c)
- [apps/web/src/components/InsertMenu.test.tsx](../../../mechanisms/apps/web/src/components/InsertMenu.tsx.md#output-d0524ef3ebdd)
- [apps/web/src/components/MathPanel.test.tsx](implementation/apps/web/src/components/MathPanel.test.tsx.md#output-a7d8fefd10d5)
- [apps/web/src/components/MeetingImport.test.tsx](implementation/apps/web/src/components/MeetingImport.test.tsx.md#output-4adb88414524)
- [apps/web/src/components/MeetingRecording.test.tsx](../../../mechanisms/apps/web/src/components/MeetingRecording.tsx.md#output-eda62f7ee0d9)
- [apps/web/src/components/MergeDriverNotice.test.tsx](../../../mechanisms/apps/web/src/components/MergeDriverNotice.tsx.md#output-38c5ddfe6997)
- [apps/web/src/components/MergeView.test.tsx](implementation/apps/web/src/components/MergeView.test.tsx.md#output-8d7ee66b0e18)
- [apps/web/src/components/MoreActions.test.tsx](implementation/apps/web/src/components/MoreActions.test.tsx.md#output-590ab520e87f)
- [apps/web/src/components/OutputEditorPane.test.tsx](implementation/apps/web/src/components/OutputEditorPane.test.tsx.md#output-0c1c6071248d)
- [apps/web/src/components/PlainFileActions.test.tsx](implementation/apps/web/src/components/PlainFileActions.test.tsx.md#output-ce14f95f9a34)
- [apps/web/src/components/PlainFilePane.test.tsx](../../../mechanisms/apps/web/src/components/PlainFilePane.tsx.md#output-d923b2fdae48)
- [apps/web/src/components/PrivatePredictionCoach.test.tsx](../../../mechanisms/apps/web/src/components/PrivatePredictionCoach.tsx.md#output-4b3612758c98)
- [apps/web/src/components/PromptPanel.test.tsx](../../../mechanisms/apps/web/src/components/PromptPanel.test.tsx.md#output-6e5d4425a8d2)
- [apps/web/src/components/ReviewPanel.test.tsx](../../../mechanisms/apps/web/src/components/ReviewPanel.tsx.md#output-d6bd48d3bc5c)
- [apps/web/src/components/SnapshotExport.test.tsx](implementation/apps/web/src/components/SnapshotExport.test.tsx.md#output-b566afcc22ee)
- [apps/web/src/components/TableGestureActions.test.tsx](implementation/apps/web/src/components/TableGestureActions.test.tsx.md#output-e3d9d7d798b0)
- [apps/web/src/components/TooltipLayer.test.tsx](implementation/apps/web/src/components/TooltipLayer.test.tsx.md#output-4920bdec172e)
- [apps/web/src/components/UiCheckbox.test.tsx](implementation/apps/web/src/components/UiCheckbox.test.tsx.md#output-4557771256d8)
- [apps/web/src/components/UiLink.test.tsx](implementation/apps/web/src/components/UiLink.test.tsx.md#output-9503a258f59f)
- [apps/web/src/components/lineageUiActions.test.tsx](implementation/apps/web/src/components/lineageUiActions.test.tsx.md#output-b841ae76cd46)
- [apps/web/src/components/uiConversationActions.test.tsx](implementation/apps/web/src/components/uiConversationActions.test.tsx.md#output-f7ffa11bc8cc)
- [apps/web/src/components/uiDatasetActions.test.tsx](implementation/apps/web/src/components/uiDatasetActions.test.tsx.md#output-e98fa629eb67)
- [apps/web/src/components/uiResultLists.test.tsx](implementation/apps/web/src/components/uiResultLists.test.tsx.md#output-7b4789da89ae)
- [apps/web/src/components/uiTableActions.test.tsx](implementation/apps/web/src/components/uiTableActions.test.tsx.md#output-75db5720569d)
- [apps/web/src/components/useMediaActions.test.tsx](implementation/apps/web/src/components/useMediaActions.test.tsx.md#output-fb9d2ad5052a)
- [apps/web/src/debug/DebugStrip.test.tsx](../../../mechanisms/apps/web/src/debug/DebugStrip.tsx.md#output-1063c352c210)
- [apps/web/src/debug/ProviderReading.test.tsx](implementation/apps/web/src/debug/ProviderReading.test.tsx.md#output-9b03d99a7bfc)
- [apps/web/src/debug/projectDebugSurface.test.tsx](../../../mechanisms/apps/web/src/debug/projectDebugSurface.test.tsx.md#output-777a3021cd65)
- [apps/web/src/debug/runtimeHistory.test.tsx](../../../mechanisms/apps/web/src/debug/runtimeHistory.ts.md#output-b01fe4e29028)
- [apps/web/src/debug/uiDebugActions.test.tsx](implementation/apps/web/src/debug/uiDebugActions.test.tsx.md#output-4e4cda819949)
- [apps/web/src/debug/uiRuntimeActions.test.tsx](implementation/apps/web/src/debug/uiRuntimeActions.test.tsx.md#output-1790b7a5242a)
- [apps/web/src/editor/DocumentEditor.test.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.test.tsx.md#output-71bbcfe029bb)
- [apps/web/src/editor/codeActionChoices.test.tsx](implementation/apps/web/src/editor/codeActionChoices.test.tsx.md#output-7d80c08f42fb)
- [apps/web/src/editor/completionActions.test.tsx](implementation/apps/web/src/editor/completionActions.test.tsx.md#output-d0dde764b787)
- [apps/web/src/editor/languageAnnotations.test.tsx](implementation/apps/web/src/editor/languageAnnotations.test.tsx.md#output-54c8647e6438)
- [apps/web/src/editor/languageEdits.test.tsx](implementation/apps/web/src/editor/languageEdits.test.tsx.md#output-241995db076e)
- [apps/web/src/editor/mdLinks.test.ts](../../../mechanisms/apps/web/src/editor/mdLinks.ts.md#output-419d07f37b80)
- [apps/web/src/editor/taskList.controls.test.tsx](implementation/apps/web/src/editor/taskList.controls.test.tsx.md#output-9f04cfb579bb)
- [apps/web/src/editor/uiEditorActions.test.tsx](implementation/apps/web/src/editor/uiEditorActions.test.tsx.md#output-20ef02f76b56)
- [apps/web/src/editor/useEditorEditing.test.tsx](implementation/apps/web/src/editor/useEditorEditing.test.tsx.md#output-efe4b52de8b3)
- [apps/web/src/editor/useEditorLanguageActions.test.tsx](implementation/apps/web/src/editor/useEditorLanguageActions.test.tsx.md#output-a7aead7d56cd)
- [apps/web/src/embed/AssetPreviewImage.test.tsx](implementation/apps/web/src/embed/AssetPreviewImage.test.tsx.md#output-d6a4681e6566)
- [apps/web/src/embed/BrowserWorkspace.controls.test.tsx](implementation/apps/web/src/embed/BrowserWorkspace.controls.test.tsx.md#output-4bb8f636db00)
- [apps/web/src/embed/EmbeddingFixture.test.tsx](implementation/apps/web/src/embed/EmbeddingFixture.test.tsx.md#output-8cbb75d36cd7)
- [apps/web/src/embed/StaticUiSurface.test.tsx](implementation/apps/web/src/embed/StaticUiSurface.test.tsx.md#output-b72f7c25d76c)
- [apps/web/src/embed/publicationAsset.test.ts](implementation/apps/web/src/embed/publicationAsset.test.ts.md#output-f80f5c932bce)
- [apps/web/src/environments/environmentActions.test.tsx](implementation/apps/web/src/environments/environmentActions.test.tsx.md#output-562f20f75499)
- [apps/web/src/lib/nativeWindowActions.test.ts](../../../mechanisms/apps/web/src/lib/nativeWindowActions.ts.md#output-fa3030dd95b0)
- [apps/web/src/lib/uiActionBridge.test.tsx](../../../mechanisms/apps/web/src/lib/uiActionBridge.tsx.md#output-2cc5832c66b7)
- [apps/web/src/lib/uiActionConnection.test.tsx](implementation/apps/web/src/lib/uiActionConnection.test.tsx.md#output-2217191ba72c)
- [apps/web/src/lib/uiActions.test.ts](../../../mechanisms/apps/web/src/lib/uiActions.ts.md#output-f5b85cef7913)
- [apps/web/src/shell/CommandBar.test.tsx](../../../mechanisms/apps/web/src/shell/CommandBar.test.tsx.md#output-f9af94fac1a9)
- [apps/web/src/shell/GeneratedBufferRecovery.test.tsx](../../../mechanisms/apps/web/src/shell/GeneratedBufferRecovery.tsx.md#output-9cef161ebf1f)
- [apps/web/src/shell/TreePrompt.controls.test.tsx](implementation/apps/web/src/shell/TreePrompt.controls.test.tsx.md#output-130fb158c588)
- [apps/web/src/shell/treeMenu.clipboard.test.ts](implementation/apps/web/src/shell/treeMenu.clipboard.test.ts.md#output-249d5b90c8f3)
- [apps/web/src/shell/uiDiredActions.test.tsx](implementation/apps/web/src/shell/uiDiredActions.test.tsx.md#output-e5212391d228)
- [apps/web/src/shell/useRibbonActions.test.tsx](implementation/apps/web/src/shell/useRibbonActions.test.tsx.md#output-2089413b681b)
- [apps/web/src/shell/workspaceStatusControls.test.tsx](implementation/apps/web/src/shell/workspaceStatusControls.test.tsx.md#output-79955af4100c)
- [apps/web/src/terminal/uiTerminalActions.test.tsx](implementation/apps/web/src/terminal/uiTerminalActions.test.tsx.md#output-91310c95ec4c)
- [apps/web/src/views/BisectPane.test.tsx](implementation/apps/web/src/views/BisectPane.test.tsx.md#output-8c59f0b8becf)
- [apps/web/src/views/CommandPathSettings.test.tsx](../../../mechanisms/apps/web/src/views/CommandPathSettings.tsx.md#output-7f450949f2a8)
- [apps/web/src/views/DocumentReading.controls.test.tsx](implementation/apps/web/src/views/DocumentReading.controls.test.tsx.md#output-c7ae8a3155cc)
- [apps/web/src/views/GitPane.test.tsx](../../../mechanisms/apps/web/src/views/GitPane.tsx.md#output-0e5613784fdf)
- [apps/web/src/views/HistoryLens.test.tsx](../../../mechanisms/apps/web/src/views/HistoryLens.tsx.md#output-8206e3ef4d86)
- [apps/web/src/views/RepresentationActions.test.tsx](implementation/apps/web/src/views/RepresentationActions.test.tsx.md#output-92d47e8f9d7f)
- [apps/web/src/views/RepresentationLibrary.test.tsx](implementation/apps/web/src/views/RepresentationLibrary.test.tsx.md#output-b21809301432)
- [apps/web/src/views/RepresentationPane.test.tsx](../../../mechanisms/apps/web/src/views/RepresentationPane.tsx.md#output-17926fbc73d7)
- [apps/web/src/views/RepresentationVersions.test.tsx](implementation/apps/web/src/views/RepresentationVersions.test.tsx.md#output-6b2ad7fe6164)
- [apps/web/src/views/SettingsView.test.tsx](../../../mechanisms/apps/web/src/views/SettingsView.tsx.md#output-5bfac230ae5d)
- [apps/web/src/views/WelcomePane.controls.test.tsx](implementation/apps/web/src/views/WelcomePane.controls.test.tsx.md#output-a6400780f2db)
- [apps/web/src/views/uiContextProjectionActions.test.tsx](implementation/apps/web/src/views/uiContextProjectionActions.test.tsx.md#output-75b9b8324e45)
- [apps/web/src/views/uiDocumentRunActions.test.tsx](implementation/apps/web/src/views/uiDocumentRunActions.test.tsx.md#output-14d5d529a63c)
- [apps/web/src/views/uiGitHistoryActions.test.tsx](implementation/apps/web/src/views/uiGitHistoryActions.test.tsx.md#output-a3c9920d0b05)
- [apps/web/src/views/uiWorkspaceMenuActions.test.tsx](../../../mechanisms/apps/web/src/views/uiWorkspaceMenuActions.ts.md#output-34e80d20ccfb)
- [apps/web/src/views/workspaceMenuActions.test.ts](../../../mechanisms/apps/web/src/views/workspaceMenuActions.ts.md#output-c4f5a89c7b3e)
- [crates/hickory-cli/src/engine/client_prediction_tests.rs](../../../mechanisms/crates/hickory-cli/src/engine/client_prediction_tests.rs.md#output-014c7e3acc59)
- [crates/hickory-cli/src/up/state/resolution_tests.rs](implementation/crates/hickory-cli/src/up/state/resolution_tests.rs.md#output-3b41d39d831f)
- [crates/hickory-cli/tests/engineer_tour.rs](../../../mechanisms/crates/hickory-cli/tests/engineer_tour.rs.md#output-ae34e687590d)
- [crates/hickory-cli/tests/serve_output_resolution.rs](implementation/crates/hickory-cli/tests/serve_output_resolution.rs.md#output-074b9e71856c)
- [crates/hickory-cli/tests/serve_python_agent.rs](../../../mechanisms/crates/hickory-cli/tests/serve_python_agent.rs.md#output-08f0cd3133cc)
- [crates/hickory-cli/tests/serve_terminals.rs](../../../mechanisms/crates/hickory-cli/tests/serve_terminals.rs.md#output-5d8946e53299)
- [scripts/ui-reachability.test.mjs](implementation/scripts/ui-reachability.test.mjs.md#output-fe2b8d41f747)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
