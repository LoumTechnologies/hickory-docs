<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/literate_agent_projects.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c0298ec2aa35"></a>
## `crates/hickory-cli/tests/literate_agent_projects.rs`

Related contracts: [Finish an agent turn against its captured project boundary](../../../../guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md), [A final changeset revision applies B to C](../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md), [Check the agent's transition separately from maintained-source reconstruction](../../../../guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities/contract.md), [Keep language tools and debug launches tied to their captured project](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md).

<a id="f-c0298ec2aa35-Host"></a>
### `Host`

<storycode:copy id="f-c0298ec2aa35-Host">
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

struct Host {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
    base: String,
    doc: String,
}
</storycode:copy>
<a id="f-c0298ec2aa35-host"></a>
### `host`

<storycode:copy id="f-c0298ec2aa35-host">
async fn host() -> Host {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(
        root.join("project.md"),
        "<hick:project/><hick:file path=\"app.ts\">declared\n</hick:file>",
    )
    .unwrap();
    std::fs::write(root.join("app.ts"), "dirty A\n").unwrap();
    let prepared = prepare(ServeOptions {
        target: root.clone(),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join(".hick-cache/ui.json")),
    })
    .await
    .unwrap();
    prepared.state.agent.set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([
        "<hick:next>done</hick:next>\nKept the current program.",
        "{\"project.md\":\"Current program.\\n<hick:project/><hick:file path=\\\"app.ts\\\">dirty A\\n</hick:file>\"}",
    ])));
    let doc = prepared.state.index.id_for_path("project.md");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    Host {
        _dir: dir,
        root,
        base,
        doc,
    }
}
</storycode:copy>
<a id="f-c0298ec2aa35-request"></a>
### `request`

<storycode:copy id="f-c0298ec2aa35-request">
async fn request(host: &Host, path: &str, body: Option<Value>) -> (u16, Value) {
    let client = reqwest::Client::new();
    let request = if let Some(body) = body {
        client.post(format!("{}{path}", host.base)).json(&body)
    } else {
        client.get(format!("{}{path}", host.base))
    };
    let response = request.send().await.unwrap();
    (response.status().as_u16(), response.json().await.unwrap())
}
</storycode:copy>
<a id="f-c0298ec2aa35-until"></a>
### `until`

<storycode:copy id="f-c0298ec2aa35-until">
async fn until(host: &Host, path: &str, check: impl Fn(&Value) -> bool) -> Value {
    tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let (status, value) = request(host, path, None).await;
            assert_eq!(status, 200, "{value}");
            if check(&value) {
                return value;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("flow did not finish")
}
</storycode:copy>
<a id="f-c0298ec2aa35-finalized"></a>
### `finalized`

<storycode:copy id="f-c0298ec2aa35-finalized">
async fn finalized(host: &Host) -> String {
    let (status, value) = request(
        host,
        &format!("/api/docs/{}/agent", host.doc),
        Some(json!({"prompt":"Keep current code","backend":"builtin"})),
    )
    .await;
    assert_eq!(status, 202, "{value}");
    let turn = value["session_id"].as_str().unwrap();
    let turns = until(host, &format!("/api/docs/{}/agent/turns", host.doc), |v| {
        v["turns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"] == turn && t["status"] != "running")
    })
    .await;
    assert_eq!(turns["turns"][0]["status"], "ok", "{turns}");
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (status, capture) = request(host, &path, None).await;
    assert_eq!(status, 200, "{capture}");
    assert_eq!(
        capture["starting"]["entries"]["app.ts"]["bytes"],
        json!(b"dirty A\n".to_vec())
    );
    assert_eq!(
        capture["completed"]["entries"]["app.ts"]["bytes"],
        json!(b"dirty A\n".to_vec())
    );
    let (status, response) =
        request(host, &path, Some(json!({"resolve":capture["ambiguous"]}))).await;
    assert_eq!(status, 200, "{response}");
    assert_eq!(response["verified"], true);
    turn.to_owned()
}

</storycode:copy>
[Read the test `native_context_registration_freezes_memory_without_borrowing_working_code` beside its requirement](../../../../guarantees/literate-projects/tools-select-complete-isolated-endpoints/contract.md#f-c0298ec2aa35-native_context_registration_freezes_memory_without_borrowing_working_code).
[Read the test `dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapplying_code` beside its requirement](../../../../guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities/contract.md#f-c0298ec2aa35-dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapply).
<a id="f-c0298ec2aa35-changed_code_and_review_edits_that_omit_outputs_cannot_publish"></a>
### `changed_code_and_review_edits_that_omit_outputs_cannot_publish`

<storycode:copy id="f-c0298ec2aa35-changed_code_and_review_edits_that_omit_outputs_cannot_publish">
#[tokio::test]
async fn changed_code_and_review_edits_that_omit_outputs_cannot_publish() {
    let host = host().await;
    let turn = finalized(&host).await;
    let path = format!("/api/docs/{}/agent/projects/{turn}/incorporate", host.doc);
    request(&host, &path, Some(json!({}))).await;
    let review = until(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        |v| {
            v["edits"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["status"] == "pending")
        },
    )
    .await;
    let original = std::fs::read_to_string(host.root.join("project.md")).unwrap();
    request(&host,&format!("/api/docs/{}/agent/review",host.doc),Some(json!({"id":review["edits"]["changes"][0]["id"],"accepted":true,"new_text":"<hick:project/>"}))).await;
    let failed = until(&host, &path, |v| v["status"] == "failed").await;
    assert!(
        failed["error"].as_str().unwrap().contains("constructs B"),
        "{failed}"
    );
    assert_eq!(
        std::fs::read_to_string(host.root.join("project.md")).unwrap(),
        original
    );
    // An authored candidate follows the same review and destination guard.
    request(&host,&path,Some(json!({"documents":{"project.md":"<hick:project/><hick:file path=\"app.ts\">dirty A\n</hick:file>"}}))).await;
    let review = until(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        |v| {
            v["edits"]["changes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|c| c["status"] == "pending")
        },
    )
    .await;
    let change = review["edits"]["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["status"] == "pending")
        .unwrap();
    std::fs::write(host.root.join("app.ts"), "concurrent edit\n").unwrap();
    request(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        Some(json!({"id":change["id"],"accepted":true})),
    )
    .await;
    let failed = until(&host, &path, |v| v["status"] == "failed").await;
    assert!(
        failed["error"].as_str().unwrap().contains("code changed"),
        "{failed}"
    );
    assert_eq!(
        std::fs::read_to_string(host.root.join("project.md")).unwrap(),
        original
    );
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "concurrent edit\n"
    );
}

</storycode:copy>
<a id="f-c0298ec2aa35-native_acp_capture_omits_abandoned_bytes_and_requires_attribution_resolution"></a>
### `native_acp_capture_omits_abandoned_bytes_and_requires_attribution_resolution`

Native ACP commands intentionally bypass the document-tool queue. This is a
real child process, not an imported diff presented as replay evidence.

<storycode:copy id="f-c0298ec2aa35-native_acp_capture_omits_abandoned_bytes_and_requires_attribution_resolution">
// Native ACP commands intentionally bypass the document-tool queue. This is a
// real child process, not an imported diff presented as replay evidence.
#[tokio::test]
async fn native_acp_capture_omits_abandoned_bytes_and_requires_attribution_resolution() {
    let host = host().await;
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(format!(
            "examples/acp_fixture{}",
            std::env::consts::EXE_SUFFIX
        ));
    assert!(
        binary.is_file(),
        "Run just test-literate-agent-projects to build the protocol peer."
    );
    let response = reqwest::Client::new()
        .put(format!("{}/api/agents", host.base))
        .json(&json!([{"id":"fixture","name":"Native ACP fixture","command":binary,"args":[]}]))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let (status, value) = request(
        &host,
        &format!("/api/docs/{}/agent/acp", host.doc),
        Some(json!({"backend":"fixture"})),
    )
    .await;
    assert_eq!(status, 200, "{value}");
    let (status, value) = request(
        &host,
        &format!("/api/docs/{}/agent", host.doc),
        Some(json!({"backend":"fixture","prompt":"literate-native"})),
    )
    .await;
    assert_eq!(status, 202, "{value}");
    let turn = value["session_id"].as_str().unwrap();
    until(&host, &format!("/api/docs/{}/agent/turns", host.doc), |v| {
        v["turns"][0]["status"] != "running"
    })
    .await;
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (status, capture) = request(&host, &path, None).await;
    assert_eq!(status, 200, "{capture}");
    assert_eq!(
        capture["completed"]["entries"]["app.ts"]["bytes"],
        json!(b"native completed\n".to_vec())
    );
    assert!(
        capture["ambiguous"]
            .as_array()
            .unwrap()
            .contains(&json!("app.ts"))
    );
    let (status, refused) = request(&host, &path, Some(json!({"resolve":[]}))).await;
    assert_eq!(status, 409, "{refused}");
    let (status, response) =
        request(&host, &path, Some(json!({"resolve":capture["ambiguous"]}))).await;
    assert_eq!(status, 200, "{response}");
    let source = response["response"].as_str().unwrap();
    assert!(source.contains("native completed"));
    assert!(!source.contains("abandoned draft"));
    let log = std::fs::read_dir(host.root.join("sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        std::fs::read_to_string(log)
            .unwrap()
            .contains("abandoned draft")
    );
}

</storycode:copy>
[Read the test `revision` beside its requirement](../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md#f-c0298ec2aa35-revision).
<a id="f-c0298ec2aa35-final_revision_publishes_b_to_c_without_changing_the_accepted_artifact"></a>
### `final_revision_publishes_b_to_c_without_changing_the_accepted_artifact`

<storycode:copy id="f-c0298ec2aa35-final_revision_publishes_b_to_c_without_changing_the_accepted_artifact">
#[tokio::test]
async fn final_revision_publishes_b_to_c_without_changing_the_accepted_artifact() {
    let host = host().await;
    let turn = finalized(&host).await;
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (_, original) = request(&host, &path, None).await;
    let (apply, body) = revision(&host, &turn, &[("app.ts", "authored C\n")]).await;
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
    let (status, result) = request(&host, &apply, Some(body.clone())).await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["error"], Value::Null, "{result}");
    assert_eq!(result["constructed"], true);
    assert_eq!(result["runtime"], "unrun");
    assert_eq!(result["applied"], json!(["app.ts"]));
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "authored C\n"
    );
    let (_, after) = request(&host, &path, None).await;
    assert_eq!(after, original);
    let (status, recorded) = request(
        &host,
        &format!("{path}/draft/result"),
        Some(json!({"editorResults":[],"applied":["app.ts"]})),
    )
    .await;
    assert_eq!(status, 200, "{recorded}");
    assert_eq!(request(&host, &path, None).await.1, original);

    let (status, _) = request(&host, &apply, Some(body)).await;
    assert_eq!(status, 409, "A-to-C must never replay on already applied C");
}
</storycode:copy>
<a id="f-c0298ec2aa35-final_revision_refuses_stale_inventory_and_invalid_construction_without_writes"></a>
### `final_revision_refuses_stale_inventory_and_invalid_construction_without_writes`

<storycode:copy id="f-c0298ec2aa35-final_revision_refuses_stale_inventory_and_invalid_construction_without_writes">
#[tokio::test]
async fn final_revision_refuses_stale_inventory_and_invalid_construction_without_writes() {
    let host = host().await;
    let turn = finalized(&host).await;
    let (path, body) = revision(&host, &turn, &[("app.ts", "C\n")]).await;
    std::fs::write(host.root.join("app.ts"), "newer work\n").unwrap();
    assert_eq!(request(&host, &path, Some(body.clone())).await.0, 409);
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "newer work\n"
    );
    std::fs::write(host.root.join("app.ts"), "dirty A\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(
            host.root.join("app.ts"),
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();
        assert_eq!(request(&host, &path, Some(body.clone())).await.0, 409);
        std::fs::set_permissions(
            host.root.join("app.ts"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
    }
    std::fs::write(host.root.join("created.txt"), "newer creation").unwrap();
    assert_eq!(request(&host, &path, Some(body.clone())).await.0, 409);
    std::fs::remove_file(host.root.join("created.txt")).unwrap();
    std::fs::rename(host.root.join("app.ts"), host.root.join("renamed.ts")).unwrap();
    assert_eq!(request(&host, &path, Some(body.clone())).await.0, 409);
    std::fs::rename(host.root.join("renamed.ts"), host.root.join("app.ts")).unwrap();
    let mut broken = body;
    broken["sources"]["response.md"] = json!(
        "<hick:project/><hick:patch id=\"broken\">\n--- a/app.ts\n+++ b/app.ts\n@@ -1 +1 @@\n-wrong baseline\n+broken\n</hick:patch>"
    );
    assert_eq!(request(&host, &path, Some(broken)).await.0, 422);
    let downstream = json!({"context":{"buffers":[]},"sources":{"response.md":"<hick:project/><hick:patch id=\"first\">\n--- a/app.ts\n+++ b/app.ts\n@@ -1 +1 @@\n-dirty A\n+broken early edit\n</hick:patch><hick:patch id=\"later\" after=\"first\">\n--- a/app.ts\n+++ b/app.ts\n@@ -1 +1 @@\n-new\n+final\n</hick:patch>"}});
    assert_eq!(request(&host, &path, Some(downstream)).await.0, 422);

    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
}
</storycode:copy>
<a id="f-c0298ec2aa35-final_revision_reports_partial_publication_and_keeps_guarded_recovery_bytes"></a>
### `final_revision_reports_partial_publication_and_keeps_guarded_recovery_bytes`

<storycode:copy id="f-c0298ec2aa35-final_revision_reports_partial_publication_and_keeps_guarded_recovery_bytes">
#[cfg(unix)]
#[tokio::test]
async fn final_revision_reports_partial_publication_and_keeps_guarded_recovery_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let host = host().await;
    std::fs::write(host.root.join("z.txt"), "B\n").unwrap();
    let turn = finalized(&host).await;
    let (path, body) = revision(&host, &turn, &[("app.ts", "C\n"), ("z.txt", "C\n")]).await;
    std::fs::set_permissions(
        host.root.join("z.txt"),
        std::fs::Permissions::from_mode(0o444),
    )
    .unwrap();
    let (status, result) = request(&host, &path, Some(body)).await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["applied"], json!(["app.ts"]));
    assert!(
        result["error"].as_str().unwrap().contains("read-only"),
        "{result}"
    );
    assert_eq!(result["undo"][0]["before"], "dirty A\n");
    assert_eq!(result["undo"][0]["after"], "C\n");
    assert_eq!(
        std::fs::read_to_string(host.root.join("z.txt")).unwrap(),
        "B\n"
    );
    std::fs::set_permissions(
        host.root.join("z.txt"),
        std::fs::Permissions::from_mode(0o644),
    )
    .unwrap();
}

</storycode:copy>
[Read the test `final_revision_preserves_the_accepted_executable_mode` beside its requirement](../../../../guarantees/agent/final-changeset-revisions-apply-b-to-c/contract.md#f-c0298ec2aa35-final_revision_preserves_the_accepted_executable_mode).

### Assemble `crates/hickory-cli/tests/literate_agent_projects.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/literate_agent_projects.rs"><storycode:paste select="#f-c0298ec2aa35-Host"/><storycode:paste select="#f-c0298ec2aa35-host"/><storycode:paste select="#f-c0298ec2aa35-request"/><storycode:paste select="#f-c0298ec2aa35-until"/><storycode:paste select="#f-c0298ec2aa35-finalized"/><storycode:paste select="#f-c0298ec2aa35-native_context_registration_freezes_memory_without_borrowing_working_code"/><storycode:paste select="#f-c0298ec2aa35-dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapply"/><storycode:paste select="#f-c0298ec2aa35-changed_code_and_review_edits_that_omit_outputs_cannot_publish"/><storycode:paste select="#f-c0298ec2aa35-native_acp_capture_omits_abandoned_bytes_and_requires_attribution_resolution"/><storycode:paste select="#f-c0298ec2aa35-revision"/><storycode:paste select="#f-c0298ec2aa35-final_revision_publishes_b_to_c_without_changing_the_accepted_artifact"/><storycode:paste select="#f-c0298ec2aa35-final_revision_refuses_stale_inventory_and_invalid_construction_without_writes"/><storycode:paste select="#f-c0298ec2aa35-final_revision_reports_partial_publication_and_keeps_guarded_recovery_bytes"/><storycode:paste select="#f-c0298ec2aa35-final_revision_preserves_the_accepted_executable_mode"/></storycode:file>

</storycode:doc>
