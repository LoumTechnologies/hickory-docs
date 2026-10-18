//! Built-in agents use the same review queue and UI as ACP document edits.
use super::{
    LocalState,
    acp::{EditDecision, edits::Edits},
    api::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
};
use serde_json::{Value, json};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Default)]
pub struct NativeReview {
    pub edits: Arc<Edits>,
    pub cancel: Mutex<Option<Arc<AtomicBool>>>,
    pub session: Mutex<Option<std::path::PathBuf>>,
    accepted: Mutex<Option<String>>,
}

#[async_trait::async_trait]
impl hickory_agent::ToolReview for NativeReview {
    fn replacement(&self) -> Option<String> {
        self.accepted
            .lock()
            .unwrap()
            .as_deref()
            .and_then(|id| self.edits.replacement(id))
    }
    fn mode(&self) -> &'static str {
        if *self.edits.mode.lock().unwrap() == super::acp::edits::Mode::AutoAccept {
            "auto-accept"
        } else {
            "review"
        }
    }
    fn completed(&self, ok: bool) {
        if let Some(id) = self.accepted.lock().unwrap().take() {
            self.edits.finished(&id, ok);
        }
    }
    async fn approve(
        &self,
        _: &hickory_agent::ToolInvocation,
        preview: &(String, String, String),
    ) -> anyhow::Result<()> {
        let waiting = self.edits.submit(
            preview.0.clone(),
            Some(preview.0.clone()),
            preview.1.clone(),
            preview.2.clone(),
            false,
        );
        tokio::pin!(waiting);
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
        loop {
            tokio::select! {
                result = &mut waiting => {
                    let id = result?;
                    *self.accepted.lock().unwrap() = Some(id);
                    return Ok(());
                },
                _ = interval.tick() => {
                    if self.cancel.lock().unwrap().as_ref().is_some_and(|cancel| cancel.load(Ordering::Relaxed)) {
                        self.edits.cancel();
                        anyhow::bail!(hickory_agent::STOPPED_BY_USER);
                    }
                }
            }
        }
    }
}

pub fn queue(state: &LocalState, doc: &str) -> Arc<NativeReview> {
    state
        .agent
        .reviews
        .lock()
        .unwrap()
        .entry(doc.into())
        .or_default()
        .clone()
}

pub async fn snapshot(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
) -> ApiResult<Json<Value>> {
    super::agent_context::subject(&state, &doc)?;
    let review = queue(&state, &doc);
    Ok(Json(
        json!({"backend":"builtin","ready":true,"edits":review.edits.snapshot()}),
    ))
}

pub async fn decide(
    State(state): State<LocalState>,
    Path(doc): Path<String>,
    Json(body): Json<EditDecision>,
) -> ApiResult<Json<Value>> {
    super::agent_context::subject(&state, &doc)?;
    let review = queue(&state, &doc);
    if let (Some(id), Some(context)) = (body.capture_request, body.context) {
        review
            .edits
            .refreshed(&id, context.buffers)
            .map_err(|e| ApiError::conflict(e.to_string()))?;
    }
    if let Some(mode) = body.mode {
        if state
            .agent
            .turns
            .lock()
            .unwrap()
            .get(&doc)
            .is_some_and(|turns| turns.iter().any(|turn| turn.status == "running"))
        {
            return Err(ApiError::conflict(
                "Wait for the turn to finish before changing review policy.",
            ));
        }
        if let Some(path) = review.session.lock().unwrap().clone() {
            use hickory_agent::{HickSessionLog, SessionEvent, SessionLog};
            let log = HickSessionLog::append_or_create(path)
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
            log.record(SessionEvent::Context {
                kind: "acp-edit-mode",
                data: &json!({"mode":mode}),
            });
            log.record(SessionEvent::End);
            log.check()
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
        }
        *review.edits.mode.lock().unwrap() = mode;
    }
    if let Some(id) = body.id {
        if let Some(text) = body.new_text {
            review
                .edits
                .revise(&id, text)
                .map_err(|e| ApiError::unprocessable(e.to_string()))?;
        }
        review
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
