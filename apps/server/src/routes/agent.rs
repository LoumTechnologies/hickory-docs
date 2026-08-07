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
use hickory_agent::{AGENT_EXEC_ID, AgentConfig, AgentEvent, PriorTurn, run_agent};
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
    /// Turn this message continues from. Omitted starts a new conversation;
    /// naming an OLDER turn forks a branch from there (rewind), leaving the
    /// turns that followed it in place on their own branch.
    #[serde(default)]
    pub parent_id: Option<Uuid>,
}

/// One turn as the client sees it.
#[derive(serde::Serialize, sqlx::FromRow)]
pub struct TurnRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub prompt: String,
    pub answer: Option<String>,
    pub status: String,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/docs/:id/agent/turns — the whole conversation tree for a document.
pub async fn list_turns(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    let doc = load_doc(&state, id).await?;
    if doc.owner_id != user.id {
        return Err(ApiError::forbidden("not your document"));
    }
    let turns: Vec<TurnRow> = sqlx::query_as(
        "SELECT id, parent_id, prompt, answer, status, error, created_at
         FROM agent_turns WHERE doc_id = $1 ORDER BY created_at",
    )
    .bind(doc.id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(json!({ "turns": turns })))
}

/// Walk `parent_id` up to the root and return the exchanges oldest-first.
///
/// Only turns that actually produced an answer are replayed: a failed or
/// still-running ancestor has nothing to contribute, and inventing an empty
/// assistant message would teach the model that silence is a valid reply.
async fn ancestor_turns(
    state: &AppState,
    doc_id: Uuid,
    mut cursor: Option<Uuid>,
) -> Result<Vec<PriorTurn>, ApiError> {
    let mut chain = Vec::new();
    // The chain is bounded by construction (each parent is strictly older),
    // but a cycle introduced by a bad write must not hang the request.
    for _ in 0..200 {
        let Some(id) = cursor else { break };
        let row: Option<(Option<Uuid>, String, Option<String>)> = sqlx::query_as(
            "SELECT parent_id, prompt, answer FROM agent_turns WHERE id = $1 AND doc_id = $2",
        )
        .bind(id)
        .bind(doc_id)
        .fetch_optional(&state.db)
        .await?;
        let Some((parent_id, prompt, answer)) = row else {
            break;
        };
        if let Some(answer) = answer {
            chain.push(PriorTurn { prompt, answer });
        }
        cursor = parent_id;
    }
    chain.reverse();
    Ok(chain)
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
    let Some(llm_config) = state.config.agent_llm.clone() else {
        // Name the variable that is missing for the provider actually
        // selected — "set some key somewhere" is not an error message.
        let provider = &state.config.agent_provider;
        let key_env = hickory_agent::ProviderSelection::parse(provider)
            .map(|s| s.key_env())
            .unwrap_or("ANTHROPIC_API_KEY");
        return Err(ApiError::service_unavailable(format!(
            "agent not configured ({key_env} unset for provider {provider})"
        )));
    };
    if body.prompt.trim().is_empty() {
        return Err(ApiError::bad_request("prompt required"));
    }
    // Agent runs execute through the same metered executor time.
    crate::runs::check_email_verified(&state, &user)?;
    crate::runs::check_exec_quota(&state, &user).await?;

    // The conversation so far, along the branch this message continues.
    let prior_turns = ancestor_turns(&state, doc.id, body.parent_id).await?;

    // One id identifies the turn, the run, and the WS stream: the client
    // subscribes to `run_id` before this response even arrives.
    let session_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO runs (id, doc_id, user_id, kind, status) VALUES ($1, $2, $3, 'agent', 'running')",
    )
    .bind(session_id)
    .bind(doc.id)
    .bind(user.id)
    .execute(&state.db)
    .await?;
    sqlx::query(
        "INSERT INTO agent_turns (id, doc_id, user_id, parent_id, prompt) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(session_id)
    .bind(doc.id)
    .bind(user.id)
    .bind(body.parent_id)
    .bind(&body.prompt)
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
        let status =
            match run_agent_session(&state2, session_id, &doc, &prompt, prior_turns, &llm_config)
                .await
            {
                Ok(summary) => {
                    let _ = sqlx::query(
                        "UPDATE agent_turns SET answer = $1, status = 'ok' WHERE id = $2",
                    )
                    .bind(&summary)
                    .bind(session_id)
                    .execute(&state2.db)
                    .await;
                    "ok"
                }
                Err(e) => {
                    log::warn!("agent session {session_id} failed: {e:#}");
                    let _ = sqlx::query(
                        "UPDATE agent_turns SET status = 'error', error = $1 WHERE id = $2",
                    )
                    .bind(format!("{e:#}"))
                    .bind(session_id)
                    .execute(&state2.db)
                    .await;
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

/// The document's first `hick:container` declaration, as `(name, image)`.
///
/// First rather than "best": a document usually declares its main environment
/// first, and a wrong guess is visible in the transcript rather than silent.
/// Documents that declare none keep the standalone agent container.
fn primary_container(source: &str) -> Option<(String, String)> {
    let doc = hick_lang::parse(source).ok()?;
    doc.find_tags("container").into_iter().find_map(|tag| {
        let name = tag.get_attribute("name")?.to_string();
        let image = tag.get_attribute("image").unwrap_or("host").to_string();
        Some((name, image))
    })
}

async fn run_agent_session(
    state: &AppState,
    session_id: Uuid,
    doc: &DocRow,
    prompt: &str,
    prior_turns: Vec<PriorTurn>,
    llm_config: &crate::config::AgentLlmConfig,
) -> anyhow::Result<String> {
    // Workspace: temp dir seeded from the project checkout (the agent's
    // scripts and its session file live here until persisted to git).
    let tmp = tempfile::tempdir()?;
    state
        .git
        .seed_checkout_async(doc.project_id, tmp.path())
        .await?;
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

    let executor = crate::executor::build_executor(state.config.executor).await?;
    let llm = hickory_agent::client_for(
        &llm_config.provider,
        llm_config.model.as_deref(),
        Some(&llm_config.api_key),
    )?;
    let mut config = AgentConfig::new(prompt, tmp.path());
    // Naming the primary document is what enables the document tool set
    // (read_doc / read_output / edit_output / edit_doc / verify). Without it
    // the agent could only write scripts and hope — the doctrine this product
    // is built around would be off in the product itself.
    config.doc_path = Some(doc_file.clone());
    config.doc_context = Some(doc.source.clone());
    config.prior_turns = prior_turns;
    // Run in the document's OWN container when it declares one. Otherwise the
    // agent works in a side room: its scripts land in a different filesystem
    // and toolchain from the `hick:exec` cells it is editing, so a file it
    // writes is not a file the document can run.
    if let Some((name, image)) = primary_container(&doc.source) {
        config.container = name;
        config.image = image;
    }

    let mut on_event = |event: AgentEvent| {
        let _ = event_tx.send(event);
    };
    let outcome = run_agent(&llm, executor.clone(), &config, &mut on_event).await;
    drop(event_tx);
    forwarder.await.ok();
    executor.shutdown().await.ok();

    let outcome = outcome?;

    // Write the agent's document edits back.
    //
    // The agent works in a temp checkout, so `edit_doc` / `edit_output` change
    // a file that is about to be deleted. Without this the tool truthfully
    // reports "the document was updated through lineage and re-woven" and the
    // user sees absolutely nothing change — the whole premise that agent
    // output is literate-programming state in git, silently dropped on the
    // floor.
    let edited = std::fs::read_to_string(&doc_file).unwrap_or_default();
    if !edited.is_empty() && edited != doc.source {
        sqlx::query("UPDATE docs SET source = $1, updated_at = now() WHERE id = $2")
            .bind(&edited)
            .bind(doc.id)
            .execute(&state.db)
            .await?;
        // Anyone with the document open holds their own CRDT copy, which would
        // otherwise win the next persist and revert the agent's work.
        state
            .rooms
            .apply_external_source(state, doc.id, &edited)
            .await;
        state
            .git
            .save_file(
                doc.project_id,
                &doc.path,
                &edited,
                &format!("Agent edit: {}", doc.path),
            )
            .await?;
        log::info!(
            "agent session {session_id} rewrote {} ({} -> {} bytes)",
            doc.path,
            doc.source.len(),
            edited.len()
        );
    }

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
    Ok(outcome.summary)
}

#[cfg(test)]
mod tests {
    use super::primary_container;

    const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
<hick:container name="py" image="python:3.12" />
<hick:container name="duck" image="duckdb/duckdb:v1.5.5" />
</hick:doc>
"#;

    #[test]
    fn the_agent_joins_the_document_s_first_container() {
        assert_eq!(
            primary_container(DOC),
            Some(("py".to_string(), "python:3.12".to_string()))
        );
    }

    #[test]
    fn a_document_with_no_containers_keeps_the_standalone_agent_room() {
        let bare = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">hi</hick:doc>";
        assert_eq!(primary_container(bare), None);
        // Unparseable source must not panic or invent a container.
        assert_eq!(primary_container("<hick:doc"), None);
    }

    #[test]
    fn a_container_without_an_image_still_names_a_room() {
        let src = "<hick:doc xmlns:hick=\"http://www.hickorydocs.com/1.0\">\
                   <hick:container name=\"shell\" /></hick:doc>";
        assert_eq!(
            primary_container(src),
            Some(("shell".to_string(), "host".to_string()))
        );
    }
}
