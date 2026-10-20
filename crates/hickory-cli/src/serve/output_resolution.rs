use super::{
    LocalState,
    api::{ApiError, ApiResult},
};
use crate::up::state::ResolutionMode;
use axum::{Json, extract::State};
use serde_json::{Value, json};

#[derive(serde::Deserialize)]
pub struct ResolveBody {
    pub path: String,
    pub content: String,
    pub expected_current: String,
    pub expected_produced: String,
}

/// Resolve only the observed held output; acknowledge the actual loop result.
pub async fn resolve_output(
    State(state): State<LocalState>,
    Json(body): Json<ResolveBody>,
) -> ApiResult<Json<Value>> {
    apply(state, body, ResolutionMode::Merge).await
}

#[derive(serde::Deserialize)]
pub struct RegenerateBody {
    pub path: String,
    pub expected_current: String,
    pub expected_produced: String,
}

pub async fn regenerate_output(
    State(state): State<LocalState>,
    Json(body): Json<RegenerateBody>,
) -> ApiResult<Json<Value>> {
    apply(
        state,
        ResolveBody {
            path: body.path,
            content: body.expected_produced.clone(),
            expected_current: body.expected_current,
            expected_produced: body.expected_produced,
        },
        ResolutionMode::Regenerate,
    )
    .await
}

async fn apply(
    state: LocalState,
    body: ResolveBody,
    mode: ResolutionMode,
) -> ApiResult<Json<Value>> {
    let relative = std::path::Path::new(&body.path);
    if relative
        .components()
        .any(|c| !matches!(c, std::path::Component::Normal(_)))
        || body.path.is_empty()
    {
        return Err(ApiError::bad_request(
            "Resolution must name a file inside the open folder",
        ));
    }
    let root = state
        .index
        .root()
        .canonicalize()
        .map_err(|e| ApiError::bad_request(e.to_string()))?;
    let sender = state
        .up_commands
        .lock()
        .expect("up command inbox")
        .clone()
        .ok_or_else(|| ApiError::unavailable("The folder is not being watched"))?;
    let (reply, result) = tokio::sync::oneshot::channel();
    sender
        .send(crate::up::UpCommand::Resolve {
            request: crate::up::state::OutputResolution {
                path: root.join(relative),
                root,
                content: body.content,
                expected_current: body.expected_current,
                expected_produced: body.expected_produced,
                mode,
            },
            reply,
        })
        .map_err(|_| ApiError::unavailable("The watch loop stopped"))?;
    match tokio::time::timeout(std::time::Duration::from_secs(30), result).await {
        Ok(Ok(Ok(()))) => Ok(Json(
            json!({ "ok": true, "path": body.path, "applied": true, "carried_back": false }),
        )),
        Ok(Ok(Err(reason))) => Err(ApiError::conflict(reason)),
        Ok(Err(_)) => Err(ApiError::unavailable(
            "The watch loop did not return a resolution result",
        )),
        Err(_) => Err(ApiError::unavailable(
            "Timed out waiting for resolution; inspect current output before retrying",
        )),
    }
}
