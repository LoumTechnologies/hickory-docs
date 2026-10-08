//! Python requests actions; Rust owns the conversation, facts and workspace effects.
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use hickory_agent::{HickSessionLog, LlmClient, Message, Role, SessionEvent, SessionLog};
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::mcp::Server;
const MAX_FRAME: usize = 1_048_576;
const INSTRUCTIONS: &str = r#"You are Hickory's Python agent. Reply with ONLY one JSON object:
{"python":"Python snippet"} or {"done":"summary with limitations"}.
Only Python snippets are executable. They run in Monty, not CPython: no filesystem,
shell, network or CPython execution. Call tool(name, arguments) to request a Hickory tool.
For example: tool("list_docs", {}) or tool("read_doc", {"doc":"notes.md"}).
Use the supplied tool schemas. Read a surface before editing, and carry its snapshot
and SHA-256 anchors. Project documents may contain the team's own language.
Every edit requires separate human approval of its exact preview. Acceptance never
turns AI-authored bytes into human-authored bytes. Existing and unknown authorship
are protected alike. Record consequential assumptions and decisions BEFORE relying
on them; preserve corrections and omissions for retrospective debugging.
The current bridge does not expose command execution or verify: those tools can
write derived workspace files without a preview. Executable edits remain unchecked.
A completion summary is a model declaration, never verification. State tool refusals,
failed checks and material unresolved assumptions in your summary.
"#;

pub struct Config {
    pub python: PathBuf,
    pub root: PathBuf,
    pub doc: Option<PathBuf>,
    pub prompt: String,
    pub max_turns: usize,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Proposal {
    pub tool: String,
    pub arguments: Value,
    pub surface: String,
    pub before: String,
    pub after: String,
    pub revision: Option<String>,
    pub sources: Vec<Value>,
}

#[async_trait::async_trait]
pub trait Review: Send {
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool>;
}

pub struct TerminalReview;
#[async_trait::async_trait]
impl Review for TerminalReview {
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool> {
        use std::io::{IsTerminal, Write};
        if !std::io::stdin().is_terminal() {
            eprintln!("agent: edit refused: separate interactive human approval required");
            return Ok(false);
        }
        let proposal = proposal.clone();
        // A blocked terminal read cannot be cancelled. Do not put it in Tokio's
        // blocking pool: runtime shutdown would wait for it after Ctrl-C.
        let (send, receive) = tokio::sync::oneshot::channel();
        std::thread::Builder::new()
            .name("hickory-agent-review".into())
            .spawn(move || {
                let result = (|| -> Result<bool> {
                    let diff = similar::TextDiff::from_lines(&proposal.before, &proposal.after);
                    eprintln!(
                        "\nAgent edit proposal: {} ({})\n{}",
                        proposal.surface,
                        proposal.tool,
                        diff.unified_diff().header("before", "proposed")
                    );
                    eprint!("Modify these protected bytes? Type 'approve' to accept: ");
                    std::io::stderr().flush()?;
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line)?;
                    Ok(line.trim() == "approve")
                })();
                let _ = send.send(result);
            })?;
        receive
            .await
            .context("Terminal review ended without a decision")?
    }
}

fn append(path: &Path, event: SessionEvent<'_>) -> Result<()> {
    let log = HickSessionLog::append_or_create(path)?;
    log.record(event);
    log.record(SessionEvent::End);
    log.check()
}
fn metadata(path: &Path, kind: &str, data: &Value) -> Result<()> {
    append(path, SessionEvent::Context { kind, data })
}
fn contained(root: &Path, path: &Path) -> Result<PathBuf> {
    let actual = path
        .canonicalize()
        .with_context(|| format!("Unavailable path {}", path.display()))?;
    ensure!(
        actual.starts_with(root),
        "Path is outside the selected workspace"
    );
    Ok(actual)
}
fn validate_source(root: &Path, path: &Path, source: &str, seen: &mut Vec<PathBuf>) -> Result<()> {
    ensure!(seen.len() < 100, "Document closure exceeds 100 files");
    if seen.contains(&path.to_path_buf()) {
        return Ok(());
    }
    seen.push(path.to_path_buf());
    let doc = hick_lang::parse(source)?;
    for tag in doc.all_tags() {
        if !matches!(tag.name.as_str(), "include" | "upstream") {
            continue;
        }
        let rel = tag
            .get_attribute("file")
            .context("Document dependency has no file")?;
        let next = contained(
            root,
            &path
                .parent()
                .context("Document has no directory")?
                .join(rel),
        )?;
        validate_source(root, &next, &std::fs::read_to_string(&next)?, seen)?;
    }
    Ok(())
}
fn allowed(name: &str) -> bool {
    matches!(
        name,
        "read_doc"
            | "read_output"
            | "read_file"
            | "edit_doc"
            | "edit_output"
            | "create_doc"
            | "list_docs"
            | "search"
    ) || hickory_agent::audit::is_tool(name)
}
struct Host<'a> {
    config: &'a Config,
    client: &'a dyn LlmClient,
    review: &'a mut dyn Review,
    server: Server,
    session: PathBuf,
    messages: Vec<Message>,
    turns: usize,
    tools: usize,
    last_action: Option<Value>,
    in_python: bool,
    finished: bool,
    unchecked_edits: bool,
}
impl Host<'_> {
    fn observation(&mut self, data: Value) -> Result<Value> {
        metadata(
            &self.session,
            "python-observation",
            &json!({"actor":"worker-reported","value":data}),
        )?;
        self.messages.push(Message::new(
            Role::User,
            format!("Python worker observation (not independent verification): {data}"),
        ));
        Ok(json!({}))
    }
    async fn proposal(&mut self, name: &str, args: &Value) -> Result<Proposal> {
        let (surface, before, after) = self
            .server
            .preview_edit(name, args)
            .await
            .map_err(anyhow::Error::msg)?;
        let (revision, sources) = if name == "create_doc" {
            (None, vec![])
        } else {
            let (revision, sources) = self
                .server
                .agent_revision(args)
                .await
                .map_err(anyhow::Error::msg)?;
            (Some(revision), sources)
        };
        let doc = args
            .get("doc")
            .and_then(Value::as_str)
            .map(|d| self.config.root.join(d))
            .or_else(|| self.config.doc.clone());
        if name == "create_doc" {
            let path = self.config.root.join(&surface);
            contained(
                &self.config.root,
                path.parent().context("No document directory")?,
            )?;
            validate_source(&self.config.root, &path, &after, &mut vec![])?;
        } else if name == "edit_doc" {
            let path = if args.get("upstream").is_some() {
                self.config.root.join(&surface)
            } else {
                doc.context("No document")?
            };
            validate_source(&self.config.root, &path, &after, &mut vec![])?;
        }
        Ok(Proposal {
            tool: name.into(),
            arguments: args.clone(),
            surface,
            before,
            after,
            revision,
            sources,
        })
    }
    async fn tool(&mut self, params: &Value) -> Result<Value> {
        ensure!(self.in_python, "Tools require a recorded Python action");
        self.tools += 1;
        ensure!(
            self.tools <= self.config.max_turns * 64,
            "Tool budget exhausted"
        );
        let name = params["name"]
            .as_str()
            .context("Tool name must be a string")?;
        ensure!(
            allowed(name),
            "Tool {name} is not exposed by the Python agent"
        );
        let args = params.get("arguments").cloned().unwrap_or(json!({}));
        ensure!(args.is_object(), "Tool arguments must be an object");
        let doc = args
            .get("doc")
            .and_then(Value::as_str)
            .map(|d| self.config.root.join(d))
            .or_else(|| self.config.doc.clone());
        if matches!(
            name,
            "read_doc" | "read_output" | "read_file" | "edit_doc" | "edit_output"
        ) {
            let path = contained(
                &self.config.root,
                &doc.context("Pass doc or start with --doc")?,
            )?;
            validate_source(
                &self.config.root,
                &path,
                &std::fs::read_to_string(&path)?,
                &mut vec![],
            )?;
            if name == "read_file" {
                let file = args["path"].as_str().context("Pass a file path")?;
                contained(&self.config.root, &path.parent().unwrap().join(file))?;
            }
        }
        if matches!(name, "edit_doc" | "edit_output" | "create_doc") {
            let proposal = self.proposal(name, &args).await?;
            metadata(
                &self.session,
                "python-edit-proposal",
                &serde_json::to_value(&proposal)?,
            )?;
            let approved = self.review.approve(&proposal).await?;
            metadata(
                &self.session,
                "python-edit-review",
                &json!({"actor":"user-declared","approved":approved,"proposal":proposal}),
            )?;
            if !approved {
                return Ok(
                    json!({"isError":true,"content":[{"type":"text","text":"Human declined the proposed edit; no edit ran."}]}),
                );
            }
            ensure!(
                proposal == self.proposal(name, &args).await?,
                "Proposal changed during approval; no edit ran"
            );
            self.unchecked_edits = true;
        }
        let result = self
            .server
            .handle("tools/call", &json!({"name":name,"arguments":args}))
            .await
            .map_err(|(_, text)| anyhow::anyhow!(text))?;
        self.messages.push(Message::new(
            Role::User,
            format!("Observed Rust tool result ({name}): {result}"),
        ));
        Ok(result)
    }
    async fn handle(&mut self, method: &str, params: &Value) -> Result<Value> {
        ensure!(!self.finished, "The model already stopped");
        match method {
            "model" => {
                ensure!(
                    !self.in_python,
                    "Missing observation for previous Python action"
                );
                ensure!(
                    self.turns < self.config.max_turns,
                    "Model turn budget exhausted"
                );
                ensure!(
                    self.messages.iter().map(|m| m.content.len()).sum::<usize>() <= 8_000_000,
                    "Conversation context budget exhausted; inspect session"
                );
                metadata(
                    &self.session,
                    "python-model-request",
                    &json!({"messages":self.messages,"turn":self.turns,"model":self.client.model_name()}),
                )?;
                let (raw, usage) = tokio::time::timeout(
                    Duration::from_secs(180),
                    self.client.complete_with_usage(self.messages.clone()),
                )
                .await??;
                ensure!(
                    raw.len() <= MAX_FRAME / 2,
                    "Model response exceeds protocol budget"
                );
                append(
                    &self.session,
                    SessionEvent::Assistant {
                        prose: &raw,
                        action: None,
                        reasoning: None,
                    },
                )?;
                append(
                    &self.session,
                    SessionEvent::Usage {
                        turn: Some(self.turns),
                        usage,
                        cost_usd: None,
                    },
                )?;
                self.messages.push(Message::new(Role::Assistant, &raw));
                self.turns += 1;
                self.last_action = serde_json::from_str(&raw).ok();
                Ok(Value::String(raw))
            }
            "begin_python" => {
                ensure!(!self.in_python, "A Python action is already running");
                let code = params["code"].as_str().context("Missing Python code")?;
                ensure!(code.len() <= 65_536, "Python snippet exceeds 64 KiB");
                ensure!(
                    self.last_action.as_ref() == Some(&json!({"python":code})),
                    "Python code must match the recorded model response"
                );
                metadata(
                    &self.session,
                    "python-action",
                    &json!({"code":code,"interpreter":"monty","state":"requested"}),
                )?;
                self.in_python = true;
                self.last_action = None;
                Ok(json!({}))
            }
            "tool" => self.tool(params).await,
            "observation" => {
                self.in_python = false;
                self.observation(params.clone())
            }
            "finish" => {
                ensure!(!self.in_python, "Python action has not finished");
                let summary = params["summary"]
                    .as_str()
                    .context("Missing completion summary")?;
                ensure!(
                    self.last_action.as_ref() == Some(&json!({"done":summary})),
                    "Completion must match the recorded model response"
                );
                metadata(
                    &self.session,
                    "python-completion",
                    &json!({"actor":"harness","model_summary":summary,"model_stopped":true,"verification":"unrun","unchecked_edits":self.unchecked_edits,"assumption_coverage":"Not established"}),
                )?;
                println!("Model summary (checks unrun):\n{summary}");
                eprintln!(
                    "agent: model stopped; checks unrun{}",
                    if self.unchecked_edits {
                        "; edits remain unchecked"
                    } else {
                        ""
                    }
                );
                self.finished = true;
                Ok(json!({"accepted":true}))
            }
            _ => bail!("Unknown agent protocol method {method}"),
        }
    }
}
struct ChildGuard {
    child: tokio::process::Child,
    pid: u32,
}
impl Drop for ChildGuard {
    fn drop(&mut self) {
        log::debug!("Stopping Python agent process {}", self.pid);
        #[cfg(unix)]
        // SAFETY: the child was launched in its own process group.
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGKILL);
        }
        let _ = self.child.start_kill();
    }
}

pub async fn run(
    config: Config,
    client: &dyn LlmClient,
    review: &mut dyn Review,
) -> Result<PathBuf> {
    ensure!(
        (1..=100).contains(&config.max_turns),
        "max-turns must be between 1 and 100"
    );
    let mut config = config;
    config.root = config.root.canonicalize()?;
    for path in [
        "/usr",
        "/lib",
        "/lib64",
        "/bin",
        "/System",
        "/Library/Apple",
    ] {
        ensure!(
            !config.root.starts_with(path),
            "Workspace cannot be inside a system runtime read root"
        );
    }
    for root in hickory_executor_sandbox::agent_process::runtime_roots(&config.python)? {
        ensure!(
            !config.root.starts_with(&root),
            "Workspace cannot be inside a trusted runtime read root"
        );
    }
    if let Some(doc) = &config.doc {
        config.doc = Some(contained(&config.root, &config.root.join(doc))?);
    }
    let scratch = tempfile::tempdir()?;
    let mut cmd = tokio::process::Command::from(hickory_executor_sandbox::agent_process::command(
        &config.python,
        scratch.path(),
    )?);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
    let session_dir = config.root.join("sessions");
    std::fs::create_dir_all(&session_dir)?;
    contained(&config.root, &session_dir)?;
    let (_, session) = tempfile::Builder::new()
        .prefix("python-agent-")
        .suffix(".md")
        .tempfile_in(&session_dir)?
        .keep()?;
    let initial = HickSessionLog::create_for(&session, config.doc.as_deref())?;
    initial.record(SessionEvent::User {
        text: &config.prompt,
    });
    initial.record(SessionEvent::End);
    initial.check()?;
    eprintln!("agent: session {}", session.display());
    let executor = crate::ExecutorChoice::Sandbox
        .build_for(Some(&config.root))
        .await?;
    let mut server = Server::agent_bridge(
        config.root.clone(),
        config.doc.clone(),
        executor.clone(),
        session.clone(),
    );
    let mut catalogue = server
        .handle("tools/list", &json!({}))
        .await
        .map_err(|(_, msg)| anyhow::anyhow!(msg))?;
    catalogue["tools"]
        .as_array_mut()
        .unwrap()
        .retain(|tool| tool["name"].as_str().is_some_and(allowed));
    let instructions = format!(
        "{INSTRUCTIONS}\n{}\nTool schemas:\n{catalogue}",
        hickory_agent::audit::INSTRUCTIONS
    );
    let mut host = Host {
        config: &config,
        client,
        review,
        server,
        session: session.clone(),
        messages: vec![
            Message::new(Role::System, instructions),
            Message::new(Role::User, &config.prompt),
        ],
        turns: 0,
        tools: 0,
        last_action: None,
        in_python: false,
        finished: false,
        unchecked_edits: false,
    };
    let worker_run = async {
        let child = cmd.spawn().context("Starting confined Python agent")?;
        let pid = child.id().context("Worker has no process id")?;
        let mut guard = ChildGuard { child, pid };
        let mut stdin = guard.child.stdin.take().context("No worker input pipe")?;
        let mut stdout =
            BufReader::new(guard.child.stdout.take().context("No worker output pipe")?);
        stdin
            .write_all(format!("{}\n", json!({"max_turns":config.max_turns})).as_bytes())
            .await?;
        let mut sequence = 0u64;
        for _ in 0..(config.max_turns * 70 + 1) {
            initial.check()?;
            let mut frame = Vec::new();
            let read = tokio::time::timeout(
                Duration::from_secs(30),
                (&mut stdout)
                    .take((MAX_FRAME + 1) as u64)
                    .read_until(b'\n', &mut frame),
            )
            .await??;
            ensure!(
                read > 0 && read <= MAX_FRAME && frame.last() == Some(&b'\n'),
                "Worker ended or exceeded protocol frame limit"
            );
            let request: Value = serde_json::from_slice(&frame)?;
            let method = request["method"]
                .as_str()
                .context("Missing protocol method")?;
            if method == "exit" {
                ensure!(
                    host.finished,
                    "Worker exited without model completion (turn limit or failure)"
                );
                return Ok(());
            }
            sequence += 1;
            ensure!(
                request["id"].as_u64() == Some(sequence),
                "Replayed or out-of-order worker request"
            );
            let params = request.get("params").cloned().unwrap_or(json!({}));
            metadata(
                &session,
                "python-rpc-request",
                &json!({"sequence":sequence,"method":method,"params":params}),
            )?;
            let response = match host.handle(method, &params).await {
                Ok(result) => json!({"id":sequence,"result":result}),
                Err(error) => {
                    metadata(
                        &session,
                        "python-rpc-refusal",
                        &json!({"sequence":sequence,"error":format!("{error:#}")}),
                    )?;
                    json!({"id":sequence,"error":format!("{error:#}")})
                }
            };
            initial.check()?;
            let output = format!("{response}\n");
            ensure!(
                output.len() <= MAX_FRAME,
                "Tool response exceeds frame budget; inspect session"
            );
            stdin.write_all(output.as_bytes()).await?;
            stdin.flush().await?;
        }
        bail!("Worker request budget exhausted")
    };
    let result: Result<()> = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(3600), worker_run) =>
            result.unwrap_or_else(|_| Err(anyhow::anyhow!("Python agent exceeded the one-hour runtime limit"))),
        signal = tokio::signal::ctrl_c() => {
            match signal {
                Ok(()) => Err(anyhow::anyhow!("Python agent cancelled; inspect the recorded session")),
                Err(error) => Err(error.into()),
            }
        }
    };
    executor.shutdown().await?;
    if let Err(error) = &result {
        metadata(
            &session,
            "python-runtime-failure",
            &json!({"error":format!("{error:#}"),"verification":"unrun"}),
        )?;
    }
    result?;
    Ok(session)
}
