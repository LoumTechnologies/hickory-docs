//! Script-first ReAct loop: LLM writes code, we execute it in a sandbox.
//!
//! The LLM produces code in fenced markdown blocks (shell, Python, JS, etc.).
//! The react loop:
//! 1. Extracts code blocks from the LLM response
//! 2. Writes each script to `.hick/tmp/script_N.{py,csx,ts}`
//! 3. Executes inside a WASM container via [`CommandExecutor`]
//! 4. Captures stdout/stderr as observation
//! 5. Returns observation to LLM for next turn
//!
//! If no code block in the LLM response → treat as final answer.
//!
//! The loop communicates with the UI exclusively through [`AgentEvent`]s,
//! never touching stdout/stderr directly.

use std::path::PathBuf;
use std::sync::Arc;

use futures::StreamExt;
use hick_agent_core::CommandExecutor;
use hick_policy::PolicyEvaluator;
use tokio::sync::{Mutex, Notify, mpsc};

use crate::agent::Agent;
use crate::context;
use crate::events::AgentEvent;
use crate::llm::{Message, Role};
use crate::sink_guard::SinkGuard;
use crate::state::AgentState;
use crate::tools::{CodeBlock, Language, ScriptResult, extract_code_blocks};

const SYSTEM_PROMPT: &str = r#"You are an AI coding agent for projects using Hick (hickory docs) — an XML format for defining containers, file outputs, and code generation.

## Response format (REQUIRED)

Every response MUST begin with one of these two XML tags on the very first line:

  <hick:next>code</hick:next>
  <hick:next>done</hick:next>

Rules:
- `<hick:next>code</hick:next>` — you are going to run code. You MUST include exactly one fenced code block in this response.
- `<hick:next>done</hick:next>` — you are finished. Write your final answer as plain text; do NOT include any code blocks.

The system validates these tags. If you write `<hick:next>code</hick:next>` without a code block, or `<hick:next>done</hick:next>` with a code block, you will be asked to correct your response.

Write ONE fenced code block per response, then STOP and wait for the real execution output. Do NOT predict or guess what the output will be — the system runs your code and shows you the actual stdout/stderr.

## Executable fenced blocks (runner map)

Pick a fence whose runner exists in your environment (see the runtime probe in the system prompt when using the sandbox).

- Shell: ```bash, ```sh, ```shell, ```posix-sh — saved as POSIX `sh`; use for `apk add`, `ls`, pipelines, POSIX scripts, installers, compilers, etc.
- Python: ```python — `python3` + Hickory SDK
- JavaScript (Node): ```javascript / ```js — `node`
- C#: ```csharp / ```cs — `dotnet-script`
- TypeScript: ```typescript / ```ts — `tsx`

You may install language runtimes or packages yourself inside the sandbox (e.g. `apk add …`, `npm install -g …`, pip, curling binaries) whenever you need them. No separate permission step—go ahead and fix the environment if something is missing (subject to Hick resource limits you may hit as normal errors).

## Available languages / patterns

### Shell (recommended for discovery and installs)
- Prefer ```bash when you want simple commands: `find`, `ls`, `apk add python3 py3-pip`, `wget`, etc.
- Scripts run as `sh /workspace/.hick/tmp/…` — the interpreter is POSIX `sh` for these fences (shebang lines are ignored). Call another interpreter explicitly (`python3 …`, `bash …`) once installed.

### Python
- Read/write files with `open()`, `os`, `pathlib`
- Run shell commands with `subprocess.run()`
- Install packages: `subprocess.run(['pip', 'install', ...])` (after installing pip if needed)
- Hickory SDK: `from hickory import hick` (Python only — SDK under `/workspace/.hick/sdk`)

### JavaScript (Node)
- Runs with `node`. Use `.mjs` style or plain CommonJS depending on what's available after install.

### C# (dotnet-script)
- Write C# script files (.csx) — no Main() or class wrapper needed
- Install NuGet packages: `#r "nuget: Newtonsoft.Json, 13.0.3"`
- Requires `dotnet` / `dotnet-script` when you choose this fence.

### TypeScript (tsx)
- Write TypeScript; requires `tsx` (install globally or via npm/npx depending on probe output).

## Hickory SDK (Python only)
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

3. **`<hick:exec>` vs one-off commands:**
   - Use one-off scripts for: running tests, installing deps, checking status
   - Use `<hick:exec container="...">` in .hick files for recurring code generation
   - Rule: if the command's output becomes part of generated code, put it in a `<hick:exec>` block

4. **Print your results.** Your script's stdout is what you see as observation. Use print/Console.WriteLine/console.log in typed languages, or echo/printf for shell fences, to report findings.
"#;

/// Appended to the system prompt when code runs in the WASM sandbox (normal mode).
///
/// The model must not use the host project path here — scripts only see `/workspace`.
const SANDBOX_EXECUTION_CONTEXT: &str = r#"

## Execution environment (sandbox)

Your fenced code blocks run in an isolated Linux/WASM container. The project is mounted at **`/workspace`** and shell commands start from there (`cd /workspace`, etc.).

In every script, use **`/workspace`**, **`WORKSPACE`** (set for Python), or paths **relative to `/workspace`**. **Do not** use host absolute paths such as `/home/...`, `/Users/...`, or `~/...` — those paths do not exist inside the container.

The prompt includes a short **automatic runtime probe** from this container listing which binaries are installed; trust that output over assumptions from training data. If something is missing, install it yourself in-code (apk, npm, etc.) unless the error clearly blocks that.
"#;

/// Probe command run once per session inside the sandbox to guide the LLM.
const SANDBOX_RUNTIME_PROBE: &str = r#"echo '### Sandbox runtime probe'; echo '---'; uname -a 2>&1 || true; echo '--- interpreters / tools ---'; for c in sh ash bash python3 python dotnet dotnet-script node npm npx tsx bun deno rustc cargo ruby lua apk apt-get wget curl git; do printf '%s: ' "$c"; command -v "$c" >/dev/null 2>&1 && command -v "$c" || echo '(not found)'; done"#;

// ── Response protocol ────────────────────────────────────────────────────────

/// Intent declared by the LLM in its `<hick:next>` tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseIntent {
    /// LLM declared it will run a code block this turn.
    Code,
    /// LLM declared it is done; no code block should follow.
    Done,
    /// No `<hick:next>` tag found — protocol not followed.
    Unknown,
}

fn parse_intent(response: &str) -> ResponseIntent {
    let t = response.trim_start();
    if t.starts_with("<hick:next>code</hick:next>") {
        ResponseIntent::Code
    } else if t.starts_with("<hick:next>done</hick:next>") {
        ResponseIntent::Done
    } else {
        ResponseIntent::Unknown
    }
}

/// Validate a response against the `<hick:next>` protocol.
///
/// Returns `None` when the response is valid, or `Some(reason)` describing
/// the first violation found. Callers use this to decide whether to re-prompt.
fn validate_response(response: &str) -> Option<String> {
    let intent = parse_intent(response);
    let has_code = !extract_code_blocks(response).is_empty();
    match (intent, has_code) {
        (ResponseIntent::Code, true) => None,
        (ResponseIntent::Done, false) => None,
        (ResponseIntent::Code, false) => Some(
            "declared <hick:next>code</hick:next> but did not include a code block".into(),
        ),
        (ResponseIntent::Done, true) => Some(
            "declared <hick:next>done</hick:next> but included a code block".into(),
        ),
        (ResponseIntent::Unknown, _) => Some(
            "response did not start with <hick:next>code</hick:next> or <hick:next>done</hick:next>".into(),
        ),
    }
}

/// Correction message injected as a user turn after a protocol violation.
///
/// Kept as a standalone function so it's easy to tune independently of the
/// validation logic.
fn correction_message(reason: &str) -> String {
    format!(
        "Your last response was invalid: {reason}.\n\n\
         Please try again. Start your response with exactly one of:\n\
         - `<hick:next>code</hick:next>` followed immediately by a fenced code block \
           (shell: bash/sh/shell — or python, javascript/js, csharp/cs, typescript/ts)\n\
         - `<hick:next>done</hick:next>` followed by your final plain-text answer with no code blocks"
    )
}

// ── ReAct loop ───────────────────────────────────────────────────────────────

/// ReAct loop runner — script-first model.
///
/// Communicates with the UI through event channels, never printing directly.
pub struct ReactLoop {
    pub agent: Agent,
    pub project_dir: PathBuf,
    max_turns: usize,
    /// Maximum re-prompt attempts when the LLM violates the response protocol.
    /// After this many retries the last response is processed as-is.
    max_reprompts: usize,
    /// Whether to use streaming LLM output (sends many small Token events)
    /// vs. a single complete call (sends one large Token event).
    pub stream_output: bool,
    /// Sandboxed command executor for running scripts in WASM containers.
    /// `None` falls back to direct host execution (legacy/unsandboxed mode).
    executor: Option<Arc<Mutex<dyn CommandExecutor>>>,
    /// Container image to use for sandbox execution.
    sandbox_image: String,
    /// Optional policy evaluator for pre-execution checks.
    policy_evaluator: Option<PolicyEvaluator>,
    /// Optional sink guard for classification-based API call gating.
    sink_guard: Option<Arc<SinkGuard>>,
}

impl ReactLoop {
    pub fn new(agent: Agent, project_dir: PathBuf) -> Self {
        Self {
            agent,
            project_dir,
            max_turns: 50,
            max_reprompts: 2,
            stream_output: true,
            executor: None,
            sandbox_image: "alpine".to_string(),
            policy_evaluator: None,
            sink_guard: None,
        }
    }

    /// Override the maximum number of re-prompt retries on protocol violations.
    pub fn with_max_reprompts(mut self, n: usize) -> Self {
        self.max_reprompts = n;
        self
    }

    /// Set the sandboxed command executor for running scripts.
    pub fn with_executor(
        mut self,
        executor: Arc<Mutex<dyn CommandExecutor>>,
        image: String,
    ) -> Self {
        self.executor = Some(executor);
        self.sandbox_image = image;
        self
    }

    /// Set the policy evaluator for pre-execution script checks.
    pub fn with_policy(mut self, evaluator: PolicyEvaluator) -> Self {
        self.policy_evaluator = Some(evaluator);
        self
    }

    /// Set the sink guard for classification-based API call gating.
    pub fn with_sink_guard(mut self, guard: Arc<SinkGuard>) -> Self {
        self.sink_guard = Some(guard);
        self
    }

    /// Get the sink guard (if configured), for SDK API call validation.
    pub fn sink_guard(&self) -> Option<&Arc<SinkGuard>> {
        self.sink_guard.as_ref()
    }

    /// Call the LLM once and return the full response text.
    ///
    /// Returns `None` when the user interrupted. When `emit_tokens` is true,
    /// [`AgentEvent::Token`] events are fired for each streamed chunk (used for
    /// the first call in a turn). Re-prompt retries pass `emit_tokens = false`
    /// so only the final accepted response appears in the UI.
    async fn call_llm(
        &self,
        messages: &[Message],
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
        emit_tokens: bool,
    ) -> anyhow::Result<Option<String>> {
        if emit_tokens && self.stream_output {
            let mut stream = self.agent.llm_stream(messages.to_vec()).await?;
            let mut full = String::new();
            let interrupt_fut = interrupt.notified();
            tokio::pin!(interrupt_fut);
            loop {
                tokio::select! {
                    biased;
                    _ = &mut interrupt_fut => return Ok(None),
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
            Ok(Some(full))
        } else {
            let interrupt_fut = interrupt.notified();
            tokio::pin!(interrupt_fut);
            tokio::select! {
                biased;
                _ = &mut interrupt_fut => Ok(None),
                result = self.agent.llm_complete(messages.to_vec()) => {
                    let response = result?;
                    if emit_tokens {
                        events.send(AgentEvent::Token(response.clone())).ok();
                    }
                    Ok(Some(response))
                }
            }
        }
    }

    /// Probe the WASM sandbox once and return human-readable text for the LLM.
    ///
    /// Best-effort: failures are inlined so models do not invent capabilities.
    async fn sandbox_runtime_probe_text(&self) -> Option<String> {
        let executor = self.executor.as_ref()?;
        let mut exec = executor.lock().await;
        match exec
            .execute(
                "agent-sandbox",
                SANDBOX_RUNTIME_PROBE,
                &self.sandbox_image,
                Some(120),
            )
            .await
        {
            Ok(cmd_out) => {
                let mut out = cmd_out.output;
                match cmd_out.exit_code {
                    Some(0) => {}
                    Some(code) => {
                        use std::fmt::Write as _;
                        let _ = writeln!(
                            out,
                            "---\nSandbox runtime probe reported exit_code: {}",
                            code
                        );
                    }
                    None => {
                        use std::fmt::Write as _;
                        let _ = writeln!(
                            out,
                            "---\nSandbox runtime probe exited (code unknown)"
                        );
                    }
                }
                Some(out)
            }
            Err(e) => Some(format!("Sandbox runtime probe failed: {e}")),
        }
    }

    /// Execute a code block inside the sandbox container.
    ///
    /// Writes the script to `.hick/tmp/script_N.{ext}` on the host for
    /// debugging, then injects it directly into the container filesystem and
    /// runs it via [`CommandExecutor`].
    async fn execute_code_block(&self, block: &CodeBlock, index: usize) -> ScriptResult {
        let tmp_dir = self.project_dir.join(".hick").join("tmp");
        if let Err(e) = std::fs::create_dir_all(&tmp_dir) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to create tmp dir: {e}"),
                exit_code: None,
            };
        }

        let filename = format!("script_{index}.{}", block.language.extension());
        let script_path = tmp_dir.join(&filename);
        if let Err(e) = std::fs::write(&script_path, &block.code) {
            return ScriptResult {
                stdout: String::new(),
                stderr: format!("Failed to write script: {e}"),
                exit_code: None,
            };
        }

        let container_script_path = format!("/workspace/.hick/tmp/{filename}");

        if let Some(ref executor) = self.executor {
            // Build the run command with any language-specific env vars.
            let run_cmd = block.language.run_command(&container_script_path);
            let cmd = match block.language {
                Language::Python => {
                    format!("PYTHONPATH=/workspace/.hick/sdk WORKSPACE=/workspace {run_cmd}")
                }
                Language::Shell
                | Language::JavaScript
                | Language::TypeScript
                | Language::CSharp => format!("cd /workspace && {run_cmd}"),
            };

            let mut exec = executor.lock().await;

            // Inject the script into the container filesystem. The sandbox
            // creates /workspace as a plain directory (WASI preopened dirs are
            // not visible as real VFS paths to shell commands), so we must push
            // the file in explicitly before running it.
            if let Err(e) = exec
                .inject_file(
                    "agent-sandbox",
                    &self.sandbox_image,
                    block.code.as_bytes(),
                    &container_script_path,
                )
                .await
            {
                return ScriptResult {
                    stdout: String::new(),
                    stderr: format!("Failed to inject script into sandbox: {e}"),
                    exit_code: None,
                };
            }

            match exec
                .execute("agent-sandbox", &cmd, &self.sandbox_image, Some(300))
                .await
            {
                Ok(result) => ScriptResult {
                    stdout: result.output.clone(),
                    stderr: String::new(),
                    exit_code: result.exit_code,
                },
                Err(e) => ScriptResult {
                    stdout: String::new(),
                    stderr: format!("Sandbox execution failed: {e}"),
                    exit_code: None,
                },
            }
        } else {
            // Unsandboxed fallback: direct host execution.
            async fn spawn_and_collect<'a>(
                program: &'a str,
                args: &[std::ffi::OsString],
                cwd: &'a std::path::Path,
                env_pairs: &[(&'a std::ffi::OsStr, &'a std::ffi::OsStr)],
            ) -> ScriptResult {
                let mut cmd = tokio::process::Command::new(program);
                cmd.args(args).current_dir(cwd);
                cmd.stdout(std::process::Stdio::piped());
                cmd.stderr(std::process::Stdio::piped());
                for (k, v) in env_pairs {
                    cmd.env(*k, *v);
                }
                match cmd.spawn() {
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
                        stderr: format!("Failed to spawn {program}: {e}"),
                        exit_code: None,
                    },
                }
            }

            let cwd = self.project_dir.as_path();
            let path_os = script_path.into_os_string();
            let sdk_path = self.project_dir.join(".hick").join("sdk");
            match block.language {
                Language::Python => {
                    let py_env = [
                        (
                            std::ffi::OsStr::new("PYTHONPATH"),
                            sdk_path.as_os_str(),
                        ),
                        (
                            std::ffi::OsStr::new("WORKSPACE"),
                            self.project_dir.as_os_str(),
                        ),
                    ];
                    spawn_and_collect("python3", &[path_os], cwd, &py_env).await
                }
                Language::Shell => spawn_and_collect("sh", &[path_os], cwd, &[]).await,
                Language::JavaScript => {
                    spawn_and_collect("node", &[path_os], cwd, &[]).await
                }
                Language::TypeScript => {
                    spawn_and_collect("tsx", &[path_os], cwd, &[]).await
                }
                Language::CSharp => {
                    spawn_and_collect("dotnet-script", &[path_os], cwd, &[]).await
                }
            }
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

                // Initial LLM call — stream tokens to the UI.
                let Some(mut response) =
                    self.call_llm(messages, events, interrupt, true).await?
                else {
                    events.send(AgentEvent::Interrupted).ok();
                    return Ok(AgentState::to_done("(interrupted)".into()));
                };

                // Re-prompt loop: keep retrying until the response follows the
                // <hick:next> protocol or we exhaust max_reprompts.
                for attempt in 1..=self.max_reprompts {
                    let Some(reason) = validate_response(&response) else {
                        break; // valid
                    };
                    events
                        .send(AgentEvent::Reprompt {
                            reason: reason.clone(),
                            attempt,
                        })
                        .ok();
                    // Fold the invalid response + correction into history.
                    messages.push(Message {
                        role: Role::Assistant,
                        content: response.clone(),
                    });
                    messages.push(Message {
                        role: Role::User,
                        content: correction_message(&reason),
                    });
                    // Retry without streaming so only the final answer appears.
                    events.send(AgentEvent::Thinking).ok();
                    let Some(r) =
                        self.call_llm(messages, events, interrupt, false).await?
                    else {
                        events.send(AgentEvent::Interrupted).ok();
                        return Ok(AgentState::to_done("(interrupted)".into()));
                    };
                    response = r;
                }

                events
                    .send(AgentEvent::ResponseComplete(response.clone()))
                    .ok();

                // Add assistant message to history.
                messages.push(Message {
                    role: Role::Assistant,
                    content: response.clone(),
                });

                let all_blocks = extract_code_blocks(&response);

                if all_blocks.is_empty() {
                    // No code blocks → final answer
                    return Ok(AgentState::to_done(response.trim().to_string()));
                }

                // Only execute the first code block per turn to prevent
                // the LLM from hallucinating observations for later blocks.
                let blocks = vec![all_blocks.into_iter().next().unwrap()];

                let thought = extract_thought(&response);

                Ok(AgentState::to_executing(blocks, thought))
            }
            AgentState::Executing { blocks, .. } => {
                let mut results = Vec::with_capacity(blocks.len());

                for (i, block) in blocks.iter().enumerate() {
                    // Policy check: evaluate script before execution.
                    if let Some(ref evaluator) = self.policy_evaluator {
                        let input = hick_policy::FlowPolicyInput {
                            column_classifications: serde_json::json!({}),
                            sink_type: "script-execution".to_string(),
                            slot_name: format!("script_{i}"),
                            data_classification: serde_json::json!(["agent-generated"]),
                            token_caveats: vec![],
                        };
                        match evaluator.evaluate_flow(&input).await {
                            Ok(hick_policy::PolicyDecision::Allow) => {}
                            Ok(hick_policy::PolicyDecision::Deny { reason }) => {
                                events
                                    .send(AgentEvent::SecurityViolation {
                                        description: format!(
                                            "Policy denied script execution: {reason}"
                                        ),
                                    })
                                    .ok();
                                results.push(ScriptResult {
                                    stdout: String::new(),
                                    stderr: format!("Policy denied execution: {reason}"),
                                    exit_code: Some(1),
                                });
                                continue;
                            }
                            Err(e) => {
                                events
                                    .send(AgentEvent::SecurityViolation {
                                        description: format!("Policy evaluation error: {e}"),
                                    })
                                    .ok();
                                results.push(ScriptResult {
                                    stdout: String::new(),
                                    stderr: format!("Policy evaluation failed: {e}"),
                                    exit_code: Some(1),
                                });
                                continue;
                            }
                        }
                    }

                    events
                        .send(AgentEvent::ScriptStarted {
                            code: block.code.clone(),
                        })
                        .ok();
                    let result = self.execute_code_block(block, i).await;
                    events.send(AgentEvent::ScriptFinished(result.clone())).ok();
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
    /// `history` is the persistent conversation across multiple user turns. On
    /// the first call it must be empty; the system prompt and initial user
    /// message are prepended automatically. On subsequent calls only the new
    /// user message is appended, preserving the full conversation context.
    ///
    /// Emits [`AgentEvent`]s on the channel and checks `interrupt` for
    /// user-requested cancellation.
    pub async fn run(
        &self,
        user_message: &str,
        history: &mut Vec<Message>,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
    ) -> anyhow::Result<()> {
        if history.is_empty() {
            // First turn: prepend system prompt and wrap user message with context.
            let agent_context = context::load_agent_context(&self.project_dir);
            let mut system = SYSTEM_PROMPT.to_string();
            if self.executor.is_some() {
                system.push_str(SANDBOX_EXECUTION_CONTEXT);
                if let Some(probe) = self.sandbox_runtime_probe_text().await {
                    system.push_str("\n\n## Detected sandbox runtimes (automatic probe)\n\n```text\n");
                    system.push_str(probe.trim_end());
                    system.push_str("\n```\n");
                }
            }
            if !agent_context.is_empty() {
                system.push_str("\n\n---\n\n");
                system.push_str(&agent_context);
            }
            history.push(Message {
                role: Role::System,
                content: system,
            });
            // In the sandbox, the LLM only sees `/workspace`; telling it the host path makes
            // it emit scripts that list the wrong directory. Unsandboxed mode still needs cwd.
            let first_user = if self.executor.is_some() {
                user_message.to_string()
            } else {
                format!(
                    "Project directory: {}\n\nRequest: {}",
                    self.project_dir.display(),
                    user_message
                )
            };
            history.push(Message {
                role: Role::User,
                content: first_user,
            });
        } else {
            // Subsequent turns: append user message directly.
            history.push(Message {
                role: Role::User,
                content: user_message.to_string(),
            });
        }

        let messages = history;
        let mut state = AgentState::Thinking { messages_sent: messages.len() };
        let mut turns = 0;

        while !state.is_terminal() && turns < self.max_turns {
            state = self.step(state, messages, events, interrupt).await?;
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

/// Extract the "thought" text from a response — everything before the first code block,
/// with the `<hick:next>` intent tag stripped.
fn extract_thought(response: &str) -> Option<String> {
    let stripped = response
        .trim_start()
        .trim_start_matches("<hick:next>code</hick:next>")
        .trim_start_matches("<hick:next>done</hick:next>")
        .trim_start();
    let text = if let Some(pos) = stripped.find("```") {
        stripped[..pos].trim().to_string()
    } else {
        stripped.trim().to_string()
    };
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
