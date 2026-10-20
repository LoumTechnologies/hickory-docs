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

const SUBMIT: &str = r#"import json
s = json.loads(tool('changeset_status', {})['content'][0]['text'])
tool('submit_changeset', {'boundary':s['boundary'], 'expected':s['expected'], 'entry':s['entry'], 'sources':{s['entry']:s['template']}})"#;

struct Model {
    replies: Mutex<VecDeque<String>>,
    requests: Mutex<Vec<Vec<Message>>>,
}
impl Model {
    fn new(replies: impl IntoIterator<Item = Value>) -> Arc<Self> {
        Arc::new(Self {
            replies: Mutex::new(
                replies
                    .into_iter()
                    .flat_map(|v| {
                        let mut actions = vec![];
                        if v.get("done").is_some() {
                            actions.push(json!({"python":SUBMIT}).to_string());
                        }
                        actions.push(v.to_string());
                        actions
                    })
                    .collect(),
            ),
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
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> anyhow::Result<hickory_agent::ChatStream> {
        let (raw, usage) = self.complete_with_usage(messages).await?;
        if raw == json!({"pending_stream":true}).to_string() {
            return futures::future::pending().await;
        }
        Ok(Box::pin(futures::stream::iter([
            Ok(hickory_agent::ChatChunk::reasoning("fixture reasoning")),
            Ok(hickory_agent::ChatChunk::text(raw)),
            Ok(hickory_agent::ChatChunk {
                usage: Some(usage),
                finish_reason: Some("stop".into()),
                ..Default::default()
            }),
        ])))
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
        (status, response.json().await.unwrap_or(Value::Null))
    }
    async fn configure(&self, python: &str) {
        let (status, value) = self
            .request("PUT", "/api/agents/python", json!({"python":python}))
            .await;
        assert_eq!(status, 200, "{value}");
        assert_eq!(value["available"], true);
    }
    async fn send(&self, prompt: &str, parent: Option<&str>) -> String {
        let (status,value) = self.request("POST", &self.route(""), json!({"backend":"python","prompt":prompt,"parent_id":parent,"context":{"live":true,"buffers":[{"id":"note-editor","name":"Meeting note","path":"note.md","content":"Editor context to retain","focused":true}]}})).await;
        assert_eq!(status, 202, "{value}");
        value["session_id"].as_str().unwrap().into()
    }
    async fn pending(&self) -> Value {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                self.refresh_capture().await;
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
    async fn refresh_capture(&self) {
        let (_, snapshot) = self
            .request("GET", &self.route("/python"), Value::Null)
            .await;
        if let Some(id) = snapshot["edits"]["captureRequest"].as_str() {
            if let Some(lens) = self.doc.strip_prefix("lens:") {
                let (_, view) = self.request("GET", &format!("/api/representations/{lens}"), Value::Null).await;
                self.request("POST", &self.route("/python/review"), json!({"capture_request":id,"context":{"live":true,"buffers":[{
                    "id":self.doc,"name":"Literate fixture","path":null,"content":view["source"],"focused":true
                }]}})).await;
                return;
            }
            let content = self.state.rooms.get(&self.doc).await.unwrap().text().await;
            self.request("POST", &self.route("/python/review"), json!({"capture_request":id,"context":{"live":true,"buffers":[{
                "id":"note-editor","name":"Meeting note","path":"note.md","content":content,"focused":true
            }]}})).await;
        }
    }
    async fn wait(&self, id: &str) -> Value {
        tokio::time::timeout(Duration::from_secs(20), async {
            loop {
                self.refresh_capture().await;
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

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
// Real confined Python + HTTP broker, scripted provider and UI endpoint.
// The separate component tests witness the actual shared UI controller.
#[tokio::test]
async fn python_agent_uses_originating_window_ui_tools_with_retained_context() {
    ui_transport_case(false).await;
}

#[tokio::test]
async fn python_literate_view_uses_ui_without_creating_a_synthetic_document() {
    ui_transport_case(true).await;
}

async fn ui_transport_case(lens: bool) {
    let Some(python) = python() else {
        return;
    };
    let mut app = App::start().await;
    let mut buffers = json!([]);
    if lens {
        std::fs::write(app.root.join("code.py"), "print(42)\n").unwrap();
        let (status, view) = app.request("POST", "/api/representations", json!({"backing":{"kind":"files","paths":["code.py"]}})).await;
        assert_eq!(status, 200, "{view}");
        app.doc = format!("lens:{}", view["id"].as_str().unwrap());
        buffers = json!([{"id":app.doc,"name":"Literate fixture","path":null,"content":view["source"],"focused":true}]);
    }
    app.configure(&python).await;
    let model = Model::new([
        json!({"python":"import json\no = json.loads(tool('ui_observe', {'parent':'fixture-rail'})['content'][0]['text'])\nprint(tool('ui_act', {'handle':o['actions'][0]['handle'], 'arguments':{}, 'request_id':'one-run'}))"}),
        json!({"done":"Requested a run through the shared UI; runtime success is not established."}),
    ]);
    app.state.agent.set_llm_override(model.clone());
    let (status, scope) = app.request("POST", "/api/v1/ui/windows", json!({})).await;
    assert_eq!(status, 200, "{scope}");
    let window = scope["window"].as_str().unwrap();
    let (status, sent) = app.request("POST", &app.route(""), json!({"backend":"python","prompt":"Discover and run the registered cell","context":{"live":lens,"ui_window":window,"buffers":buffers}})).await;
    assert_eq!(status, 202, "{sent}");
    let turn = sent["session_id"].as_str().unwrap();
    let poll_route = format!("/api/v1/ui/windows/{window}/requests");
    let reply_route = format!("/api/v1/ui/windows/{window}/reply");
    for (method, result) in [
        (
            "ui_observe",
            json!({"actions":[{"label":"Run fixture","handle":"action:fixture-current"}],"coverage":"scripted UI endpoint; component parity tested separately"}),
        ),
        (
            "ui_act",
            json!({"status":"result","outcome":{"status":"queued","job":"fixture-runtime-job"}}),
        ),
    ] {
        let poll = app.request("GET", &poll_route, Value::Null);
        tokio::pin!(poll);
        let (status, frame) = loop {
            tokio::select! {
                result = &mut poll => break result,
                _ = tokio::time::sleep(Duration::from_millis(25)) => app.refresh_capture().await,
            }
        };
        assert_eq!(status, 200, "{frame}");
        if frame["request"].is_null() {
            panic!("missing UI request: {}", app.wait(turn).await);
        }
        let frame = &frame["request"];
        assert_eq!(frame["method"], method, "{frame}");
        assert_eq!(
            frame["actor"],
            json!({"id":format!("python:{turn}"),"kind":"agent"})
        );
        if method == "ui_act" {
            assert_eq!(
                frame["arguments"],
                json!({"handle":"action:fixture-current","arguments":{},"request_id":"one-run"})
            );
        }
        let (status, response) = app
            .request(
                "POST",
                &reply_route,
                json!({"id":frame["id"],"result":result}),
            )
            .await;
        assert_eq!(status, 200, "{response}");
    }
    let outcome = app.wait(turn).await;
    assert_eq!(outcome["status"], "ok", "{outcome}");
    let session = app.root.join(outcome["session"].as_str().unwrap());
    if lens {
        assert!(!session.starts_with(&app.root), "lens sessions belong to workspace storage");
        assert!(!app.root.join(format!(".hick-lens-{}", app.doc.strip_prefix("lens:").unwrap())).exists());
        assert!(!app.root.join("sessions").exists());
    }
    let source = std::fs::read_to_string(session).unwrap();
    assert!(source.contains("python-ui-request"));
    assert!(source.contains("fixture-runtime-job"));
    assert!(source.contains("context_references"));
    let requests = model.requests.lock().unwrap();
    assert!(
        requests[0]
            .iter()
            .any(|m| m.content.contains("ui_observe") && m.content.contains("ui_act"))
    );
    assert!(
        requests
            .iter()
            .flatten()
            .any(|m| m.content.contains("fixture-runtime-job"))
    );
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
    // No runtime is configured and the test executable ships none, so the
    // Settings route reports nothing available and no bundled default.
    let (status, settings) = app.request("GET", "/api/agents/python", Value::Null).await;
    assert_eq!(status, 200);
    assert_eq!(settings["available"], false);
    assert!(settings["bundled"].is_null());
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
        // The retired native review route is not a compatibility endpoint.
        assert_eq!(
            app.request("POST", &app.route("/review"), json!({"mode":"auto-accept"}))
                .await
                .0,
            404
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
                .starts_with("Changeset verified")
        );
        assert_eq!(turn["usage"]["input_tokens"], 21);
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
        assert!(events.contains(&"reasoning".into()));
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
        // Keep submitted messages for exact manifest comparison below.
        let path = app.root.join(turn["session"].as_str().unwrap());
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(source.contains("python-edit-review"));
        assert!(source.contains("python-model-response"));
        // Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md
        let contexts = |kind: &str| -> Vec<Value> {
            let prefix = format!("<hick:context kind=\"{kind}\">");
            source
                .lines()
                .filter_map(|line| {
                    line.strip_prefix(&prefix)
                        .and_then(|s| s.strip_suffix("</hick:context>"))
                })
                .map(|s| serde_json::from_str(s).unwrap())
                .collect()
        };
        let manifests = contexts("python-context-manifest");
        let deliveries = contexts("python-context-delivery");
        assert_eq!(manifests.len(), requests.len());
        for (manifest, submitted) in manifests.iter().zip(&requests) {
            let mut validated = manifest.clone();
            validated.as_object_mut().unwrap().remove("recorded_at");
            hick_context_protocol::validate_bundle(&validated).unwrap();
            let parts = manifest["request"]["parts"].as_array().unwrap();
            assert_eq!(parts.len(), submitted.len());
            for (part, message) in parts.iter().zip(submitted) {
                assert_eq!(part["text"], message.content);
                assert_eq!(part["role"], serde_json::to_value(message.role).unwrap());
            }
            let states: Vec<_> = deliveries
                .iter()
                .filter(|d| d["request"] == manifest["request"]["id"])
                .map(|d| d["state"].as_str().unwrap())
                .collect();
            assert_eq!(
                states,
                ["prepared", "transport_started", "provider_confirmed"]
            );
        }
        assert!(
            manifests
                .iter()
                .any(|m| m["productions"].as_array().unwrap().len() > 0)
        );
        assert!(source.contains("<hick:reasoning>"));
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
        assert!(recovered.iter().all(|t| {
            t["answer"]
                .as_str()
                .unwrap()
                .starts_with("Changeset verified")
        }));
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

// Guarantee: docs/guarantees/agent/python-requests-have-validated-context.md
#[tokio::test(flavor = "multi_thread")]
async fn cancelling_active_inference_records_uncertain_delivery() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    let model = Model::new([json!({"pending_stream":true})]);
    app.state.agent.set_llm_override(model.clone());
    let id = app.send("Cancel during transport", None).await;
    tokio::time::timeout(Duration::from_secs(20), async {
        loop {
            app.refresh_capture().await;
            if !model.requests.lock().unwrap().is_empty() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        app.request("POST", &app.route("/stop"), json!({})).await.0,
        200
    );
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "stopped");
    let source = std::fs::read_to_string(app.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(source.contains("cancelled_uncertain"));
    assert!(!source.contains("provider_confirmed"));
    assert_eq!(model.requests.lock().unwrap().len(), 1);
}

// Guarantee: docs/guarantees/authoring/review-investigations-link-observed-agent-work.md
#[tokio::test(flavor = "multi_thread")]
async fn review_investigation_links_real_response_refused_edits_and_cancellation() {
    use sha2::{Digest, Sha256};
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    let (_, initial) = app.request("GET", "/api/reviews", Value::Null).await;
    let scope = initial["scope"].clone();
    let (status, root) = app
        .request(
            "POST",
            "/api/reviews",
            json!({"scope":scope,"target":{
        "kind":"source-relationship","path":"note.md","quote":"Original human input",
        "source_hash":format!("{:x}",Sha256::digest("Original human input\n")),"lines":[1,1]},
        "action":"comment","body":"The generated requirement contradicts this source"}),
        )
        .await;
    assert_eq!(status, 200, "{root}");
    let thread = root["thread"].clone();
    app.state.agent.set_llm_override(Model::new([
        json!({"python":"tool('create_doc', {'path':'refused.md','input':'Unapproved correction'})"}),
        json!({"done":"The proposed edit was refused; human review remains open."}),
    ]));
    let id = app.send("Investigate the source objection", None).await;
    let change = app.pending().await;
    app.request(
        "POST",
        &app.route("/python/review"),
        json!({"id":change["id"],"accepted":false,"error":"Human refuses this correction"}),
    )
    .await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok");
    assert!(!app.root.join("refused.md").exists());
    let link = json!({"version":1,"id":"investigation-1","conversation":app.doc,"turn":id,"state":"running"});
    assert_eq!(
        app.request(
            "POST",
            "/api/reviews",
            json!({"scope":scope,"thread":thread,"action":"investigate","body":link.to_string()})
        )
        .await
        .0,
        200
    );
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["investigations"][0]["state"], "responded");
    assert_eq!(history["investigations"][0]["response"], turn["answer"]);
    assert_eq!(history["investigations"][0]["session"], turn["session"]);
    assert!(
        !history["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["action"] == "resolve")
    );
    std::fs::write(
        app.root.join("note.md"),
        "Original human input\nOriginal human input\n",
    )
    .unwrap();
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["targets"][0]["current_state"], "ambiguous");
    for action in ["resolve", "reopen"] {
        assert_eq!(app.request("POST", "/api/reviews", json!({"scope":scope,"thread":thread,"action":action,"body":"Human inspected the recorded evidence"})).await.0, 200);
    }
    app.state
        .agent
        .set_llm_override(Model::new([json!({"pending_stream":true})]));
    let cancelled = app.send("Cancelled investigation", Some(&id)).await;
    let link = json!({"version":1,"id":"investigation-2","conversation":app.doc,"turn":cancelled,"state":"running"});
    app.request(
        "POST",
        "/api/reviews",
        json!({"scope":scope,"thread":thread,"action":"investigate","body":link.to_string()}),
    )
    .await;
    app.request("POST", &app.route("/stop"), json!({})).await;
    app.wait(&cancelled).await;
    let (_, history) = app.request("GET", "/api/reviews", Value::Null).await;
    assert_eq!(history["investigations"][1]["state"], "cancelled");
}

// Guarantee: docs/guarantees/agent/explicit-context-updates.md
#[tokio::test]
async fn explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    std::fs::write(app.root.join("AGENTS.md"), "Original instructions\n").unwrap();
    app.configure(&python).await;
    let model = Model::new([
        json!({"python":"tool('read_file', {'path':'AGENTS.md'})"}),
        json!({"done":"Read source"}),
        json!({"done":"Continue"}),
    ]);
    app.state.agent.set_llm_override(model.clone());
    let first = app.send("Read instructions", None).await;
    let outcome = app.wait(&first).await;
    assert_eq!(outcome["status"], "ok", "{outcome}");
    std::fs::write(
        app.root.join("AGENTS.md"),
        "Changed instructions after pull\n",
    )
    .unwrap();
    let context = json!({"buffers":[{"id":"note-editor","name":"Meeting note","path":"note.md","content":"Unsaved change","focused":true}]});
    let body = json!({"branch":first,"context":context});
    let (status, view) = app
        .request("POST", &app.route("/python/context"), body.clone())
        .await;
    assert_eq!(status, 200, "{view}");
    let items = view["items"].as_array().unwrap();
    assert!(
        items
            .iter()
            .any(|i| i["path"] == "AGENTS.md" && i["status"] == "changed_observed_passage"),
        "{view}"
    );
    assert!(
        items
            .iter()
            .any(|i| i["path"] == "note-editor" && i["after"] == "Unsaved change")
    );
    let selected: Vec<_> = items
        .iter()
        .filter(|i| i["eligible"] == true)
        .map(|i| i["reference"].clone())
        .collect();
    let revisions: serde_json::Map<String, Value> = items
        .iter()
        .map(|i| {
            (
                i["reference"].as_str().unwrap().into(),
                i["revision"].clone(),
            )
        })
        .collect();
    let (status, queued) = app
        .request(
            "POST",
            &app.route("/python/context"),
            json!({"branch":first,"context":context,"selected":selected,"revisions":revisions}),
        )
        .await;
    assert_eq!(status, 200, "{queued}");
    assert!(
        queued["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "queued")
    );
    assert_eq!(
        std::fs::read_to_string(app.root.join("note.md")).unwrap(),
        "Original human input\n"
    );
    std::fs::write(app.root.join("AGENTS.md"), "Changed again while queued\n").unwrap();
    let child = app.send("Continue", Some(&first)).await;
    let outcome = app.wait(&child).await;
    assert_eq!(outcome["status"], "ok", "{outcome}");
    let (_, delivered) = app
        .request(
            "POST",
            &app.route("/python/context"),
            json!({"branch":child,"context":context}),
        )
        .await;
    assert!(
        delivered["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "provider_confirmed"),
        "{delivered}"
    );
    let requests = model.requests.lock().unwrap();
    assert!(
        requests
            .last()
            .unwrap()
            .iter()
            .any(|m| m.content.contains("Changed instructions after pull"))
    );
    assert!(
        !requests
            .last()
            .unwrap()
            .iter()
            .any(|m| m.content.contains("Changed again while queued"))
    );
    let (_, parent) = app
        .request("POST", &app.route("/python/context"), body)
        .await;
    assert!(
        parent["updates"]
            .as_array()
            .unwrap()
            .iter()
            .all(|u| u["state"] == "queued")
    );
}
