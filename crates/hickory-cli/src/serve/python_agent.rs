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
    anyhow::ensure!(
        !settings.python.is_empty(),
        "Configure the Python agent runtime in Settings → Agents."
    );
    let path = PathBuf::from(settings.python);
    let scratch = tempfile::tempdir()?;
    // No local executor fallback, regardless of the current cell-executor preference.
    hickory_executor_sandbox::agent_process::command(&path, scratch.path())?;
    Ok(path)
}
pub fn catalogue(state: &LocalState) -> Value {
    let result = runtime(state);
    json!({"id":"python","name":"Hickory Python (experimental)","command":load(state).unwrap_or_default().python,"args":[],"available":result.is_ok(),"error":result.err().map(|e| format!("{e:#}")),"kind":"python"})
}
pub async fn settings(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let settings = load(&state).map_err(|e| ApiError::unprocessable(format!("{e:#}")))?;
    Ok(Json(
        json!({"python":settings.python,"available":runtime(&state).is_ok()}),
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
fn queue(state: &LocalState, doc: &str) -> Arc<super::agent_review::NativeReview> {
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
    if body.mode == Some(super::acp::edits::Mode::AutoAccept) {
        return Err(ApiError::bad_request(
            "Python agent edits always require independent approval.",
        ));
    }
    super::agent_context::subject(&state, &doc)?;
    if let Some(id) = body.id {
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
    queue: Arc<super::agent_review::NativeReview>,
    accepted: Option<String>,
}
#[async_trait::async_trait]
impl crate::python_agent::Review for Review {
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
        python: identity.python.expect("Python run has a runtime"),
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
    };
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
        .map(|outcome| (outcome.summary, outcome.usage))
        .map_err(|e| format!("{e:#}"))
}

pub(super) fn routes() -> axum::Router<LocalState> {
    use axum::routing::{get, post};
    axum::Router::new()
        .route("/agents/python", get(settings).put(configure))
        .route("/docs/{id}/agent/python", get(snapshot))
        .route("/docs/{id}/agent/python/review", post(decide))
}
