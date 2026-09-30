use super::super::LocalState;
use super::{config::AgentCommand, mcp, record::Record, transport::Rpc};
use anyhow::{Context, Result, bail};
use hickory_agent::SessionEvent;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tokio::sync::{Mutex as AsyncMutex, oneshot};

pub struct PendingPermission {
    pub view: Value,
    pub reply: oneshot::Sender<Value>,
}

#[derive(Default)]
pub struct Active {
    pub turn: String,
    pub answer: String,
    pub reasoning: String,
    pub tools: HashMap<String, Value>,
    pub tool_order: Vec<String>,
    pub message_id: Option<String>,
}

pub struct Client {
    pub agent: String,
    pub rpc: Arc<Rpc>,
    pub initialized: Value,
    pub session: Mutex<Option<String>>,
    pub view: Mutex<Value>,
    pub record: Arc<AsyncMutex<Record>>,
    workspace_gate: Arc<AsyncMutex<()>>,
    file_reads: Mutex<HashMap<String, super::workspace::Snapshot>>,
    pub active: Mutex<Option<Active>>,
    pub permissions: Mutex<HashMap<String, PendingPermission>>,
    pub operation: AsyncMutex<()>,
    pub closed: std::sync::atomic::AtomicBool,
    pub last_turn: Mutex<Option<String>>,
    worker: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub host: mcp::Host,
    pub filesystem: Option<super::super::workspace_fs::Mount>,
    pub context: RoomContext,
    pub doc_id: String,
}

#[derive(Clone)]
pub struct RoomContext {
    pub index: Arc<super::super::store::DocIndex>,
    pub rooms: Arc<hickory_collab::RoomRegistry>,
}
impl RoomContext {
    pub async fn publish(&self, doc: &str, turn: &str, event: Value) {
        self.rooms
            .publish_run_event(
                doc,
                &json!({"run_id": turn,"exec_id":"agent","event":event}),
            )
            .await;
    }
}

impl Client {
    fn cwd(&self) -> &std::path::Path {
        self.filesystem
            .as_ref()
            .map(|m| m.path.as_path())
            .unwrap_or_else(|| self.context.index.root())
    }
    pub async fn start(
        state: &LocalState,
        doc_id: &str,
        agent: AgentCommand,
        path: PathBuf,
    ) -> Result<Arc<Self>> {
        let mut command = super::config::process(&agent, state)?;
        let doc = state
            .index
            .absolute(doc_id)
            .context("document is no longer open")?;
        let record = Arc::new(AsyncMutex::new(Record::open(path, &doc)?));
        let workspace_gate = Arc::new(AsyncMutex::new(()));
        let host =
            mcp::Host::start(state.clone(), doc, record.clone(), workspace_gate.clone()).await?;
        let filesystem = if agent.workspace_filesystem {
            let engine = super::super::workspace_fs::Engine::new(
                state.clone(),
                record.clone(),
                workspace_gate.clone(),
            );
            let filesystem_host = super::super::workspace_fs::Host::start(engine).await?;
            let mount = super::super::workspace_fs::Mount::start(filesystem_host).await?;
            command.current_dir(&mount.path);
            Some(mount)
        } else {
            None
        };
        let (rpc, mut events) = Rpc::spawn(command)?;
        // Do not advertise terminal support: harness commands use the harness's
        // execution boundary, rather than being silently redirected into ours.
        let initialized = rpc.request("initialize", json!({"protocolVersion":1,
            "clientInfo":{"name":"hickory-docs","title":"Hickory Docs","version":env!("CARGO_PKG_VERSION")},
            "clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"session":{"configOptions":{"boolean":{}}}}
        }), Duration::from_secs(30)).await?;
        if initialized["protocolVersion"] != 1 {
            rpc.kill().await;
            bail!(
                "{} requires an unsupported ACP version. Update the adapter or Hickory Docs.",
                agent.name
            );
        }
        let client = Arc::new(Self {
            agent: agent.id,
            rpc,
            initialized,
            session: Mutex::new(None),
            view: Mutex::new(json!({})),
            record,
            workspace_gate,
            file_reads: Mutex::new(HashMap::new()),
            active: Mutex::new(None),
            permissions: Mutex::new(HashMap::new()),
            operation: AsyncMutex::new(()),
            closed: false.into(),
            last_turn: Mutex::new(None),
            worker: Mutex::new(None),
            host,
            filesystem,
            context: RoomContext {
                index: state.index.clone(),
                rooms: state.rooms.clone(),
            },
            doc_id: doc_id.into(),
        });
        let weak: Weak<Self> = Arc::downgrade(&client);
        let worker = tokio::spawn(async move {
            while let Some(event) = events.recv().await {
                let Some(client) = weak.upgrade() else {
                    break;
                };
                if event.get("method").is_some() && event.get("id").is_some() {
                    // Permissions may wait for a person while more updates arrive.
                    tokio::spawn(async move {
                        client.respond(event).await;
                    });
                } else {
                    client.update(event).await;
                }
            }
        });
        *client.worker.lock().unwrap() = Some(worker);
        client
            .rpc
            .ordered
            .store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(client)
    }

    pub async fn setup(&self) -> Result<()> {
        if self.session.lock().unwrap().is_some() {
            return Ok(());
        }
        let path = self.record.lock().await.path.clone();
        let saved = super::record::metadata(&path);
        let mcp = mcp::configuration(
            &self.host,
            self.initialized["agentCapabilities"]["mcpCapabilities"]["http"] == true,
        )?;
        let mut params = json!({"cwd":self.cwd().display().to_string(), "mcpServers":[mcp]});
        let method = if let Some(saved) = saved.filter(|v| v["agent"] == self.agent) {
            if self.initialized["agentCapabilities"]["loadSession"] != true {
                bail!(
                    "This agent cannot resume its saved session. Start a new thread; the existing record remains readable."
                );
            }
            params["sessionId"] = saved["sessionId"].clone();
            "session/load"
        } else {
            "session/new"
        };
        let result = self
            .rpc
            .request(method, params.clone(), Duration::from_secs(60))
            .await?;
        let session = result["sessionId"]
            .as_str()
            .or_else(|| params["sessionId"].as_str())
            .context("the agent did not return a session id")?
            .to_string();
        *self.session.lock().unwrap() = Some(session.clone());
        if let Some(fields) = result.as_object() {
            let mut view = self.view.lock().unwrap();
            for (key, value) in fields {
                view[key] = value.clone();
            }
        }
        self.record.lock().await.session(&self.agent, &session)?;
        Ok(())
    }

    pub fn snapshot(&self) -> Value {
        let mut view = self.view.lock().unwrap().clone();
        view["backend"] = json!(self.agent);
        view["canRewind"] = json!(self.can_rewind());
        view["ready"] = json!(
            self.session.lock().unwrap().is_some()
                && !self.closed.load(std::sync::atomic::Ordering::Relaxed)
        );
        view["authMethods"] = self.initialized["authMethods"].clone();
        view["permissions"] = json!(
            self.permissions
                .lock()
                .unwrap()
                .values()
                .map(|p| p.view.clone())
                .collect::<Vec<_>>()
        );
        view["tools"] = json!(
            self.active
                .lock()
                .unwrap()
                .as_ref()
                .map(|a| a
                    .tool_order
                    .iter()
                    .filter_map(|id| a.tools.get(id).cloned())
                    .collect::<Vec<_>>())
                .unwrap_or_default()
        );
        view
    }

    pub async fn prompt(
        &self,
        turn: &str,
        parent: Option<&str>,
        prompt: &str,
        cancel: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<String> {
        let _operation = self.operation.lock().await;
        self.branch(parent).await?;
        let session = self
            .session
            .lock()
            .unwrap()
            .clone()
            .context("Connect and sign in to the agent before sending a message.")?;
        let model = self.model();
        self.record.lock().await.event(SessionEvent::UserTurn {
            text: prompt,
            turn,
            parent,
            provider: &format!("acp:{}", self.agent),
            model: &model,
        })?;
        if let Some(filesystem) = &self.filesystem {
            filesystem
                .exchange(json!({"op":"turn","turn":turn}))
                .await?;
        }
        *self.active.lock().unwrap() = Some(Active {
            turn: turn.into(),
            ..Default::default()
        });
        let call = self.rpc.request(
            "session/prompt",
            json!({"sessionId":session,"prompt":[{"type":"text","text":prompt}]}),
            Duration::from_secs(24 * 60 * 60),
        );
        tokio::pin!(call);
        let mut pulse = tokio::time::interval(Duration::from_millis(100));
        let mut stopped_at = None;
        let outcome = loop {
            tokio::select! {
                result = &mut call => break result,
                _ = pulse.tick() => {
                    if cancel.load(std::sync::atomic::Ordering::Relaxed) && stopped_at.is_none() {
                        stopped_at = Some(std::time::Instant::now());
                        self.cancel_permissions();
                        let _ = self.rpc.notify("session/cancel", json!({"sessionId":session})).await;
                    }
                    if stopped_at.is_some_and(|at| at.elapsed() >= Duration::from_secs(3)) {
                        self.rpc.kill().await; self.closed.store(true, std::sync::atomic::Ordering::Relaxed);
                        break Err(anyhow::anyhow!(hickory_agent::STOPPED_BY_USER));
                    }
                }
            }
        };
        // RPC responses pass through the same queue as updates, so the last
        // token is recorded before the prompt's response can complete.
        self.cancel_permissions();
        let active = self.active.lock().unwrap().take().unwrap_or_default();
        let record = self.record.lock().await;
        if !active.answer.is_empty() || !active.reasoning.is_empty() {
            record.event(SessionEvent::Assistant {
                prose: &active.answer,
                action: None,
                reasoning: (!active.reasoning.is_empty()).then_some(active.reasoning.as_str()),
            })?;
            record.compact_stream(turn)?;
        }
        let stopped = stopped_at.is_some()
            || outcome
                .as_ref()
                .is_ok_and(|r| r["stopReason"] == "cancelled");
        let error = outcome.as_ref().err().map(|e| format!("{e:#}"));
        record.context("acp-turn-status", &json!({"turn":turn,"status":if stopped {"stopped"} else if error.is_some() {"error"} else {"ok"},"error":error,"sessionId":session,"messageId":active.message_id}))?;
        *self.last_turn.lock().unwrap() = Some(turn.into());
        drop(record);
        if stopped {
            bail!(hickory_agent::STOPPED_BY_USER);
        }
        outcome?;
        Ok(active.answer)
    }

    /// Exact rewind requires the Codex adapter's documented AIR fork point.
    /// A generic ACP fork only copies the latest context, which is not rewind.
    pub fn can_rewind(&self) -> bool {
        self.initialized["agentInfo"]["name"]
            .as_str()
            .is_some_and(|name| name.contains("codex-acp"))
            && self.initialized["agentCapabilities"]["sessionCapabilities"]
                .get("fork")
                .is_some()
    }

    async fn branch(&self, parent: Option<&str>) -> Result<()> {
        if parent == self.last_turn.lock().unwrap().as_deref() {
            return Ok(());
        }
        anyhow::ensure!(
            self.can_rewind(),
            "This adapter cannot rewind. Start a new thread instead."
        );
        let parent = parent.context("Start a new thread to return to the beginning.")?;
        let path = self.record.lock().await.path.clone();
        let point = super::record::status(&path, parent)
            .context("This turn has no recorded fork point.")?;
        let message = point["messageId"]
            .as_str()
            .context("The adapter did not report a message id for this turn.")?;
        let source_session = point["sessionId"]
            .as_str()
            .context("This turn has no recorded agent session.")?;
        let mcp = mcp::configuration(
            &self.host,
            self.initialized["agentCapabilities"]["mcpCapabilities"]["http"] == true,
        )?;
        let result = self
            .rpc
            .request(
                "session/fork",
                json!({"sessionId":source_session,
                    "cwd":self.cwd().display().to_string(), "mcpServers":[mcp],
                    "_meta":{"jetbrains":{"air":{"fork":{"version":1,"messageId":message}}}}
                }),
                Duration::from_secs(60),
            )
            .await?;
        let session = result["sessionId"]
            .as_str()
            .context("The adapter did not return the forked session id.")?;
        *self.session.lock().unwrap() = Some(session.into());
        // ACP forks create an inactive session. Resume it before prompting so
        // the harness subscribes to turn updates and can complete the request.
        let method = if self.initialized["agentCapabilities"]["sessionCapabilities"]
            .get("resume")
            .is_some()
        {
            "session/resume"
        } else {
            "session/load"
        };
        let result = self
            .rpc
            .request(
                method,
                json!({"sessionId":session,
                    "cwd":self.cwd().display().to_string(),"mcpServers":[mcp]
                }),
                Duration::from_secs(60),
            )
            .await?;
        if let Some(fields) = result.as_object() {
            let mut view = self.view.lock().unwrap();
            for (key, value) in fields {
                view[key] = value.clone();
            }
        }
        self.record.lock().await.session(&self.agent, session)?;
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.closed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.cancel_permissions();
        self.rpc.kill().await;
    }

    pub fn model(&self) -> String {
        let view = self.view.lock().unwrap();
        view["configOptions"]
            .as_array()
            .and_then(|options| {
                options
                    .iter()
                    .find(|o| o["category"] == "model" || o["id"] == "model")
            })
            .and_then(|o| o["currentValue"].as_str())
            .or_else(|| view["models"]["currentModelId"].as_str())
            .unwrap_or("agent default")
            .to_string()
    }

    fn cancel_permissions(&self) {
        for (_, pending) in self.permissions.lock().unwrap().drain() {
            let _ = pending
                .reply
                .send(json!({"outcome":{"outcome":"cancelled"}}));
        }
    }

    async fn update(&self, message: Value) {
        if message.get("method").is_none() && message.get("id").is_some() {
            self.rpc.deliver(&message);
            return;
        }
        if message.get("transportClosed").is_some() || message.get("transportError").is_some() {
            self.closed
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.cancel_permissions();
            self.rpc.disconnected();
            return;
        }
        if message["method"] != "session/update" {
            return;
        }
        let update = &message["params"]["update"];
        let remote = self.session.lock().unwrap().clone();
        if remote
            .as_ref()
            .is_some_and(|s| message["params"]["sessionId"] != *s)
        {
            return;
        }
        let kind = update["sessionUpdate"].as_str().unwrap_or("");
        let mut text = update["content"]["text"].as_str().unwrap_or("").to_string();
        match kind {
            "config_option_update" => {
                self.view.lock().unwrap()["configOptions"] = update["configOptions"].clone()
            }
            "available_commands_update" => {
                self.view.lock().unwrap()["commands"] = update["availableCommands"].clone()
            }
            "current_mode_update" => {
                self.view.lock().unwrap()["modes"]["currentModeId"] =
                    update["currentModeId"].clone()
            }
            _ => {}
        }
        let turn = {
            let mut active = self.active.lock().unwrap();
            let Some(active) = active.as_mut() else {
                return;
            }; // replay is not new evidence
            match kind {
                "agent_message_chunk" => {
                    if let Some(id) = update["messageId"].as_str()
                        && active
                            .message_id
                            .as_deref()
                            .is_some_and(|previous| previous != id)
                        && !active.answer.is_empty()
                    {
                        text.insert_str(0, "\n\n");
                    }
                    active.answer.push_str(&text);
                    if let Some(id) = update["messageId"].as_str() {
                        active.message_id = Some(id.into());
                    }
                }
                "agent_thought_chunk" => active.reasoning.push_str(&text),
                "tool_call" | "tool_call_update" => {
                    let id = update["toolCallId"].as_str().unwrap_or("tool").to_string();
                    if !active.tools.contains_key(&id) {
                        active.tool_order.push(id.clone());
                    }
                    let tool = active.tools.entry(id).or_insert(json!({}));
                    if let Some(fields) = update.as_object() {
                        for (k, v) in fields {
                            if !v.is_null() {
                                tool[k] = v.clone();
                            }
                        }
                    }
                }
                _ => {}
            }
            active.turn.clone()
        };
        if matches!(kind, "agent_message_chunk" | "agent_thought_chunk") && !text.is_empty() {
            let _ = self
                .record
                .lock()
                .await
                .context("acp-stream", &json!({"turn":turn,"kind":kind,"text":text}));
        }
        if !matches!(kind, "agent_message_chunk" | "agent_thought_chunk")
            && let Err(e) = self.record.lock().await.context("acp-activity", update)
        {
            self.context.publish(&self.doc_id, &turn, json!({"kind":"error","message":format!("Could not save agent activity: {e:#}")})).await;
        }
        let event = match kind {
            "agent_message_chunk" => json!({"kind":"token", "data":text}),
            "agent_thought_chunk" => json!({"kind":"reasoning", "data":text}),
            _ => json!({"kind":"acp_update", "update":update}),
        };
        self.context.publish(&self.doc_id, &turn, event).await;
    }

    async fn respond(&self, message: Value) {
        let id = message["id"].clone();
        let method = message["method"].as_str().unwrap_or("");
        let params = &message["params"];
        let result: Result<Value> = async {
            if params["sessionId"].as_str() != self.session.lock().unwrap().as_deref() {
                bail!("unknown ACP session");
            }
            match method {
                "session/request_permission" => {
                    let key = format!("{:016x}", super::super::rand_id());
                    let (tx, rx) = oneshot::channel();
                    let view =
                        json!({"id":key,"toolCall":params["toolCall"],"options":params["options"]});
                    self.record
                        .lock()
                        .await
                        .context("acp-permission-request", &view)?;
                    self.permissions
                        .lock()
                        .unwrap()
                        .insert(key.clone(), PendingPermission { view, reply: tx });
                    let reply = tokio::time::timeout(Duration::from_secs(60 * 60), rx)
                        .await
                        .ok()
                        .and_then(Result::ok)
                        .unwrap_or(json!({"outcome":{"outcome":"cancelled"}}));
                    self.permissions.lock().unwrap().remove(&key);
                    self.record
                        .lock()
                        .await
                        .context("acp-permission-result", &json!({"id":key,"result":reply}))?;
                    Ok(reply)
                }
                "fs/read_text_file" | "fs/write_text_file" => self.file(method, params).await,
                _ => bail!("unsupported client method {method}"),
            }
        }
        .await;
        let reply = match result {
            Ok(value) => json!({"jsonrpc":"2.0","id":id,"result":value}),
            Err(e) => {
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32603,"message":format!("{e:#}")}})
            }
        };
        let _ = self.rpc.send(reply).await;
    }

    async fn file(&self, method: &str, params: &Value) -> Result<Value> {
        if let Some(filesystem) = &self.filesystem {
            let raw = params["path"].as_str().context("file path missing")?;
            let rel = std::path::Path::new(raw).strip_prefix(&filesystem.path).context("ACP files must be inside the mounted workspace; use Hickory MCP for document paths")?.to_string_lossy().to_string();
            let mut request = params.clone();
            request["path"] = json!(rel);
            request["op"] = json!(if method == "fs/read_text_file" {
                "read_text"
            } else {
                "write_text"
            });
            return filesystem.exchange(request).await;
        }
        let _workspace = self.workspace_gate.lock().await;
        let path = mcp::confined(
            self.context.index.root(),
            params["path"].as_str().context("file path missing")?,
        )?;
        let rel = path
            .strip_prefix(self.context.index.root())?
            .display()
            .to_string();
        let id = self.context.index.id_for_path(&rel);
        if method == "fs/read_text_file" {
            let text = if let Some(room) = self.context.rooms.get(&id).await {
                let snapshot = room.snapshot().await;
                self.file_reads
                    .lock()
                    .unwrap()
                    .insert(rel.clone(), snapshot.clone());
                snapshot.0
            } else {
                std::fs::read_to_string(&path)?
            };
            let start = params["line"].as_u64().unwrap_or(1).saturating_sub(1) as usize;
            let limit = params["limit"].as_u64().map_or(usize::MAX, |n| n as usize);
            use sha2::Digest as _;
            let count = text.lines().count().max(1);
            let shown = count.saturating_sub(start).min(limit);
            if shown > 0 {
                let read = hickory_agent::ContextRead {
                    path: rel.clone(),
                    commit: None,
                    sha256: hex::encode(sha2::Sha256::digest(text.as_bytes())),
                    first_line: start + 1,
                    last_line: start + shown,
                };
                self.record
                    .lock()
                    .await
                    .event(SessionEvent::Read { read: &read })?;
            }
            let content = if params.get("line").is_none() && params.get("limit").is_none() {
                text
            } else {
                text.split_inclusive('\n')
                    .skip(start)
                    .take(limit)
                    .collect::<String>()
            };
            return Ok(json!({"content":content}));
        }
        let content = params["content"].as_str().context("file content missing")?;
        // Generated bytes must go through lineage, not through a generic write.
        for (doc_id, _) in self.context.index.entries() {
            if let Some(doc) = self.context.index.absolute(&doc_id)
                && let Ok(source) = std::fs::read_to_string(&doc)
            {
                let outputs = hick_lang::parse(&source).ok();
                if outputs.as_ref().is_some_and(|d| {
                    d.all_tags().into_iter().any(|t| {
                        t.name == "file"
                            && t.get_attribute("path").is_some_and(|p| {
                                let output =
                                    doc.parent().unwrap_or(self.context.index.root()).join(p);
                                mcp::confined(
                                    self.context.index.root(),
                                    &output.display().to_string(),
                                )
                                .is_ok_and(|output| output == path)
                            })
                    })
                }) {
                    bail!(
                        "{rel} is generated. Use Hickory's edit_output MCP tool so the edit lands in its document."
                    );
                }
            }
        }
        let merged = if self.context.rooms.get(&id).await.is_some() {
            let base = self.file_reads.lock().unwrap().remove(&rel).context(
                "Read this open document before writing it, so concurrent edits can be preserved.",
            )?;
            super::workspace::apply(&self.context, &id, &base, content).await?
        } else {
            content.to_string()
        };
        super::super::store::write_atomic(&path, merged.as_bytes())?;
        let record = self.record.lock().await;
        record.context("acp-file-write", &json!({"path":rel,"content":merged}))?;
        let wrote = hickory_agent::Wrote {
            file: rel,
            first_line: 1,
            last_line: merged.lines().count().max(1),
            hashes: merged
                .lines()
                .map(hickory_agent::hashline::line_hash)
                .collect(),
        };
        record.event(SessionEvent::Wrote { wrote: &wrote })?;
        Ok(json!({}))
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Some(task) = self.worker.lock().unwrap().take() {
            task.abort();
        }
        self.cancel_permissions();
    }
}
