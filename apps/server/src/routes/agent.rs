//! POST /api/docs/:id/agent — the agent integration boundary.
//!
//! This module is the only server code that may know `hickory-agent`. The
//! ReAct loop runs server-side through the configured executor; its events
//! stream on the WS run channel with `exec_id: "agent"` and
//! `run_id == session_id`; the finished session is persisted as a
//! `hick:session` document in the project git repo (and as a doc row so it
//! shows up in the project).

use std::time::Instant;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use hickory_agent::{AGENT_EXEC_ID, AgentConfig, AgentEvent, AnthropicClient, run_agent};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;
use crate::auth::AuthUser;
use crate::error::{ApiError, ApiResult};
use crate::routes::docs::{DocRow, load_doc};

#[derive(Deserialize)]
pub struct AgentRequest {
    pub prompt: String,
}

pub async fn start_agent(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<AgentRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden(
            "only the project owner can start an agent session",
        ));
    }
    let Some(api_key) = state.config.anthropic_api_key.clone() else {
        return Err(ApiError::service_unavailable(
            "agent not configured (ANTHROPIC_API_KEY unset)",
        ));
    };
    if body.prompt.trim().is_empty() {
        return Err(ApiError::bad_request("prompt required"));
    }
    // Agent runs execute through the same metered executor time.
    crate::runs::check_exec_quota(&state, &user).await?;

    let session_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (id, doc_id, user_id, kind, status) VALUES ($1, $2, $3, 'agent', 'running')",
    )
    .bind(session_id)
    .bind(doc.id)
    .bind(user.id)
    .execute(&state.db)
    .await?;

    state.analytics.capture(
        &user.id.to_string(),
        "agent_session_started",
        json!({ "doc_id": doc.id, "session_id": session_id }),
    );

    let prompt = body.prompt.clone();
    let user_id = user.id;
    let state2 = state.clone();
    tokio::spawn(async move {
        let started = Instant::now();
        let status = match run_agent_session(&state2, session_id, &doc, &prompt, &api_key).await {
            Ok(()) => "ok",
            Err(e) => {
                log::warn!("agent session {session_id} failed: {e:#}");
                let _ = sqlx::query("UPDATE runs SET error = $1 WHERE id = $2")
                    .bind(format!("{e:#}"))
                    .bind(session_id)
                    .execute(&state2.db)
                    .await;
                "failed"
            }
        };
        let wall_ms = started.elapsed().as_millis() as i64;
        let _ = sqlx::query(
            "UPDATE runs SET status = $1, finished_at = now(), wall_ms = $2 WHERE id = $3",
        )
        .bind(status)
        .bind(wall_ms)
        .bind(session_id)
        .execute(&state2.db)
        .await;
        crate::runs::record_usage(&state2, user_id, wall_ms).await;
        state2
            .rooms
            .publish_run_event(doc.id, &json!({ "run_id": session_id, "status": status }))
            .await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "session_id": session_id })),
    ))
}

async fn run_agent_session(
    state: &AppState,
    session_id: Uuid,
    doc: &DocRow,
    prompt: &str,
    api_key: &str,
) -> anyhow::Result<()> {
    // Workspace: temp dir seeded from the project checkout (the agent's
    // scripts and its session file live here until persisted to git).
    let tmp = tempfile::tempdir()?;
    state.git.seed_checkout(doc.project_id, tmp.path())?;
    let doc_file = tmp.path().join(&doc.path);
    if let Some(parent) = doc_file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&doc_file, &doc.source)?;

    // Stream agent events on the run channel as they happen.
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();
    let forwarder = {
        let state = state.clone();
        let doc_id = doc.id;
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                state
                    .rooms
                    .publish_run_event(
                        doc_id,
                        &json!({
                            "run_id": session_id,
                            "exec_id": AGENT_EXEC_ID,
                            "event": event,
                        }),
                    )
                    .await;
            }
        })
    };

    let executor = crate::executor::build_executor(state.config.executor)?;
    let llm = AnthropicClient::new().with_api_key(api_key);
    let mut config = AgentConfig::new(prompt, tmp.path());
    config.doc_context = Some(doc.source.clone());

    let mut on_event = |event: AgentEvent| {
        let _ = event_tx.send(event);
    };
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event).await;
    drop(event_tx);
    forwarder.await.ok();
    executor.shutdown().await.ok();

    let outcome = outcome?;

    // Persist the hick:session document into the project (git + doc row).
    let rel_path = outcome
        .session_path
        .strip_prefix(tmp.path())
        .unwrap_or(&outcome.session_path)
        .to_string_lossy()
        .replace('\\', "/");
    let session_source = std::fs::read_to_string(&outcome.session_path)?;
    state
        .git
        .save_file(
            doc.project_id,
            &rel_path,
            &session_source,
            &format!("Agent session: {rel_path}"),
        )
        .await?;
    sqlx::query(
        "INSERT INTO docs (id, project_id, path, source) VALUES ($1, $2, $3, $4)
         ON CONFLICT (project_id, path)
         DO UPDATE SET source = EXCLUDED.source, updated_at = now()",
    )
    .bind(Uuid::new_v4())
    .bind(doc.project_id)
    .bind(&rel_path)
    .bind(&session_source)
    .execute(&state.db)
    .await?;

    log::info!(
        "agent session {session_id} done in {} turns → {rel_path}",
        outcome.turns
    );
    Ok(())
}
