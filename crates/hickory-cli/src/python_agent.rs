//! Confined Python process supervisor and capability boundary. Python owns agent policy.
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use hickory_agent::{
    AgentEvent, HickSessionLog, LlmClient, Message, PriorTurn, SessionEvent, SessionLog, Usage,
};
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};

use crate::mcp::Server;
const MAX_FRAME: usize = 1_048_576;
pub struct Config {
    pub python: PathBuf,
    pub root: PathBuf,
    pub doc: Option<PathBuf>,
    pub prompt: String,
    pub max_turns: usize,
}

/// Optional conversation identity supplied by the UI. Standalone runs use defaults.
#[derive(Default)]
pub struct Conversation {
    pub session: Option<PathBuf>,
    pub subject: Option<PathBuf>,
    pub turn: Option<String>,
    pub parent: Option<String>,
    pub provider: String,
    pub context: String,
    pub prior_turns: Vec<PriorTurn>,
    pub cancel: Option<Arc<AtomicBool>>,
    pub changeset: Option<crate::changeset::Contract>,
}
pub struct Outcome {
    pub session: PathBuf,
    pub summary: String,
    pub usage: Usage,
    pub artifact: crate::changeset::Artifact,
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
    fn replacement(&self) -> Option<String> {
        None
    }
    async fn refresh(&mut self) -> Result<()> {
        Ok(())
    }
    /// Current host editor observations, separate from model inclusion and edit snapshots.
    async fn editor_heads(&mut self) -> Result<Option<Vec<EditorHead>>> {
        Ok(None)
    }
    async fn completed(&mut self, _ok: bool) -> Result<()> {
        Ok(())
    }
}

pub struct EditorHead {
    pub identity: String,
    pub content: String,
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
            | "changeset_status"
            | "submit_changeset"
    ) || hickory_agent::audit::is_tool(name)
}
#[derive(serde::Deserialize)]
struct HostEditorContext {
    buffers: Vec<HostEditorBuffer>,
}
#[derive(serde::Deserialize)]
struct HostEditorBuffer {
    id: Option<String>,
    provenance_uri: Option<String>,
    path: Option<String>,
    content: String,
}
struct Host<'a> {
    config: &'a Config,
    client: &'a dyn LlmClient,
    review: &'a mut dyn Review,
    server: Server,
    context_store: hick_context_store::host::Service,
    session: PathBuf,
    turns: usize,
    tools: usize,
    turn_id: Option<String>,
    finished: bool,
    unchecked_edits: bool,
    summary: String,
    usage: Usage,
    on_event: &'a mut (dyn FnMut(AgentEvent) + Send),
}
impl Host<'_> {
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
        if matches!(name, "changeset_status" | "submit_changeset") {
            self.review.refresh().await?;
        }
        let mut args = params.get("arguments").cloned().unwrap_or(json!({}));
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
        let mut created = None;
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
            if let Some(replacement) = self.review.replacement() {
                if name == "create_doc" {
                    args["input"] = json!(replacement);
                } else {
                    args["input"] = json!(replacement);
                    args["whole"] = json!("true");
                    args["snapshot"] =
                        json!(hickory_agent::hashline::LineIndex::new(&proposal.before).snapshot);
                }
                self.proposal(name, &args).await?;
                metadata(
                    &self.session,
                    "python-reviewed-replacement",
                    &json!({"actor":"user-declared","arguments":args}),
                )?;
            }
            self.unchecked_edits = true;
            if name == "create_doc" {
                let effective = self.proposal(name, &args).await?;
                created = Some((effective.surface, effective.after));
            }
        }
        let mut result = self
            .server
            .handle("tools/call", &json!({"name":name,"arguments":args}))
            .await
            .map_err(|(_, text)| anyhow::anyhow!(text))?;
        {
            let mut references = Vec::new();
            for read in self.server.take_context_reads() {
                let text = &read.content;
                let path = read
                    .source_path
                    .as_deref()
                    .unwrap_or_else(|| Path::new(&read.path));
                let absolute = if path.is_absolute() {
                    path.to_path_buf()
                } else {
                    self.config.root.join(path)
                };
                let locator = absolute.strip_prefix(&self.config.root)?.to_string_lossy();
                let kind = if name == "read_output" {
                    "generated_output"
                } else {
                    "saved_file"
                };
                let mut source = self.context_store.store.register(kind, &locator)?;
                source.revision = read.sha256;
                let start = read.byte_span.0;
                let end = read.byte_span.1;
                let nodes = self.context_store.store.import_chunks_anchored(
                    source,
                    hick_context_store::span(start, end),
                    text,
                    &hick_context_store::fresh_id(),
                    read.unique,
                )?;
                references.extend(nodes.into_iter().map(|node| node.id));
            }
            result["context_references"] = json!(references);
        }
        if result["isError"] != true
            && let Some((file, expected)) = created
        {
            let content = std::fs::read_to_string(contained(
                &self.config.root,
                &self.config.root.join(&file),
            )?)?;
            ensure!(
                content == expected,
                "Created document changed before its write receipt; actual state remains captured"
            );
            let index = hickory_agent::hashline::LineIndex::new(&content);
            let wrote = hickory_agent::Wrote {
                file,
                first_line: 1,
                last_line: index.lines.len(),
                hashes: index.hashes,
                fingerprints: index
                    .lines
                    .iter()
                    .map(|line| hickory_agent::hashline::line_fingerprint(line))
                    .collect(),
            };
            append(&self.session, SessionEvent::Wrote { wrote: &wrote })?;
        }
        Ok(result)
    }
    async fn handle(&mut self, method: &str, params: &Value) -> Result<Value> {
        ensure!(!self.finished, "The model already stopped");
        if method.starts_with("context.") {
            if matches!(
                method,
                "context.hello" | "context.capture" | "context.changes" | "context.reconcile"
            ) {
                self.context_store.editors.0.clear();
                // A missing/timed-out editor feed cannot certify unchanged. File heads
                // remain independently checkable, and old editor captures remain intact.
                if let Ok(Some(heads)) = self.review.editor_heads().await {
                    let sources = self.context_store.store.sources();
                    for head in heads {
                        if let Some(source) = sources
                            .iter()
                            .find(|s| s.kind == "editor_buffer" && s.locator == head.identity)
                        {
                            self.context_store
                                .editors
                                .0
                                .insert(source.id.clone(), head.content);
                        }
                    }
                }
            }
            return self.context_store.call(method, params);
        }
        match method {
            "inference" => {
                // Transport/resource limits only. Never add prompts, replay history,
                // retry a provider request, parse an action, or choose the next step.
                ensure!(self.turns < 600, "Inference resource budget exhausted");
                let messages: Vec<Message> = serde_json::from_value(params["messages"].clone())?;
                ensure!(
                    serde_json::to_vec(&messages)?.len() <= 800_000,
                    "Inference request exceeds transport budget"
                );
                metadata(
                    &self.session,
                    "python-model-request",
                    &json!({"messages":messages,"backend":"python","turn":self.turn_id,
                    "iteration":self.turns,"model":self.client.model_name()}),
                )?;
                (self.on_event)(AgentEvent::Thinking);
                use futures::StreamExt;
                let request = async {
                    let mut stream = self.client.complete_stream(messages).await?;
                    let mut raw = String::new();
                    let mut usage = Usage::default();
                    let mut reasoning = String::new();
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        raw.push_str(&chunk.delta);
                        ensure!(
                            raw.len() <= MAX_FRAME / 2,
                            "Model response exceeds protocol budget"
                        );
                        if !chunk.reasoning.is_empty() {
                            reasoning.push_str(&chunk.reasoning);
                            ensure!(
                                reasoning.len() <= MAX_FRAME / 2,
                                "Reasoning exceeds protocol budget"
                            );
                            (self.on_event)(AgentEvent::Reasoning {
                                data: chunk.reasoning,
                            });
                        }
                        if !chunk.delta.is_empty() {
                            (self.on_event)(AgentEvent::Token { data: chunk.delta });
                        }
                        if let Some(value) = chunk.usage {
                            usage.add(&value);
                        }
                    }
                    Ok::<_, anyhow::Error>((raw, usage, reasoning))
                };
                let (raw, usage, reasoning) =
                    tokio::time::timeout(Duration::from_secs(180), request).await??;
                metadata(
                    &self.session,
                    "python-model-response",
                    &json!({"actor":"model-declared","raw":raw,"reasoning":reasoning}),
                )?;
                if !reasoning.is_empty() {
                    append(
                        &self.session,
                        SessionEvent::Assistant {
                            prose: "",
                            action: None,
                            reasoning: Some(&reasoning),
                        },
                    )?;
                }
                self.usage.add(&usage);
                (self.on_event)(AgentEvent::TurnUsage {
                    turn: self.turns,
                    usage,
                    cost_usd: hickory_agent::cost_usd(self.client.model_name(), &usage),
                    total_usage: self.usage,
                    total_cost_usd: hickory_agent::cost_usd(self.client.model_name(), &self.usage),
                });
                append(
                    &self.session,
                    SessionEvent::Usage {
                        turn: Some(self.turns),
                        usage,
                        cost_usd: None,
                    },
                )?;
                self.turns += 1;
                Ok(Value::String(raw))
            }
            "record" => {
                let kind = params["kind"].as_str().context("Missing record kind")?;
                ensure!(
                    matches!(kind, "action" | "observation" | "model-error"),
                    "Unknown worker record kind"
                );
                metadata(
                    &self.session,
                    &format!("python-{kind}"),
                    &json!({"actor":"worker-reported","value":params["data"]}),
                )?;
                Ok(json!({}))
            }
            "tool" => {
                let name = params["name"].as_str().unwrap_or("unknown").to_string();
                (self.on_event)(AgentEvent::ToolStarted {
                    name: name.clone(),
                    data: params.to_string(),
                });
                let mut result = self.tool(params).await;
                let ok = result.as_ref().is_ok_and(|value| value["isError"] != true);
                if let Err(error) = self.review.completed(ok).await {
                    result = Err(error);
                }
                metadata(
                    &self.session,
                    "python-tool-result",
                    &json!({
                        "actor":"host-observed", "name":name,
                        "result":result.as_ref().ok(), "error":result.as_ref().err().map(|e| format!("{e:#}"))
                    }),
                )?;
                (self.on_event)(AgentEvent::ToolFinished {
                    name,
                    ok: ok && result.is_ok(),
                    text: match &result {
                        Ok(value) => value.to_string(),
                        Err(error) => format!("{error:#}"),
                    },
                });
                result
            }
            "complete" => {
                let summary = params["summary"]
                    .as_str()
                    .context("Missing completion summary")?;
                self.review.refresh().await?;
                // Independent byte verification, never a model assertion.
                let artifact = self.server.agent_completion()?;
                metadata(
                    &self.session,
                    "python-verified-changeset",
                    &serde_json::to_value(artifact)?,
                )?;
                metadata(
                    &self.session,
                    "python-completion",
                    &json!({"actor":"harness","model_summary":summary,"model_stopped":true,"verification":"source-output-equality-only","runtime_checks":"unrun","unchecked_edits":self.unchecked_edits,"assumption_coverage":"Not established"}),
                )?;
                self.summary = format!(
                    "Changeset verified (source-output equality); runtime checks unrun. Model summary:\n\n{summary}"
                );
                append(
                    &self.session,
                    SessionEvent::Assistant {
                        prose: &self.summary,
                        action: None,
                        reasoning: None,
                    },
                )?;
                (self.on_event)(AgentEvent::Token {
                    data: self.summary.clone(),
                });
                (self.on_event)(AgentEvent::ResponseComplete {
                    text: self.summary.clone(),
                });
                (self.on_event)(AgentEvent::Done {
                    summary: self.summary.clone(),
                });
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
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    let signal = tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            flag.store(true, Ordering::Relaxed);
        }
    });
    let result = run_conversation(
        config,
        client,
        review,
        Conversation {
            cancel: Some(cancel),
            ..Default::default()
        },
        &mut |_| {},
    )
    .await;
    signal.abort();
    let outcome = result.map_err(|error| {
        if error.to_string() == hickory_agent::STOPPED_BY_USER {
            anyhow::anyhow!("Python agent cancelled; inspect the recorded session")
        } else {
            error
        }
    })?;
    println!("{}", outcome.summary);
    Ok(outcome.session)
}

pub async fn run_conversation(
    config: Config,
    client: &dyn LlmClient,
    review: &mut dyn Review,
    conversation: Conversation,
    on_event: &mut (dyn FnMut(AgentEvent) + Send),
) -> Result<Outcome> {
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
    let session = match conversation.session {
        Some(path) => {
            contained(
                &config.root,
                path.parent().context("Session has no directory")?,
            )?;
            if path.exists() {
                contained(&config.root, &path)?;
            }
            path
        }
        None => {
            tempfile::Builder::new()
                .prefix("python-agent-")
                .suffix(".md")
                .tempfile_in(&session_dir)?
                .keep()?
                .1
        }
    };
    let initial = if session.metadata().is_ok_and(|m| m.len() > 0) {
        HickSessionLog::append_or_create(&session)?
    } else {
        HickSessionLog::create_for(
            &session,
            conversation.subject.as_deref().or(config.doc.as_deref()),
        )?
    };
    if let Some(turn) = &conversation.turn {
        initial.record(SessionEvent::UserTurn {
            text: &config.prompt,
            turn,
            parent: conversation.parent.as_deref(),
            provider: &conversation.provider,
            model: client.model_name(),
        });
    } else {
        initial.record(SessionEvent::User {
            text: &config.prompt,
        });
    }
    initial.record(SessionEvent::Context { kind: "python-runtime", data: &json!({"backend":"python","turn":conversation.turn,"context":conversation.context,"prior_turns":conversation.prior_turns.iter().map(|t| json!({"prompt":t.prompt,"answer":t.answer})).collect::<Vec<_>>()}) });
    if !conversation.context.is_empty() {
        initial.record(SessionEvent::EditorContext {
            text: &conversation.context,
        });
    }
    initial.record(SessionEvent::End);
    initial.check()?;
    on_event(AgentEvent::SessionStarted {
        model: client.model_name().into(),
        session_path: session.display().to_string(),
    });
    on_event(AgentEvent::UserMessage {
        text: config.prompt.clone(),
    });
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
    if let Some(contract) = conversation.changeset {
        server.agent_set_changeset(contract);
    } else {
        server.agent_begin_changeset()?;
    }
    let bootstrap = json!({
        "prompt":config.prompt,"max_turns":config.max_turns,"context":conversation.context,
        "document":config.doc.as_ref().map(|doc| doc.strip_prefix(&config.root).unwrap_or(doc).display().to_string()),
        "prior_turns":conversation.prior_turns.iter().map(|t| json!({"prompt":t.prompt,"answer":t.answer})).collect::<Vec<_>>(),
        "tools":catalogue
    });
    // A session owns its references, separately from the append-only agent narrative.
    let mut context_store = hick_context_store::host::Service::open(
        &config.root,
        &session.with_extension("context.json"),
        conversation.turn.as_deref().unwrap_or("standalone"),
    )?;
    if let Some(doc) = &config.doc
        && let Ok(locator) = doc.strip_prefix(&config.root)
        && let Some(locator) = locator.to_str()
    {
        context_store.saved(locator)?;
    }
    if let Some(line) = conversation.context.lines().last()
        && let Ok(value) = serde_json::from_str::<Value>(line)
    {
        let value = value.get("editors").unwrap_or(&value);
        if let Ok(editor) = serde_json::from_value::<HostEditorContext>(value.clone()) {
            for buffer in editor.buffers {
                let identity = buffer
                    .id
                    .as_deref()
                    .or(buffer.provenance_uri.as_deref())
                    .or(buffer.path.as_deref())
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("snapshot:{}", hick_context_store::fresh_id()));
                context_store.editor(&identity, &buffer.content)?;
                context_store.snapshots(
                    "editor_buffer",
                    &identity,
                    &buffer.content,
                    &hick_context_store::fresh_id(),
                )?;
            }
        }
    }
    let mut host = Host {
        config: &config,
        client,
        review,
        server,
        context_store,
        session: session.clone(),
        turns: 0,
        tools: 0,
        turn_id: conversation.turn.clone(),
        finished: false,
        unchecked_edits: false,
        summary: String::new(),
        usage: Usage::default(),
        on_event,
    };
    let worker_run = async {
        let child = cmd.spawn().context("Starting confined Python agent")?;
        let pid = child.id().context("Worker has no process id")?;
        let mut guard = ChildGuard { child, pid };
        let mut stdin = guard.child.stdin.take().context("No worker input pipe")?;
        let mut stdout =
            BufReader::new(guard.child.stdout.take().context("No worker output pipe")?);
        stdin
            .write_all(format!("{}\n", bootstrap).as_bytes())
            .await?;
        let mut sequence = 0u64;
        for _ in 0..50_000 {
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
                    "Worker exited without model completion: {}",
                    request["params"]["error"]
                        .as_str()
                        .unwrap_or("No current accepted changeset")
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
                    if method.starts_with("context.") {
                        json!({"id":sequence,"error":hick_context_store::host::protocol_error(&error)})
                    } else {
                        json!({"id":sequence,"error":format!("{error:#}")})
                    }
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
        _ = async {
            loop {
                if conversation.cancel.as_ref().is_some_and(|flag| flag.load(Ordering::Relaxed)) { break; }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        } => Err(anyhow::anyhow!(hickory_agent::STOPPED_BY_USER)),
    };
    executor.shutdown().await?;
    if let Err(error) = &result {
        metadata(
            &session,
            "python-runtime-failure",
            &json!({"error":format!("{error:#}"),"cancellation":if error.to_string() == hickory_agent::STOPPED_BY_USER { Some("Python agent cancelled") } else { None },"verification":"unrun"}),
        )?;
    }
    result?;
    let artifact = host.server.agent_completion()?;
    Ok(Outcome {
        artifact,
        session,
        summary: host.summary,
        usage: host.usage,
    })
}

/// Check the installed worker in the same confinement used for real turns, without a model call.
pub async fn probe(python: &Path) -> Result<()> {
    let scratch = tempfile::tempdir()?;
    let mut command = tokio::process::Command::from(
        hickory_executor_sandbox::agent_process::command(python, scratch.path())?,
    );
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true);
    let child = command.spawn()?;
    let pid = child.id().context("Worker has no process id")?;
    let mut guard = ChildGuard { child, pid };
    let mut input = guard.child.stdin.take().context("No worker input pipe")?;
    let mut output = BufReader::new(guard.child.stdout.take().context("No worker output pipe")?);
    input.write_all(b"{\"probe\":true}\n").await?;
    let mut frame = Vec::new();
    let read = tokio::time::timeout(
        Duration::from_secs(10),
        (&mut output)
            .take((MAX_FRAME + 1) as u64)
            .read_until(b'\n', &mut frame),
    )
    .await??;
    ensure!(
        read > 0 && read <= MAX_FRAME,
        "Worker did not start; install the locked hickory-agent-worker package into this virtual environment"
    );
    let request: Value = serde_json::from_slice(&frame)?;
    ensure!(
        request["method"] == "ready",
        "Unexpected worker startup protocol"
    );
    Ok(())
}
