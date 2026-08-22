//! The in-app agent: the same ReAct loop `hick agent` runs, behind the
//! document view's chat dock.
//!
//! One `POST /api/docs/:id/agent` is one agent run — up to `max_turns`
//! internal LLM turns, scripts through this session's executor, the document
//! edit tool set on the posted document. Nothing is reduced relative to the
//! CLI: the decision on record is that the dock gets the full tool surface.
//!
//! The conversation is a TREE. Each POST names the turn it continues from
//! (`parent_id`); replaying the ancestor chain into
//! [`AgentConfig::prior_turns`] is what makes a reply a reply, and what makes
//! rewinding cheap — a branch is just a different ancestor chain fed to a
//! fresh loop. The tree lives in server state for the session's lifetime;
//! the durable record is the `hick:session` file each run writes under
//! `<served folder>/sessions/`, exactly as the CLI does.
//!
//! Events stream on the WS run channel (`0x01`) as
//! `{run_id: <turn id>, exec_id: "agent", event: <AgentEvent>}` with a
//! terminal `{run_id, status}`, per `docs/specs/freeform/api.md` — the same
//! frames, socket, and discipline as cell transcripts, so the dock needs no
//! second transport.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use hickory_agent::{
    AGENT_EXEC_ID, AgentConfig, AgentEvent, DEFAULT_ANTHROPIC_MODEL, LlmClient, PriorTurn,
    ProviderSelection, Usage, client_for_with_store, cost_usd, resolve_selector_with_store,
    run_agent,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::api::{ApiError, ApiResult};
use super::{LocalState, now_rfc3339, rand_id, store};

/// One exchange in a document's conversation, in the wire shape the dock
/// renders (`apps/web/src/api/types.ts::AgentTurn`).
#[derive(Clone, Serialize)]
pub struct TurnRecord {
    pub id: String,
    pub parent_id: Option<String>,
    pub prompt: String,
    pub answer: Option<String>,
    /// `"running"`, `"ok"`, or `"error"`.
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
    /// Provider selector this turn ran on (`"anthropic"`, `"openai"`, …;
    /// `"scripted"` under the test seam with no explicit choice).
    pub provider: String,
    /// Model id this turn ran on. A turn records what it actually ran on,
    /// so changing the model mid-conversation stays visible per turn.
    pub model: String,
    /// The turn's final four-way token usage. `None` while running or when
    /// the run failed before reporting usage.
    pub usage: Option<Usage>,
    /// The session file this turn is recorded in, relative to the served
    /// folder. One file per conversation: a child turn appends to its
    /// parent's file, so the tree is on disk and survives a restart.
    pub session: String,
}

/// The per-document model choice the dock last posted. `None` fields mean
/// "the resolved default".
#[derive(Default, Clone)]
pub struct DocSelection {
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// The conversation state one local session holds.
#[derive(Default)]
pub struct AgentHub {
    /// Turn tree per document id, in creation order (the dock treats the
    /// last element as the default tip).
    turns: Mutex<HashMap<String, Vec<TurnRecord>>>,
    /// The model choice each document's dock last posted, applied to
    /// subsequent turns until changed (see [`AgentRequest`]).
    selection: Mutex<HashMap<String, DocSelection>>,
    /// Test seam: a canned client instead of a provider from the
    /// environment. Set through [`AgentHub::set_llm_override`] by the serve
    /// integration tests so a full turn runs without any API key.
    llm_override: Mutex<Option<Arc<dyn LlmClient>>>,
}

impl AgentHub {
    /// Replace provider resolution with a fixed client (tests, offline dev).
    pub fn set_llm_override(&self, llm: Arc<dyn LlmClient>) {
        *self.llm_override.lock().unwrap() = Some(llm);
    }

    /// Rebuild a document's turns from its session files when the hub holds
    /// none for it — the app was restarted, or this is the first look. The
    /// files are the durable record; memory is a cache of them.
    fn hydrate(&self, root: &std::path::Path, doc_id: &str, doc_path: &std::path::Path) {
        let mut map = self.turns.lock().unwrap();
        if map.contains_key(doc_id) {
            return;
        }
        let mut turns: Vec<TurnRecord> = Vec::new();
        for (path, view) in
            hickory_agent::session_view::conversations_for(&root.join("sessions"), doc_path)
        {
            let rel = path
                .strip_prefix(root)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|_| path.display().to_string());
            for t in view.turns {
                turns.push(TurnRecord {
                    id: t.id,
                    parent_id: t.parent,
                    prompt: t.prompt,
                    answer: t.answer,
                    status: "ok".into(),
                    error: None,
                    created_at: view.start.clone().unwrap_or_default(),
                    provider: t.provider.unwrap_or_default(),
                    model: t.model.unwrap_or_default(),
                    usage: t.usage.map(|u| Usage {
                        input_tokens: u.input,
                        cache_creation_input_tokens: u.cache_write,
                        cache_read_input_tokens: u.cache_read,
                        output_tokens: u.output,
                    }),
                    session: rel.clone(),
                });
            }
        }
        map.insert(doc_id.to_string(), turns);
    }

    fn snapshot(&self, doc_id: &str) -> Vec<TurnRecord> {
        self.turns
            .lock()
            .unwrap()
            .get(doc_id)
            .cloned()
            .unwrap_or_default()
    }

    fn finish(&self, doc_id: &str, turn_id: &str, outcome: Result<(String, Usage), String>) {
        let mut map = self.turns.lock().unwrap();
        let Some(turn) = map
            .get_mut(doc_id)
            .and_then(|turns| turns.iter_mut().find(|t| t.id == turn_id))
        else {
            return;
        };
        match outcome {
            Ok((answer, usage)) => {
                turn.answer = Some(answer);
                turn.usage = Some(usage);
                turn.status = "ok".into();
            }
            Err(message) => {
                turn.error = Some(message);
                turn.status = "error".into();
            }
        }
    }

    /// Fold the fields of one POST into the document's stored selection and
    /// return the result. A field absent from the body keeps what is stored;
    /// a present-but-empty field clears back to the default — so the dock's
    /// controls are always exactly what the next turn runs on.
    fn update_selection(
        &self,
        doc_id: &str,
        provider: Option<String>,
        model: Option<String>,
    ) -> DocSelection {
        let mut map = self.selection.lock().unwrap();
        let sel = map.entry(doc_id.to_string()).or_default();
        if let Some(p) = provider {
            sel.provider = Some(p.trim().to_string()).filter(|s| !s.is_empty());
        }
        if let Some(m) = model {
            sel.model = Some(m.trim().to_string()).filter(|s| !s.is_empty());
        }
        sel.clone()
    }

    fn selection_of(&self, doc_id: &str) -> DocSelection {
        self.selection
            .lock()
            .unwrap()
            .get(doc_id)
            .cloned()
            .unwrap_or_default()
    }
}

/// The model a provider selector runs by default, for the turns listing —
/// what a turn would use when nothing is chosen. Mirrors the client
/// constructors: Anthropic's alias constant, and each OpenAI-compatible
/// vendor's `MODEL` environment override then its built-in default.
fn default_model_for(selector: &str) -> String {
    match ProviderSelection::parse(selector) {
        Some(ProviderSelection::Compatible(p)) => std::env::var(p.model_env())
            .ok()
            .filter(|m| !m.trim().is_empty())
            .unwrap_or_else(|| p.default_model().to_string()),
        Some(ProviderSelection::Anthropic) | None => DEFAULT_ANTHROPIC_MODEL.to_string(),
    }
}

/// Session totals across a document's turns: the summed four-way usage and
/// the summed USD cost. The cost is `None` when any usage-bearing turn ran
/// on a model with no known price — an unknown price makes the *total*
/// unknown, never silently understated.
pub fn totals_of(turns: &[TurnRecord]) -> (Usage, Option<f64>) {
    let mut total = Usage::default();
    let mut usd = 0.0;
    let mut known = true;
    for turn in turns {
        let Some(usage) = &turn.usage else { continue };
        total.add(usage);
        match cost_usd(&turn.model, usage) {
            Some(cost) => usd += cost,
            None if usage.is_zero() => {}
            None => known = false,
        }
    }
    (total, known.then_some(usd))
}

/// The ancestor chain of `tip`, oldest first, as replayable history.
///
/// Only completed exchanges replay: a turn that errored produced no answer
/// to stand in the assistant seat, and replaying a half-exchange would feed
/// the model a conversation that never happened. A cycle (impossible through
/// the API, cheap to guard) terminates instead of hanging the request.
pub fn prior_turns_of(turns: &[TurnRecord], tip: Option<&str>) -> Vec<PriorTurn> {
    let by_id: HashMap<&str, &TurnRecord> = turns.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut chain = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut cursor = tip;
    while let Some(id) = cursor {
        if !seen.insert(id.to_string()) {
            break;
        }
        let Some(turn) = by_id.get(id) else { break };
        if let Some(answer) = &turn.answer {
            chain.push(PriorTurn {
                prompt: turn.prompt.clone(),
                answer: answer.clone(),
            });
        }
        cursor = turn.parent_id.as_deref();
    }
    chain.reverse();
    chain
}

/// Body of `POST /api/docs/:id/agent`.
#[derive(Deserialize)]
pub struct AgentRequest {
    pub prompt: String,
    /// The turn this one continues from. Naming an older turn forks a
    /// branch; `null` starts a new thread.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Provider selector for this and subsequent turns (`"anthropic"`,
    /// `"openai"`, `"deepseek"`, `"grok"`, `"openrouter"`). Absent keeps
    /// the document's current choice; empty clears back to the resolved
    /// default. Persists in the hub for the session.
    #[serde(default)]
    pub provider: Option<String>,
    /// Model id override, same absent/empty semantics as `provider`.
    /// Free text: providers accept ids we cannot enumerate.
    #[serde(default)]
    pub model: Option<String>,
}

/// `POST /api/docs/:id/agent` — start one agent run on the document.
///
/// Returns `{session_id}` immediately; the run proceeds in the background,
/// streaming on the run channel. The session id doubles as the turn id and
/// the `run_id` of its stream.
pub async fn start_turn(
    State(state): State<LocalState>,
    Path(id): Path<String>,
    Json(body): Json<AgentRequest>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let doc_path = state
        .index
        .absolute(&id)
        .ok_or_else(|| ApiError::not_found(format!("no document {id} in this session")))?;
    let prompt = body.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(ApiError::bad_request(
            "the agent needs a prompt — say what to do to this document",
        ));
    }

    // Reject an unknown provider before anything is recorded or stored: a
    // typo must not silently run (or persist) as some other vendor.
    if let Some(p) = body.provider.as_deref().map(str::trim)
        && !p.is_empty()
        && ProviderSelection::parse(p).is_none()
    {
        return Err(ApiError::bad_request(format!(
            "unknown provider {p:?} — choose one of: {}",
            ProviderSelection::ALL.join(", ")
        )));
    }
    let selection = state
        .agent
        .update_selection(&id, body.provider.clone(), body.model.clone());

    // Provider next, before anything is recorded: a machine with no key
    // gets a configuration note, never a half-started turn. The leading
    // phrase is a contract with the desktop client: ChatDock matches
    // "agent not available" to render a quiet note instead of a red error
    // (apps/web/src/components/ChatDock.tsx). Change both together.
    let mut provider_name = selection.provider.clone();
    let llm: Arc<dyn LlmClient> = {
        let overridden = state.agent.llm_override.lock().unwrap().clone();
        match overridden {
            Some(llm) => llm,
            None => {
                // Keys saved in Settings come first, the environment second
                // — the desktop app has no shell profile to export from, so
                // the store is its keys' home. A clone, not the lock, is
                // held across the resolution: nothing here may block a
                // concurrent PUT /api/settings/keys.
                let store = state
                    .keys
                    .store
                    .read()
                    .expect("key store lock poisoned")
                    .clone();
                let provider = resolve_selector_with_store(selection.provider.as_deref(), &store)
                    .map_err(|e| {
                    ApiError::unavailable(format!("agent not available — {e:#}"))
                })?;
                let llm = client_for_with_store(&provider, selection.model.as_deref(), &store)
                    .map_err(|e| ApiError::unavailable(format!("agent not available — {e:#}")))?;
                provider_name = Some(provider);
                llm
            }
        }
    };
    // What this turn runs on, recorded up front so the listing tells the
    // truth even while the turn is still streaming. Under the test seam with
    // no explicit choice the provider is the seam's name, "scripted".
    let provider_name = provider_name.unwrap_or_else(|| "scripted".to_string());
    let model_name = selection
        .model
        .clone()
        .unwrap_or_else(|| llm.model_name().to_string());

    let turn_id = format!("{:016x}", rand_id());
    state.agent.hydrate(state.index.root(), &id, &doc_path);
    let parent_for_run = body.parent_id.clone();
    let (prior_turns, session_rel) = {
        let mut map = state.agent.turns.lock().unwrap();
        let turns = map.entry(id.clone()).or_default();
        if turns.iter().any(|t| t.status == "running") {
            // The edit session is single-writer; a second concurrent turn
            // would race it on the same file.
            return Err(ApiError::conflict(
                "an agent turn is already running on this document — wait for it to \
                 finish, then send again",
            ));
        }
        if let Some(parent) = &body.parent_id
            && !turns.iter().any(|t| &t.id == parent)
        {
            return Err(ApiError::bad_request(format!(
                "parent turn {parent} does not exist on this document — refresh the \
                 conversation and pick a turn that does"
            )));
        }
        let prior = prior_turns_of(turns, body.parent_id.as_deref());
        // The conversation's file: the parent's when continuing, a fresh one
        // at the root of a new thread.
        let session_rel = body
            .parent_id
            .as_deref()
            .and_then(|p| turns.iter().find(|t| t.id == p))
            .map(|t| t.session.clone())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                let abs = hickory_agent::session_file_path(state.index.root(), &prompt);
                abs.strip_prefix(state.index.root())
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| abs.display().to_string())
            });
        turns.push(TurnRecord {
            id: turn_id.clone(),
            parent_id: body.parent_id.clone(),
            prompt: prompt.clone(),
            answer: None,
            status: "running".into(),
            error: None,
            created_at: now_rfc3339(),
            provider: provider_name,
            model: model_name,
            usage: None,
            session: session_rel.clone(),
        });
        (prior, session_rel)
    };

    let task_state = state.clone();
    let doc_id = id.clone();
    let run_key = turn_id.clone();
    tokio::spawn(async move {
        let outcome = run_turn(
            &task_state,
            &doc_id,
            llm,
            doc_path,
            prompt,
            prior_turns,
            TurnIdentity {
                session: task_state.index.root().join(&session_rel),
                turn: run_key.clone(),
                parent: parent_for_run.clone(),
            },
        )
        .await;
        let status = if outcome.is_ok() { "ok" } else { "failed" };
        task_state.agent.finish(&doc_id, &run_key, outcome);

        // The agent may have edited the document (or an upstream one) on
        // disk; a live room still serving pre-run text would write that
        // stale text back over the agent's work on its next debounce, so
        // every room is reconciled before anyone types again — the same
        // move `api::edit_outputs` makes.
        for (other_id, _) in task_state.index.entries() {
            if let Some(path) = task_state.index.absolute(&other_id)
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                task_state
                    .rooms
                    .apply_external_source(&other_id, &text)
                    .await;
            }
        }

        task_state
            .rooms
            .publish_run_event(&doc_id, &json!({ "run_id": run_key, "status": status }))
            .await;
    });

    Ok((StatusCode::ACCEPTED, Json(json!({ "session_id": turn_id }))))
}

/// Run one agent turn to completion, streaming its events on the run
/// channel. Returns the final answer plus the turn's total token usage, or
/// the error message to record.
/// Where a turn is recorded and how it is named in the file.
struct TurnIdentity {
    session: std::path::PathBuf,
    turn: String,
    parent: Option<String>,
}

async fn run_turn(
    state: &LocalState,
    doc_id: &str,
    llm: Arc<dyn LlmClient>,
    doc_path: std::path::PathBuf,
    prompt: String,
    prior_turns: Vec<PriorTurn>,
    identity: TurnIdentity,
) -> Result<(String, Usage), String> {
    let turn_id: &str = &identity.turn.clone();
    // Flush the live room to disk first: the agent's edit session reads the
    // file, and running against text missing the newest keystrokes would
    // edit a document the user is no longer looking at.
    if let Some(room) = state.rooms.get(doc_id).await {
        let live = room.text().await;
        if live != std::fs::read_to_string(&doc_path).unwrap_or_default()
            && let Err(e) = store::write_atomic(&doc_path, live.as_bytes())
        {
            return Err(format!(
                "could not flush unsaved edits to {}: {e:#}",
                doc_path.display()
            ));
        }
    }

    // The session's executor choice (HICKORY_EXECUTOR; sandbox by default)
    // applies to the agent's scripts exactly as it does to cell runs.
    let executor = match state.executor.build().await {
        Ok(executor) => executor,
        Err(e) => {
            return Err(format!(
                "could not start the {} executor: {e:#}",
                state.executor.as_str()
            ));
        }
    };

    // `AgentConfig::new` defaults to 20 internal turns per prompt, matching
    // `hick agent`. The session file lands in `<served folder>/sessions/`.
    let mut config = AgentConfig::new(prompt, state.index.root());
    config.doc_path = Some(doc_path);
    config.prior_turns = prior_turns;
    // One file per conversation, each turn naming its parent: the tree is
    // on disk, and `hydrate` rebuilds the dock from it after a restart.
    config.session_path = Some(identity.session);
    config.turn_id = Some(identity.turn);
    config.parent_turn_id = identity.parent;

    // `run_agent`'s callback is synchronous; publishing is async. A channel
    // decouples them: the loop pushes, a forwarder task publishes in order.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<AgentEvent>();
    let forwarder = {
        let state = state.clone();
        let doc_id = doc_id.to_string();
        let run_id = turn_id.to_string();
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                state
                    .rooms
                    .publish_run_event(
                        &doc_id,
                        &json!({ "run_id": run_id, "exec_id": AGENT_EXEC_ID, "event": event }),
                    )
                    .await;
            }
        })
    };

    let mut on_event = move |event: AgentEvent| {
        let _ = tx.send(event);
    };
    let outcome = run_agent(llm.as_ref(), executor.clone(), &config, &mut on_event).await;
    if let Err(e) = executor.shutdown().await {
        log::warn!("agent executor shutdown failed: {e:#}");
    }
    // Dropping the callback drops the sender; the forwarder then drains what
    // is queued and exits, so the terminal status frame cannot overtake the
    // last token.
    drop(on_event);
    let _ = forwarder.await;

    match outcome {
        Ok(outcome) => Ok((outcome.summary, outcome.total_usage)),
        Err(e) => {
            log::warn!("agent turn {turn_id} failed: {e:#}");
            Err(format!("{e:#}"))
        }
    }
}

/// `GET /api/docs/:id/agent/turns` — the document's turn tree, in creation
/// order, plus what the next turn would run on and the session's spend:
///
/// - `provider` / `model` — the current choice with defaults resolved
///   (e.g. `"anthropic"` / `"claude-sonnet-5"` on a machine with only an
///   Anthropic key and nothing chosen).
/// - `totals` — `{usd, input, output, cache_read, cache_write}` summed
///   across the document's finished turns, each turn priced on the model it
///   ran on. `usd` is `null` when any turn's model has no known price.
///
/// Empty turns for a document nothing has asked about yet, which the dock
/// renders as an empty conversation rather than an error.
pub async fn list_turns(State(state): State<LocalState>, Path(id): Path<String>) -> Json<Value> {
    if let Some(doc_path) = state.index.absolute(&id) {
        state.agent.hydrate(state.index.root(), &id, &doc_path);
    }
    let turns = state.agent.snapshot(&id);
    let selection = state.agent.selection_of(&id);
    let provider = selection.provider.clone().unwrap_or_else(|| {
        // Under the test seam provider resolution is bypassed (and reading
        // key variables here would race the serve tests' env scrub), so the
        // listing reports the stock default.
        if state.agent.llm_override.lock().unwrap().is_some() {
            return "anthropic".to_string();
        }
        let store = state
            .keys
            .store
            .read()
            .expect("key store lock poisoned")
            .clone();
        // With no keys anywhere this GET must still answer (the dock shows
        // the note only when a POST fails), so fall back to the default
        // vendor name rather than erroring the listing.
        resolve_selector_with_store(None, &store).unwrap_or_else(|_| "anthropic".to_string())
    });
    let model = selection
        .model
        .clone()
        .unwrap_or_else(|| default_model_for(&provider));
    let (total, usd) = totals_of(&turns);
    Json(json!({
        "turns": turns,
        "provider": provider,
        "model": model,
        "totals": {
            "usd": usd,
            "input": total.input_tokens,
            "output": total.output_tokens,
            "cache_read": total.cache_read_input_tokens,
            "cache_write": total.cache_creation_input_tokens,
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn turn(id: &str, parent: Option<&str>, answer: Option<&str>) -> TurnRecord {
        TurnRecord {
            id: id.into(),
            parent_id: parent.map(Into::into),
            prompt: format!("p:{id}"),
            answer: answer.map(Into::into),
            status: if answer.is_some() { "ok" } else { "error" }.into(),
            error: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            provider: "anthropic".into(),
            model: "claude-sonnet-5".into(),
            usage: None,
            session: String::new(),
        }
    }

    //        root
    //       /    \
    //      a      b        <- a rewind at `root` forked `b`
    //      |
    //      a2
    fn tree() -> Vec<TurnRecord> {
        vec![
            turn("root", None, Some("a:root")),
            turn("a", Some("root"), Some("a:a")),
            turn("a2", Some("a"), Some("a:a2")),
            turn("b", Some("root"), Some("a:b")),
        ]
    }

    #[test]
    fn a_reply_replays_exactly_its_own_branch_oldest_first() {
        let prior = prior_turns_of(&tree(), Some("a2"));
        let prompts: Vec<&str> = prior.iter().map(|t| t.prompt.as_str()).collect();
        assert_eq!(prompts, ["p:root", "p:a", "p:a2"]);

        // The fork sees the shared ancestor, never its sibling branch.
        let prior = prior_turns_of(&tree(), Some("b"));
        let prompts: Vec<&str> = prior.iter().map(|t| t.prompt.as_str()).collect();
        assert_eq!(prompts, ["p:root", "p:b"]);
    }

    #[test]
    fn a_new_thread_replays_nothing() {
        assert!(prior_turns_of(&tree(), None).is_empty());
    }

    #[test]
    fn an_errored_ancestor_is_skipped_not_replayed_half_finished() {
        let turns = vec![
            turn("root", None, Some("a:root")),
            turn("broken", Some("root"), None),
            turn("next", Some("broken"), Some("a:next")),
        ];
        let prior = prior_turns_of(&turns, Some("next"));
        let prompts: Vec<&str> = prior.iter().map(|t| t.prompt.as_str()).collect();
        assert_eq!(prompts, ["p:root", "p:next"]);
    }

    fn priced_turn(id: &str, model: &str, usage: Option<Usage>) -> TurnRecord {
        TurnRecord {
            model: model.into(),
            usage,
            ..turn(id, None, Some("a"))
        }
    }

    #[test]
    fn totals_sum_usage_and_price_each_turn_on_its_own_model() {
        let usage = Usage {
            input_tokens: 1_000_000,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
            output_tokens: 1_000_000,
        };
        let turns = vec![
            priced_turn("a", "claude-sonnet-5", Some(usage)), // $3 + $15
            priced_turn("b", "claude-haiku-4-5", Some(usage)), // $1 + $5
            priced_turn("running", "claude-sonnet-5", None),  // no usage yet
        ];
        let (total, usd) = totals_of(&turns);
        assert_eq!(total.input_tokens, 2_000_000);
        assert_eq!(total.output_tokens, 2_000_000);
        let usd = usd.expect("both models are priced");
        assert!((usd - 24.0).abs() < 1e-9, "got {usd}");
    }

    #[test]
    fn an_unpriced_model_with_real_usage_makes_the_total_unknown_not_understated() {
        let usage = Usage {
            input_tokens: 10,
            ..Default::default()
        };
        let turns = vec![
            priced_turn("a", "claude-sonnet-5", Some(usage)),
            priced_turn("b", "mystery-model", Some(usage)),
        ];
        let (total, usd) = totals_of(&turns);
        assert_eq!(total.input_tokens, 20, "tokens still sum");
        assert!(usd.is_none(), "an unknown price is unknown, never a guess");

        // But a zero-usage turn on an unpriced model (the scripted seam)
        // costs nothing and must not poison the total.
        let turns = vec![
            priced_turn("a", "claude-sonnet-5", Some(usage)),
            priced_turn("b", "scripted", Some(Usage::default())),
        ];
        let (_, usd) = totals_of(&turns);
        assert!(usd.is_some());
    }

    #[test]
    fn no_turns_total_to_zero_usage_and_zero_cost() {
        let (total, usd) = totals_of(&[]);
        assert!(total.is_zero());
        assert_eq!(usd, Some(0.0));
    }

    #[test]
    fn a_cycle_terminates_instead_of_hanging_the_request() {
        let turns = vec![
            turn("x", Some("y"), Some("a")),
            turn("y", Some("x"), Some("a")),
        ];
        assert!(prior_turns_of(&turns, Some("x")).len() <= 2);
    }
}
