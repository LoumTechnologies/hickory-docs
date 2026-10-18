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

// Guarantee: docs/guarantees/literate-projects/tools-select-complete-isolated-endpoints.md
#[tokio::test]
async fn native_context_registration_freezes_memory_without_borrowing_working_code() {
    let host = host().await;
    let mut tree = hick_lang::ProjectTree::default();
    tree.insert(
        "app.ts",
        hick_lang::ProjectEntry::file(b"captured B\n".to_vec()),
    )
    .unwrap();
    tree.insert(
        "library.ts",
        hick_lang::ProjectEntry::file(b"later library\n".to_vec()),
    )
    .unwrap();
    let (status, registered) = request(
        &host,
        "/api/project-contexts",
        Some(json!({"snapshot":hick_lang::ProjectSnapshot::capture(&tree)})),
    )
    .await;
    assert_eq!(status, 200, "{registered}");
    assert_eq!(registered["materialized"], false);
    assert!(!host.root.join("library.ts").exists());
    std::fs::write(host.root.join("app.ts"), "concurrent working code\n").unwrap();
    let mut url = reqwest::Url::parse(&format!("{}/api/project-contexts/file", host.base)).unwrap();
    url.query_pairs_mut().append_pair(
        "uri",
        &format!("{}app.ts", registered["uri"].as_str().unwrap()),
    );
    let response = reqwest::get(url.clone()).await.unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let file: Value = response.json().await.unwrap();
    assert_eq!(file["bytes"], json!(b"captured B\n".to_vec()));
    assert_eq!(file["files"].as_array().unwrap().len(), 2);
    let removed = reqwest::Client::new()
        .delete(format!(
            "{}/api/project-contexts/{}",
            host.base,
            registered["id"].as_str().unwrap()
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(removed.status(), reqwest::StatusCode::OK);
    assert_eq!(
        reqwest::get(url).await.unwrap().status(),
        reqwest::StatusCode::NOT_FOUND
    );
}

// Guarantee: docs/guarantees/literate-projects/finalization-and-incorporation-check-distinct-equalities.md
#[tokio::test]
async fn dirty_start_is_finalized_then_ai_source_is_verified_reviewed_and_published_without_reapplying_code()
 {
    let host = host().await;
    let turn = finalized(&host).await;
    let path = format!("/api/docs/{}/agent/projects/{turn}/incorporate", host.doc);
    let (status, value) = request(&host, &path, Some(json!({}))).await;
    assert_eq!(status, 200, "{value}");
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
    let change = &review["edits"]["changes"][0];
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
    let revised = format!(
        "Reviewed explanation.\n{}",
        change["newText"].as_str().unwrap()
    );
    let (status, value) = request(
        &host,
        &format!("/api/docs/{}/agent/review", host.doc),
        Some(json!({"id":change["id"],"accepted":true,"new_text":revised})),
    )
    .await;
    assert_eq!(status, 200, "{value}");
    let result = until(&host, &path, |v| {
        matches!(v["status"].as_str(), Some("applied" | "failed"))
    })
    .await;
    assert_eq!(result["status"], "applied", "{result}");
    assert_eq!(
        std::fs::read_to_string(host.root.join("project.md")).unwrap(),
        revised
    );
    assert_eq!(
        std::fs::read_to_string(host.root.join("app.ts")).unwrap(),
        "dirty A\n"
    );
    let log = std::fs::read_dir(host.root.join("sessions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(
        std::fs::read_to_string(log)
            .unwrap()
            .contains("literate-incorporation-applied")
    );
}

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

// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
async fn revision(host: &Host, turn: &str, changes: &[(&str, &str)]) -> (String, Value) {
    let path = format!("/api/docs/{}/agent/projects/{turn}", host.doc);
    let (_, capture) = request(host, &path, None).await;
    let starting: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["starting"].clone()).unwrap();
    let completed: hick_lang::ProjectSnapshot =
        serde_json::from_value(capture["completed"].clone()).unwrap();
    let a = starting.restore().unwrap();
    let mut c = completed.restore().unwrap();
    for (name, text) in changes {
        let mut entry = c
            .get(name)
            .map(|e| (**e).clone())
            .unwrap_or_else(|| hick_lang::ProjectEntry::file(Vec::new()));
        entry.bytes = text.as_bytes().to_vec().into();
        entry.origins.clear();
        c.insert(name, entry).unwrap();
    }
    let response = hick_lang::transition_response(&a, &c, "Authored revision").unwrap();
    (
        format!("{path}/draft/apply"),
        json!({"sources":{"response.md":response},"context":{"buffers":[]}}),
    )
}
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

// Guarantee: docs/guarantees/agent/final-changeset-revisions-apply-b-to-c.md
#[cfg(unix)]
#[tokio::test]
async fn final_revision_preserves_the_accepted_executable_mode() {
    use std::os::unix::fs::PermissionsExt;
    let host = host().await;
    std::fs::set_permissions(
        host.root.join("app.ts"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    let turn = finalized(&host).await;
    let (path, body) = revision(&host, &turn, &[("app.ts", "C\n")]).await;
    let (status, result) = request(&host, &path, Some(body)).await;
    assert_eq!(status, 200, "{result}");
    assert_eq!(result["error"], Value::Null, "{result}");
    assert_eq!(
        std::fs::metadata(host.root.join("app.ts"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}
