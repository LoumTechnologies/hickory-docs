//! The in-app agent, driven the way the desktop app's chat dock drives it:
//! `POST /api/docs/:id/agent`, a WebSocket on the run channel, and
//! `GET /api/docs/:id/agent/turns`.
//!
//! Protects docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.
//!
//! No test here spends a token: the full-turn tests inject a
//! [`hickory_agent::ScriptedLlmClient`] through the serve state's test seam,
//! and the no-key test scrubs the provider variables from this test binary's
//! environment. The other tests never read those variables (the scripted
//! client bypasses provider resolution), so the scrub cannot race them.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt as _;
use hickory_cli::ExecutorChoice;
use hickory_cli::serve::{ServeOptions, prepare};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message as TtMessage;

const DOC: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="reading.md">
# Demo

<hick:copy id="greet">fn greet() { println!("hello"); }
</hick:copy>
<hick:file path="greet.rs"><hick:paste select="#greet" /></hick:file>
</hick:doc>
"##;

struct Session {
    base: String,
    doc_id: String,
    root: PathBuf,
    state: hickory_cli::serve::LocalState,
    _dir: tempfile::TempDir,
}

async fn start() -> Session {
    start_with_folder(false).await
}

async fn start_with_folder(folder: bool) -> Session {
    start_session(folder, false).await
}

async fn start_session(folder: bool, editor_only: bool) -> Session {
    let dir = tempfile::tempdir().unwrap();
    if !editor_only {
        std::fs::write(dir.path().join("demo.md"), DOC).unwrap();
    }
    let root = dir.path().canonicalize().unwrap();

    let opts = ServeOptions {
        target: if folder || editor_only {
            root.clone()
        } else {
            root.join("demo.md")
        },
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    };
    let prepared = if editor_only {
        hickory_cli::serve::prepare_without_folder(opts).await
    } else {
        prepare(opts).await
    }
    .expect("session prepares");

    let state = prepared.state.clone();
    let doc_id = if editor_only {
        assert!(state.index.entries().is_empty());
        "workspace".into()
    } else {
        state.index.sole().expect("one document").0
    };

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });

    Session {
        base: format!("http://127.0.0.1:{port}"),
        doc_id,
        root,
        state,
        _dir: dir,
    }
}

async fn get(session: &Session, path: &str) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .get(format!("{}{path}", session.base))
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

async fn post(session: &Session, path: &str, body: Value) -> (u16, Value) {
    let resp = reqwest::Client::new()
        .post(format!("{}{path}", session.base))
        .json(&body)
        .send()
        .await
        .unwrap();
    (
        resp.status().as_u16(),
        resp.json().await.unwrap_or(Value::Null),
    )
}

/// A `<hick:next>done</hick:next>` response carrying `summary`.
fn done(summary: &str) -> String {
    format!("<hick:next>done</hick:next>\n\n{summary}")
}

/// Start a turn and wait until the turns listing reports it finished.
async fn run_turn_to_completion(
    session: &Session,
    prompt: &str,
    parent_id: Option<&str>,
) -> (String, Value) {
    let (status, body) = post(
        session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": prompt, "parent_id": parent_id }),
    )
    .await;
    assert_eq!(status, 202, "{body}");
    let turn_id = body["session_id"].as_str().expect("session_id").to_string();
    wait_for_turn(session, &turn_id).await
}

/// With no provider key anywhere in the environment, the agent route answers
/// 503 with a message that STARTS with "agent not available" — the phrase
/// ChatDock matches to render a quiet configuration note instead of a red
/// error — and then names the variables that would fix it.
///
/// Protects docs/guarantees/agent/a-missing-key-degrades-to-a-note.md.
#[tokio::test(flavor = "multi_thread")]
async fn no_key_degrades_to_the_note_never_a_crash() {
    for var in [
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "DEEPSEEK_API_KEY",
        "XAI_API_KEY",
        "OPENROUTER_API_KEY",
        "HICKORY_LLM_PROVIDER",
    ] {
        unsafe { std::env::remove_var(var) };
    }

    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "hello", "parent_id": null }),
    )
    .await;
    assert_eq!(status, 503, "{body}");
    let error = body["error"].as_str().expect("an {{error}} body");
    assert!(
        error.starts_with("agent not available"),
        "the client contract is the leading phrase: {error}"
    );
    assert!(
        error.contains("ANTHROPIC_API_KEY"),
        "the note must say what to set: {error}"
    );

    // Degraded, not broken: the rest of the session still answers.
    let (status, _) = get(&session, "/api/health").await;
    assert_eq!(status, 200);
}

/// One POST is one full agent run: the scripted answer streams on the run
/// channel under `exec_id: "agent"`, the turn lands in the listing in the
/// shape the dock renders, and the session is persisted as a `hick:session`
/// file under `<served folder>/sessions/`.
#[tokio::test(flavor = "multi_thread")]
async fn a_turn_streams_on_the_run_channel_and_persists_a_session_file() {
    let session = start().await;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([done(
            "All done: the document greets the world.",
        )])));

    // The dock's window: a socket on the document's room.
    let url = format!(
        "ws://{}/api/ws?doc=doc:{}",
        session.base.trim_start_matches("http://"),
        session.doc_id
    );
    let (mut ws, _) = tokio_tungstenite::connect_async(url).await.unwrap();

    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "finish the greeting", "parent_id": null }),
    )
    .await;
    assert_eq!(status, 202, "{body}");
    let turn_id = body["session_id"].as_str().unwrap().to_string();

    // Watch the run channel until the terminal status frame.
    let mut saw_token = false;
    let terminal = tokio::time::timeout(Duration::from_secs(30), async {
        while let Some(Ok(msg)) = ws.next().await {
            let TtMessage::Binary(data) = msg else {
                continue;
            };
            if data.first() != Some(&0x01) {
                continue;
            }
            let event: Value = serde_json::from_slice(&data[1..]).unwrap();
            if event["run_id"] != turn_id.as_str() {
                continue;
            }
            if let Some(status) = event["status"].as_str() {
                return status.to_string();
            }
            assert_eq!(event["exec_id"], "agent");
            if event["event"]["kind"] == "token" {
                saw_token = true;
            }
        }
        panic!("the socket closed before the terminal status");
    })
    .await
    .expect("the turn must publish a terminal status");
    assert_eq!(terminal, "ok");
    assert!(saw_token, "the answer must stream as token events");

    // The listing has the finished turn, in the dock's wire shape.
    let (_, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    let turns = listing["turns"].as_array().unwrap();
    assert_eq!(turns.len(), 1);
    let turn = &turns[0];
    assert_eq!(turn["id"], turn_id.as_str());
    assert_eq!(turn["parent_id"], Value::Null);
    assert_eq!(turn["prompt"], "finish the greeting");
    assert_eq!(turn["status"], "ok");
    assert_eq!(turn["error"], Value::Null);
    assert!(
        turn["answer"]
            .as_str()
            .unwrap()
            .contains("greets the world"),
        "{turn}"
    );
    assert!(turn["created_at"].as_str().is_some());

    // Nothing is lost on quit: the run wrote a hick:session document.
    let sessions: Vec<_> = std::fs::read_dir(session.root.join("sessions"))
        .expect("a sessions/ directory exists in the served folder")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|x| x == "md"))
        .collect();
    assert_eq!(sessions.len(), 1, "one turn, one session file");
}

/// The dock's model control: `provider`/`model` posted with a turn are
/// validated, persist for the document, apply to subsequent turns, and come
/// back in the turns listing along with session totals priced per turn.
///
/// Protects
/// docs/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model.md.
#[tokio::test(flavor = "multi_thread")]
async fn model_choice_is_accepted_persisted_and_priced_in_totals() {
    let session = start().await;
    let usage = hickory_agent::Usage {
        input_tokens: 1000,
        cache_creation_input_tokens: 100,
        cache_read_input_tokens: 400,
        output_tokens: 200,
    };
    session.state.agent.set_llm_override(Arc::new(
        hickory_agent::ScriptedLlmClient::with_usages([
            (done("first"), usage),
            (done("second"), usage),
        ])
        .with_model_name("claude-sonnet-5"),
    ));

    // A typo'd provider is refused up front — it must never silently run
    // (or persist) as some other vendor.
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "x", "provider": "opennai" }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    let error = body["error"].as_str().unwrap();
    assert!(error.contains("unknown provider"), "{error}");
    assert!(
        error.contains("openrouter"),
        "the error must list the valid selectors: {error}"
    );

    // An explicit choice rides the POST and is echoed on the turn.
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({
            "prompt": "go",
            "parent_id": null,
            "provider": "openai",
            "model": "claude-sonnet-5",
        }),
    )
    .await;
    assert_eq!(status, 202, "{body}");
    let first_id = body["session_id"].as_str().unwrap().to_string();
    let (_, first) = wait_for_turn(&session, &first_id).await;
    assert_eq!(first["provider"], "openai");
    assert_eq!(first["model"], "claude-sonnet-5");
    assert_eq!(first["usage"]["input_tokens"], 1000);
    assert_eq!(first["usage"]["output_tokens"], 200);

    // The choice persists: the next POST names neither field and still runs
    // (and records) the same provider and model.
    let (_, second) = run_turn_to_completion(&session, "again", Some(&first_id)).await;
    assert_eq!(second["provider"], "openai");
    assert_eq!(second["model"], "claude-sonnet-5");

    // The listing carries the current choice plus totals: tokens summed
    // four ways, and USD priced with each turn's own model
    // (sonnet-5: 1000*3 + 100*1.25*3 + 400*0.1*3 + 200*15 per MTok).
    let (_, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    assert_eq!(listing["provider"], "openai");
    assert_eq!(listing["model"], "claude-sonnet-5");
    let totals = &listing["totals"];
    assert_eq!(totals["input"], 2000);
    assert_eq!(totals["output"], 400);
    assert_eq!(totals["cache_read"], 800);
    assert_eq!(totals["cache_write"], 200);
    let usd = totals["usd"].as_f64().expect("a priced model sums to USD");
    assert!((usd - 2.0 * 0.006495).abs() < 1e-9, "got {usd}");
}

/// Before anything is chosen, the listing resolves defaults so the dock has
/// something truthful to display, and totals start at zero dollars.
///
/// Protects
/// docs/guarantees/agent/the-dock-reports-spend-and-runs-the-chosen-model.md.
#[tokio::test(flavor = "multi_thread")]
async fn an_untouched_document_lists_resolved_defaults_and_zero_totals() {
    let session = start().await;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([done(
            "hi",
        )])));

    let (status, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(listing["provider"], "anthropic");
    assert_eq!(listing["model"], "claude-sonnet-5");
    assert_eq!(listing["totals"]["usd"], 0.0);
    assert_eq!(listing["totals"]["input"], 0);
}

/// Wait until `turn_id` leaves the running state; returns its listing entry.
async fn wait_for_turn(session: &Session, turn_id: &str) -> (String, Value) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let (status, listing) = get(
            session,
            &format!("/api/docs/{}/agent/turns", session.doc_id),
        )
        .await;
        assert_eq!(status, 200, "{listing}");
        let turn = listing["turns"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == turn_id)
            .cloned();
        if let Some(turn) = turn
            && turn["status"] != "running"
        {
            return (turn_id.to_string(), turn);
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the turn never finished"
        );
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// `parent_turn_id` selects the conversation tip: a reply records its
/// parent, and naming an earlier turn again forks a sibling branch rather
/// than overwriting what followed it. An unknown parent is refused before
/// anything runs.
#[tokio::test(flavor = "multi_thread")]
async fn replies_record_their_parent_and_rewinding_forks_a_branch() {
    let session = start().await;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([
            done("first answer"),
            done("second answer"),
            done("forked answer"),
        ])));

    let (root_id, root) = run_turn_to_completion(&session, "start", None).await;
    assert_eq!(root["status"], "ok");

    let (reply_id, reply) = run_turn_to_completion(&session, "continue", Some(&root_id)).await;
    assert_eq!(reply["parent_id"], root_id.as_str());
    assert_eq!(reply["status"], "ok");

    // Rewind to the root and send again: a sibling of the first reply.
    let (fork_id, fork) = run_turn_to_completion(&session, "try differently", Some(&root_id)).await;
    assert_eq!(fork["parent_id"], root_id.as_str());

    let (_, listing) = get(
        &session,
        &format!("/api/docs/{}/agent/turns", session.doc_id),
    )
    .await;
    let children: Vec<String> = listing["turns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|t| t["parent_id"] == root_id.as_str())
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(children, vec![reply_id, fork_id]);

    // A parent that does not exist is a bad request, not a broken run.
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "x", "parent_id": "no-such-turn" }),
    )
    .await;
    assert_eq!(status, 400, "{body}");
}

// ---------------------------------------------------------------------------
// The stop button.
// Protects docs/guarantees/agent/a-running-agent-can-be-stopped.md.
// ---------------------------------------------------------------------------

/// The runaway that forced this feature: a model degenerating into an
/// endless `<sh:exec></sh:exec>` stream, billing tokens until somebody
/// killed the whole program.
struct EndlessLlm;

#[async_trait::async_trait]
impl hickory_agent::LlmClient for EndlessLlm {
    async fn complete(&self, _messages: Vec<hickory_agent::Message>) -> anyhow::Result<String> {
        anyhow::bail!("the runaway only streams")
    }

    async fn complete_stream(
        &self,
        _messages: Vec<hickory_agent::Message>,
    ) -> anyhow::Result<hickory_agent::ChatStream> {
        let stream = futures::stream::unfold(0u64, |n| async move {
            tokio::time::sleep(Duration::from_millis(2)).await;
            Some((
                Ok(hickory_agent::ChatChunk::text("<sh:exec></sh:exec>")),
                n + 1,
            ))
        });
        Ok(Box::pin(stream))
    }

    fn provider_name(&self) -> &str {
        "endless"
    }

    fn model_name(&self) -> &str {
        "endless"
    }
}

/// `POST /api/docs/:id/agent/stop` cuts a run that would otherwise stream
/// forever, and the turn finishes as `"stopped"` — the user's own act, which
/// the dock renders quietly, never as a red error.
#[tokio::test(flavor = "multi_thread")]
async fn the_stop_route_halts_a_runaway_turn_and_records_a_stop() {
    let session = start().await;
    session.state.agent.set_llm_override(Arc::new(EndlessLlm));

    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent", session.doc_id),
        json!({ "prompt": "loop forever", "parent_id": null }),
    )
    .await;
    assert_eq!(status, 202, "{body}");
    let turn_id = body["session_id"].as_str().expect("session_id").to_string();

    // Let it stream a moment: the stop must work MID-generation, because
    // that is when the tokens are being billed.
    tokio::time::sleep(Duration::from_millis(150)).await;

    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent/stop", session.doc_id),
        json!({}),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["stopping"], turn_id.as_str(), "{body}");

    let (_, turn) = wait_for_turn(&session, &turn_id).await;
    assert_eq!(turn["status"], "stopped", "{turn}");
    assert!(
        turn["answer"].is_null(),
        "a stopped turn has no answer to replay — the next message continues \
         as if it never ran: {turn}"
    );
}

/// Stopping when nothing runs is answered with a sentence, not a shrug —
/// the usual cause is the turn finishing in the race with the click.
#[tokio::test(flavor = "multi_thread")]
async fn stopping_an_idle_document_names_the_situation() {
    let session = start().await;
    let (status, body) = post(
        &session,
        &format!("/api/docs/{}/agent/stop", session.doc_id),
        json!({}),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert!(
        body["error"]
            .as_str()
            .unwrap_or_default()
            .contains("no agent turn is running"),
        "{body}"
    );
}

// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
#[tokio::test(flavor = "multi_thread")]
async fn workspace_agent_receives_unsaved_buffers_without_saving_them() {
    #[derive(Default)]
    struct Recorder(std::sync::Mutex<Vec<Vec<hickory_agent::Message>>>);
    #[async_trait::async_trait]
    impl hickory_agent::LlmClient for Recorder {
        async fn complete(&self, messages: Vec<hickory_agent::Message>) -> anyhow::Result<String> {
            self.0.lock().unwrap().push(messages);
            Ok(done("I can see the open editors"))
        }
        fn provider_name(&self) -> &str {
            "scripted"
        }
        fn model_name(&self) -> &str {
            "scripted"
        }
    }
    let session = start_session(false, true).await;
    let recorder = Arc::new(Recorder::default());
    session.state.agent.set_llm_override(recorder.clone());
    std::fs::write(session.root.join("plain.txt"), "disk bytes").unwrap();
    let (mut socket, _) = tokio_tungstenite::connect_async(format!(
        "{}/api/ws?doc=workspace",
        session.base.replace("http:", "ws:")
    ))
    .await
    .unwrap();
    let request = json!({"prompt":"What is open?", "context":{"buffers":[
        {"name":"Untitled 1","path":null,"content":"draft with <hick:exec> tags","focused":true},
        {"name":"plain.txt","path":"plain.txt","content":"unsaved plain bytes","focused":false},
        {"name":"deleted.txt","path":"deleted.txt","content":"deleted but still open","focused":false}
    ]}});
    let (status, response) = post(&session, "/api/docs/workspace/agent", request.clone()).await;
    assert_eq!(status, 202, "{response}");
    let id = response["session_id"].as_str().unwrap();
    let (_, turn) = wait_for_turn(&session, id).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    let messages = recorder.0.lock().unwrap()[0].clone();
    let context = messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for expected in [
        "No folder is open",
        "draft with <hick:exec> tags",
        "unsaved plain bytes",
        "deleted but still open",
    ] {
        assert!(context.contains(expected), "missing {expected}: {context}");
    }
    assert_eq!(
        std::fs::read_to_string(session.root.join("plain.txt")).unwrap(),
        "disk bytes"
    );
    assert!(!session.root.join("deleted.txt").exists());
    assert!(!session.root.join(".hick-workspace-agent").exists());
    let recorded =
        std::fs::read_to_string(session.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(recorded.contains("editor-buffers"));
    assert_eq!(
        hickory_agent::session_view::session_view(&recorded)
            .turns
            .len(),
        1
    );
    let streamed = tokio::time::timeout(Duration::from_secs(5), socket.next())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert!(matches!(streamed, TtMessage::Binary(frame) if frame[0] == 1));
    // A later message refreshes snapshots in the same conversation.
    let mut next = request;
    next["parent_id"] = json!(id);
    next["context"]["buffers"][0]["content"] = json!("revised draft");
    let (status, response) = post(&session, "/api/docs/workspace/agent", next).await;
    assert_eq!(status, 202, "{response}");
    let (_, second) = wait_for_turn(&session, response["session_id"].as_str().unwrap()).await;
    assert_eq!(second["session"], turn["session"]);
    assert!(
        recorder.0.lock().unwrap()[1]
            .iter()
            .any(|message| message.content.contains("revised draft"))
    );
}

// Guarantee: docs/guarantees/agent/the-agent-sees-the-open-editors.md
#[tokio::test(flavor = "multi_thread")]
async fn workspace_agent_reads_a_folder_without_a_primary_document_and_recovers() {
    let mut session = start_with_folder(true).await;
    session.doc_id = "workspace".into();
    std::fs::write(session.root.join("folder-only.txt"), "folder contents").unwrap();
    let read = r#"<hick:next>tool</hick:next>
<hick:tool name="read_file"><hick:arg name="path">folder-only.txt</hick:arg></hick:tool>"#;
    session
        .state
        .agent
        .set_llm_override(Arc::new(hickory_agent::ScriptedLlmClient::new([
            read.to_string(),
            done("read the folder"),
        ])));
    let (_, turn) = run_turn_to_completion(&session, "inspect the folder", None).await;
    assert_eq!(turn["status"], "ok", "{turn}");
    let source =
        std::fs::read_to_string(session.root.join(turn["session"].as_str().unwrap())).unwrap();
    assert!(source.contains("folder contents"), "{source}");
    assert!(source.contains("name=\"read_file\" ok=\"true\""));
    assert!(source.contains("doc=\".hick-workspace-agent\""));
    // A new server has an empty in-memory hub and hydrates the workspace thread.
    let prepared = prepare(ServeOptions {
        target: session.root.clone(),
        port: 0,
        params: Vec::new(),
        executor: ExecutorChoice::Local,
        key_store_path: None,
        ui_settings_path: None,
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move {
        axum::serve(listener, prepared.router).await.unwrap();
    });
    let response: Value = reqwest::Client::new()
        .get(format!("{base}/api/docs/workspace/agent/turns"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(response["turns"][0]["id"], turn["id"], "{response}");
}
