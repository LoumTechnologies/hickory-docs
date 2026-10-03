//! Protects docs/guarantees/agent/acp-agents-are-first-class.md.
//! A protocol peer is necessary to test races, permissions and hung-process
//! cancellation deterministically. It is never a fallback for missing login.
use hickory_cli::{
    ExecutorChoice,
    serve::{ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{path::PathBuf, time::Duration};

const DOC: &str = r##"<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct App {
    base: String,
    doc: String,
    root: PathBuf,
    _task: tokio::task::JoinHandle<()>,
    state: hickory_cli::serve::LocalState,
}
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
impl Drop for App {
    fn drop(&mut self) {
        self._task.abort();
    }
}
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

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
async fn pending_change(app: &App) -> Value {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
            if let Some(change) = state["edits"]["changes"].as_array().and_then(|cs| {
                cs.iter()
                    .find(|c| c["status"] == "pending" || c["status"] == "applying")
            }) {
                return change.clone();
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_untitled_edits_use_tools_and_wait_for_client_review() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    let ready = app.connect(None).await;
    assert_eq!(ready["edits"]["mode"], "review");
    for (accepted, parent) in [(false, None), (true, Some("previous"))] {
        let parent = if parent.is_some() {
            let (_, turns) = app.request("GET", &app.route("/turns"), json!({})).await;
            Some(turns["turns"][0]["id"].as_str().unwrap().to_string())
        } else {
            None
        };
        let (status, response) = app.request("POST", &app.route(""), json!({
            "backend":"fixture", "prompt":"buffer-edit", "parent_id":parent,
            "context":{"buffers":[{"id":"untitled-tab","kind":"untitled","name":"Untitled 1","path":null,"content":"# Original 📝\n","focused":true}]}
        })).await;
        assert_eq!(status, 202, "{response}");
        let change = pending_change(&app).await;
        assert_eq!(change["oldText"], "# Original 📝\n");
        assert_eq!(change["newText"], "# Changed by ACP 📝\n");
        assert_eq!(change["name"], "Untitled 1");
        assert_eq!(change["buffer"], "untitled-tab");
        assert!(change["path"].is_null());
        assert!(!app.root.join("Untitled 1").exists());
        assert_eq!(
            std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
            DOC
        );
        let (status, result) = app
            .request(
                "POST",
                &app.route("/acp/edits"),
                json!({"id":change["id"],"accepted":accepted}),
            )
            .await;
        assert_eq!(status, 200, "{result}");
        let turn = app.wait(response["session_id"].as_str().unwrap()).await;
        assert_eq!(turn["status"], "ok", "{turn}");
        assert!(turn["answer"].as_str().unwrap().contains(if accepted {
            "Edited the live buffer"
        } else {
            "rejected this edit"
        }));
        let (status, _) = app
            .request(
                "POST",
                &app.route("/acp/edits"),
                json!({"id":change["id"],"accepted":true}),
            )
            .await;
        assert_eq!(status, 422); // Decisions are consumed once.
    }
}

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_auto_accept_is_conversation_scoped_and_restored() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    app.connect(None).await;
    let (status, state) = app
        .request(
            "POST",
            &app.route("/acp/edits"),
            json!({"mode":"auto-accept"}),
        )
        .await;
    assert_eq!(status, 200);
    assert_eq!(state["edits"]["mode"], "auto-accept");
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"buffer-edit","context":{"buffers":[{"id":"tab","name":"Untitled","path":null,"content":"original\n","focused":true}]}})).await;
    let change = pending_change(&app).await;
    assert_eq!(change["status"], "applying");
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    let session = turn["session"].as_str().unwrap().to_string();
    let root = app.root.clone();
    drop(app);
    let mut app = start(&root).await;
    app.doc = "workspace".into();
    assert_eq!(
        app.connect(Some(&session)).await["edits"]["mode"],
        "auto-accept"
    );
    // An independent conversation gets its own default.
    app.doc = app.state.index.sole().unwrap().0;
    assert_eq!(app.connect(None).await["edits"]["mode"], "review");
}

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn acp_existing_output_tool_waits_for_review_before_writing() {
    let (_dir, app) = fixture(vec![]).await;
    app.state.rooms.get_or_create(&app.doc).await.unwrap();
    app.connect(None).await;
    let id = app.send("edit", None).await;
    let change = pending_change(&app).await;
    assert_eq!(change["editor"], false);
    assert!(change["newText"].as_str().unwrap().contains("ACP"));
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    assert_eq!(app.wait(&id).await["status"], "ok");
    assert!(
        std::fs::read_to_string(app.root.join("demo.md"))
            .unwrap()
            .contains("println!(\"ACP\")")
    );
}

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
async fn stopping_a_review_cancels_the_edit() {
    let (_dir, mut app) = fixture(vec![]).await;
    app.doc = "workspace".into();
    app.connect(None).await;
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"buffer-edit","context":{"buffers":[{"id":"tab","name":"Untitled","path":null,"content":"original\n","focused":true}]}})).await;
    let change = pending_change(&app).await;
    app.request("POST", &app.route("/stop"), json!({})).await;
    assert_eq!(
        app.wait(response["session_id"].as_str().unwrap()).await["status"],
        "stopped"
    );
    let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
    assert_eq!(state["edits"]["changes"][0]["status"], "rejected");
    assert_eq!(
        std::fs::read_to_string(app.root.join("demo.md")).unwrap(),
        DOC
    );
    assert_eq!(
        app.request(
            "POST",
            &app.route("/acp/edits"),
            json!({"id":change["id"],"accepted":true})
        )
        .await
        .0,
        422
    );
}

// Guarantee: docs/guarantees/agent/conversation-edits-use-the-client-review-policy.md
#[tokio::test]
#[ignore = "uses the installed authenticated Codex ACP adapter and a real model turn"]
async fn live_codex_edits_current_note_through_review() {
    let command = std::env::var("HICKORY_ACP_LIVE_COMMAND").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap();
    let before = "# Original heading\n\nKeep this paragraph unchanged.\n";
    std::fs::write(root.join("demo.md"), before).unwrap();
    let mut app = start(&root).await;
    app.doc = "workspace".into();
    app.request(
        "PUT",
        "/api/agents",
        json!([{"id":"fixture","name":"Codex live","command":command,"args":[]}]),
    )
    .await;
    let connected = app.connect(None).await;
    assert_eq!(connected["ready"], true, "{connected}");
    let (_, response) = app.request("POST", &app.route(""), json!({"backend":"fixture","prompt":"Change the heading in the current document to Meeting notes. Keep its paragraph unchanged.","context":{"buffers":[{"id":"note-tab","kind":"document","name":"demo.md","path":"demo.md","content":before,"focused":true}]}})).await;
    let change = tokio::time::timeout(Duration::from_secs(90), async {
        loop {
            let (_, state) = app.request("GET", &app.route("/acp"), json!({})).await;
            if let Some(change) = state["edits"]["changes"]
                .as_array()
                .and_then(|cs| cs.iter().find(|c| c["status"] == "pending"))
            {
                break change.clone();
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    })
    .await
    .expect("Codex should submit a tool edit rather than printing the replacement document");
    assert_eq!(change["oldText"], before);
    let after = "# Meeting notes\n\nKeep this paragraph unchanged.\n";
    assert_eq!(change["newText"], after, "{change}");
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        before
    );
    // The real UI uses its editor transaction. Here the HTTP client applies
    // the approved bytes through the public document route before acknowledging.
    let doc = app.state.index.sole().unwrap().0;
    app.request("PUT", &format!("/api/docs/{doc}"), json!({"source":after}))
        .await;
    app.request(
        "POST",
        &app.route("/acp/edits"),
        json!({"id":change["id"],"accepted":true}),
    )
    .await;
    let turn = app.wait(response["session_id"].as_str().unwrap()).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    assert_eq!(
        std::fs::read_to_string(root.join("demo.md")).unwrap(),
        after
    );
    println!(
        "Live Codex current-note tool edit reviewed and accepted: {}",
        turn["answer"]
    );
}
