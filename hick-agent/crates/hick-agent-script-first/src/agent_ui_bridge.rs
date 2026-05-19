//! Bridges [`AgentEvent`](crate::events::AgentEvent) / [`AgentCommand`](crate::events::AgentCommand)
//! to the shared SDK UI model ([`UiEvent`](hick_agent_sdk::UiEvent), [`UiCommand`](hick_agent_sdk::UiCommand)).

use hick_agent_sdk::{NetworkApprovalRequest, UiCommand, UiEvent, UiNetworkApproval};


use crate::events::{AgentCommand, AgentEvent};
use crate::tools::ScriptResult;

/// Carries monotonic [`NodeId`](hick_agent_sdk::NodeId) allocation and minimal parent pointers so
/// sequential [`AgentEvent`] streams map cleanly onto a [`hick_agent_sdk::ConversationTree`].
#[derive(Debug, Default)]
pub struct AgentUiBridge {
    ids: hick_agent_sdk::IdGen,
    /// Optional synthetic session root emitted before the first [`Self::user_turn`].
    session_parent: Option<hick_agent_sdk::NodeId>,
    /// Parent link for the next assistant [`UiEvent::Thinking`] node (`None` = forest root).
    turn_parent: Option<hick_agent_sdk::NodeId>,
    /// Active assistant response node receiving [`UiEvent::Token`] deltas.
    streaming_assistant: Option<hick_agent_sdk::NodeId>,
    /// Last completed assistant node (used as parent for in-turn actions).
    last_assistant: Option<hick_agent_sdk::NodeId>,
    /// `ActionStart` / `ActionEnd` pairing.
    pending_action: Option<hick_agent_sdk::NodeId>,
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
            AgentEvent::Thinking => {
                let node_id = self.ids.next();
                let parent_id = self.turn_parent;
                self.streaming_assistant = Some(node_id);
                vec![UiEvent::Thinking {
                    node_id,
                    parent_id,
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
                vec![UiEvent::AssistantDone {
                    node_id,
                    text,
                }]
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
/// UI-only commands ([`UiCommand::ExpandNode`], [`UiCommand::CollapseNode`], [`UiCommand::Cancel`])
/// return `None` until the core exposes matching [`AgentCommand`] variants.
pub fn ui_command_to_agent(cmd: UiCommand) -> Option<AgentCommand> {
    match cmd {
        UiCommand::Message(s) => Some(AgentCommand::Message(s)),
        UiCommand::DirectCommand(s) => Some(AgentCommand::DirectCommand(s)),
        UiCommand::Quit => Some(AgentCommand::Quit),
        UiCommand::Cancel | UiCommand::ExpandNode(_) | UiCommand::CollapseNode(_) => None,
    }
}

/// Inverse helper for tests / future UI stacks that speak [`UiCommand`] only.
pub fn agent_command_to_ui(cmd: AgentCommand) -> UiCommand {
    match cmd {
        AgentCommand::Message(s) => UiCommand::Message(s),
        AgentCommand::DirectCommand(s) => UiCommand::DirectCommand(s),
        AgentCommand::Quit => UiCommand::Quit,
    }
}

/// Merges core [`AgentEvent`]s with [`NetworkApprovalRequest`] prompts into [`UiEvent`] for [`hick_agent_tui`] (single [`AgentUiBridge`], monotonic ids).
///
/// Drops the approvals branch cleanly when all [`NetworkApprovalRequest`] senders are gone so the task keeps running until the agent event stream ends.
pub async fn run_ui_event_fanout_task(
    mut agent_rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
    mut approval_rx: tokio::sync::mpsc::UnboundedReceiver<NetworkApprovalRequest>,
    ui_tx: tokio::sync::mpsc::UnboundedSender<UiEvent>,
    model: Option<String>,
    sandbox_image: Option<String>,
    provider_banner: Option<String>,
) {
    let mut bridge = AgentUiBridge::new();
    let boot = bridge.bootstrap_session(model, sandbox_image, provider_banner);
    let _ = ui_tx.send(boot);

    let mut approvals_live = true;

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
                    let _ = ui_tx.send(UiEvent::NetworkApproval(
                        UiNetworkApproval::from_network_request(id, req),
                    ));
                }
                None => approvals_live = false,
            },
        }
    }
}
