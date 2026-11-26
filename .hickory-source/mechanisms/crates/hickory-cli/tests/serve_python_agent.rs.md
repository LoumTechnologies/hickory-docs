<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hickory-cli/tests/serve_python_agent.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-08f0cd3133cc"></a>
## `crates/hickory-cli/tests/serve_python_agent.rs`

Related contracts: [A desktop app bundles its own Python agent](../../../../guarantees/agent/a-desktop-app-bundles-its-python-agent/contract.md), [Read the retained branch of an agent conversation](../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [Explicit context updates preserve confirmed history](../../../../guarantees/agent/explicit-context-updates/contract.md), [Separate model snippets from trusted host operations](../../../../guarantees/agent/python-execution-has-two-boundaries/contract.md), [Python model requests carry validated context manifests](../../../../guarantees/agent/python-requests-have-validated-context/contract.md), [Explicit UI action dispatch checks scope, state, policy and retries](../../../../guarantees/agent/ui-action-dispatch/contract.md), [Review investigations link observed agent work without resolving human review](../../../../guarantees/authoring/review-investigations-link-observed-agent-work/contract.md).

<a id="f-08f0cd3133cc-SUBMIT"></a>
### `SUBMIT`

and docs/guarantees/agent/a-session-is-the-conversation.md.
Real confined workers and HTTP/WS transport; only the paid model is replaced.

<storycode:copy id="f-08f0cd3133cc-SUBMIT">
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

</storycode:copy>
<a id="f-08f0cd3133cc-Model"></a>
### `Model`

<storycode:copy id="f-08f0cd3133cc-Model">
struct Model {
    replies: Mutex<VecDeque<String>>,
    requests: Mutex<Vec<Vec<Message>>>,
}
</storycode:copy>
<a id="f-08f0cd3133cc-impl-Model"></a>
### `impl-Model`

<storycode:copy id="f-08f0cd3133cc-impl-Model">
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
</storycode:copy>
<a id="f-08f0cd3133cc-impl-LlmClient-for-Model"></a>
### `impl-LlmClient for Model`

<storycode:copy id="f-08f0cd3133cc-impl-LlmClient-for-Model">
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
</storycode:copy>
<a id="f-08f0cd3133cc-App"></a>
### `App`

<storycode:copy id="f-08f0cd3133cc-App">
struct App {
    base: String,
    doc: String,
    root: PathBuf,
    state: LocalState,
    _dir: tempfile::TempDir,
    task: tokio::task::JoinHandle<()>,
}
</storycode:copy>
<a id="f-08f0cd3133cc-impl-Drop-for-App"></a>
### `impl-Drop for App`

<storycode:copy id="f-08f0cd3133cc-impl-Drop-for-App">
impl Drop for App {
    fn drop(&mut self) {
        self.task.abort();
    }
}
</storycode:copy>
<a id="f-08f0cd3133cc-impl-App"></a>
### `impl-App`

<storycode:copy id="f-08f0cd3133cc-impl-App">
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
                let (_, view) = self
                    .request("GET", &format!("/api/representations/{lens}"), Value::Null)
                    .await;
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
</storycode:copy>
<a id="f-08f0cd3133cc-python"></a>
### `python`

The runtime helper makes an unavailable confined worker explicit. The Git
witness uses that real worker and HTTP transport with a scripted model. It
rejects repository-selection arguments, then observes branch and literal paths
without an originating UI window. Compare source and index bytes afterward;
the final answer must retain its Markdown without document/project boilerplate.
This witness does not exercise a paid provider or a native frontend.

The assistant-branch witness saves an edited answer through real HTTP without
calling a model, checks idempotent retries and stale-hash/path-collision refusals,
and reopens its immutable recording after restart. With the confined worker
configured, its follow-up must replay the edited text in the assistant role,
never the replaced original answer. The test's explicit non-subscription choice
uses the scripted model transport; it does not relax subscription admission.

<storycode:copy id="f-08f0cd3133cc-python">
fn python() -> Option<String> {
    let value = std::env::var("HICKORY_AGENT_TEST_PYTHON").ok();
    if value.is_none() {
        eprintln!("SKIP real Python UI integration: run scripts/check-python-agent.sh");
    }
    value
}

// Contract: .hickory-source/frontends/contract.md
#[tokio::test(flavor = "multi_thread")]
async fn python_git_status_is_read_only_and_answers_with_bare_markdown() {
    let Some(python) = python() else { return };
    let app = App::start().await;
    app.configure(&python).await;
    let git = |args: &[&str]| {
        let output = std::process::Command::new("git").current_dir(&app.root).args(args).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    };
    git(&["init", "--initial-branch=agent-status"]);
    git(&["add", "note.md"]);
    let index = std::fs::read(app.root.join(".git/index")).unwrap();
    std::fs::write(app.root.join("untracked space.md"), "Untouched source\n").unwrap();
    let model = Model::new([
        json!({"tool":"git_status", "arguments":{"path":"../outside"}}),
        json!({"tool":"git_status", "arguments":{}}),
        json!({"done":"## Repository status\n\n- Staged note.md.\n- Untracked source remains on disk."}),
    ]);
    app.state.agent.set_llm_override(model.clone());
    let id = app.send("Read Git status without changing files", None).await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    let requests = model.requests.lock().unwrap();
    let observed: String = requests.iter().flatten().map(|message| message.content.as_str()).collect();
    assert!(observed.contains("git_status accepts an empty argument object"), "Repository-selection arguments must be refused");
    assert!(observed.contains("agent-status") && observed.contains("untracked space.md"), "The actual Git receipt must reach the model");
    assert!(!turn["answer"].as_str().unwrap().contains("<hick:doc"));
    assert!(!turn["answer"].as_str().unwrap().contains("<hick:project"));
    assert_eq!(std::fs::read(app.root.join(".git/index")).unwrap(), index);
    assert_eq!(std::fs::read_to_string(app.root.join("untracked space.md")).unwrap(), "Untouched source\n");
    assert_eq!(std::fs::read_to_string(app.root.join("note.md")).unwrap(), "Original human input\n");
}

// Contract: .hickory-source/frontends/contract.md
#[tokio::test(flavor = "multi_thread")]
async fn assistant_branch_saves_an_immutable_idempotent_human_answer() {
    use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
    use sha2::Digest;
    let app = App::start().await;
    let original = app.root.join("sessions/original.md");
    let record = HickSessionLog::create_for(&original, Some(&app.root.join("note.md"))).unwrap();
    record.record(SessionEvent::UserTurn {text:"Original prompt",turn:"original",parent:None,parent_is_user:false,provider:"codex-subscription",model:"original-model"});
    record.record(SessionEvent::Context {kind:"python-runtime",data:&json!({"backend":"python"})});
    record.record(SessionEvent::Assistant {prose:"Original answer",action:None,reasoning:None});
    record.record(SessionEvent::Context {kind:"acp-turn-status",data:&json!({"turn":"original","status":"ok"})});
    record.record(SessionEvent::End); record.check().unwrap();
    let bytes = std::fs::read(&original).unwrap();
    let model = Model::new([]); app.state.agent.set_llm_override(model.clone());
    let body = json!({"turn":"original","source_hash":hex::encode(sha2::Sha256::digest(b"Original answer")),"answer":"Edited answer\n\n```python\nprint(2)\n```","request_id":"a".repeat(32)});
    let (status, edited) = app.request("POST", &app.route("/branch"), body.clone()).await;
    assert_eq!(status,200,"{edited}");
    assert_eq!(edited["prompt"],"Original prompt"); assert_eq!(edited["answer"],body["answer"]);
    assert_eq!(edited["parent_id"],Value::Null); assert_eq!(edited["provider"],"human-edited");
    assert_eq!(edited["usage"]["input_tokens"],0);
    let (status, repeated) = app.request("POST", &app.route("/branch"), body.clone()).await;
    assert_eq!(status,200); assert_eq!(repeated["id"],edited["id"]);
    let mut stale = body.clone(); stale["source_hash"]=json!("0".repeat(64));
    assert_eq!(app.request("POST", &app.route("/branch"), stale).await.0,409);
    let mut conflicting = body.clone(); conflicting["answer"]=json!("Different answer");
    assert_eq!(app.request("POST", &app.route("/branch"), conflicting).await.0,409);
    #[cfg(unix)] {
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("must-not-be-created");
        let collision = app.root.join("sessions/edited-bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.md");
        std::os::unix::fs::symlink(&target, &collision).unwrap();
        let mut linked = body.clone(); linked["request_id"]=json!("b".repeat(32));
        assert_eq!(app.request("POST", &app.route("/branch"), linked).await.0,409);
        assert!(!target.exists()); assert!(std::fs::symlink_metadata(&collision).unwrap().file_type().is_symlink());
        std::fs::remove_file(collision).unwrap();
    }
    assert_eq!(std::fs::read(&original).unwrap(),bytes); assert!(model.requests.lock().unwrap().is_empty());
    let source = std::fs::read_to_string(app.root.join(edited["session"].as_str().unwrap())).unwrap();
    assert!(source.contains("human-edited") && source.contains("\"verification\":\"none\""));
    assert!(!source.contains("python-verified-changeset"));
    let restarted = prepare(ServeOptions {target:app.root.clone(),port:0,params:vec![],executor:ExecutorChoice::Local,key_store_path:Some(app.root.join("keys.json")),ui_settings_path:Some(app.root.join("ui.json"))}).await.unwrap();
    let history = hickory_cli::serve::agent::list_turns(axum::extract::State(restarted.state),axum::extract::Path(app.doc.clone())).await.0;
    assert_eq!(history["provider"],"codex-subscription"); assert_eq!(history["model"],"original-model");
    let turns = history["turns"].as_array().unwrap(); assert_eq!(turns.len(),2);
    let restored = turns.iter().find(|turn|turn["id"]==edited["id"]).unwrap();
    assert_eq!(restored["answer"],body["answer"]); assert_eq!(restored["status"],"ok"); assert_eq!(restored["backend"],"python");
    if let Ok(python) = std::env::var("HICKORY_AGENT_TEST_PYTHON") {
        app.configure(&python).await;
        let continuation = Model::new([json!({"done":"Continued from the edited assistant answer."})]);
        app.state.agent.set_llm_override(continuation.clone());
        let (status, started) = app.request("POST", &app.route(""), json!({"backend":"python","prompt":"Follow up","parent_id":edited["id"],"provider":"anthropic","model":""})).await;
        assert_eq!(status,202,"{started}");
        let next = started["session_id"].as_str().unwrap().to_string();
        let completed = app.wait(&next).await; assert_eq!(completed["status"],"ok","{completed}");
        let requests = continuation.requests.lock().unwrap();
        assert!(requests.first().unwrap().iter().any(|message| matches!(message.role,hickory_agent::Role::Assistant) && message.content == body["answer"].as_str().unwrap()));
        assert!(!requests.first().unwrap().iter().any(|message| matches!(message.role,hickory_agent::Role::Assistant) && message.content == "Original answer"));
        assert_eq!(std::fs::read(&original).unwrap(),bytes);
    }
}

</storycode:copy>
[Read the test `python_agent_uses_originating_window_ui_tools_with_retained_context` beside its requirement](../../../../guarantees/agent/ui-action-dispatch/contract.md#f-08f0cd3133cc-python_agent_uses_originating_window_ui_tools_with_retained_context).
<a id="f-08f0cd3133cc-python_literate_view_uses_ui_without_creating_a_synthetic_document"></a>
### `python_literate_view_uses_ui_without_creating_a_synthetic_document`

<storycode:copy id="f-08f0cd3133cc-python_literate_view_uses_ui_without_creating_a_synthetic_document">
#[tokio::test]
async fn python_literate_view_uses_ui_without_creating_a_synthetic_document() {
    ui_transport_case(true, false).await;
}

</storycode:copy>
<a id="f-08f0cd3133cc-python_agent_delivers_captured_ui_images_as_selected_provider_content"></a>
### `python_agent_delivers_captured_ui_images_as_selected_provider_content`

<storycode:copy id="f-08f0cd3133cc-python_agent_delivers_captured_ui_images_as_selected_provider_content">
#[tokio::test]
async fn python_agent_delivers_captured_ui_images_as_selected_provider_content() {
    ui_transport_case(false, true).await;
}

</storycode:copy>
<a id="f-08f0cd3133cc-ui_transport_case"></a>
### `ui_transport_case`

<storycode:copy id="f-08f0cd3133cc-ui_transport_case">
async fn ui_transport_case(lens: bool, image: bool) {
    let Some(python) = python() else {
        return;
    };
    let mut app = App::start().await;
    let mut buffers = json!([]);
    if lens {
        std::fs::write(app.root.join("code.py"), "print(42)\n").unwrap();
        let (status, view) = app
            .request(
                "POST",
                "/api/representations",
                json!({"backing":{"kind":"files","paths":["code.py"]}}),
            )
            .await;
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
    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==";
    use base64::Engine;
    let pixels = base64::engine::general_purpose::STANDARD
        .decode(PNG)
        .unwrap();
    let action_reply = if image {
        json!({"status":"result","outcome":{"status":"completed","value":{"path":"fixture.png","ui_image":{"media_type":"image/png","data":PNG,"digest":hick_context_protocol::digest(&pixels),"byte_length":pixels.len()}}}})
    } else {
        json!({"status":"result","outcome":{"status":"queued","job":"fixture-runtime-job"}})
    };
    for (method, result) in [
        (
            "ui_observe",
            json!({"actions":[{"label":"Run fixture","handle":"action:fixture-current"}],"coverage":"scripted UI endpoint; component parity tested separately"}),
        ),
        ("ui_act", action_reply),
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
        assert!(
            !session.starts_with(&app.root),
            "lens sessions belong to workspace storage"
        );
        assert!(
            !app.root
                .join(format!(
                    ".hick-lens-{}",
                    app.doc.strip_prefix("lens:").unwrap()
                ))
                .exists()
        );
        assert!(!app.root.join("sessions").exists());
    }
    let source = std::fs::read_to_string(session).unwrap();
    assert!(source.contains("python-ui-request"));
    if !image {
        assert!(source.contains("fixture-runtime-job"));
    } else {
        assert!(source.contains("captured-ui-image"));
        assert!(source.contains("artifact"));
    }
    assert!(source.contains("context_references"));
    let requests = model.requests.lock().unwrap();
    assert!(
        requests[0]
            .iter()
            .any(|m| m.content.contains("ui_observe") && m.content.contains("ui_act"))
    );
    if image {
        assert!(requests.iter().skip(1).flatten().any(|m| {
            m.images
                .iter()
                .any(|i| i.media_type == "image/png" && i.data == PNG)
        }));
        assert!(
            !requests[0].iter().any(|m| !m.images.is_empty()),
            "image is selected only after the explicit UI read"
        );
    } else {
        assert!(
            requests
                .iter()
                .flatten()
                .any(|m| m.content.contains("fixture-runtime-job"))
        );
    }
}

</storycode:copy>
<a id="f-08f0cd3133cc-runtime_setup_refuses_invalid_installations_and_reserved_acp_ids"></a>
### `runtime_setup_refuses_invalid_installations_and_reserved_acp_ids`

<storycode:copy id="f-08f0cd3133cc-runtime_setup_refuses_invalid_installations_and_reserved_acp_ids">
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

</storycode:copy>
<a id="f-08f0cd3133cc-python_turn_requires_review_streams_receipts_and_rehydrates_its_branch"></a>
### `python_turn_requires_review_streams_receipts_and_rehydrates_its_branch`

The initial proposal waits with no disk write. Authorizing future proposals during
that wait records the policy without resolving the current one. Both Accept and
Reject retain their existing results, and continuing the session keeps the policy.

Completion and restart must retain the accepted literate construction as the answer, rather than a model summary. The existing approval, actual-file and streaming-receipt assertions remain in force.

<storycode:copy id="f-08f0cd3133cc-python_turn_requires_review_streams_receipts_and_rehydrates_its_branch">
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
            json!({"tool":"create_doc","arguments":{"path":"second.md","input":"Approved by conversation policy"}}),
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
            200
        );
        assert!(!app.root.join("proposal.md").exists());
        assert_eq!(
            app.request("GET", &app.route("/python"), json!({})).await.1["edits"]["mode"],
            "auto-accept"
        );
        use sha2::{Digest, Sha256};
        let digest = |text: &str| format!("{:x}", Sha256::digest(text.as_bytes()));
        let exact_route = app.route("/python/review-exact");
        let mut decision = json!({"turn":turn_id,"id":change["id"],"accepted":true,
            "before":digest(""),"after":digest("obsolete preview")});
        assert_eq!(app.request("POST", &exact_route, decision.clone()).await.0, 409);
        assert!(!app.root.join("proposal.md").exists());
        if accept {
            decision["after"] = json!(digest("AI proposal\n"));
            assert_eq!(app.request("POST", &exact_route, decision).await.0, 200);
        } else {
            assert_eq!(
                app.request(
                    "POST",
                    &app.route("/python/review"),
                    json!({"id":change["id"],"accepted":false,"error":"Human rejects this summary's premise"})
                )
                .await
                .0,
                200
            );
        }
        let turn = app.wait(&turn_id).await;
        assert_eq!(turn["status"], "ok");
        assert_eq!(turn["backend"], "python");
        assert_eq!(turn["provider"], "scripted");
        assert!(!turn["answer"].as_str().unwrap().contains("<hick:doc"));
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
        // Reopening restores policy; a new thread cannot inherit prior approvals.
        let fresh = app
            .request("GET", &app.route("/python?session="), json!({}))
            .await;
        assert_eq!(fresh.1["edits"]["mode"], "review");
        assert_eq!(fresh.1["edits"]["changes"], json!([]));
        let restored = app
            .request(
                "GET",
                &app.route(&format!(
                    "/python?session={}",
                    turn["session"].as_str().unwrap()
                )),
                json!({}),
            )
            .await;
        assert_eq!(restored.0, 200, "{restored:?}");
        assert_eq!(restored.1["edits"]["mode"], "auto-accept");
        assert!(
            app.root.join("second.md").exists(),
            "future proposal proceeds through the authorized policy"
        );
        assert_eq!(
            app.request("GET", &app.route("/python"), json!({})).await.1["edits"]["mode"],
            "auto-accept"
        );
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
                .any(|m| matches!(&m.role, hickory_agent::Role::Assistant)
                    && m.content == turn["answer"].as_str().unwrap())
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
        assert!(
            source.contains("Stopped without verification."),
            "The declaration remains historical session evidence"
        );
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
        // Verification lives in the host's evidence, not in a phrase the model
        // must put in the authored construction.
        let artifacts = contexts("python-verified-changeset");
        let artifact = artifacts.first().expect("host-verified changeset");
        let entry = artifact["entry"].as_str().unwrap();
        assert_eq!(turn["answer"], artifact["sources"][entry]);
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
        assert!(
            recovered
                .iter()
                .all(|t| { !t["answer"].as_str().unwrap().contains("<hick:doc") })
        );
    }
}
</storycode:copy>
<a id="f-08f0cd3133cc-stop_cancels_a_pending_python_review_and_keeps_the_audit"></a>
### `stop_cancels_a_pending_python_review_and_keeps_the_audit`

<storycode:copy id="f-08f0cd3133cc-stop_cancels_a_pending_python_review_and_keeps_the_audit">
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

</storycode:copy>
<a id="f-08f0cd3133cc-accepting_a_stale_proposal_preserves_live_human_typing"></a>
### `accepting_a_stale_proposal_preserves_live_human_typing`

<storycode:copy id="f-08f0cd3133cc-accepting_a_stale_proposal_preserves_live_human_typing">
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

</storycode:copy>
[Read the test `cancelling_active_inference_records_uncertain_delivery` beside its requirement](../../../../guarantees/agent/python-requests-have-validated-context/contract.md#f-08f0cd3133cc-cancelling_active_inference_records_uncertain_delivery).
[Read the test `review_investigation_links_real_response_refused_edits_and_cancellation` beside its requirement](../../../../guarantees/authoring/review-investigations-link-observed-agent-work/contract.md#f-08f0cd3133cc-review_investigation_links_real_response_refused_edits_and_cancellation).
[Read the test `explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary` beside its requirement](../../../../guarantees/agent/explicit-context-updates/contract.md#f-08f0cd3133cc-explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary).

### Keep a large compiler artifact out of a live agent turn

This uses real HTTP, the confined Python worker and a scripted provider that waits
in inference. A sparse 9 GiB target file must not delay the first model request,
cancellation, or inflate retained session evidence. Removing its explicit exclusion
must fail the next turn before any model call. The current-thread runtime also
exposes blocking startup work that would starve the HTTP server.

<storycode:copy id="bounded-workspace-agent-http">
// Guarantee: .hickory-source/guarantees/agent/changeset-completion-is-negotiated-and-checked/contract.md
// Guarantee: .hickory-source/guarantees/agent/a-running-agent-can-be-stopped/contract.md
#[tokio::test]
async fn bounded_workspace_capture_reaches_inference_and_stops_with_large_excluded_artifact() {
    let Some(python) = python() else {
        return;
    };
    let app = App::start().await;
    app.configure(&python).await;
    std::fs::create_dir(app.root.join("target")).unwrap();
    std::fs::File::create(app.root.join("target/huge.bin"))
        .unwrap()
        .set_len(9 * 1024 * 1024 * 1024)
        .unwrap();
    std::fs::write(
        app.root.join("project.md"),
        "<hick:project exclude=\"target/\" />",
    )
    .unwrap();
    let model = Model::new([json!({"pending_stream":true})]);
    app.state.agent.set_llm_override(model.clone());
    let id = app.send("Wait in inference", None).await;
    tokio::time::timeout(Duration::from_secs(10), async {
        while model.requests.lock().unwrap().is_empty() {
            app.refresh_capture().await;
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    })
    .await
    .expect("excluded artifacts must not block inference");
    assert_eq!(
        app.request("POST", &app.route("/stop"), json!({})).await.0,
        200
    );
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "stopped");
    let session = app.root.join(turn["session"].as_str().unwrap());
    assert!(std::fs::metadata(session).unwrap().len() < 1_000_000);
    let (status, history) = app
        .request("GET", &app.route(&format!("/projects/{id}")), Value::Null)
        .await;
    assert_eq!(status, 200, "{history}");
    assert_eq!(history["workspace"]["exclude"], json!(["target/"]));
    assert!(
        history["starting"]["entries"]
            .get("target/huge.bin")
            .is_none()
    );
    std::fs::write(app.root.join("project.md"), "<hick:project />").unwrap();
    let id = app.send("Refuse oversized boundary", None).await;
    let turn = app.wait(&id).await;
    assert_eq!(turn["status"], "error");
    assert!(
        turn["error"].as_str().unwrap().contains("byte budget"),
        "{turn}"
    );
    assert_eq!(model.requests.lock().unwrap().len(), 1);
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/serve_python_agent.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/serve_python_agent.rs"><storycode:paste select="#f-08f0cd3133cc-SUBMIT"/><storycode:paste select="#f-08f0cd3133cc-Model"/><storycode:paste select="#f-08f0cd3133cc-impl-Model"/><storycode:paste select="#f-08f0cd3133cc-impl-LlmClient-for-Model"/><storycode:paste select="#f-08f0cd3133cc-App"/><storycode:paste select="#f-08f0cd3133cc-impl-Drop-for-App"/><storycode:paste select="#f-08f0cd3133cc-impl-App"/><storycode:paste select="#f-08f0cd3133cc-python"/><storycode:paste select="#f-08f0cd3133cc-python_agent_uses_originating_window_ui_tools_with_retained_context"/><storycode:paste select="#f-08f0cd3133cc-python_literate_view_uses_ui_without_creating_a_synthetic_document"/><storycode:paste select="#f-08f0cd3133cc-python_agent_delivers_captured_ui_images_as_selected_provider_content"/><storycode:paste select="#f-08f0cd3133cc-ui_transport_case"/><storycode:paste select="#f-08f0cd3133cc-runtime_setup_refuses_invalid_installations_and_reserved_acp_ids"/><storycode:paste select="#f-08f0cd3133cc-python_turn_requires_review_streams_receipts_and_rehydrates_its_branch"/><storycode:paste select="#f-08f0cd3133cc-stop_cancels_a_pending_python_review_and_keeps_the_audit"/><storycode:paste select="#f-08f0cd3133cc-accepting_a_stale_proposal_preserves_live_human_typing"/><storycode:paste select="#f-08f0cd3133cc-cancelling_active_inference_records_uncertain_delivery"/><storycode:paste select="#f-08f0cd3133cc-review_investigation_links_real_response_refused_edits_and_cancellation"/><storycode:paste select="#f-08f0cd3133cc-explicit_context_updates_compare_actual_reads_and_deliver_only_on_child_boundary"/><storycode:paste select="#bounded-workspace-agent-http"/></storycode:file>

</storycode:doc>
