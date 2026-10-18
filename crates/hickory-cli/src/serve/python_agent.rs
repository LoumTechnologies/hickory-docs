//! Python's UI adapter shares conversation transport, never ACP process permissions.
use super::{
    LocalState,
    agent::TurnIdentity,
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use hickory_agent::{AgentEvent, LlmClient, PriorTurn, Usage};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{path::PathBuf, sync::Arc};

#[derive(Default, Deserialize, serde::Serialize)]
pub struct Settings {
    pub python: String,
}
fn load(state: &LocalState) -> anyhow::Result<Settings> {
    let path = super::acp::config::directory(state).join("python.json");
    match std::fs::read(path) {
        Ok(raw) => Ok(serde_json::from_slice(&raw)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}
pub fn runtime(state: &LocalState) -> anyhow::Result<PathBuf> {
    let settings = load(state)?;
    let path = effective_runtime(&settings, state)?;
    let scratch = tempfile::tempdir()?;
    // No local executor fallback, regardless of the current cell-executor preference.
    hickory_executor_sandbox::agent_process::command(&path, scratch.path())?;
    Ok(path)
}

/// The runtime to use, in the order that makes each one honest: what a person
/// chose in Settings, then the runtime the app that started this engine handed
/// over, then one shipped inside the app.
///
/// The last is discovered from the running executable, not configured, because
/// that is what it literally is — the same bytes the app was installed with.
/// It is the zero-configuration default that makes the experimental Python
/// agent work in a downloaded app without naming a path.
fn effective_runtime(settings: &Settings, state: &LocalState) -> anyhow::Result<PathBuf> {
    if !settings.python.trim().is_empty() {
        return Ok(PathBuf::from(&settings.python));
    }
    if let Some(path) = &state.python_runtime_path {
        return Ok(path.clone());
    }
    bundled_runtime()
        .ok_or_else(|| anyhow::anyhow!("Configure the Python agent runtime in Settings → Agents."))
}

/// A Python runtime shipped inside this app, located from the running
/// executable's own path — the same bytes the app was installed with.
///
/// On macOS that is `Hickory Docs.app/Contents/MacOS/<binary>`, with the
/// runtime at `Contents/Resources/python-agent/bin/python`; a bare executable
/// looks for `python-agent/` beside it. `None` — the CLI, the tests, a build
/// that ships none — means no runtime was bundled and Settings must name one.
pub fn bundled_runtime() -> Option<PathBuf> {
    bundled_runtime_for(std::env::current_exe().ok()?)
}

/// The path logic behind [`bundled_runtime`], split out so a test can place a
/// fake bundle and ask without moving the test executable into it.
fn bundled_runtime_for(exe: PathBuf) -> Option<PathBuf> {
    let dir = exe.parent()?;
    [
        // A macOS application bundle: Contents/MacOS/<binary>.
        dir.parent()
            .map(|contents| contents.join("Resources/python-agent/bin/python")),
        // A bare executable with the runtime beside it.
        Some(dir.join("python-agent/bin/python")),
    ]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
}
pub fn catalogue(state: &LocalState) -> Value {
    let result = runtime(state);
    json!({"id":"python","name":"Hickory Python","command":load(state).unwrap_or_default().python,"args":[],"available":result.is_ok(),"error":result.err().map(|e| format!("{e:#}")),"kind":"python"})
}
pub async fn settings(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let settings = load(&state).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    // `bundled` lets Settings say "this app already ships a runtime" without
    // pretending the person configured one: the field is a hint, never saved.
    let bundled = state
        .python_runtime_path
        .clone()
        .or_else(bundled_runtime)
        .map(|path| path.display().to_string());
    Ok(Json(
        json!({"python":settings.python,"available":runtime(&state).is_ok(),"bundled":bundled}),
    ))
}
pub async fn configure(
    State(state): State<LocalState>,
    Json(body): Json<Settings>,
) -> ApiResult<Json<Value>> {
    if !body.python.is_empty() {
        crate::python_agent::probe(&PathBuf::from(&body.python))
            .await
            .map_err(|e| {
                ApiError::unprocessable(format!(
                    "Python agent runtime could not start in confinement: {e:#}"
                ))
            })?;
    }
    let dir = super::acp::config::directory(&state);
    std::fs::create_dir_all(&dir).map_err(|e| ApiError::unprocessable(e.to_string()))?;
    super::store::write_atomic(
        &dir.join("python.json"),
        &serde_json::to_vec_pretty(&body).map_err(|e| ApiError::unprocessable(e.to_string()))?,
    )
    .map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    settings(State(state)).await
}
fn queue(state: &LocalState, doc: &str) -> Arc<super::agent_review::ReviewQueue> {
    // ACP/native policy and historical proposals cannot contaminate this runtime's queue.
    super::agent_review::queue(state, &format!("python:{doc}"))
}
pub async fn snapshot(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
) -> ApiResult<Json<Value>> {
    super::agent_context::subject(&state, &doc)?;
    let result = runtime(&state);
    let mut edits = queue(&state, &doc).edits.snapshot();
    edits["mode"] = json!("review");
    Ok(Json(
        json!({"backend":"python","ready":result.is_ok(),"canRewind":true,"error":result.err().map(|e| format!("{e:#}")),"edits":edits}),
    ))
}
pub async fn decide(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<super::acp::EditDecision>,
) -> ApiResult<Json<Value>> {
    if let (Some(id), Some(context)) = (body.capture_request, body.context) {
        queue(&state, &doc)
            .edits
            .refreshed(&id, context.buffers)
            .map_err(|e| ApiError::conflict(e.to_string()))?;
    }
    if body.mode == Some(super::acp::edits::Mode::AutoAccept) {
        return Err(ApiError::bad_request(
            "Python agent edits always require independent approval.",
        ));
    }
    super::agent_context::subject(&state, &doc)?;
    if let Some(id) = body.id {
        if let Some(text) = body.new_text {
            queue(&state, &doc)
                .edits
                .revise(&id, text)
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
        }
        queue(&state, &doc)
            .edits
            .resolve(
                &id,
                if body.accepted == Some(true) {
                    Ok(())
                } else {
                    Err(body
                        .error
                        .unwrap_or_else(|| "The user rejected the edit.".into()))
                },
            )
            .map_err(|e| ApiError::unprocessable(e.to_string()))?;
    }
    snapshot(State(state), Path(doc)).await
}
pub fn session_backend(path: &std::path::Path, provider: &str) -> String {
    if let Some(backend) = provider.strip_prefix("acp:") {
        return backend.into();
    }
    if std::fs::read_to_string(path)
        .ok()
        .and_then(|s| hick_lang::parse(&s).ok())
        .is_some_and(|doc| {
            doc.all_tags().iter().any(|tag| {
                tag.name == "context" && tag.get_attribute("kind") == Some("python-runtime")
            })
        })
    {
        "python".into()
    } else {
        "builtin".into()
    }
}
struct Review {
    context: super::acp::RoomContext,
    snapshots: std::collections::HashMap<String, super::acp::workspace::Snapshot>,
    queue: Arc<super::agent_review::ReviewQueue>,
    accepted: Option<String>,
}
#[async_trait::async_trait]
impl crate::python_agent::Review for Review {
    fn replacement(&self) -> Option<String> {
        self.accepted
            .as_deref()
            .and_then(|id| self.queue.edits.replacement(id))
    }
    async fn approve(&mut self, proposal: &crate::python_agent::Proposal) -> anyhow::Result<bool> {
        let surface = std::path::Path::new(&proposal.surface);
        let display = surface
            .strip_prefix(self.context.index.root())
            .unwrap_or(surface)
            .display()
            .to_string();
        match self
            .queue
            .edits
            .submit_review(
                proposal.tool.clone(),
                Some(display),
                proposal.before.clone(),
                proposal.after.clone(),
            )
            .await
        {
            Ok(id) => {
                self.accepted = Some(id);
                // Include keystrokes not yet persisted by the editor debounce in
                // the supervisor's post-approval freshness check.
                self.snapshots = super::acp::workspace::capture(&self.context).await?;
                Ok(true)
            }
            Err(error) => {
                use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
                if let Some(path) = self.queue.session.lock().unwrap().clone() {
                    let log = HickSessionLog::append_or_create(path)?;
                    log.record(SessionEvent::Context { kind: "python-review-refusal", data: &json!({"actor":"user-declared","proposal":proposal,"reason":format!("{error:#}")}) });
                    log.record(SessionEvent::End);
                    log.check()?;
                }
                Ok(false)
            }
        }
    }
    async fn refresh(&mut self) -> anyhow::Result<()> {
        if self
            .queue
            .edits
            .live_context
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.queue.edits.refresh().await?;
        }
        Ok(())
    }
    async fn completed(&mut self, ok: bool) -> anyhow::Result<()> {
        let result =
            super::acp::workspace::finish(&self.context, std::mem::take(&mut self.snapshots)).await;
        if let Some(id) = self.accepted.take() {
            self.queue.edits.finished(&id, ok && result.is_ok());
        }
        result
    }
}
impl Drop for Review {
    fn drop(&mut self) {
        if let Some(id) = self.accepted.take() {
            self.queue.edits.finished(&id, false);
        }
        self.queue.edits.cancel();
    }
}
pub(super) async fn run_turn(
    state: &LocalState,
    doc: &str,
    llm: Arc<dyn LlmClient>,
    subject: PathBuf,
    prompt: String,
    prior_turns: Vec<PriorTurn>,
    identity: TurnIdentity,
) -> Result<(String, Usage), String> {
    let queue = queue(state, doc);
    *queue.session.lock().unwrap() = Some(identity.session.clone());
    *queue.edits.mode.lock().unwrap() = super::acp::edits::Mode::Review;
    let mut review = Review {
        context: super::acp::RoomContext {
            index: state.index.clone(),
            rooms: state.rooms.clone(),
        },
        snapshots: Default::default(),
        queue,
        accepted: None,
    };
    let provider = state
        .agent
        .turns
        .lock()
        .unwrap()
        .get(doc)
        .and_then(|turns| turns.iter().find(|t| t.id == identity.turn))
        .map(|t| t.provider.clone())
        .unwrap_or_default();
    let config = crate::python_agent::Config {
        python: identity.python,
        root: state.index.root().to_path_buf(),
        doc: if doc == super::agent_context::WORKSPACE_AGENT {
            identity.primary
        } else {
            Some(subject.clone())
        },
        prompt,
        max_turns: 20,
    };
    let conversation = crate::python_agent::Conversation {
        session: Some(identity.session),
        subject: Some(subject),
        turn: Some(identity.turn.clone()),
        parent: identity.parent,
        provider,
        context: identity.context,
        prior_turns,
        cancel: Some(identity.cancel),
        changeset: identity.changeset,
    };
    let identity_turn = identity.turn.clone();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();
    let forwarder = {
        let state = state.clone();
        let doc = doc.to_string();
        let run = identity.turn;
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                state
                    .rooms
                    .publish_run_event(
                        &doc,
                        &json!({"run_id":run,"exec_id":hickory_agent::AGENT_EXEC_ID,"event":event}),
                    )
                    .await;
            }
        })
    };
    let mut emit = move |event| {
        let _ = tx.send(event);
    };
    let result = crate::python_agent::run_conversation(
        config,
        llm.as_ref(),
        &mut review,
        conversation,
        &mut emit,
    )
    .await;
    drop(emit);
    let _ = forwarder.await;
    result
        .map(|outcome| {
            if let Some(capture) = state.agent.projects.lock().unwrap().get_mut(&identity_turn) {
                capture.artifact = Some(outcome.artifact);
            }
            (outcome.summary, outcome.usage)
        })
        .map_err(|e| format!("{e:#}"))
}

pub(super) fn routes() -> axum::Router<LocalState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/agents/python", get(settings).put(configure))
        .route("/docs/{id}/agent/python", get(snapshot))
        .route("/docs/{id}/agent/python/review", post(decide))
}

#[cfg(test)]
mod tests {
    use super::bundled_runtime_for;

    // Guarantee: docs/guarantees/agent/a-desktop-app-bundles-its-python-agent.md
    #[test]
    fn a_runtime_is_found_inside_an_app_bundle_and_beside_a_bare_executable() {
        let root = tempfile::tempdir().unwrap();
        let bundle = root.path().join("Hickory Docs.app");
        let exe = bundle.join("Contents/MacOS/hickory-desktop");
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        let bundled = bundle.join("Contents/Resources/python-agent/bin/python");
        std::fs::create_dir_all(bundled.parent().unwrap()).unwrap();
        std::fs::write(&bundled, b"").unwrap();
        assert_eq!(bundled_runtime_for(exe), Some(bundled));

        // A bare executable with the runtime beside it.
        let bare = root.path().join("bin/hick");
        std::fs::create_dir_all(bare.parent().unwrap()).unwrap();
        let sibling = root.path().join("bin/python-agent/bin/python");
        std::fs::create_dir_all(sibling.parent().unwrap()).unwrap();
        std::fs::write(&sibling, b"").unwrap();
        assert_eq!(bundled_runtime_for(bare), Some(sibling));
    }

    // Guarantee: docs/guarantees/agent/a-desktop-app-bundles-its-python-agent.md
    #[test]
    fn nothing_is_found_when_no_runtime_was_shipped() {
        let root = tempfile::tempdir().unwrap();
        assert_eq!(bundled_runtime_for(root.path().join("hick")), None);
    }
}
