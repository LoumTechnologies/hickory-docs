//! Isolated adapter for the existing Python supervisor and edit-review queue.
use super::{
    LocalState,
    api::{ApiError, ApiResult},
    private_predictions::Bound,
};
use axum::{Json, extract::State};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

pub(super) struct Run {
    id: String,
    status: String,
    answer: Option<String>,
    error: Option<crate::python_agent::PrivateFailure>,
    cancel: Arc<AtomicBool>,
}
struct Review {
    edits: Arc<super::acp::edits::Edits>,
    accepted: Option<String>,
    ui: Arc<super::ui_actions::Hub>,
    window: Option<String>,
    actor: String,
}
#[async_trait::async_trait]
impl crate::python_agent::Review for Review {
    fn private_prediction_safe(&self) -> bool {
        true
    }
    fn ui_tools(&self) -> Vec<Value> {
        if self.window.is_some() {
            super::ui_actions::catalogue()
        } else {
            vec![]
        }
    }
    async fn ui_call(&mut self, name: &str, args: Value) -> anyhow::Result<Value> {
        let window = self
            .window
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("No originating private window"))?;
        Ok(super::ui_actions::tool_result(
            self.ui.call(window, &self.actor, name, args).await?,
        ))
    }
    fn replacement(&self) -> Option<String> {
        self.accepted
            .as_deref()
            .and_then(|id| self.edits.replacement(id))
    }
    async fn approve(&mut self, proposal: &crate::python_agent::Proposal) -> anyhow::Result<bool> {
        match self
            .edits
            .submit_review(
                proposal.tool.clone(),
                Some(proposal.surface.clone()),
                proposal.before.clone(),
                proposal.after.clone(),
            )
            .await
        {
            Ok(id) => {
                self.accepted = Some(id);
                Ok(true)
            }
            Err(_) => Ok(false),
        }
    }
    async fn completed(&mut self, ok: bool) -> anyhow::Result<()> {
        if let Some(id) = self.accepted.take() {
            self.edits.finished(&id, ok);
        }
        Ok(())
    }
}
impl Drop for Review {
    fn drop(&mut self) {
        if let Some(id) = self.accepted.take() {
            self.edits.finished(&id, false);
        }
        self.edits.cancel();
    }
}
fn bound(state: &LocalState) -> ApiResult<Arc<Bound>> {
    state
        .private_prediction
        .clone()
        .ok_or_else(|| ApiError::unprocessable("No private prediction session"))
}
fn snapshot(bound: &Bound, run: Option<&Run>) -> Value {
    json!({"run":run.map(|run|json!({"id":run.id,"status":run.status,"answer":run.answer,"error_code":run.error})),
        "backend":"python","ready":true,"canRewind":false,"edits":bound.edits.snapshot(),
        "actual_values":"withheld","current_certification":false,"output_token_limit":2048})
}
pub(super) async fn status(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = bound(&state)?;
    let run = bound.agent.lock().await;
    Ok(Json(snapshot(&bound, run.as_ref())))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Start {
    prompt: String,
    provider: Option<String>,
    model: Option<String>,
    ui_window: Option<String>,
}
pub(super) async fn start(
    State(state): State<LocalState>,
    Json(body): Json<Start>,
) -> ApiResult<Json<Value>> {
    let bound = bound(&state)?;
    if bound.startup_fingerprint.get().is_none() {
        return Err(ApiError::unprocessable(
            "Private provider conversations require the shared engine binding",
        ));
    }
    if body.model.as_ref().is_some_and(|model| model.len() > 1024) {
        return Err(ApiError::bad_request("Model identifier is too long"));
    }
    if body.prompt.trim().is_empty() || body.prompt.len() > 32768 {
        return Err(ApiError::bad_request(
            "A bounded investigation request is required",
        ));
    }
    if body
        .provider
        .as_deref()
        .is_some_and(|provider| hickory_agent::ProviderSelection::parse(provider).is_none())
    {
        return Err(ApiError::bad_request("Unknown provider"));
    }
    if let Some(window) = &body.ui_window {
        state
            .ui_actions
            .ensure_window(window)
            .map_err(|_| ApiError::conflict("Originating private window is unavailable"))?;
    }
    let python = super::python_agent::runtime(&state).map_err(|_| {
        ApiError::unavailable("Configure the confined Python runtime before starting this session")
    })?;
    let (llm, _, _) = super::agent::selected_client_with_limit(
        &state,
        &super::agent::DocSelection {
            provider: body.provider,
            model: body.model,
        },
        Some(2048),
    )?;
    let mut current = bound.agent.lock().await;
    if current.as_ref().is_some_and(|run| run.status == "running") {
        return Err(ApiError::conflict(
            "A private investigation is already running",
        ));
    }
    let id = format!("{:016x}", super::rand_id());
    let cancel = Arc::new(AtomicBool::new(false));
    *current = Some(Run {
        id: id.clone(),
        status: "running".into(),
        answer: None,
        error: None,
        cancel: cancel.clone(),
    });
    let reply = snapshot(&bound, current.as_ref());
    drop(current);
    let task_bound = bound.clone();
    tokio::spawn(async move {
        // A fresh private narrative for each request; safe durable gate history is
        // supplied by the engine. No ordinary session/editor context is hydrated.
        let mut review = Review {
            edits: task_bound.edits.clone(),
            accepted: None,
            ui: state.ui_actions.clone(),
            window: body.ui_window,
            actor: format!("python-private:{id}"),
        };
        let config = crate::python_agent::Config {
            python,
            root: state.index.root().into(),
            doc: Some(task_bound.source_path().into()),
            prompt: body.prompt,
            max_turns: 30,
        };
        let result = crate::python_agent::run_conversation(
            config,
            llm.as_ref(),
            &mut review,
            crate::python_agent::Conversation {
                private_prediction: true,
                require_private_verification: true,
                cancel: Some(cancel),
                ..Default::default()
            },
            &mut |_| {},
        )
        .await;
        let mut current = task_bound.agent.lock().await;
        if let Some(run) = current.as_mut().filter(|run| run.id == id) {
            match result {
                Ok(outcome) => {
                    run.status = "finished".into();
                    run.answer = Some(outcome.summary);
                }
                Err(error) => {
                    run.status = "failed".into();
                    run.error = error
                        .downcast_ref::<crate::python_agent::PrivateFailure>()
                        .cloned();
                    run.answer=Some(run.error.as_ref().map(ToString::to_string).unwrap_or_else(||"Private investigation did not complete. Inspect the retained comparison history and review state; held-out diagnostics remain withheld.".into()));
                }
            }
        }
    });
    Ok(Json(reply))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Decision {
    id: String,
    accepted: bool,
    new_text: Option<String>,
}
pub(super) async fn decide(
    State(state): State<LocalState>,
    Json(body): Json<Decision>,
) -> ApiResult<Json<Value>> {
    let bound = bound(&state)?;
    if let Some(text) = body.new_text {
        bound
            .edits
            .revise(&body.id, text)
            .map_err(|_| ApiError::conflict("Proposal is no longer editable"))?;
    }
    bound
        .edits
        .resolve(
            &body.id,
            if body.accepted {
                Ok(())
            } else {
                Err("Human declined the edit".into())
            },
        )
        .map_err(|_| ApiError::conflict("Proposal is no longer pending"))?;
    status(State(state)).await
}
pub(super) async fn stop(State(state): State<LocalState>) -> ApiResult<Json<Value>> {
    let bound = bound(&state)?;
    {
        let current = bound.agent.lock().await;
        if let Some(run) = current.as_ref() {
            run.cancel.store(true, Ordering::Relaxed);
        }
    }
    bound.edits.cancel();
    status(State(state)).await
}
