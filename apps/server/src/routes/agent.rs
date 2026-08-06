//! POST /api/docs/:id/agent — the agent integration boundary.
//!
//! TODO(agent): `crates/hickory-agent` had not landed when this server
//! shipped. When it does, wire it HERE (this module is the only place that
//! may know the agent crate): run the ReAct loop server-side with the
//! configured executor, stream its events on the run WS channel with
//! `exec_id: "agent"` and `run_id == session_id`, and persist the session as
//! a `hick:session` document into the project git repo
//! (`state.git.save_file(...)`). Until then this endpoint answers
//! `501 {"error": "agent not yet enabled"}` per the graceful-degradation
//! convention.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::routes::docs::load_doc;

#[derive(Deserialize)]
pub struct AgentRequest {
    #[allow(dead_code)]
    pub prompt: String,
}

pub async fn start_agent(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(_body): Json<AgentRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden("only the project owner can start an agent session"));
    }
    Err(ApiError::new(StatusCode::NOT_IMPLEMENTED, "agent not yet enabled"))
}
