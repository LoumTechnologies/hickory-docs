<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_acp.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2fce65a349e6"></a>
## `crates/hickory-cli/tests/serve_acp.rs`

Related contracts: [ACP Agents Work In The Agent Pane And Leave A Hickory Record](../../../../guarantees/agent/acp-agents-are-first-class/contract.md), [Conversation edits use the client's review policy](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md), [Select model context without saving open buffers](../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md), [A literate view writes through to ordinary source](../../../../guarantees/authoring/a-literate-view-writes-through-to-ordinary-source/contract.md).

[Read the test `DOC` beside its requirement](../../../../guarantees/agent/acp-agents-are-first-class/contract.md#f-2fce65a349e6-DOC).
<a id="f-2fce65a349e6-App"></a>
### `App`

<storycode:copy id="f-2fce65a349e6-App">
struct App {
    base: String,
    doc: String,
    root: PathBuf,
    _task: tokio::task::JoinHandle<()>,
    state: hickory_cli::serve::LocalState,
}
</storycode:copy>
<a id="f-2fce65a349e6-start"></a>
### `start`

<storycode:copy id="f-2fce65a349e6-start">
async fn start(root: &std::path::Path) -> App {
    let prepared = prepare(ServeOptions {
        target: root.join("demo.md"),
        port: 0,
        params: vec![],
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: Some(root.join("ui.json")),
    })
    .await
    .unwrap();
    let state = prepared.state.clone();
    let doc = state.index.sole().unwrap().0;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let task = tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    App {
        base,
        doc,
        root: root.into(),
        _task: task,
        state,
    }
}
</storycode:copy>
<a id="f-2fce65a349e6-impl-Drop-for-App"></a>
### `impl-Drop for App`

<storycode:copy id="f-2fce65a349e6-impl-Drop-for-App">
impl Drop for App {
    fn drop(&mut self) {
        self._task.abort();
    }
}
</storycode:copy>
<a id="f-2fce65a349e6-impl-App"></a>
### `impl-App`

<storycode:copy id="f-2fce65a349e6-impl-App">
impl App {
    async fn request(&self, method: &str, path: &str, body: Value) -> (u16, Value) {
        let result = reqwest::Client::new()
            .request(method.parse().unwrap(), format!("{}{path}", self.base))
            .json(&body)
            .send()
            .await
            .unwrap();
        (result.status().as_u16(), result.json().await.unwrap())
    }
    fn route(&self, suffix: &str) -> String {
        format!("/api/docs/{}/agent{suffix}", self.doc)
    }
    async fn connect(&self, session: Option<&str>) -> Value {
        let (status, result) = self
            .request(
                "POST",
                &self.route("/acp"),
                json!({"backend":"fixture","session":session}),
            )
            .await;
        assert_eq!(status, 200, "{result}");
        result
    }
    async fn send(&self, prompt: &str, parent: Option<&str>) -> String {
        let (status, result) = self
            .request(
                "POST",
                &self.route(""),
                json!({"backend":"fixture","prompt":prompt,"parent_id":parent}),
            )
            .await;
        assert_eq!(status, 202, "{result}");
        result["session_id"].as_str().unwrap().into()
    }
    async fn wait(&self, id: &str) -> Value {
        let outcome = tokio::time::timeout(
            Duration::from_secs(if std::env::var_os("HICKORY_ACP_LIVE_COMMAND").is_some() {
                180
            } else {
                15
            }),
            async {
                loop {
                    let (_, result) = self
                        .request("GET", &self.route("/turns"), Value::Null)
                        .await;
                    if let Some(turn) = result["turns"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|t| t["id"] == id && t["status"] != "running")
                    {
                        return turn.clone();
                    }
                    tokio::time::sleep(Duration::from_millis(25)).await;
                }
            },
        )
        .await;
        match outcome {
            Ok(turn) => turn,
            Err(e) => {
                let (_, snapshot) = self.request("GET", &self.route("/acp"), Value::Null).await;
                let (_, listing) = self
                    .request("GET", &self.route("/turns"), Value::Null)
                    .await;
                self.request("POST", &self.route("/stop"), Value::Null)
                    .await;
                panic!("Timed out {id}: {e}; state={snapshot}; turns={listing}")
            }
        }
    }
}
</storycode:copy>
<a id="f-2fce65a349e6-fixture"></a>
### `fixture`

<storycode:copy id="f-2fce65a349e6-fixture">
async fn fixture(args: Vec<&str>) -> (tempfile::TempDir, App) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("demo.md"), DOC).unwrap();
    let app = start(&root).await;
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
        binary.exists(),
        "build the ACP protocol peer with just test-acp"
    );
    let (status, result) = app
        .request(
            "PUT",
            "/api/agents",
            json!([{"id":"fixture","name":"Test ACP","command":binary,"args":args}]),
        )
        .await;
    assert_eq!(status, 200, "{result}");
    (dir, app)
}

</storycode:copy>
<a id="f-2fce65a349e6-acp_records_exact_final_tokens_tools_and_resumes_after_restart"></a>
### `acp_records_exact_final_tokens_tools_and_resumes_after_restart`

<storycode:copy id="f-2fce65a349e6-acp_records_exact_final_tokens_tools_and_resumes_after_restart">
#[tokio::test]
async fn acp_records_exact_final_tokens_tools_and_resumes_after_restart() {
    let (_dir, app) = fixture(vec![]).await;
    let connected = app.connect(None).await;
    assert_eq!(connected["ready"], true);
    let id = app.send("hello", None).await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    assert_eq!(
        turn["answer"],
        "Hello from ACP. <hick:file> is literal here."
    );
    let session = turn["session"].as_str().unwrap().to_string();
    let source = std::fs::read_to_string(app.root.join(&session)).unwrap();
    hick_lang::parse_session(&source).unwrap();
    assert!(source.ends_with("</hick:session>\n"));
    assert!(source.contains("Checking the document."));
    assert!(source.contains("Read successfully"));
    let (_, settings) = app
        .request(
            "POST",
            &app.route("/acp/configure"),
            json!({"config_id":"model","value":"other"}),
        )
        .await;
    assert_eq!(settings["configOptions"][0]["currentValue"], "other");
    let root = app.root.clone();
    drop(app);
    let app = start(&root).await;
    let resumed = app.connect(Some(&session)).await;
    assert_eq!(resumed["ready"], true, "{resumed}");
    let next = app.send("continue", Some(&id)).await;
    assert_eq!(app.wait(&next).await["status"], "ok");
    let branch = app.send("branch from the first turn", Some(&id)).await;
    let branched = app.wait(&branch).await;
    assert_eq!(branched["status"], "ok", "{branched}");
    assert_eq!(branched["parent_id"], id);
    assert_eq!(branched["session"], session);
}

</storycode:copy>
<a id="f-2fce65a349e6-permissions_validate_choices_and_do_not_block_other_updates"></a>
### `permissions_validate_choices_and_do_not_block_other_updates`

<storycode:copy id="f-2fce65a349e6-permissions_validate_choices_and_do_not_block_other_updates">
#[tokio::test]
async fn permissions_validate_choices_and_do_not_block_other_updates() {
    let (_dir, app) = fixture(vec![]).await;
    app.connect(None).await;
    let id = app.send("permission", None).await;
    let permission = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let (_, state) = app.request("GET", &app.route("/acp"), Value::Null).await;
            if let Some(p) = state["permissions"].as_array().and_then(|p| p.first()) {
                break p.clone();
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        app.request(
            "POST",
            &app.route("/acp/permission"),
            json!({"request_id":permission["id"],"option_id":"invented"})
        )
        .await
        .0,
        400
    );
    assert_eq!(
        app.request(
            "POST",
            &app.route("/acp/permission"),
            json!({"request_id":permission["id"],"option_id":"allow"})
        )
        .await
        .0,
        200
    );
    assert_eq!(app.wait(&id).await["answer"], "Permission allow");
}

</storycode:copy>
<a id="f-2fce65a349e6-stop_kills_a_hung_adapter_and_restored_status_stays_stopped"></a>
### `stop_kills_a_hung_adapter_and_restored_status_stays_stopped`

<storycode:copy id="f-2fce65a349e6-stop_kills_a_hung_adapter_and_restored_status_stays_stopped">
#[tokio::test]
async fn stop_kills_a_hung_adapter_and_restored_status_stays_stopped() {
    let (_dir, app) = fixture(vec![]).await;
    app.connect(None).await;
    let id = app.send("hang", None).await;
    assert_eq!(
        app.request("POST", &app.route("/stop"), Value::Null)
            .await
            .0,
        200
    );
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "stopped", "{turn}");
    let root = app.root.clone();
    drop(app);
    let app = start(&root).await;
    let (_, listing) = app.request("GET", &app.route("/turns"), Value::Null).await;
    assert_eq!(listing["turns"][0]["status"], "stopped");
}

</storycode:copy>
<a id="f-2fce65a349e6-authentication_is_offered_without_a_hickory_model_key"></a>
### `authentication_is_offered_without_a_hickory_model_key`

<storycode:copy id="f-2fce65a349e6-authentication_is_offered_without_a_hickory_model_key">
#[tokio::test]
async fn authentication_is_offered_without_a_hickory_model_key() {
    let (_dir, app) = fixture(vec!["--auth"]).await;
    let connected = app.connect(None).await;
    assert_eq!(connected["ready"], false);
    assert_eq!(connected["authMethods"][0]["id"], "login");
    let (status, auth) = app
        .request(
            "POST",
            &app.route("/acp/authenticate"),
            json!({"method_id":"login"}),
        )
        .await;
    assert_eq!(status, 200, "{auth}");
    assert_eq!(auth["ready"], true);
    let id = app.send("hello", None).await;
    assert_eq!(app.wait(&id).await["status"], "ok");
}

</storycode:copy>
<a id="f-2fce65a349e6-acp_mcp_edit_maps_back_and_verifies_in_the_live_document_room"></a>
### `acp_mcp_edit_maps_back_and_verifies_in_the_live_document_room`

<storycode:copy id="f-2fce65a349e6-acp_mcp_edit_maps_back_and_verifies_in_the_live_document_room">
#[tokio::test]
async fn acp_mcp_edit_maps_back_and_verifies_in_the_live_document_room() {
    let (_dir, app) = fixture(vec![]).await;
    app.state.rooms.get_or_create(&app.doc).await.unwrap();
    app.connect(None).await;
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"mode":"auto-accept"}),
    )
    .await;
    let id = app.send("edit", None).await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    let source = std::fs::read_to_string(app.root.join("demo.md")).unwrap();
    assert!(source.contains("println!(\"ACP\")"), "{source}");
    assert_eq!(
        app.state.rooms.get(&app.doc).await.unwrap().text().await,
        source
    );
    let session =
        std::fs::read_to_string(app.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(session.contains("<hick:read"));
    assert!(session.contains("<hick:wrote"));
    hick_lang::parse_session(&session).unwrap();
}

</storycode:copy>
<a id="f-2fce65a349e6-acp_file_reads_record_context_and_writes_refuse_generated_and_outside_paths"></a>
### `acp_file_reads_record_context_and_writes_refuse_generated_and_outside_paths`

<storycode:copy id="f-2fce65a349e6-acp_file_reads_record_context_and_writes_refuse_generated_and_outside_paths">
#[tokio::test]
async fn acp_file_reads_record_context_and_writes_refuse_generated_and_outside_paths() {
    let (_dir, app) = fixture(vec![]).await;
    app.connect(None).await;
    let read = app.send("file-read", None).await;
    let read_turn = app.wait(&read).await;
    assert!(
        read_turn["answer"]
            .as_str()
            .unwrap()
            .starts_with("<hick:doc")
    );
    let record =
        std::fs::read_to_string(app.root.join(read_turn["session"].as_str().unwrap())).unwrap();
    assert!(record.contains("<hick:read"));
    let generated = app.send("file-generated", Some(&read)).await;
    let result = app.wait(&generated).await;
    assert!(
        result["answer"].as_str().unwrap().contains("edit_output"),
        "{result}"
    );
    let outside = app.send("file-outside", Some(&generated)).await;
    let result = app.wait(&outside).await;
    assert!(
        result["answer"]
            .as_str()
            .unwrap()
            .contains("outside this workspace"),
        "{result}"
    );
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
}

</storycode:copy>
<a id="f-2fce65a349e6-a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot"></a>
### `a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot`

agent/an-answer-in-the-agent-pane-has-ribbons.md.

<storycode:copy id="f-2fce65a349e6-a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot">
// Guarantees: agent/context-provenance-is-derived-from-the-session.md;
// agent/an-answer-in-the-agent-pane-has-ribbons.md.
#[tokio::test]
async fn a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot() {
    use sha2::{Digest, Sha256};
    let (_dir, app) = fixture(vec![]).await;
    app.connect(None).await;
    let content = "# Unsaved note\nThe paragraph mentions X, Y and Z. 📝\nUnrequested material\n";
    let (status, response) = app.request("POST", &app.route(""), json!({
        "backend":"fixture", "prompt":"file-read-range", "context":{"buffers":[
            {"id":"note-tab","kind":"document","name":"demo.md","path":"demo.md","content":content,"focused":true}
        ]}
    })).await;
    assert_eq!(status, 202, "{response}");
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    assert_eq!(turn["answer"], "The paragraph mentions X, Y and Z. 📝\n");
    let session = turn["session"].as_str().unwrap();
    let source = std::fs::read_to_string(app.root.join(session)).unwrap();
    let hash = hex::encode(Sha256::digest(content.as_bytes()));
    assert!(
        source.contains(&format!("sha256=\"{hash}\" lines=\"2-2\"")),
        "{source}"
    );
    let (status, view) = app
        .request(
            "GET",
            &format!("/api/sessions/view?path={session}"),
            Value::Null,
        )
        .await;
    assert_eq!(status, 200, "{view}");
    let receipt = view["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|link| {
            link["title"]
                .as_str()
                .is_some_and(|title| title.starts_with("Recorded read"))
        })
        .unwrap();
    assert_eq!(receipt["family"], "context");
    assert_eq!(receipt["to"]["path"], "demo.md");
    assert_eq!(receipt["to"]["lines"], json!([2, 2]));
    assert!(receipt["title"].as_str().unwrap().contains(&hash));
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
}

</storycode:copy>
<a id="f-2fce65a349e6-live_codex_uses_hickory_tools_and_records_a_session"></a>
### `live_codex_uses_hickory_tools_and_records_a_session`

<storycode:copy id="f-2fce65a349e6-live_codex_uses_hickory_tools_and_records_a_session">
#[tokio::test]
#[ignore = "requires a real, authenticated Codex ACP adapter; just test-acp-live"]
async fn live_codex_uses_hickory_tools_and_records_a_session() {
    let command = std::env::var("HICKORY_ACP_LIVE_COMMAND")
        .expect("set HICKORY_ACP_LIVE_COMMAND to codex-acp's absolute path");
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    std::fs::write(root.join("demo.md"), DOC).unwrap();
    let app = start(&root).await;
    app.request(
        "PUT",
        "/api/agents",
        json!([{"id":"fixture","name":"Codex live","command":command,"args":[]}]),
    )
    .await;
    let ready = app.connect(None).await;
    assert_eq!(ready["ready"], true, "{ready}");
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"mode":"auto-accept"}),
    )
    .await;
    let id=app.send("Use the hick MCP server to read_doc, read_output with lineage for greet.rs, edit_output to change hello to ACP, then verify. Report done. Do not use shell tools or edit files directly.",None).await;
    let turn = tokio::time::timeout(Duration::from_secs(180), async {
        loop {
            let (_, listing) = app.request("GET", &app.route("/turns"), Value::Null).await;
            if let Some(turn) = listing["turns"]
                .as_array()
                .unwrap()
                .iter()
                .find(|t| t["id"] == id && t["status"] != "running")
            {
                break turn.clone();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(turn["status"], "ok", "{turn}");
    let source = std::fs::read_to_string(root.join("demo.md")).unwrap();
    assert!(source.contains("ACP"), "{source}");
    let record = std::fs::read_to_string(root.join(turn["session"].as_str().unwrap())).unwrap();
    hick_lang::parse_session(&record).unwrap();
    assert!(record.contains("<hick:wrote"), "no Hickory tool evidence");
    println!("Live Codex verified: {}", turn["answer"]);
    println!("Live: beginning follow-up");
    let next = app
        .send(
            "Remember my branch marker: cactus-purple-73. Reply only ACK. Do not use tools.",
            Some(&id),
        )
        .await;
    assert_eq!(app.wait(&next).await["status"], "ok");
    println!("Live: beginning exact rewind");
    let branch = app.send("What branch marker did I give you in the previous turn? If none, reply only NO_MARKER. Do not use tools.", Some(&id)).await;
    let branched = app.wait(&branch).await;
    assert_eq!(branched["status"], "ok", "{branched}");
    assert!(
        branched["answer"].as_str().unwrap().contains("NO_MARKER"),
        "{branched}"
    );
    println!("Live: rewind completed");
    let saved = branched["session"].as_str().unwrap().to_string();
    drop(app);
    let app = start(&root).await;
    let ready = app.connect(Some(&saved)).await;
    assert_eq!(ready["ready"], true, "{ready}");
    let resumed = app.send("What string replaced hello in the file we edited? Reply only the string. Do not use tools.", Some(&branch)).await;
    let resumed = app.wait(&resumed).await;
    assert_eq!(resumed["status"], "ok", "{resumed}");
    assert!(
        resumed["answer"].as_str().unwrap().contains("ACP"),
        "{resumed}"
    );
    println!("Live Codex rewind and restart-resume verified.");
}

</storycode:copy>
<a id="f-2fce65a349e6-an_uncited_acp_answer_has_context_to_the_untitled_editor"></a>
### `an_uncited_acp_answer_has_context_to_the_untitled_editor`

<storycode:copy id="f-2fce65a349e6-an_uncited_acp_answer_has_context_to_the_untitled_editor">
// Guarantee: agent/an-answer-in-the-agent-pane-has-ribbons.md.
#[tokio::test]
async fn an_uncited_acp_answer_has_context_to_the_untitled_editor() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    app.connect(None).await;
    let (status, response) = app.request("POST", &app.route(""), json!({
        "backend":"fixture", "prompt":"untitled-paragraph", "context":{"buffers":[
            {"id":"startup-note","provenance_uri":"hickory-buffer://window/startup-note","kind":"untitled","name":"Untitled","path":null,"content":"Short.\n\nThe longest paragraph. 📝","focused":true}
        ]}
    })).await;
    assert_eq!(status, 202, "{response}");
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    assert_eq!(
        turn["answer"],
        "The second paragraph has the most characters."
    );
    let path = turn["session"].as_str().unwrap();
    let (status, view) = app
        .request(
            "GET",
            &format!("/api/sessions/view?path={path}"),
            Value::Null,
        )
        .await;
    assert_eq!(status, 200, "{view}");
    let source = view["source"].as_str().unwrap();
    let link = view["links"]
        .as_array()
        .unwrap()
        .iter()
        .find(|link| {
            let span = link["span"].as_array().unwrap();
            source[span[0].as_u64().unwrap() as usize..span[1].as_u64().unwrap() as usize]
                .starts_with("The second paragraph")
        })
        .unwrap_or_else(|| panic!("final claim carries recorded input context: {view}"));
    assert_eq!(link["family"], "context");
    assert_eq!(link["to"]["path"], "hickory-buffer://window/startup-note");
    assert_eq!(link["to"]["lines"], json!([1, 3]));
    assert!(
        link["title"]
            .as_str()
            .unwrap()
            .contains("not support for a claim")
    );
    assert!(!app.root.join("Untitled").exists());
}

</storycode:copy>
[Read the test `acp_connects_to_workspace_with_untitled_context_and_resumes` beside its requirement](../../../../guarantees/agent/the-agent-sees-the-open-editors/contract.md#f-2fce65a349e6-acp_connects_to_workspace_with_untitled_context_and_resumes).
<a id="f-2fce65a349e6-acp_organizes_and_edits_a_disposable_view_without_repository_session_files"></a>
### `acp_organizes_and_edits_a_disposable_view_without_repository_session_files`

<storycode:copy id="f-2fce65a349e6-acp_organizes_and_edits_a_disposable_view_without_repository_session_files">
// Guarantee: authoring/a-literate-view-writes-through-to-ordinary-source.md.
#[tokio::test]
async fn acp_organizes_and_edits_a_disposable_view_without_repository_session_files() {
    let (_dir, mut app) = fixture(vec![]).await;
    std::fs::write(app.root.join("plain.py"), "value = 1\n").unwrap();
    let (status, view) = app
        .request(
            "POST",
            "/api/representations",
            json!({"backing":{"kind":"files","paths":["plain.py"]}}),
        )
        .await;
    assert_eq!(status, 200, "{view}");
    let view_id = view["id"].as_str().unwrap();
    app.doc = format!("lens:{view_id}");
    app.connect(None).await;
    let id = app.send(&format!("literate-edit:{view_id}"), None).await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    assert_eq!(
        std::fs::read_to_string(app.root.join("plain.py")).unwrap(),
        "value = 2\n"
    );
    assert!(!app.root.join("sessions").exists());
    assert!(!app.root.join(format!(".hick-lens-{view_id}")).exists());
    let session = PathBuf::from(turn["session"].as_str().unwrap());
    assert!(!session.starts_with(&app.root));
    let text = std::fs::read_to_string(&session).unwrap();
    assert!(text.contains("literate-view-tool"));
    assert!(text.contains("value = 1"));
    hick_lang::parse_session(&text).unwrap();
    let (status, v) = app
        .request("GET", &format!("/api/representations/{view_id}"), json!({}))
        .await;
    assert_eq!(status, 200, "{v}");
    assert!(
        v["source"]
            .as_str()
            .unwrap()
            .contains("ACP reading of the source.")
    );
    app.request("POST", &app.route("/stop"), json!({})).await;
}

</storycode:copy>
[Read the test `pending_change` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-pending_change).
[Read the test `acp_untitled_edits_use_tools_and_wait_for_client_review` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-acp_untitled_edits_use_tools_and_wait_for_client_review).
[Read the test `acp_auto_accept_is_conversation_scoped_and_restored` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-acp_auto_accept_is_conversation_scoped_and_restored).
[Read the test `acp_existing_output_tool_waits_for_review_before_writing` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-acp_existing_output_tool_waits_for_review_before_writing).
[Read the test `stopping_a_review_cancels_the_edit` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-stopping_a_review_cancels_the_edit).
[Read the test `live_codex_edits_current_note_through_review` beside its requirement](../../../../guarantees/agent/conversation-edits-use-the-client-review-policy/contract.md#f-2fce65a349e6-live_codex_edits_current_note_through_review).

### Assemble `crates/hickory-cli/tests/serve_acp.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_acp.rs"><storycode:paste select="#f-2fce65a349e6-DOC"/><storycode:paste select="#f-2fce65a349e6-App"/><storycode:paste select="#f-2fce65a349e6-start"/><storycode:paste select="#f-2fce65a349e6-impl-Drop-for-App"/><storycode:paste select="#f-2fce65a349e6-impl-App"/><storycode:paste select="#f-2fce65a349e6-fixture"/><storycode:paste select="#f-2fce65a349e6-acp_records_exact_final_tokens_tools_and_resumes_after_restart"/><storycode:paste select="#f-2fce65a349e6-permissions_validate_choices_and_do_not_block_other_updates"/><storycode:paste select="#f-2fce65a349e6-stop_kills_a_hung_adapter_and_restored_status_stays_stopped"/><storycode:paste select="#f-2fce65a349e6-authentication_is_offered_without_a_hickory_model_key"/><storycode:paste select="#f-2fce65a349e6-acp_mcp_edit_maps_back_and_verifies_in_the_live_document_room"/><storycode:paste select="#f-2fce65a349e6-acp_file_reads_record_context_and_writes_refuse_generated_and_outside_paths"/><storycode:paste select="#f-2fce65a349e6-a_live_buffer_read_records_the_returned_range_and_links_to_its_snapshot"/><storycode:paste select="#f-2fce65a349e6-live_codex_uses_hickory_tools_and_records_a_session"/><storycode:paste select="#f-2fce65a349e6-an_uncited_acp_answer_has_context_to_the_untitled_editor"/><storycode:paste select="#f-2fce65a349e6-acp_connects_to_workspace_with_untitled_context_and_resumes"/><storycode:paste select="#f-2fce65a349e6-acp_organizes_and_edits_a_disposable_view_without_repository_session_files"/><storycode:paste select="#f-2fce65a349e6-pending_change"/><storycode:paste select="#f-2fce65a349e6-acp_untitled_edits_use_tools_and_wait_for_client_review"/><storycode:paste select="#f-2fce65a349e6-acp_auto_accept_is_conversation_scoped_and_restored"/><storycode:paste select="#f-2fce65a349e6-acp_existing_output_tool_waits_for_review_before_writing"/><storycode:paste select="#f-2fce65a349e6-stopping_a_review_cancels_the_edit"/><storycode:paste select="#f-2fce65a349e6-live_codex_edits_current_note_through_review"/></storycode:file>

</storycode:doc>
