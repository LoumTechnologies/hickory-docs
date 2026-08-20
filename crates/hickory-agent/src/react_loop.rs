//! The script-first ReAct loop.
//!
//! LLM emits `<hick:next>code</hick:next>` + a fenced code block → the script
//! runs through the [`Executor`] → stdout/stderr comes back as the next
//! observation. `<hick:next>done</hick:next>` (no code block) ends the loop.
//! Every step is written incrementally to a `hick:session` document and
//! emitted as an [`AgentEvent`] for the WS run channel.

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result};
use futures::StreamExt as _;
use hickory_executor::Executor;

use crate::events::AgentEvent;
use crate::llm::{LlmClient, Message, Role};
use crate::protocol::{
    SYSTEM_PROMPT, TOOLS_SYSTEM_PROMPT, Turn, correction_message, parse_response,
};
use crate::script::{AGENT_CONTAINER, run_script};
use crate::session::{HickSessionLog, SessionEvent, SessionLog, session_file_path};
use crate::tools::{EditSession, execute_tool};
use crate::usage::{Usage, cost_usd};

/// Configuration for one agent run.
pub struct AgentConfig {
    /// The user's task prompt.
    pub prompt: String,
    /// Optional document source given as inline context (pure-script
    /// sessions; superseded by [`AgentConfig::doc_path`]).
    pub doc_context: Option<String>,
    /// The session's primary document (`--doc`). When set, the document
    /// edit tool set (read_doc/read_output/edit_output/edit_doc/verify) is
    /// enabled and an [`EditSession`] is opened on this path.
    pub doc_path: Option<PathBuf>,
    /// Project directory: session files land in `<project_dir>/sessions/`.
    pub project_dir: PathBuf,
    /// Maximum LLM turns before giving up (default 20).
    pub max_turns: usize,
    /// Limits applied to every script this session runs.
    pub script_limits: crate::script::ScriptLimits,
    /// Container image recorded for the agent workspace (LocalExecutor
    /// ignores it).
    pub image: String,
    /// Container the agent's scripts run in.
    ///
    /// Set this to one the document declares and the agent stops working in a
    /// side room: its scripts and the document's `hick:exec` cells share a
    /// filesystem and a toolchain, so a file the agent writes is a file the
    /// cells can run. Defaults to [`AGENT_CONTAINER`].
    pub container: String,
    /// Earlier exchanges to replay as conversation history, oldest first.
    ///
    /// This is what makes a chat a chat rather than a series of unrelated
    /// sessions — and what makes rewinding cheap: a branch is just a different
    /// ancestor chain, replayed into a fresh loop.
    pub prior_turns: Vec<PriorTurn>,
}

/// One completed exchange being replayed as history.
#[derive(Debug, Clone)]
pub struct PriorTurn {
    pub prompt: String,
    pub answer: String,
}

impl AgentConfig {
    /// Config with defaults for `prompt` in `project_dir`.
    pub fn new(prompt: impl Into<String>, project_dir: impl Into<PathBuf>) -> Self {
        Self {
            prompt: prompt.into(),
            doc_context: None,
            doc_path: None,
            project_dir: project_dir.into(),
            max_turns: 20,
            script_limits: crate::script::ScriptLimits::default(),
            image: "host".into(),
            container: AGENT_CONTAINER.to_string(),
            prior_turns: Vec::new(),
        }
    }
}

/// The result of a completed agent run.
pub struct AgentOutcome {
    /// The agent's final answer.
    pub summary: String,
    /// Path of the written `hick:session` file.
    pub session_path: PathBuf,
    /// Number of LLM turns used.
    pub turns: usize,
    /// Accumulated four-way token usage for the whole session.
    pub total_usage: Usage,
    /// Accumulated USD spend (`None` when the model has no known price).
    pub total_cost_usd: Option<f64>,
}

/// Maximum consecutive protocol violations before the run aborts.
const MAX_CONSECUTIVE_INVALID: usize = 3;

/// Run the script-first ReAct loop to completion.
///
/// Streams [`AgentEvent`]s to `on_event` and writes the session to
/// `<project_dir>/sessions/<timestamp>-<slug>.hick`. Returns the final
/// answer and the session path.
pub async fn run_agent(
    llm: &dyn LlmClient,
    executor: Arc<dyn Executor>,
    config: &AgentConfig,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<AgentOutcome> {
    let session_path = session_file_path(&config.project_dir, &config.prompt);
    let session = HickSessionLog::create(&session_path)
        .with_context(|| format!("failed to create session file {}", session_path.display()))?;

    on_event(AgentEvent::SessionStarted {
        model: llm.model_name().to_string(),
        session_path: session_path.display().to_string(),
    });

    executor
        .ensure_started(&config.container, &config.image)
        .await
        .context("failed to start agent container")?;

    // The primary document opens an edit session: single-writer, re-woven
    // after every successful edit, so staleness is impossible inside it.
    let mut edit_session: Option<EditSession> =
        match &config.doc_path {
            Some(path) => Some(EditSession::open(path, &[]).await.with_context(|| {
                format!("failed to open the primary document {}", path.display())
            })?),
            None => None,
        };

    // Prompt-cache contract: the FIRST system message is the frozen,
    // byte-stable prefix (protocol + tool doctrine — no timestamps, no
    // session ids, no interpolation), shared across every session of the
    // same mode (with/without tools) on the same build. Anything
    // per-session (doc path, doc context) goes in a SEPARATE later system
    // message so it gets its own cache breakpoint without invalidating the
    // frozen one. See `llm_anthropic`'s module docs for the breakpoint
    // layout.
    let mut frozen = SYSTEM_PROMPT.to_string();
    if edit_session.is_some() {
        frozen.push_str(TOOLS_SYSTEM_PROMPT);
    }
    let mut history = vec![Message::new(Role::System, frozen)];
    // The shell the model is writing FOR belongs here rather than in the
    // frozen prefix: it is a property of this session's executor, and the
    // frozen text has to stay byte-identical across sessions to keep its cache
    // breakpoint. Without it the model writes bash on a Windows machine and
    // every block fails for a reason it cannot see.
    let mut session_context = format!(
        "Your code runs on {}. Shell blocks are saved as a script and run by \
         that shell, so write for it.",
        executor.script_platform().describe()
    );
    if let Some(es) = &edit_session {
        session_context.push_str(&format!(
            "\n\nPrimary document of this session: {}",
            es.doc_path().display()
        ));
    }
    if let Some(doc) = &config.doc_context {
        if !session_context.is_empty() {
            session_context.push_str("\n\n");
        }
        session_context.push_str(&format!("## Document under discussion\n\n{doc}"));
    }
    if !session_context.is_empty() {
        history.push(Message::new(Role::System, session_context));
    }
    // Replay earlier exchanges before the new prompt. They go AFTER the system
    // messages so the frozen cache prefix stays byte-identical, and the growing
    // conversation extends that prefix instead of invalidating it.
    for turn in &config.prior_turns {
        history.push(Message::new(Role::User, turn.prompt.clone()));
        history.push(Message::new(Role::Assistant, turn.answer.clone()));
    }
    history.push(Message::new(Role::User, config.prompt.clone()));
    session.record(SessionEvent::User {
        text: &config.prompt,
    });
    on_event(AgentEvent::UserMessage {
        text: config.prompt.clone(),
    });

    let mut action_index = 0usize;
    let mut consecutive_invalid = 0usize;
    let mut total_usage = Usage::default();

    for turn in 0..config.max_turns {
        on_event(AgentEvent::Thinking);
        let (response, turn_usage) = stream_completion(llm, history.clone(), on_event).await?;
        total_usage.add(&turn_usage);
        let turn_cost = cost_usd(llm.model_name(), &turn_usage);
        let total_cost = cost_usd(llm.model_name(), &total_usage);
        session.record(SessionEvent::Usage {
            turn: Some(turn),
            usage: turn_usage,
            cost_usd: turn_cost,
        });
        on_event(AgentEvent::TurnUsage {
            turn,
            usage: turn_usage,
            cost_usd: turn_cost,
            total_usage,
            total_cost_usd: total_cost,
        });
        on_event(AgentEvent::ResponseComplete {
            text: response.clone(),
        });

        match parse_response(&response) {
            Turn::Invalid { reason } => {
                consecutive_invalid += 1;
                if consecutive_invalid > MAX_CONSECUTIVE_INVALID {
                    session.record(SessionEvent::End);
                    let message =
                        format!("agent gave {consecutive_invalid} malformed responses in a row");
                    on_event(AgentEvent::Error {
                        message: message.clone(),
                    });
                    anyhow::bail!(message);
                }
                on_event(AgentEvent::Reprompt {
                    reason: reason.clone(),
                    attempt: consecutive_invalid,
                });
                // An empty response is a malformed response, and replaying it
                // verbatim is fatal: every provider rejects an empty content
                // block (Anthropic: 400 "text content blocks must be
                // non-empty"), so the correction turn kills the session it was
                // trying to rescue — losing every edit made so far. Say what
                // happened instead, which is also more useful to the model
                // than a blank turn.
                let recorded = if response.trim().is_empty() {
                    "(no output)".to_string()
                } else {
                    response
                };
                history.push(Message::new(Role::Assistant, recorded));
                history.push(Message::new(Role::User, correction_message(&reason)));
            }
            Turn::Code { thought, block } => {
                consecutive_invalid = 0;
                session.record(SessionEvent::Assistant {
                    prose: thought.as_deref().unwrap_or(""),
                    action: Some((block.language.hick_lang(), &block.code)),
                });
                on_event(AgentEvent::ScriptStarted {
                    lang: block.language.to_string(),
                    data: block.code.clone(),
                });

                let result = run_script(
                    executor.as_ref(),
                    &config.container,
                    &block,
                    action_index,
                    config.script_limits,
                )
                .await?;

                let mut observation_text = result.stdout.clone();
                if !result.stderr.is_empty() {
                    if !observation_text.is_empty() {
                        observation_text.push('\n');
                    }
                    observation_text.push_str(&result.stderr);
                }
                session.record(SessionEvent::Observation {
                    source: &format!("action-{action_index}"),
                    exit_code: result.exit_code,
                    text: &observation_text,
                });
                on_event(AgentEvent::ScriptFinished {
                    result: result.clone(),
                });
                action_index += 1;

                history.push(Message::new(Role::Assistant, response));
                history.push(Message::new(
                    Role::User,
                    format!("Observation:\n{}", result.as_observation()),
                ));
            }
            Turn::Tool {
                thought,
                invocation,
            } => {
                consecutive_invalid = 0;
                session.record(SessionEvent::ToolCall {
                    prose: thought.as_deref().unwrap_or(""),
                    xml: &invocation.raw_xml,
                });
                on_event(AgentEvent::ToolStarted {
                    name: invocation.name.clone(),
                    data: invocation.raw_xml.clone(),
                });

                let outcome = match edit_session.as_mut() {
                    Some(es) => execute_tool(es, executor.clone(), &invocation).await,
                    None => crate::tools::ToolOutcome {
                        name: invocation.name.clone(),
                        ok: false,
                        text: "no primary document in this session — document tools need \
                               `hick agent --doc <file.hick>`; use a script instead"
                            .to_string(),
                    },
                };

                session.record(SessionEvent::ToolResult {
                    name: &outcome.name,
                    ok: outcome.ok,
                    text: &outcome.text,
                });
                on_event(AgentEvent::ToolFinished {
                    name: outcome.name.clone(),
                    ok: outcome.ok,
                    text: outcome.text.clone(),
                });

                history.push(Message::new(Role::Assistant, response));
                history.push(Message::new(
                    Role::User,
                    format!(
                        "<hick:tool-result name=\"{}\" ok=\"{}\">\n{}\n</hick:tool-result>",
                        outcome.name, outcome.ok, outcome.text
                    ),
                ));
            }
            Turn::Done { summary } => {
                session.record(SessionEvent::Assistant {
                    prose: &summary,
                    action: None,
                });
                let total_cost_usd = cost_usd(llm.model_name(), &total_usage);
                session.record(SessionEvent::Usage {
                    turn: None,
                    usage: total_usage,
                    cost_usd: total_cost_usd,
                });
                session.record(SessionEvent::End);
                on_event(AgentEvent::Done {
                    summary: summary.clone(),
                });
                return Ok(AgentOutcome {
                    summary,
                    session_path,
                    turns: turn + 1,
                    total_usage,
                    total_cost_usd,
                });
            }
        }
    }

    // The budget is spent. Do NOT throw the session away: the work up to
    // here is real — files were written, edits landed — and failing the run
    // discards the only account of it. Spend one more call asking for a
    // handoff instead, so an unfinished run ends with a summary a person (or
    // a follow-up turn, replayed through `prior_turns`) can continue from.
    on_event(AgentEvent::Thinking);
    history.push(Message::new(
        Role::User,
        format!(
            "You have used the whole turn budget ({} turns) and must stop now. \
             Do not start any new work and do not emit a code block or a tool \
             call. Reply with <hick:next>done</hick:next> followed by a handoff: \
             what you changed, what you verified, what is left, and the exact \
             next step you would have taken.",
            config.max_turns
        ),
    ));
    let (response, wrap_usage) = stream_completion(llm, history.clone(), on_event).await?;
    total_usage.add(&wrap_usage);
    let summary = match parse_response(&response) {
        // A handoff is what was asked for; anything else still carries the
        // model's own words, which beat a generic failure string.
        Turn::Done { summary } => summary,
        _ => response.clone(),
    };
    let summary = format!(
        "[unfinished — stopped after {} turns]\n\n{summary}",
        config.max_turns
    );
    session.record(SessionEvent::Assistant {
        prose: &summary,
        action: None,
    });
    session.record(SessionEvent::Usage {
        turn: None,
        usage: total_usage,
        cost_usd: cost_usd(llm.model_name(), &total_usage),
    });
    session.record(SessionEvent::End);
    on_event(AgentEvent::Done {
        summary: summary.clone(),
    });
    Ok(AgentOutcome {
        summary,
        session_path,
        turns: config.max_turns,
        total_usage,
        total_cost_usd: cost_usd(llm.model_name(), &total_usage),
    })
}

/// Stream a completion, emitting [`AgentEvent::Token`] per chunk, and return
/// the accumulated response text plus the call's summed [`Usage`].
async fn stream_completion(
    llm: &dyn LlmClient,
    messages: Vec<Message>,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<(String, Usage)> {
    let mut stream = llm.complete_stream(messages).await?;
    let mut response = String::new();
    let mut usage = Usage::default();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(u) = &chunk.usage {
            usage.add(u);
        }
        if !chunk.delta.is_empty() {
            on_event(AgentEvent::Token {
                data: chunk.delta.clone(),
            });
            response.push_str(&chunk.delta);
        }
    }
    Ok((response, usage))
}
