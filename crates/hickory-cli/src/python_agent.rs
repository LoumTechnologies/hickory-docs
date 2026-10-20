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
    /// Host-selected storage root; never supplied by a model or worker.
    pub session_storage: Option<PathBuf>,
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
    fn ui_tools(&self) -> Vec<Value> {
        vec![]
    }
    async fn ui_call(&mut self, _name: &str, _arguments: Value) -> Result<Value> {
        bail!("No originating UI window is attached to this agent")
    }
    async fn approve(&mut self, proposal: &Proposal) -> Result<bool>;
    fn context_inbox(&self) -> Option<Arc<std::sync::Mutex<crate::context_updates::Inbox>>> {
        None
    }
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
fn context_delivery(
    inbox: Option<Arc<std::sync::Mutex<crate::context_updates::Inbox>>>,
    request: &hick_context_protocol::Request,
    delivery: &Value,
) {
    if let Some(inbox) = inbox {
        let mut inbox = inbox.lock().unwrap();
        let refs: Vec<String> = inbox
            .0
            .get(&request.branch)
            .map(|b| {
                b.updates
                    .iter()
                    .filter(|u| {
                        request.parts.iter().any(|p| {
                            p.text.as_ref().is_some_and(|text| {
                                text.starts_with(&format!("Context update {}:", u.id))
                            })
                        })
                    })
                    .map(|u| u.id.clone())
                    .collect()
            })
            .unwrap_or_default();
        inbox.delivery(
            &request.branch,
            &request.id,
            delivery["state"].as_str().unwrap_or("uncertain"),
            &refs,
        );
        if let Err(error) = inbox.save() {
            eprintln!("Context update status could not persist: {error}");
        }
    }
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
            | "ui_observe"
            | "ui_act"
            | "ui_cancel"
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
    projection: Option<Value>,
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
    deliveries: std::collections::BTreeMap<String, (String, Option<Value>)>,
    active_delivery: Option<Value>,
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
mod host;

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
    let session_storage = conversation
        .session_storage
        .unwrap_or_else(|| config.root.clone());
    std::fs::create_dir_all(&session_storage)?;
    let session_storage = session_storage.canonicalize()?;
    let session_dir = session_storage.join("sessions");
    std::fs::create_dir_all(&session_dir)?;
    contained(&session_storage, &session_dir)?;
    let session = match conversation.session {
        Some(path) => {
            contained(
                &session_storage,
                path.parent().context("Session has no directory")?,
            )?;
            if path.exists() {
                contained(&session_storage, &path)?;
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
    catalogue["tools"]
        .as_array_mut()
        .unwrap()
        .extend(review.ui_tools());
    if let Some(contract) = conversation.changeset {
        server.agent_set_changeset(contract);
    } else {
        server.agent_begin_changeset()?;
    }
    let mut bootstrap = json!({
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
    let mut bootstrap_references = Vec::new();
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
                if let Some(projection) = &editor.projection {
                    if let Some(documents) = projection["documents"].as_array() {
                        for document in documents.iter().filter(|d| d["id"] == identity) {
                            if let Some(segments) = document["segments"].as_array() {
                                for segment in segments.iter().filter(|s| s["omitted"] == false) {
                                    let start =
                                        segment["start"].as_u64().context("projection start")?
                                            as usize;
                                    let end =
                                        segment["end"].as_u64().context("projection end")? as usize;
                                    let text = buffer
                                        .content
                                        .get(start..end)
                                        .context("invalid projection range")?;
                                    ensure!(
                                        segment["text"].as_str() == Some(text),
                                        "projection bytes differ"
                                    );
                                    let source =
                                        context_store.editor(&identity, &buffer.content)?;
                                    let nodes = context_store.store.import_chunks_anchored(
                                        source,
                                        hick_context_store::span(start, end),
                                        text,
                                        &hick_context_store::fresh_id(),
                                        hick_context_store::unique_passage(&buffer.content, text),
                                    )?;
                                    bootstrap_references.extend(nodes.into_iter().map(|n| n.id));
                                }
                            }
                        }
                    }
                } else {
                    bootstrap_references.extend(
                        context_store
                            .snapshots(
                                "editor_buffer",
                                &identity,
                                &buffer.content,
                                &hick_context_store::fresh_id(),
                            )?
                            .into_iter()
                            .map(|node| node.id),
                    );
                }
            }
        }
    }
    bootstrap["context_references"] = json!(bootstrap_references);
    let mut host = Host {
        config: &config,
        client,
        review,
        server,
        context_store,
        deliveries: Default::default(),
        active_delivery: None,
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
    if let Some(mut delivery) = host.active_delivery.take() {
        delivery["state"] = json!("cancelled_uncertain");
        metadata(&session, "python-context-delivery", &delivery)?;
        if let Some(inbox) = host.review.context_inbox() {
            let mut inbox = inbox.lock().unwrap();
            for branch in inbox.0.values_mut() {
                for update in &mut branch.updates {
                    if update.request.as_deref() == delivery["request"].as_str() {
                        update.state = "cancelled_uncertain".into();
                    }
                }
            }
        }
    }
    if let Some(inbox) = host.review.context_inbox() {
        let mut inbox = inbox.lock().unwrap();
        if let Some(branch) = host.turn_id.as_ref().and_then(|id| inbox.0.get_mut(id)) {
            for update in &mut branch.updates {
                if matches!(update.state.as_str(), "preparing" | "prepared") {
                    update.state = "failed".into();
                }
            }
        }
        inbox.save()?;
    }
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

#[cfg(test)]
mod observation_capability_tests {
    // Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
    #[test]
    fn receiving_debugger_evidence_does_not_grant_python_debugger_capabilities() {
        for name in [
            "debug_start",
            "debug_state",
            "debug_eval",
            "debug_step",
            "located",
        ] {
            assert!(
                !super::allowed(name),
                "reading evidence must not authorize {name}"
            );
        }
    }
}
