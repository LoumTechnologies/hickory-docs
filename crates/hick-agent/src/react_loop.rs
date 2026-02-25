//! Script-first ReAct loop: LLM writes Python scripts, we execute them.
//!
//! The LLM produces Python code in fenced code blocks. The react loop:
//! 1. Extracts ```python ... ``` blocks from the LLM response
//! 2. Writes each script to `.hick/tmp/script_N.py`
//! 3. Executes via `python3 .hick/tmp/script_N.py`
//! 4. Captures stdout/stderr as observation
//! 5. Returns observation to LLM for next turn
//!
//! If no code block in the LLM response → treat as final answer.
//!
//! The loop communicates with the UI exclusively through [`AgentEvent`]s,
//! never touching stdout/stderr directly.

use std::path::PathBuf;

use futures::StreamExt;
use tokio::sync::{Notify, mpsc};

use crate::agent::Agent;
use crate::context;
use crate::events::AgentEvent;
use crate::llm::{Message, Role};
use crate::state::AgentState;
use crate::tools::{ScriptResult, extract_python_blocks};

const SYSTEM_PROMPT: &str = r#"You are an AI coding agent for projects using Hick (hickory docs) — an XML format for defining containers, file outputs, and code generation.

## How you work

You accomplish tasks by writing Python scripts. Write ONE ```python code block per response, then STOP and wait for the real execution output. Do NOT predict or guess what the output will be — the system runs your script and shows you the actual stdout/stderr.

When you're done (no more scripts to run), write your final answer as plain text without any ```python blocks.

IMPORTANT: Only write ONE code block per response. Never write multiple code blocks. Never write "Observation:" or fake output — you will receive the real output automatically.

## Available tools

### Native Python
- Read/write files with `open()`, `os`, `pathlib`
- Run shell commands with `subprocess.run()`
- Install packages with `subprocess.run(['pip', 'install', ...])`

### Hickory SDK (structural code operations)
The hickory SDK is pre-installed. Usage:
  from hickory import hick
  hick.query(selector, file=path)      — find elements (fn, struct, class, #id, .class)
  hick.structure(path)                  — structural summary of a generated file
  hick.provenance(path, start, end)    — which .hick source generated these lines
  hick.edit(selector, new_code, file=path) — compute source edits
  hick.apply(edits)                     — apply edits to .hick source files

## Hick XML format

.hick files use this format:
  <hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">
    <hick:file path="output/path.py">
  file content here
    </hick:file>
  </hick:doc>

Key elements:
- Root element: `<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0">`
- File output: `<hick:file path="relative/path">content</hick:file>`
- Variables: `<hick:var name="x">value</hick:var>` and `<hick:paste select="$x"/>`
- Shell exec: `<hick:exec container="default">command</hick:exec>`

## Key concepts

1. **Output files are generated** from `.hick` XML source files. The pipeline runs transparently when `.hick` files change — you never need to run it manually.

2. **Edit .hick source, not output files.** Use the hickory SDK to map output locations back to .hick source, or edit .hick files directly.

3. **`<hick:exec>` vs `subprocess.run()`:**
   - Use `subprocess.run()` for one-time tasks: running tests, installing deps, checking status
   - Use `<hick:exec container="...">` in .hick files for recurring code generation
   - Rule: if the command's output becomes part of generated code, put it in a `<hick:exec>` block

4. **Print your results.** Your script's stdout is what you see as observation. Use `print()` to report findings.
"#;

/// ReAct loop runner — script-first model.
///
/// Communicates with the UI through event channels, never printing directly.
pub struct ReactLoop {
    pub agent: Agent,
    pub project_dir: PathBuf,
    max_turns: usize,
    /// Whether to use streaming LLM output (sends many small Token events)
    /// vs. a single complete call (sends one large Token event).
    pub stream_output: bool,
}

impl ReactLoop {
    pub fn new(agent: Agent, project_dir: PathBuf) -> Self {
        Self {
            agent,
            project_dir,
            max_turns: 50,
            stream_output: true,
        }
    }

    /// Execute a Python script and capture results.
    async fn execute_script(&self, script: &str, index: usize) -> ScriptResult {
        let tmp_dir = self.project_dir.join(".hick").join("tmp");
        if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to create tmp dir: {e}"),
                exit_code: None,
            };
        }

        let script_path = tmp_dir.join(format!("script_{index}.py"));
        if let Err(e) = std::fs::write(&script_path, script) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to write script: {e}"),
                exit_code: None,
            };
        }

        // Set PYTHONPATH so `from hickory import hick` works
        let sdk_path = self.project_dir.join(".hick").join("sdk");

        match tokio::process::Command::new("python3")
            .arg(&script_path)
            .current_dir(&self.project_dir)
            .env("PYTHONPATH", &sdk_path)
            .env("WORKSPACE", &self.project_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(child) => match child.wait_with_output().await {
                Ok(output) => ScriptResult {
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    exit_code: output.status.code(),
                },
                Err(e) => ScriptResult {
                    stdout: String::new(),
                    stderr: format!("Script execution failed: {e}"),
                    exit_code: None,
                },
            },
            Err(e) => ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to spawn python3: {e}"),
                exit_code: None,
            },
        }
    }

    /// Run one full ReAct turn: get LLM response, execute scripts, return new state.
    ///
    /// All output goes through the `events` channel; nothing is printed.
    async fn step(
        &self,
        state: AgentState,
        messages: &mut Vec<Message>,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
    ) -> anyhow::Result<AgentState> {
        match state {
            AgentState::Thinking { .. } => {
                events.send(AgentEvent::Thinking).ok();

                let response = if self.stream_output {
                    let mut stream = self.agent.llm_stream(messages.clone()).await?;
                    let mut full = String::new();

                    // Pin the interrupt future so it survives across loop iterations.
                    let interrupt_fut = interrupt.notified();
                    tokio::pin!(interrupt_fut);

                    loop {
                        tokio::select! {
                            biased;
                            _ = &mut interrupt_fut => {
                                events.send(AgentEvent::Interrupted).ok();
                                return Ok(AgentState::to_done("(interrupted)".into()));
                            }
                            chunk = stream.next() => {
                                match chunk {
                                    Some(Ok(chunk)) => {
                                        if !chunk.delta.is_empty() {
                                            events.send(AgentEvent::Token(chunk.delta.clone())).ok();
                                            full.push_str(&chunk.delta);
                                        }
                                    }
                                    Some(Err(e)) => return Err(e),
                                    None => break,
                                }
                            }
                        }
                    }
                    full
                } else {
                    // Non-streaming: race the complete call against interrupt.
                    let interrupt_fut = interrupt.notified();
                    tokio::pin!(interrupt_fut);

                    tokio::select! {
                        biased;
                        _ = &mut interrupt_fut => {
                            events.send(AgentEvent::Interrupted).ok();
                            return Ok(AgentState::to_done("(interrupted)".into()));
                        }
                        result = self.agent.llm_complete(messages.clone()) => {
                            let response = result?;
                            events.send(AgentEvent::Token(response.clone())).ok();
                            response
                        }
                    }
                };

                events
                    .send(AgentEvent::ResponseComplete(response.clone()))
                    .ok();

                // Add assistant message to history
                messages.push(Message {
                    role: Role::Assistant,
                    content: response.clone(),
                });

                let all_scripts = extract_python_blocks(&response);

                if all_scripts.is_empty() {
                    // No code blocks → final answer
                    return Ok(AgentState::to_done(response.trim().to_string()));
                }

                // Only execute the first code block per turn to prevent
                // the LLM from hallucinating observations for later blocks.
                let scripts = vec![all_scripts.into_iter().next().unwrap()];

                // Extract the non-code-block text as "thought"
                let thought = extract_thought(&response);

                Ok(AgentState::to_executing(scripts, thought))
            }
            AgentState::Executing { scripts, .. } => {
                let mut results = Vec::with_capacity(scripts.len());

                for (i, script) in scripts.iter().enumerate() {
                    events
                        .send(AgentEvent::ScriptStarted {
                            code: script.clone(),
                        })
                        .ok();
                    let result = self.execute_script(script, i).await;
                    events
                        .send(AgentEvent::ScriptFinished(result.clone()))
                        .ok();
                    results.push(result);
                }

                // Format observation for the LLM
                let observation = results
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        if results.len() == 1 {
                            format!("Script output:\n{}", r.as_observation())
                        } else {
                            format!("Script {} output:\n{}", i + 1, r.as_observation())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");

                messages.push(Message {
                    role: Role::User,
                    content: observation,
                });

                Ok(AgentState::to_observing(results))
            }
            _ => Ok(state),
        }
    }

    /// Run the full ReAct loop until done or max turns.
    ///
    /// Emits [`AgentEvent`]s on the channel and checks `interrupt` for
    /// user-requested cancellation.
    pub async fn run(
        &self,
        user_message: &str,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
    ) -> anyhow::Result<()> {
        // Load standards-based agent context (AGENTS.md, .agents/, CLAUDE.md)
        let agent_context = context::load_agent_context(&self.project_dir);
        let system = if agent_context.is_empty() {
            SYSTEM_PROMPT.to_string()
        } else {
            format!("{SYSTEM_PROMPT}\n\n---\n\n{agent_context}")
        };

        let mut messages = vec![
            Message {
                role: Role::System,
                content: system,
            },
            Message {
                role: Role::User,
                content: format!(
                    "Project directory: {}\n\nRequest: {}",
                    self.project_dir.display(),
                    user_message
                ),
            },
        ];

        let mut state = AgentState::Thinking { messages_sent: 2 };
        let mut turns = 0;

        while !state.is_terminal() && turns < self.max_turns {
            state = self.step(state, &mut messages, events, interrupt).await?;
            turns += 1;

            if let AgentState::Observing { .. } = state {
                state = AgentState::Thinking {
                    messages_sent: messages.len(),
                };
            }
        }

        match state {
            AgentState::Done { message } => {
                events.send(AgentEvent::Done(message)).ok();
                Ok(())
            }
            AgentState::Error { message } => {
                events.send(AgentEvent::Error(message.clone())).ok();
                Err(anyhow::anyhow!("Agent error: {message}"))
            }
            _ => {
                let msg = format!("Max turns ({}) exceeded", self.max_turns);
                events.send(AgentEvent::Error(msg.clone())).ok();
                Err(anyhow::anyhow!("{msg}"))
            }
        }
    }
}

/// Extract the "thought" text from a response — everything before/between code blocks.
fn extract_thought(response: &str) -> Option<String> {
    let text: String = response
        .split("```python")
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if text.is_empty() { None } else { Some(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_thought_from_response() {
        let response = "Let me check the file structure first.\n\n```python\nimport os\nprint(os.listdir('.'))\n```";
        let thought = extract_thought(response);
        assert_eq!(thought.unwrap(), "Let me check the file structure first.");
    }

    #[test]
    fn no_thought_when_starts_with_code() {
        let response = "```python\nprint('hello')\n```";
        let thought = extract_thought(response);
        assert!(thought.is_none());
    }

    #[test]
    fn extract_thought_no_code_blocks() {
        let response = "This is the final answer with no code.";
        let thought = extract_thought(response);
        assert_eq!(thought.unwrap(), "This is the final answer with no code.");
    }
}
