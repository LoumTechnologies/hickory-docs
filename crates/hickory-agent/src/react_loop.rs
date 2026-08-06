//! The script-first ReAct loop.
//!
//! LLM emits `<hick:next>code</hick:next>` + a fenced code block → the script
//! runs through the [`Executor`] → stdout/stderr comes back as the next
//! observation. `<hick:next>done</hick:next>` (no code block) ends the loop.
//! Every step is written incrementally to a `hick:session` document and
//! emitted as an [`AgentEvent`] for the WS run channel.

use std::path::PathBuf;

use anyhow::{Context as _, Result};
use futures::StreamExt as _;
use hickory_executor::Executor;

use crate::events::AgentEvent;
use crate::llm::{LlmClient, Message, Role};
use crate::protocol::{SYSTEM_PROMPT, Turn, correction_message, parse_response};
use crate::script::{AGENT_CONTAINER, run_script};
use crate::session::{HickSessionLog, SessionEvent, SessionLog, session_file_path};

/// Configuration for one agent run.
pub struct AgentConfig {
    /// The user's task prompt.
    pub prompt: String,
    /// Optional document source given as context (`--doc`).
    pub doc_context: Option<String>,
    /// Project directory: session files land in `<project_dir>/sessions/`.
    pub project_dir: PathBuf,
    /// Maximum LLM turns before giving up (default 20).
    pub max_turns: usize,
    /// Container image recorded for the agent workspace (LocalExecutor
    /// ignores it).
    pub image: String,
}

impl AgentConfig {
    /// Config with defaults for `prompt` in `project_dir`.
    pub fn new(prompt: impl Into<String>, project_dir: impl Into<PathBuf>) -> Self {
        Self {
            prompt: prompt.into(),
            doc_context: None,
            project_dir: project_dir.into(),
            max_turns: 20,
            image: "host".into(),
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
    executor: &dyn Executor,
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
        .ensure_started(AGENT_CONTAINER, &config.image)
        .await
        .context("failed to start agent container")?;

    let mut system = SYSTEM_PROMPT.to_string();
    if let Some(doc) = &config.doc_context {
        system.push_str("\n\n## Document under discussion\n\n");
        system.push_str(doc);
    }

    let mut history = vec![
        Message::new(Role::System, system),
        Message::new(Role::User, config.prompt.clone()),
    ];
    session.record(SessionEvent::User {
        text: &config.prompt,
    });
    on_event(AgentEvent::UserMessage {
        text: config.prompt.clone(),
    });

    let mut action_index = 0usize;
    let mut consecutive_invalid = 0usize;

    for turn in 0..config.max_turns {
        on_event(AgentEvent::Thinking);
        let response = stream_completion(llm, history.clone(), on_event).await?;
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
                history.push(Message::new(Role::Assistant, response));
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

                let result = run_script(executor, &block, action_index).await?;

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
            Turn::Done { summary } => {
                session.record(SessionEvent::Assistant {
                    prose: &summary,
                    action: None,
                });
                session.record(SessionEvent::End);
                on_event(AgentEvent::Done {
                    summary: summary.clone(),
                });
                return Ok(AgentOutcome {
                    summary,
                    session_path,
                    turns: turn + 1,
                });
            }
        }
    }

    session.record(SessionEvent::End);
    let message = format!("agent did not finish within {} turns", config.max_turns);
    on_event(AgentEvent::Error {
        message: message.clone(),
    });
    anyhow::bail!(message)
}

/// Stream a completion, emitting [`AgentEvent::Token`] per chunk, and return
/// the accumulated response text.
async fn stream_completion(
    llm: &dyn LlmClient,
    messages: Vec<Message>,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<String> {
    let mut stream = llm.complete_stream(messages).await?;
    let mut response = String::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if !chunk.delta.is_empty() {
            on_event(AgentEvent::Token {
                data: chunk.delta.clone(),
            });
            response.push_str(&chunk.delta);
        }
    }
    Ok(response)
}
