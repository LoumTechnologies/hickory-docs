//! Guarantees: docs/guarantees/agent/python-execution-has-two-boundaries.md
//! and docs/guarantees/agent/a-session-is-the-conversation.md.
//! Real confined workers and HTTP/WS transport; only the paid model is replaced.
use futures::StreamExt;
use hickory_agent::{LlmClient, Message, Usage};
use hickory_cli::{
    ExecutorChoice,
    serve::{LocalState, ServeOptions, prepare},
};
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

struct Model {
    replies: Mutex<VecDeque<String>>,
    requests: Mutex<Vec<Vec<Message>>>,
}
impl Model {
    fn new(replies: impl IntoIterator<Item = Value>) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(replies.into_iter().map(|v| v.to_string()).collect()),
            requests: Mutex::new(vec![]),
        })
    }
}
#[async_trait::async_trait]
impl LlmClient for Model {
    async fn complete(&self, messages: Vec<Message>) -> anyhow::Result<String> {
        self.requests.lock().unwrap().push(messages);
        self.replies
            .lock()
            .unwrap()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("Unexpected model request"))
    }
    async fn complete_with_usage(&self, messages: Vec<Message>) -> anyhow::Result<(String, Usage)> {
        Ok((
            self.complete(messages).await?,
            Usage {
                input_tokens: 7,
                output_tokens: 3,
                ..Default::default()
            },
        ))
    }
    fn model_name(&self) -> &str {
        "fixture-no-paid-calls"
    }
    fn provider_name(&self) -> &str {
        "fixture"
    }
}
struct App {
    base: String,
    doc: String,
    root: PathBuf,
    state: LocalState,
    _dir: tempfile::TempDir,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for App {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl App {
    async fn start() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join("note.md"), "Original human input\n").unwrap();
        let prepared = prepare(ServeOptions {
            target: root.clone(),
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: Some(root.join("keys.json")),
            ui_settings_path: Some(root.join("ui.json")),
        })
        .await
        .unwrap();
        let state = prepared.state.clone();
        let doc = state.index.sole().unwrap().0;
        state.rooms.get_or_create(&doc).await.unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let task = tokio::spawn(async move {
            axum::serve(listener, prepared.router).await.unwrap();
        });
        Self {
            base: format!("http://127.0.0.1:{port}"),
            doc,
            root,
            state,
            _dir: dir,
            task,
        }
    }
    fn route(&self, suffix: &str) -> String {
        format!("/api/docs/{}/agent{suffix}", self.doc)
    }
    async fn request(&self, method: &str, path: &str, value: Value) -> (u16, Value) {
        let response = reqwest::Client::new()
            .request(method.parse().unwrap(), format!("{}{path}", self.base))
            .json(&value)
            .send()
            .await
            .unwrap();
        let status = response.status().as_u16();
        (status, response.json().await.unwrap())
    }
    async fn configure(&self, python: &str) {
        let (status, value) = self
            .request("PUT", "/api/agents/python", json!({"python":python}))
            .await;
        assert_eq!(status, 200, "{value}");
        assert_eq!(value["available"], true);
    }
    async fn send(&self, prompt: &str, parent: Option<&str>) -> String {
        let (status,value) = self.request("POST", &self.route(""), json!({"backend":"python","prompt":prompt,"parent_id":parent,"context":{"buffers":[{"name":"Meeting note","path":"note.md","content":"Editor context to retain","focused":true}]}})).await;
        assert_eq!(status, 202, "{value}");
        value["session_id"].as_str().unwrap().into()
    }
    async fn pending(&self) -> Value {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let (_, state) = self
                    .request("GET", &self.route("/python"), Value::Null)
                    .await;
                if let Some(change) = state["edits"]["changes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|c| c["status"] == "pending")
                {
                    return change.clone();
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("proposal must become visible")
    }
    async fn wait(&self, id: &str) -> Value {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                let (_, listing) = self
                    .request("GET", &self.route("/turns"), Value::Null)
                    .await;
                if let Some(turn) = listing["turns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|t| t["id"] == id && t["status"] != "running")
                {
                    return turn.clone();
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("turn must finish")
    }
}
fn python() -> Option<String> {
    let value = std::env::var("HICKORY_AGENT_TEST_PYTHON").ok();
    if value.is_none() {
        eprintln!("SKIP real Python UI integration: run scripts/check-python-agent.sh");
    }
    value
}

#[tokio::test]
async fn runtime_setup_refuses_invalid_installations_and_reserved_acp_ids() {
    let app = App::start().await;
    let (status, state) = app.request("GET", &app.route("/python"), Value::Null).await;
    assert_eq!(status, 200);
    assert_eq!(state["ready"], false);
    let (status, _) = app
        .request("PUT", "/api/agents/python", json!({"python":"python3"}))
        .await;
    assert_eq!(status, 422);
    let (status, _) = app
        .request(
            "PUT",
            "/api/agents",
            json!([{"id":"python","name":"spoof","command":"python3"}]),
        )
        .await;
    assert_eq!(status, 422);
    let (status, _) = app
        .request(
            "POST",
            &app.route(""),
            json!({"backend":"python","prompt":"try"}),
        )
        .await;
    assert_eq!(status, 503);
    assert!(!app.root.join("sessions").exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn python_turn_requires_review_streams_receipts_and_rehydrates_its_branch() {
    let Some(python) = python() else {
        return;
    };
    for accept in [false, true] {
        let app = App::start().await;
        app.configure(&python).await;
        let model = Model::new([
            json!({"python":"tool('record_assumption', {'statement':'The meeting note is current', 'reason':'Summary may omit new facts', 'evidence':['User supplied context'], 'falsifier':'A newer note'})\ntool('create_doc', {'path':'proposal.md', 'input':'AI proposal'})"}),
            json!({"done":"Stopped without verification."}),
            json!({"done":"Follow-up stopped."}),
        ]);
        app.state.agent.set_llm_override(model.clone());
        // An earlier native conversation policy must not approve a Python edit.
        assert_eq!(
            app.request("POST", &app.route("/review"), json!({"mode":"auto-accept"}))
                .await
                .0,
            200
        );
        let (mut ws, _) = tokio_tungstenite::connect_async(format!(
            "{}/api/ws?doc=doc:{}",
            app.base.replace("http://", "ws://"),
            app.doc
        ))
        .await
        .unwrap();
        let turn_id = app.send("Propose a note", None).await;
        let change = app.pending().await;
        assert_eq!(change["oldText"], "");
        assert_eq!(change["newText"], "AI proposal\n");
        assert!(!app.root.join("proposal.md").exists());
        assert_eq!(
            app.request(
                "POST",
                &app.route("/python/review"),
                json!({"mode":"auto-accept"})
            )
            .await
            .0,
            400
        );
        assert_eq!(
            app.request(
                "POST",
                &app.route("/python/review"),
                json!({"id":change["id"],"accepted":accept,"error":if accept { None } else { Some("Human rejects this summary's premise") }})
            )
            .await
            .0,
            200
        );
        let turn = app.wait(&turn_id).await;
        assert_eq!(turn["status"], "ok");
        assert_eq!(turn["backend"], "python");
        assert_eq!(turn["provider"], "scripted");
        assert!(
            turn["answer"]
                .as_str()
                .unwrap()
                .starts_with("Checks unrun.")
        );
        assert_eq!(turn["usage"]["input_tokens"], 14);
        assert_eq!(app.root.join("proposal.md").exists(), accept);
        let mut events = vec![];
        tokio::time::timeout(Duration::from_secs(10), async {
            while let Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(data))) =
                ws.next().await
            {
                if data.first() != Some(&1) {
                    continue;
                }
                let frame: Value = serde_json::from_slice(&data[1..]).unwrap();
                if frame["run_id"] != turn_id {
                    continue;
                }
                if frame["status"].is_string() {
                    assert_eq!(frame["status"], "ok");
                    break;
                }
                events.push(frame["event"]["kind"].as_str().unwrap().to_string());
            }
        })
        .await
        .unwrap();
        assert!(events.contains(&"tool_started".into()));
        assert!(events.contains(&"tool_finished".into()));
        assert!(events.contains(&"done".into()));
        let child_id = app
            .send("Continue from that proposal", Some(&turn_id))
            .await;
        let child = app.wait(&child_id).await;
        assert_eq!(child["session"], turn["session"]);
        assert_eq!(child["parent_id"], turn_id);
        let requests = model.requests.lock().unwrap().clone();
        assert!(
            requests
                .last()
                .unwrap()
                .iter()
                .any(|m| m.content == "Propose a note")
        );
        assert!(
            requests
                .last()
                .unwrap()
                .iter()
                .any(|m| m.content.contains("Stopped without verification."))
        );
        assert!(
            requests
                .last()
                .unwrap()
                .iter()
                .any(|m| m.content.contains("Editor context to retain"))
        );
        drop(requests);
        let path = app.root.join(turn["session"].as_str().unwrap());
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains("python-edit-review"));
        assert!(source.contains("python-model-response"));
        if !accept {
            assert!(source.contains("Human rejects this summary's premise"));
        }
        let writes = hickory_agent::context::derive_from_session(path.to_str().unwrap(), &source);
        assert_eq!(
            writes.iter().any(
                |write| write.file == "proposal.md" && write.scope == "native-message-envelope"
            ),
            accept
        );
        let timeline = hickory_agent::audit::timeline(&source);
        assert!(timeline.iter().any(|e| e["kind"] == "assumption"));
        let restarted = prepare(ServeOptions {
            target: app.root.clone(),
            port: 0,
            params: vec![],
            executor: ExecutorChoice::Local,
            key_store_path: Some(app.root.join("keys.json")),
            ui_settings_path: Some(app.root.join("ui.json")),
        })
        .await
        .unwrap();
        let listing = hickory_cli::serve::agent::list_turns(
            axum::extract::State(restarted.state),
            axum::extract::Path(app.doc.clone()),
        )
        .await
        .0;
        assert_eq!(listing["backend"], "python");
        let recovered = listing["turns"].as_array().unwrap();
        assert_eq!(recovered.len(), 2);
        assert!(
            recovered
                .iter()
                .all(|t| t["backend"] == "python" && t["status"] == "ok")
        );
        assert!(
            recovered
                .iter()
                .all(|t| t["answer"].as_str().unwrap().starts_with("Checks unrun."))
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn stop_cancels_a_pending_python_review_and_keeps_the_audit() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    app.state.agent.set_llm_override(Model::new([
        json!({"python":"tool('create_doc', {'path':'cancelled.md','input':'not approved'})"}),
    ]));
    let id = app.send("Wait for review", None).await;
    app.pending().await;
    assert_eq!(
        app.request("POST", &app.route("/stop"), json!({})).await.0,
        200
    );
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "stopped");
    assert!(!app.root.join("cancelled.md").exists());
    let source = std::fs::read_to_string(app.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(source.contains("python-runtime-failure"));
    assert!(source.contains("stopped by you"));
    let (_, state) = app.request("GET", &app.route("/python"), Value::Null).await;
    assert!(
        state["edits"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["status"] != "pending")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn accepting_a_stale_proposal_preserves_live_human_typing() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    app.state.agent.set_llm_override(Model::new([
        json!({"python":"tool('read_doc', {'doc':'note.md'})"}),
        json!({"python":"tool('edit_doc', {'doc':'note.md','after':'^','input':'AI prefix'})"}),
        json!({"done":"The source changed; edit refused."}),
    ]));
    let id = app.send("Suggest a prefix", None).await;
    let proposal = app.pending().await;
    app.state
        .rooms
        .apply_external_source(&app.doc, "New human input while reviewing\n")
        .await;
    assert_eq!(
        app.request(
            "POST",
            &app.route("/python/review"),
            json!({"id":proposal["id"],"accepted":true})
        )
        .await
        .0,
        200
    );
    assert_eq!(app.wait(&id).await["status"], "ok");
    assert_eq!(
        std::fs::read_to_string(app.root.join("note.md")).unwrap(),
        "New human input while reviewing\n"
    );
    assert_eq!(
        app.state.rooms.get(&app.doc).await.unwrap().text().await,
        "New human input while reviewing\n"
    );
    let (_, state) = app.request("GET", &app.route("/python"), Value::Null).await;
    assert!(
        state["edits"]["changes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["status"] == "failed")
    );
}
