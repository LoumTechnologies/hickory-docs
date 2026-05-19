//! Canonical UI-facing events and commands for agent frontends (TUI, web, relay).
//!
//! [`UiEvent`] is a superset stream designed for building a [`crate::conversation_tree::ConversationTree`].
//! Network access approvals reuse the same [`crate::NetworkApprovalRequest`] channel semantics as today,
//! wrapped in [`UiNetworkApproval`] so the enum remains [`Clone`] via shared [`std::sync::Arc`] state.

use std::sync::{Arc, Mutex};

use hick_policy::PolicySuggestion;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::network_filter::{ApprovalResponse, NetworkApprovalRequest};

// ---------------------------------------------------------------------------
// NodeId & IdGen
// ---------------------------------------------------------------------------

/// Stable identifier for one node in the conversation tree (XML attributes, JSON).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Serialize as decimal string (WebSocket / loose JSON consumers).
    pub fn as_string(self) -> String {
        self.0.to_string()
    }

    /// Parse from a decimal string; returns `None` if invalid.
    pub fn from_str_radix10(s: &str) -> Option<Self> {
        s.parse::<u64>().ok().map(NodeId)
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for NodeId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(NodeId(s.parse()?))
    }
}

/// Thread-safe monotonic [`NodeId`] allocator (starts at 1; `0` reserved as sentinel).
#[derive(Debug)]
pub struct IdGen {
    next: std::sync::atomic::AtomicU64,
}

impl Default for IdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl IdGen {
    pub fn new() -> Self {
        Self {
            next: std::sync::atomic::AtomicU64::new(1),
        }
    }

    /// Next identifier (unique for this generator for the process lifetime).
    pub fn next(&self) -> NodeId {
        let n = self
            .next
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        NodeId(n)
    }
}

// ---------------------------------------------------------------------------
// UiNetworkApproval
// ---------------------------------------------------------------------------

/// Clone-friendly wrapper around [`NetworkApprovalRequest`] for embedding in [`UiEvent`].
///
/// Completing the approval consumes the inner `oneshot` sender exactly once; clones share the same slot.
#[derive(Clone)]
pub struct UiNetworkApproval {
    pub node_id: NodeId,
    pub host: String,
    pub suggestions: Vec<PolicySuggestion>,
    response_tx: Arc<Mutex<Option<oneshot::Sender<ApprovalResponse>>>>,
}

impl std::fmt::Debug for UiNetworkApproval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UiNetworkApproval")
            .field("node_id", &self.node_id)
            .field("host", &self.host)
            .field("suggestions", &self.suggestions)
            .field("response_tx", &"<oneshot>")
            .finish()
    }
}

impl UiNetworkApproval {
    /// Wrap a live SDK approval request with a UI [`NodeId`].
    pub fn from_network_request(node_id: NodeId, req: NetworkApprovalRequest) -> Self {
        let NetworkApprovalRequest {
            host,
            suggestions,
            response_tx,
        } = req;
        Self {
            node_id,
            host,
            suggestions,
            response_tx: Arc::new(Mutex::new(Some(response_tx))),
        }
    }

    /// Deliver the user's verdict. Returns `Err(response)` if the channel was already consumed.
    pub fn respond(&self, response: ApprovalResponse) -> Result<(), ApprovalResponse> {
        let tx = self.response_tx.lock().unwrap().take();
        match tx {
            Some(tx) => tx.send(response),
            None => Err(response),
        }
    }
}

// ---------------------------------------------------------------------------
// UiCommand
// ---------------------------------------------------------------------------

/// Commands sent from a UI toward the agent core or shared UI state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiCommand {
    /// User submitted a natural-language prompt.
    Message(String),
    /// Shell-style passthrough (`! cmd`).
    DirectCommand(String),
    /// Ask the core to cancel / interrupt the current step.
    Cancel,
    /// Exit the application / UI session.
    Quit,
    /// Mark a tree node expanded (for server-side collapse tracking).
    ExpandNode(NodeId),
    /// Mark a tree node collapsed.
    CollapseNode(NodeId),
}

// ---------------------------------------------------------------------------
// UiEvent
// ---------------------------------------------------------------------------

/// Event emitted toward any agent UI — TUI, local web, or relay fan-out.
#[derive(Clone)]
pub enum UiEvent {
    /// Session / runtime started (model, sandbox, provider metadata).
    SessionStarted {
        node_id: NodeId,
        model: Option<String>,
        sandbox_image: Option<String>,
        provider_info: Option<String>,
    },
    /// A user message turn (usually injected by the shell around [`UiCommand::Message`]).
    UserTurn {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        text: String,
    },
    /// Assistant is waiting on / calling the LLM for this branch.
    Thinking {
        node_id: NodeId,
        parent_id: Option<NodeId>,
    },
    /// Streaming token delta for an assistant response node.
    Token {
        node_id: NodeId,
        text: String,
    },
    /// Assistant finished streaming; carries the full consolidated text.
    AssistantDone {
        node_id: NodeId,
        text: String,
    },
    /// Script or shell action begins.
    ActionStart {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        lang: String,
        code: String,
        collapsed_by_default: bool,
    },
    /// Script or shell action completed.
    ActionEnd {
        node_id: NodeId,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
        collapsed_by_default: bool,
    },
    /// A sub-agent or parallel branch started under `parent_id`.
    SubAgentSpawned {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        label: String,
    },
    /// Sub-agent branch finished.
    SubAgentDone {
        node_id: NodeId,
    },
    /// MCP tool call (collapsed by default in UIs).
    McpCall {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        server: String,
        method: String,
        collapsed_by_default: bool,
    },
    /// MCP tool response (collapsed by default).
    McpResponse {
        node_id: NodeId,
        server: String,
        method: String,
        collapsed_by_default: bool,
    },
    /// Security policy blocked an operation.
    SecurityViolation {
        node_id: NodeId,
        description: String,
    },
    /// Prompt for outbound network access (not serialized for relay — use relay-specific approval).
    NetworkApproval(UiNetworkApproval),
    /// Extension-emitted Adaptive Card JSON.
    AdaptiveCard {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        json: String,
    },
    /// Informational status line.
    Info {
        node_id: NodeId,
        text: String,
    },
    /// Recoverable or fatal error message for display.
    Error {
        node_id: NodeId,
        text: String,
    },
    /// Turn / iteration completed.
    Done {
        node_id: NodeId,
    },
    /// User or policy interrupted execution.
    Interrupted {
        node_id: NodeId,
    },
}

impl std::fmt::Debug for UiEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UiEvent::SessionStarted { node_id, .. } => f.debug_tuple("SessionStarted").field(node_id).finish(),
            UiEvent::UserTurn { node_id, .. } => f.debug_tuple("UserTurn").field(node_id).finish(),
            UiEvent::Thinking { node_id, .. } => f.debug_tuple("Thinking").field(node_id).finish(),
            UiEvent::Token { node_id, .. } => f.debug_tuple("Token").field(node_id).finish(),
            UiEvent::AssistantDone { node_id, .. } => f.debug_tuple("AssistantDone").field(node_id).finish(),
            UiEvent::ActionStart { node_id, .. } => f.debug_tuple("ActionStart").field(node_id).finish(),
            UiEvent::ActionEnd { node_id, .. } => f.debug_tuple("ActionEnd").field(node_id).finish(),
            UiEvent::SubAgentSpawned { node_id, .. } => f.debug_tuple("SubAgentSpawned").field(node_id).finish(),
            UiEvent::SubAgentDone { node_id } => f.debug_tuple("SubAgentDone").field(node_id).finish(),
            UiEvent::McpCall { node_id, .. } => f.debug_tuple("McpCall").field(node_id).finish(),
            UiEvent::McpResponse { node_id, .. } => f.debug_tuple("McpResponse").field(node_id).finish(),
            UiEvent::SecurityViolation { node_id, .. } => f.debug_tuple("SecurityViolation").field(node_id).finish(),
            UiEvent::NetworkApproval(inner) => f.debug_tuple("NetworkApproval").field(inner).finish(),
            UiEvent::AdaptiveCard { node_id, .. } => f.debug_tuple("AdaptiveCard").field(node_id).finish(),
            UiEvent::Info { node_id, .. } => f.debug_tuple("Info").field(node_id).finish(),
            UiEvent::Error { node_id, .. } => f.debug_tuple("Error").field(node_id).finish(),
            UiEvent::Done { node_id } => f.debug_tuple("Done").field(node_id).finish(),
            UiEvent::Interrupted { node_id } => f.debug_tuple("Interrupted").field(node_id).finish(),
        }
    }
}
