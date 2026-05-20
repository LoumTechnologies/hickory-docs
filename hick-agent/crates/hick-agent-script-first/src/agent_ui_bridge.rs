//! Bridges [`AgentEvent`](crate::events::AgentEvent) / [`AgentCommand`](crate::events::AgentCommand)
//! to the shared SDK UI model ([`UiEvent`](hick_agent_sdk::UiEvent), [`UiCommand`](hick_agent_sdk::UiCommand)).

use std::collections::HashMap;

use hick_agent_sdk::{
    Message, NetworkApprovalRequest, NodeId, Role, UiCommand, UiEvent, UiNetworkApproval,
    network_filter::{ApprovalResponse, NetworkVerdict},
};
use tokio::sync::{broadcast, mpsc};

use crate::events::{AgentCommand, AgentEvent};
use crate::tools::ScriptResult;

/// Carries monotonic [`NodeId`](hick_agent_sdk::NodeId) allocation and minimal parent pointers so
/// sequential [`AgentEvent`] streams map cleanly onto a [`hick_agent_sdk::ConversationTree`].
#[derive(Debug, Default)]
pub struct AgentUiBridge {
    ids: hick_agent_sdk::IdGen,
    /// Optional synthetic session root emitted before the first [`Self::user_turn`].
    session_parent: Option<NodeId>,
    /// Parent link for the next assistant [`UiEvent::Thinking`] node (`None` = forest root).
    turn_parent: Option<NodeId>,
    /// Active assistant response node receiving [`UiEvent::Token`] deltas.
    streaming_assistant: Option<NodeId>,
    /// Last completed assistant node (used as parent for in-turn actions).
    last_assistant: Option<NodeId>,
    /// `ActionStart` / `ActionEnd` pairing.
    pending_action: Option<NodeId>,
    /// Alternating User/Assistant messages, mirroring what the agent adds to the LLM history.
    /// Used to reconstruct a truncated history when `RewindTo` arrives.
    turn_history: Vec<Message>,
    /// Maps `AssistantDone` or `UserTurn` node_id → length of `turn_history` after that turn.
    node_turn_checkpoint: HashMap<NodeId, usize>,
}

impl AgentUiBridge {
    pub fn new() -> Self {
        Self {
            ids: hick_agent_sdk::IdGen::new(),
            ..Default::default()
        }
    }

    /// Allocate a standalone [`UiEvent`] node id (e.g. for [`UiEvent::NetworkApproval`]).
    pub fn next_node_id(&self) -> hick_agent_sdk::NodeId {
        self.ids.next()
    }

    /// Emit a synthetic session root [`UiEvent::SessionStarted`] and remember its id so
    /// [`Self::user_turn`] can hang user messages under it when no explicit parent is given.
    pub fn bootstrap_session(
        &mut self,
        model: Option<String>,
        sandbox_image: Option<String>,
        provider_info: Option<String>,
    ) -> UiEvent {
        let node_id = self.ids.next();
        self.session_parent = Some(node_id);
        UiEvent::SessionStarted {
            node_id,
            model,
            sandbox_image,
            provider_info,
        }
    }

    /// Inject an explicit user turn (the core does not emit user text today).
    pub fn user_turn(&mut self, text: String, parent_id: Option<hick_agent_sdk::NodeId>) -> UiEvent {
        let parent_id = parent_id.or(self.session_parent);
        let node_id = self.ids.next();
        self.turn_parent = Some(node_id);
        UiEvent::UserTurn {
            node_id,
            parent_id,
            text,
        }
    }

    /// Map one core event to zero or more UI events.
    pub fn map_core_event(&mut self, event: AgentEvent) -> Vec<UiEvent> {
        match event {
            AgentEvent::Info(text) => {
                let node_id = self.ids.next();
                vec![UiEvent::Info { node_id, text }]
            }
            AgentEvent::UserMessage(text) => {
                let ev = self.user_turn(text.clone(), None);
                // Track so the user turn node can be used as a RewindTo target.
                let node_id = match &ev {
                    UiEvent::UserTurn { node_id, .. } => *node_id,
                    _ => unreachable!(),
                };
                self.turn_history.push(Message { role: Role::User, content: text });
                self.node_turn_checkpoint.insert(node_id, self.turn_history.len());
                vec![ev]
            }
            AgentEvent::Thinking => {
                let node_id = self.ids.next();
                let parent_id = self.turn_parent;
                self.streaming_assistant = Some(node_id);
                vec![UiEvent::Thinking {
                    node_id,
                    parent_id,
                    text: None,
                }]
            }
            AgentEvent::Token(text) => {
                let node_id = match self.streaming_assistant {
                    Some(id) => id,
                    None => {
                        let id = self.ids.next();
                        self.streaming_assistant = Some(id);
                        id
                    }
                };
                vec![UiEvent::Token { node_id, text }]
            }
            AgentEvent::ResponseComplete(text) => {
                let node_id = match self.streaming_assistant {
                    Some(id) => id,
                    None => self.ids.next(),
                };
                self.last_assistant = Some(node_id);
                self.streaming_assistant = None;
                // Checkpoint: record the turn history after adding this assistant response.
                self.turn_history.push(Message { role: Role::Assistant, content: text.clone() });
                self.node_turn_checkpoint.insert(node_id, self.turn_history.len());
                // Approximate token counts: 4 chars ≈ 1 token (rough heuristic).
                let output_tokens = (text.len() as u64).div_ceil(4);
                let input_tokens = self.turn_history
                    .iter()
                    .map(|m| m.content.len() as u64)
                    .sum::<u64>()
                    .div_ceil(4);
                let stats_id = self.ids.next();
                vec![
                    UiEvent::AssistantDone { node_id, text },
                    UiEvent::TurnStats {
                        node_id: stats_id,
                        input_tokens,
                        output_tokens,
                        cost_usd: None,
                    },
                ]
            }
            AgentEvent::ScriptStarted { code, lang } => {
                let node_id = self.ids.next();
                self.pending_action = Some(node_id);
                let parent_id = self.streaming_assistant.or(self.last_assistant).or(self.turn_parent);
                vec![UiEvent::ActionStart {
                    node_id,
                    parent_id,
                    lang,
                    code,
                    collapsed_by_default: true,
                }]
            }
            AgentEvent::ScriptFinished(result) => {
                let ScriptResult {
                    stdout,
                    stderr,
                    exit_code,
                } = result;
                let node_id = self.pending_action.unwrap_or_else(|| self.ids.next());
                self.pending_action = None;
                vec![UiEvent::ActionEnd {
                    node_id,
                    stdout,
                    stderr,
                    exit_code,
                    collapsed_by_default: true,
                }]
            }
            AgentEvent::Done(_) => {
                let node_id = self.ids.next();
                self.turn_parent = Some(node_id);
                self.streaming_assistant = None;
                self.pending_action = None;
                vec![UiEvent::Done { node_id }]
            }
            AgentEvent::Interrupted => {
                let node_id = self.ids.next();
                vec![UiEvent::Interrupted { node_id }]
            }
            AgentEvent::Error(text) => {
                let node_id = self.ids.next();
                vec![UiEvent::Error { node_id, text }]
            }
            AgentEvent::DataClassification {
                source,
                classification,
            } => {
                let node_id = self.ids.next();
                vec![UiEvent::Info {
                    node_id,
                    text: format!("classification ({source}): {classification}"),
                }]
            }
            AgentEvent::SecurityViolation { description } => {
                let node_id = self.ids.next();
                vec![UiEvent::SecurityViolation {
                    node_id,
                    description,
                }]
            }
            AgentEvent::Reprompt { reason, attempt } => {
                let node_id = self.ids.next();
                vec![UiEvent::Info {
                    node_id,
                    text: format!("reprompt (attempt {attempt}): {reason}"),
                }]
            }
            AgentEvent::McpInstallProgress(msg) => {
                let node_id = self.ids.next();
                vec![UiEvent::Info {
                    node_id,
                    text: format!("[mcp install] {msg}"),
                }]
            }
            AgentEvent::McpInstallComplete { tools } => {
                let node_id = self.ids.next();
                vec![UiEvent::Info {
                    node_id,
                    text: format!(
                        "[mcp install] complete — {} tool(s): {}",
                        tools.len(),
                        tools.join(", ")
                    ),
                }]
            }
        }
    }
}

impl AgentEvent {
    /// Convert this core event into UI events using shared [`AgentUiBridge`] state.
    pub fn into_ui_events(self, bridge: &mut AgentUiBridge) -> Vec<UiEvent> {
        bridge.map_core_event(self)
    }
}

/// Best-effort mapping for control commands originating from shared UI chrome.
///
/// `RewindTo` and `NetworkApproval` are handled inside [`run_ui_event_fanout_task`]
/// directly; other UI-only commands return `None`.
pub fn ui_command_to_agent(cmd: UiCommand) -> Option<AgentCommand> {
    match cmd {
        UiCommand::Message { text } => Some(AgentCommand::Message(text)),
        UiCommand::DirectCommand { text } => Some(AgentCommand::DirectCommand(text)),
        UiCommand::Quit => Some(AgentCommand::Quit),
        UiCommand::Cancel
        | UiCommand::ExpandNode { .. }
        | UiCommand::CollapseNode { .. }
        | UiCommand::RewindTo { .. }
        | UiCommand::NetworkApproval { .. }
        | UiCommand::McpCredential { .. }
        | UiCommand::McpCredentialSkip
        | UiCommand::AddConfigPath { .. } => None,
    }
}

/// Inverse helper for tests / future UI stacks that speak [`UiCommand`] only.
pub fn agent_command_to_ui(cmd: AgentCommand) -> UiCommand {
    match cmd {
        AgentCommand::Message(s) => UiCommand::Message { text: s },
        AgentCommand::DirectCommand(s) => UiCommand::DirectCommand { text: s },
        AgentCommand::Quit => UiCommand::Quit,
        AgentCommand::RewindTo { .. } => UiCommand::Cancel,
    }
}

/// Merges core [`AgentEvent`]s, [`NetworkApprovalRequest`] prompts, and [`UiCommand`]s
/// into [`UiEvent`]s (single [`AgentUiBridge`], monotonic ids).
///
/// Handles `RewindTo` commands directly: emits `BranchStarted` and forwards
/// a `AgentCommand::RewindTo` with the reconstructed history to `agent_cmd_tx`.
/// All other commands are forwarded via [`ui_command_to_agent`].
///
/// Events are multicasted via [`broadcast::Sender`] — every subscriber (TUI, browsers, …)
/// observes the same stream.
pub async fn run_ui_event_fanout_task(
    mut agent_rx: mpsc::UnboundedReceiver<AgentEvent>,
    mut approval_rx: mpsc::UnboundedReceiver<NetworkApprovalRequest>,
    ui_tx: broadcast::Sender<UiEvent>,
    model: Option<String>,
    sandbox_image: Option<String>,
    provider_banner: Option<String>,
    mut ui_cmd_rx: mpsc::UnboundedReceiver<UiCommand>,
    agent_cmd_tx: mpsc::UnboundedSender<AgentCommand>,
    interrupt: std::sync::Arc<tokio::sync::Notify>,
) {
    let mut bridge = AgentUiBridge::new();
    let boot = bridge.bootstrap_session(model, sandbox_image, provider_banner);
    let _ = ui_tx.send(boot);

    let mut approvals_live = true;
    // Pending network approvals keyed by NodeId — allows web UI to resolve them.
    let mut pending_approvals: HashMap<NodeId, UiNetworkApproval> = HashMap::new();

    loop {
        tokio::select! {
            ev = agent_rx.recv() => {
                match ev {
                    None => break,
                    Some(e) => {
                        for u in bridge.map_core_event(e) {
                            let _ = ui_tx.send(u);
                        }
                    }
                }
            }
            appr = approval_rx.recv(), if approvals_live => match appr {
                Some(req) => {
                    let id = bridge.next_node_id();
                    let approval = UiNetworkApproval::from_network_request(id, req);
                    pending_approvals.insert(id, approval.clone());
                    let _ = ui_tx.send(UiEvent::NetworkApproval(approval));
                }
                None => approvals_live = false,
            },
            cmd = ui_cmd_rx.recv() => {
                match cmd {
                    None => break,
                    Some(UiCommand::Cancel) => {
                        interrupt.notify_one();
                    }
                    Some(UiCommand::RewindTo { node_id }) => {
                        handle_rewind_to(node_id, &mut bridge, &ui_tx, &agent_cmd_tx, &interrupt);
                    }
                    Some(UiCommand::NetworkApproval { node_id, verdict, suggestion_index }) => {
                        handle_web_network_approval(
                            node_id, &verdict, suggestion_index,
                            &mut pending_approvals, &ui_tx, &mut bridge,
                        );
                    }
                    Some(other) => {
                        if let Some(c) = ui_command_to_agent(other) {
                            agent_cmd_tx.send(c).ok();
                        }
                    }
                }
            }
        }
    }
}

fn handle_web_network_approval(
    node_id: NodeId,
    verdict: &str,
    suggestion_index: Option<usize>,
    pending_approvals: &mut HashMap<NodeId, UiNetworkApproval>,
    ui_tx: &broadcast::Sender<UiEvent>,
    bridge: &mut AgentUiBridge,
) {
    let Some(approval) = pending_approvals.remove(&node_id) else {
        return; // already resolved or unknown
    };
    let always_allow = suggestion_index
        .and_then(|i| approval.suggestions.get(i.saturating_sub(1)).cloned());
    let net_verdict = match verdict {
        "allow" => NetworkVerdict::Allow,
        _ => NetworkVerdict::Block,
    };
    let desc = always_allow.as_ref().map(|s| s.describe().to_string());
    let _ = approval.respond(ApprovalResponse { verdict: net_verdict, always_allow });
    let info_id = bridge.ids.next();
    let _ = ui_tx.send(UiEvent::Info {
        node_id: info_id,
        text: match (verdict, desc) {
            ("allow", Some(rule)) => format!("[network] Always allow: {rule}"),
            ("allow", None) => "[network] Allowed (this request only)".to_string(),
            _ => "[network] Blocked".to_string(),
        },
    });
}

fn handle_rewind_to(
    node_id: NodeId,
    bridge: &mut AgentUiBridge,
    ui_tx: &broadcast::Sender<UiEvent>,
    agent_cmd_tx: &mpsc::UnboundedSender<AgentCommand>,
    interrupt: &std::sync::Arc<tokio::sync::Notify>,
) {
    let Some(&checkpoint_len) = bridge.node_turn_checkpoint.get(&node_id) else {
        let err_id = bridge.ids.next();
        let _ = ui_tx.send(UiEvent::Error {
            node_id: err_id,
            text: format!("RewindTo: unknown node #{node_id}"),
        });
        return;
    };

    // Cancel any in-flight LLM request.
    interrupt.notify_one();

    let fork_history = bridge.turn_history[..checkpoint_len].to_vec();
    let label = format!("Branch from #{node_id}");
    let branch_node_id = bridge.ids.next();
    let _ = ui_tx.send(UiEvent::BranchStarted {
        node_id: branch_node_id,
        fork_node_id: node_id,
        label: label.clone(),
    });

    // Reset the bridge's own turn history so new turns after the branch
    // don't accumulate stale post-fork entries.
    bridge.turn_history.truncate(checkpoint_len);
    bridge.node_turn_checkpoint.retain(|_, v| *v <= checkpoint_len);

    agent_cmd_tx.send(AgentCommand::RewindTo {
        fork_node_id: node_id,
        user_assistant_history: fork_history,
    }).ok();
}
