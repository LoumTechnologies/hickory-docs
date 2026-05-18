//! Script-first ReAct loop implemented on top of `hick-agent-sdk`.
//!
//! [`ReactLoop`] wraps an [`AgentRuntime`] and drives the script-first
//! strategy: the LLM writes fenced code blocks, the loop extracts and
//! executes them in a WASM sandbox, then feeds stdout back as the next
//! observation. History management, LLM streaming, and project-context
//! loading all delegate to the runtime.
//!
//! # Loop structure (expressed via AgentRuntime + Step)
//!
//! ```text
//! while !done && turns < max_turns:
//!   stream = runtime.complete_stream()          // Step::AssistantChunk* + AssistantDone
//!   validate <hick:next> protocol, re-prompt if needed
//!   extract code blocks from response
//!   if no blocks → Done
//!   execute first block in sandbox
//!   runtime.add_user_message(observation)       // feed stdout back
//! ```

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use hick_agent_core::CommandExecutor;
use hick_agent_sdk::runtime::Step;
use hick_agent_sdk::{AgentRuntime, CommandOutput, LlmClient, Message, Role, SinkGuard};
use hick_policy::PolicyEvaluator;
use tokio::sync::{Mutex, Notify, mpsc};

use crate::events::AgentEvent;
use crate::state::AgentState;
use crate::tools::{CodeBlock, Language, ScriptResult, extract_code_blocks};

// ── System prompt ────────────────────────────────────────────────────────────

const SYSTEM_PROMPT: &str = r#"You are an AI coding agent. You work by writing and executing code to accomplish tasks.

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

Pick a fence whose runner exists in your environment (see the runtime probe):

- Shell: ```bash, ```sh, ```shell, ```posix-sh — saved as POSIX `sh`; use for `apk add`, `ls`, pipelines, POSIX scripts, installers, compilers, etc.
- Python: ```python — `python3`
- JavaScript (Node): ```javascript / ```js — `node`
- C#: ```csharp / ```cs — `dotnet-script`
- TypeScript: ```typescript / ```ts — `tsx`

You may install language runtimes or packages yourself inside the sandbox (e.g. `apk add …`, `npm install -g …`, pip, curling binaries) whenever you need them. No separate permission step — go ahead and fix the environment if something is missing.

## Available languages

### Shell (recommended for discovery and installs)
- Prefer ```bash when you want simple commands: `find`, `ls`, `apk add python3 py3-pip`, `wget`, etc.
- Scripts run as POSIX `sh` (shebang lines are ignored). Call another interpreter explicitly (`python3 …`, `bash …`) once installed.

### Python
- Read/write files with `open()`, `os`, `pathlib`
- Run shell commands with `subprocess.run()`
- Install packages: `subprocess.run(['pip', 'install', ...])` (after installing pip if needed)

### JavaScript (Node)
- Runs with `node`. Use `.mjs` style or plain CommonJS depending on what's available after install.

### C# (dotnet-script)
- Write C# script files (.csx) — no Main() or class wrapper needed
- Install NuGet packages: `#r "nuget: Newtonsoft.Json, 13.0.3"`
- Requires `dotnet` / `dotnet-script` when you choose this fence.

### TypeScript (tsx)
- Write TypeScript; requires `tsx` (install globally or via npm/npx depending on probe output).

## Key concepts

1. **Print your results.** Your script's stdout is what you see as observation. Use print/Console.WriteLine/console.log in typed languages, or echo/printf for shell fences, to report findings.
"#;

/// Appended to the system prompt when code runs in the WASM sandbox (normal mode).
const SANDBOX_EXECUTION_CONTEXT: &str = r#"

## Execution environment (sandbox)

Your fenced code blocks run in an isolated Linux/WASM container. The project is mounted at **`/workspace`** and shell commands start from there (`cd /workspace`, etc.).

In every script, use **`/workspace`**, **`WORKSPACE`** (set for Python), or paths **relative to `/workspace`**. **Do not** use host absolute paths such as `/home/...`, `/Users/...`, or `~/...` — those paths do not exist inside the container.

The prompt includes a short **automatic runtime probe** from this container listing which binaries are installed; trust that output over assumptions from training data. If something is missing, install it yourself in-code (apk, npm, etc.) unless the error clearly blocks that.
"#;

/// Probe command run once per session inside the sandbox to guide the LLM.
const SANDBOX_RUNTIME_PROBE: &str = r#"echo '### Sandbox runtime probe'; echo '---'; uname -a 2>&1 || true; echo '--- interpreters / tools ---'; for c in sh ash bash python3 python dotnet dotnet-script node npm npx tsx bun deno rustc cargo ruby lua apk apt-get wget curl git; do printf '%s: ' "$c"; command -v "$c" >/dev/null 2>&1 && command -v "$c" || echo '(not found)'; done"#;

// ── Response protocol ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResponseIntent {
    Code,
    Done,
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

fn correction_message(reason: &str) -> String {
    format!(
        "Your last response was invalid: {reason}.\n\n\
         Please try again. Start your response with exactly one of:\n\
         - `<hick:next>code</hick:next>` followed immediately by a fenced code block \
           (shell: bash/sh/shell — or python, javascript/js, csharp/cs, typescript/ts)\n\
         - `<hick:next>done</hick:next>` followed by your final plain-text answer with no code blocks"
    )
}

// ── NoopSandbox ──────────────────────────────────────────────────────────────

/// Placeholder sandbox passed to AgentRuntime when the real executor is managed
/// separately by ReactLoop. The runtime's `run_command` is never called by the
/// script-first loop — script execution goes through `ReactLoop.executor`.
struct NoopSandbox;

#[async_trait]
impl hick_agent_sdk::CommandExecutor for NoopSandbox {
    async fn execute(
        &mut self,
        _container: &str,
        _command: &str,
        _image: &str,
        _timeout: Option<u64>,
    ) -> anyhow::Result<CommandOutput> {
        Err(anyhow::anyhow!(
            "NoopSandbox: script execution is managed by ReactLoop, not AgentRuntime"
        ))
    }
}

// ── ReactLoop ────────────────────────────────────────────────────────────────

/// Script-first ReAct loop runner.
///
/// Uses [`AgentRuntime`] for LLM streaming and conversation history. Script
/// execution is handled by the hick-agent-core [`CommandExecutor`] so that the
/// existing WASM sandbox integration is reused without change.
///
/// Communicates with the UI through [`AgentEvent`] channels, never printing directly.
pub struct ReactLoop {
    /// SDK runtime — manages history, LLM calls, and project context.
    runtime: AgentRuntime,
    /// Host project directory (used for script staging under `agent_dir`).
    pub project_dir: PathBuf,
    max_turns: usize,
    /// Maximum re-prompt attempts when the LLM violates the response protocol.
    max_reprompts: usize,
    /// Whether to stream tokens to the UI incrementally.
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
    /// Hidden directory under `project_dir` used for script staging and optional SDK.
    /// Defaults to `".agent"`. Set to `".hick"` for iron-peak.
    agent_dir: String,
    /// Overrides the built-in system prompt when set.
    custom_system_prompt: Option<String>,
    /// When true, injects `PYTHONPATH=<agent_dir>/sdk` for Python script execution.
    inject_sdk_pythonpath: bool,
}

impl ReactLoop {
    /// Create a new loop.
    ///
    /// The `llm` is wrapped in an [`AgentRuntime`] that manages streaming and
    /// history. A [`NoopSandbox`] is passed to the runtime since the real
    /// sandbox is set via [`with_executor`].
    pub fn new(llm: Box<dyn LlmClient>, project_dir: PathBuf) -> Self {
        let runtime = AgentRuntime::new(llm, Box::new(NoopSandbox), project_dir.clone());
        Self {
            runtime,
            project_dir,
            max_turns: 50,
            max_reprompts: 2,
            stream_output: true,
            executor: None,
            sandbox_image: "alpine".to_string(),
            policy_evaluator: None,
            sink_guard: None,
            agent_dir: ".agent".to_string(),
            custom_system_prompt: None,
            inject_sdk_pythonpath: false,
        }
    }

    /// Override the maximum number of re-prompt retries on protocol violations.
    pub fn with_max_reprompts(mut self, n: usize) -> Self {
        self.max_reprompts = n;
        self
    }

    /// Set the hidden agent directory used for script staging (default: `".agent"`).
    ///
    /// Scripts are staged at `<project_dir>/<agent_dir>/tmp/` on the host and
    /// `/workspace/<agent_dir>/tmp/` inside the WASM container.
    pub fn with_agent_dir(mut self, dir: impl Into<String>) -> Self {
        self.agent_dir = dir.into();
        self
    }

    /// Override the default system prompt with a custom one.
    ///
    /// The sandbox execution context and project context are still appended.
    /// Use this to inject agent-specific instructions (e.g. hick pipeline editing).
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.custom_system_prompt = Some(prompt.into());
        self
    }

    /// Enable `PYTHONPATH=<agent_dir>/sdk` injection for Python script execution.
    ///
    /// Use this when an SDK is bundled into `<project_dir>/<agent_dir>/sdk/`.
    pub fn with_sdk(mut self) -> Self {
        self.inject_sdk_pythonpath = true;
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

    /// Get the sink guard (if configured).
    pub fn sink_guard(&self) -> Option<&Arc<SinkGuard>> {
        self.sink_guard.as_ref()
    }

    // ── LLM call via AgentRuntime + Step ─────────────────────────────────────

    /// Call the LLM once using `runtime.complete_stream()` and collect the response.
    ///
    /// Processes [`Step::AssistantChunk`] and [`Step::AssistantDone`] events.
    /// When `emit_tokens` is true, [`AgentEvent::Token`] events are sent for each
    /// chunk (used for the first call per turn). Re-prompt retries pass `emit_tokens =
    /// false` so only the final accepted response appears in the UI.
    ///
    /// Returns `None` when the user interrupted.
    async fn llm_call(
        &mut self,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
        emit_tokens: bool,
    ) -> anyhow::Result<Option<String>> {
        // complete_stream clones history internally — the returned stream is 'static
        // and does not borrow self.runtime after the await returns.
        let mut stream = self.runtime.complete_stream().await?;
        let mut response = String::new();
        let interrupt_fut = interrupt.notified();
        tokio::pin!(interrupt_fut);

        loop {
            tokio::select! {
                biased;
                _ = &mut interrupt_fut => return Ok(None),
                step = stream.next() => {
                    match step {
                        Some(Ok(Step::AssistantChunk(chunk))) => {
                            if emit_tokens && self.stream_output && !chunk.delta.is_empty() {
                                events.send(AgentEvent::Token(chunk.delta.clone())).ok();
                            }
                            response.push_str(&chunk.delta);
                        }
                        Some(Ok(Step::AssistantDone(msg))) => {
                            // When not streaming, emit the full response as one Token.
                            if emit_tokens && !self.stream_output {
                                events.send(AgentEvent::Token(msg.content.clone())).ok();
                            }
                            response = msg.content;
                            break;
                        }
                        Some(Ok(_)) => {} // other Step variants not used here
                        Some(Err(e)) => return Err(e),
                        None => break,
                    }
                }
            }
        }

        Ok(Some(response))
    }

    // ── Sandbox probe ─────────────────────────────────────────────────────────

    async fn sandbox_runtime_probe_text(&mut self) -> Option<String> {
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
                        let _ = writeln!(out, "---\nSandbox runtime probe reported exit_code: {code}");
                    }
                    None => {
                        use std::fmt::Write as _;
                        let _ = writeln!(out, "---\nSandbox runtime probe exited (code unknown)");
                    }
                }
                Some(out)
            }
            Err(e) => Some(format!("Sandbox runtime probe failed: {e}")),
        }
    }

    // ── Script execution ──────────────────────────────────────────────────────

    async fn execute_code_block(&self, block: &CodeBlock, index: usize) -> ScriptResult {
        let tmp_dir = self.project_dir.join(&self.agent_dir).join("tmp");
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

        let container_script_path =
            format!("/workspace/{}/tmp/{filename}", self.agent_dir);

        if let Some(ref executor) = self.executor {
            let run_cmd = block.language.run_command(&container_script_path);
            let cmd = match block.language {
                Language::Python => {
                    if self.inject_sdk_pythonpath {
                        format!(
                            "PYTHONPATH=/workspace/{}/sdk WORKSPACE=/workspace {run_cmd}",
                            self.agent_dir
                        )
                    } else {
                        format!("WORKSPACE=/workspace {run_cmd}")
                    }
                }
                Language::Shell
                | Language::JavaScript
                | Language::TypeScript
                | Language::CSharp => format!("cd /workspace && {run_cmd}"),
            };

            let mut exec = executor.lock().await;

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
            let sdk_path = self.project_dir.join(&self.agent_dir).join("sdk");
            match block.language {
                Language::Python => {
                    if self.inject_sdk_pythonpath {
                        let py_env = [
                            (std::ffi::OsStr::new("PYTHONPATH"), sdk_path.as_os_str()),
                            (std::ffi::OsStr::new("WORKSPACE"), self.project_dir.as_os_str()),
                        ];
                        spawn_and_collect("python3", &[path_os], cwd, &py_env).await
                    } else {
                        let py_env = [(
                            std::ffi::OsStr::new("WORKSPACE"),
                            self.project_dir.as_os_str(),
                        )];
                        spawn_and_collect("python3", &[path_os], cwd, &py_env).await
                    }
                }
                Language::Shell => spawn_and_collect("sh", &[path_os], cwd, &[]).await,
                Language::JavaScript => spawn_and_collect("node", &[path_os], cwd, &[]).await,
                Language::TypeScript => spawn_and_collect("tsx", &[path_os], cwd, &[]).await,
                Language::CSharp => {
                    spawn_and_collect("dotnet-script", &[path_os], cwd, &[]).await
                }
            }
        }
    }

    async fn execute_shell_command(&self, cmd: &str) -> ScriptResult {
        if let Some(ref executor) = self.executor {
            let sandbox_cmd = format!("cd /workspace && {cmd}");
            let mut exec = executor.lock().await;
            match exec
                .execute("agent-sandbox", &sandbox_cmd, &self.sandbox_image, Some(60))
                .await
            {
                Ok(result) => ScriptResult {
                    stdout: result.output,
                    stderr: String::new(),
                    exit_code: result.exit_code,
                },
                Err(e) => ScriptResult {
                    stdout: String::new(),
                    stderr: format!("Command failed: {e}"),
                    exit_code: None,
                },
            }
        } else {
            use std::process::Stdio;
            match tokio::process::Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&self.project_dir)
                .spawn()
            {
                Ok(child) => match child.wait_with_output().await {
                    Ok(out) => ScriptResult {
                        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                        exit_code: out.status.code(),
                    },
                    Err(e) => ScriptResult {
                        stdout: String::new(),
                        stderr: format!("Failed to collect output: {e}"),
                        exit_code: None,
                    },
                },
                Err(e) => ScriptResult {
                    stdout: String::new(),
                    stderr: format!("Failed to spawn sh: {e}"),
                    exit_code: None,
                },
            }
        }
    }

    // ── State machine step ────────────────────────────────────────────────────

    /// Run one ReAct step via AgentRuntime + Step events.
    async fn step_once(
        &mut self,
        state: AgentState,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
    ) -> anyhow::Result<AgentState> {
        match state {
            AgentState::Thinking { .. } => {
                events.send(AgentEvent::Thinking).ok();

                let Some(mut response) = self.llm_call(events, interrupt, true).await? else {
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
                    // Fold the invalid response + correction into history via the runtime.
                    self.runtime.add_message(Message {
                        role: Role::Assistant,
                        content: response.clone(),
                    });
                    self.runtime
                        .add_user_message(correction_message(&reason));

                    events.send(AgentEvent::Thinking).ok();
                    let Some(r) = self.llm_call(events, interrupt, false).await? else {
                        events.send(AgentEvent::Interrupted).ok();
                        return Ok(AgentState::to_done("(interrupted)".into()));
                    };
                    response = r;
                }

                events
                    .send(AgentEvent::ResponseComplete(response.clone()))
                    .ok();

                // Add the accepted assistant message to history.
                self.runtime.add_message(Message {
                    role: Role::Assistant,
                    content: response.clone(),
                });

                let all_blocks = extract_code_blocks(&response);

                if all_blocks.is_empty() {
                    return Ok(AgentState::to_done(response.trim().to_string()));
                }

                // Execute only the first block per turn.
                let blocks = vec![all_blocks.into_iter().next().unwrap()];
                let thought = extract_thought(&response);
                Ok(AgentState::to_executing(blocks, thought))
            }

            AgentState::Executing { blocks, .. } => {
                let mut results = Vec::with_capacity(blocks.len());

                for (i, block) in blocks.iter().enumerate() {
                    // Policy check before execution.
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
                            // TODO: wire up full prompt flow (Always Allow milestone)
                            Ok(hick_policy::PolicyDecision::Prompt { reason }) => {
                                events
                                    .send(AgentEvent::SecurityViolation {
                                        description: format!(
                                            "Policy requires approval for script execution: {reason}"
                                        ),
                                    })
                                    .ok();
                                results.push(ScriptResult {
                                    stdout: String::new(),
                                    stderr: format!(
                                        "Policy requires approval: {reason}"
                                    ),
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
                            lang: block.language.extension().to_string(),
                        })
                        .ok();
                    let result = self.execute_code_block(block, i).await;
                    events.send(AgentEvent::ScriptFinished(result.clone())).ok();
                    results.push(result);
                }

                // Format observation and feed back to LLM via runtime.
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

                self.runtime.add_user_message(observation);
                Ok(AgentState::to_observing(results))
            }

            _ => Ok(state),
        }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Execute a shell command directly in the WASM sandbox without calling the LLM.
    ///
    /// Records the command + output in conversation history so subsequent LLM turns
    /// have context. Emits `ScriptStarted`/`ScriptFinished`/`Done` events.
    pub async fn run_direct_command(
        &mut self,
        cmd: &str,
        events: &mpsc::UnboundedSender<AgentEvent>,
    ) -> anyhow::Result<()> {
        events
            .send(AgentEvent::ScriptStarted {
                code: cmd.to_string(),
                lang: "sh".to_string(),
            })
            .ok();

        let result = self.execute_shell_command(cmd).await;
        events.send(AgentEvent::ScriptFinished(result.clone())).ok();

        let obs = format!(
            "User ran directly (! command): {cmd}\nOutput:\n{}",
            result.as_observation()
        );
        self.runtime.add_user_message(obs);

        events.send(AgentEvent::Done(String::new())).ok();
        Ok(())
    }

    /// Run the full ReAct loop until done or max turns.
    ///
    /// On the first call the system prompt and initial user message are added to
    /// the runtime's history. On subsequent calls only the new user message is
    /// appended, preserving the full conversation context.
    ///
    /// Emits [`AgentEvent`]s on the channel and checks `interrupt` for
    /// user-requested cancellation.
    pub async fn run(
        &mut self,
        user_message: &str,
        events: &mpsc::UnboundedSender<AgentEvent>,
        interrupt: &Notify,
    ) -> anyhow::Result<()> {
        if self.runtime.history().is_empty() {
            // First turn: build and inject the system prompt.
            let base = self
                .custom_system_prompt
                .as_deref()
                .unwrap_or(SYSTEM_PROMPT);
            let mut system = base.to_string();
            if self.executor.is_some() {
                system.push_str(SANDBOX_EXECUTION_CONTEXT);
                if let Some(probe) = self.sandbox_runtime_probe_text().await {
                    system.push_str(
                        "\n\n## Detected sandbox runtimes (automatic probe)\n\n```text\n",
                    );
                    system.push_str(probe.trim_end());
                    system.push_str("\n```\n");
                }
            }
            let agent_context = self.runtime.load_project_context();
            if !agent_context.is_empty() {
                system.push_str("\n\n---\n\n");
                system.push_str(&agent_context);
            }
            self.runtime.add_system_message(system);

            // In the sandbox the LLM only sees `/workspace`; give the project path
            // only in unsandboxed mode.
            let first_user = if self.executor.is_some() {
                user_message.to_string()
            } else {
                format!(
                    "Project directory: {}\n\nRequest: {}",
                    self.project_dir.display(),
                    user_message
                )
            };
            self.runtime.add_user_message(first_user);
        } else {
            self.runtime.add_user_message(user_message.to_string());
        }

        let mut state = AgentState::Thinking {
            messages_sent: self.runtime.history().len(),
        };
        let mut turns = 0;

        while !state.is_terminal() && turns < self.max_turns {
            state = self.step_once(state, events, interrupt).await?;
            turns += 1;

            if let AgentState::Observing { .. } = state {
                state = AgentState::Thinking {
                    messages_sent: self.runtime.history().len(),
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

// ── Helpers ───────────────────────────────────────────────────────────────────

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
        assert!(extract_thought(response).is_none());
    }

    #[test]
    fn extract_thought_no_code_blocks() {
        let response = "This is the final answer with no code.";
        assert_eq!(
            extract_thought(response).unwrap(),
            "This is the final answer with no code."
        );
    }
}
