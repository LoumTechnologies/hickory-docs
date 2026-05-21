//! Canonical UI-facing events and commands for agent frontends (TUI, web, relay).
//!
//! [`UiEvent`] is a superset stream designed for building a [`crate::conversation_tree::ConversationTree`].
//! Network access approvals reuse the same [`crate::NetworkApprovalRequest`] channel semantics as today,
//! wrapped in [`UiNetworkApproval`] so the enum remains [`Clone`] via shared [`std::sync::Arc`] state.

use std::collections::HashMap;
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
///
/// Serializes over the browser WebSocket as JSON with a **`type`** field in snake_case
/// (`message`, `direct_command`, `cancel`, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UiCommand {
    /// User submitted a natural-language prompt.
    Message {
        text: String,
    },
    /// Shell-style passthrough (`! cmd`).
    DirectCommand {
        text: String,
    },
    /// Ask the core to cancel / interrupt the current step.
    Cancel,
    /// Exit the application / UI session.
    Quit,
    /// Mark a tree node expanded (for server-side collapse tracking).
    ExpandNode {
        node_id: NodeId,
    },
    /// Mark a tree node collapsed.
    CollapseNode {
        node_id: NodeId,
    },
    /// Rewind the agent's LLM context to the given node and start a new branch.
    RewindTo {
        node_id: NodeId,
    },
    /// Web UI response to a `NetworkApproval` event.
    NetworkApproval {
        node_id: NodeId,
        /// "allow" or "deny"
        verdict: String,
        /// 1-based index into the `suggestions` list; `None` means allow-once with no persistent rule.
        suggestion_index: Option<usize>,
    },
    /// Web UI supplies a credential for an MCP install flow.
    McpCredential {
        env_var: String,
        value: String,
    },
    /// Web UI skips credential entry for an MCP install flow.
    McpCredentialSkip,
    /// User supplied API keys via the first-run setup modal.
    ProvideApiKeys {
        anthropic: Option<String>,
        openai: Option<String>,
        xai: Option<String>,
    },
    /// Request to add a new config to the current session (mid-session multi-config).
    AddConfigPath {
        /// TOML config file path. The agent core will load and spawn a new instance.
        path: String,
    },
    /// Replay a past turn against one or more alternate configs (TASK-47).
    ///
    /// `turn_node_id` must identify a `UserTurn` node visible in the conversation tree.
    /// Each path in `config_paths` spawns an independent agent instance in a new tab.
    ReplayTurn {
        turn_node_id: NodeId,
        config_paths: Vec<String>,
    },
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
    ///
    /// `text` carries extended-thinking block content when the provider
    /// returns it (e.g. Anthropic extended thinking).  `None` means the
    /// LLM was called but produced no separate thinking block.
    Thinking {
        node_id: NodeId,
        parent_id: Option<NodeId>,
        text: Option<String>,
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
    /// A new conversation branch was created by rewinding to `fork_node_id`.
    BranchStarted {
        node_id: NodeId,
        fork_node_id: NodeId,
        label: String,
    },
    /// Agent-pushed persistent markdown pane (name is stable; empty markdown clears the pane).
    PaneContent {
        node_id: NodeId,
        name: String,
        title: String,
        markdown: String,
        /// "markdown" or "html"
        pane_type: String,
        /// Entry file for html panes (relative to the pane directory).
        entry: String,
    },
    /// LLM token usage and estimated cost for the completed assistant turn.
    TurnStats {
        node_id: NodeId,
        /// Approximate input token count for this turn.
        input_tokens: u64,
        /// Approximate output token count for this turn.
        output_tokens: u64,
        /// Estimated USD cost (None when cost model is unknown).
        cost_usd: Option<f64>,
    },
    /// Judge scores are available for a completed turn — pre-populates the human rating panel (AC #3).
    JudgeScoreReady {
        node_id: NodeId,
        /// Agent config tab whose response was scored.
        config_name: String,
        /// The user query that was judged.
        query: String,
        judge_model: String,
        scores: HashMap<String, f64>,
        /// Ordered dimension names (preserves display order from config).
        dimensions: Vec<String>,
        /// Brief rationale for each backtracking reversal identified (AC #11).
        backtracking_rationale: Vec<String>,
        wall_secs: Option<f64>,
        rework_count: Option<u32>,
        /// RFC 3339 timestamp of the judge call (links to HumanRatingRecord).
        judge_timestamp: String,
    },
    /// The judge call failed — non-blocking error for UI display (AC #7).
    JudgeError {
        node_id: NodeId,
        config_name: String,
        error: String,
    },
    /// No API keys are configured; UIs should show the first-run setup screen.
    AwaitingApiKey {
        node_id: NodeId,
    },
}

fn opt_id(id: Option<NodeId>) -> Option<u64> {
    id.map(|n| n.0)
}

/// JSON wire projection for transports (compact `type` + snake_case payloads).
///
/// Interactive approval state (`oneshot`) is intentionally omitted — only display fields ship.
pub fn ui_event_wire_value(ev: &UiEvent) -> serde_json::Value {
    match ev {
        UiEvent::SessionStarted {
            node_id,
            model,
            sandbox_image,
            provider_info,
        } => serde_json::json!({
            "type": "session_started",
            "node_id": node_id.0,
            "model": model,
            "sandbox_image": sandbox_image,
            "provider_info": provider_info,
        }),
        UiEvent::UserTurn {
            node_id,
            parent_id,
            text,
        } => serde_json::json!({
            "type": "user_turn",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "text": text,
        }),
        UiEvent::Thinking { node_id, parent_id, text } => serde_json::json!({
            "type": "thinking",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "text": text,
        }),
        UiEvent::Token { node_id, text } => serde_json::json!({
            "type": "token",
            "node_id": node_id.0,
            "text": text,
        }),
        UiEvent::AssistantDone { node_id, text } => serde_json::json!({
            "type": "assistant_done",
            "node_id": node_id.0,
            "text": text,
        }),
        UiEvent::ActionStart {
            node_id,
            parent_id,
            lang,
            code,
            collapsed_by_default,
        } => serde_json::json!({
            "type": "action_start",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "lang": lang,
            "code": code,
            "collapsed_by_default": collapsed_by_default,
        }),
        UiEvent::ActionEnd {
            node_id,
            stdout,
            stderr,
            exit_code,
            collapsed_by_default,
        } => serde_json::json!({
            "type": "action_end",
            "node_id": node_id.0,
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
            "collapsed_by_default": collapsed_by_default,
        }),
        UiEvent::SubAgentSpawned {
            node_id,
            parent_id,
            label,
        } => serde_json::json!({
            "type": "sub_agent_spawned",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "label": label,
        }),
        UiEvent::SubAgentDone { node_id } => serde_json::json!({
            "type": "sub_agent_done",
            "node_id": node_id.0,
        }),
        UiEvent::McpCall {
            node_id,
            parent_id,
            server,
            method,
            collapsed_by_default,
        } => serde_json::json!({
            "type": "mcp_call",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "server": server,
            "method": method,
            "collapsed_by_default": collapsed_by_default,
        }),
        UiEvent::McpResponse {
            node_id,
            server,
            method,
            collapsed_by_default,
        } => serde_json::json!({
            "type": "mcp_response",
            "node_id": node_id.0,
            "server": server,
            "method": method,
            "collapsed_by_default": collapsed_by_default,
        }),
        UiEvent::SecurityViolation { node_id, description } => serde_json::json!({
            "type": "security_violation",
            "node_id": node_id.0,
            "description": description,
        }),
        UiEvent::NetworkApproval(a) => serde_json::json!({
            "type": "network_approval",
            "node_id": a.node_id.0,
            "host": a.host,
            "suggestions": a.suggestions.iter().map(|s| s.describe()).collect::<Vec<_>>(),
        }),
        UiEvent::AdaptiveCard {
            node_id,
            parent_id,
            json,
        } => serde_json::json!({
            "type": "adaptive_card",
            "node_id": node_id.0,
            "parent_id": opt_id(*parent_id),
            "json": json,
        }),
        UiEvent::Info { node_id, text } => serde_json::json!({
            "type": "info",
            "node_id": node_id.0,
            "text": text,
        }),
        UiEvent::Error { node_id, text } => serde_json::json!({
            "type": "error",
            "node_id": node_id.0,
            "text": text,
        }),
        UiEvent::Done { node_id } => serde_json::json!({
            "type": "done",
            "node_id": node_id.0,
        }),
        UiEvent::Interrupted { node_id } => serde_json::json!({
            "type": "interrupted",
            "node_id": node_id.0,
        }),
        UiEvent::BranchStarted { node_id, fork_node_id, label } => serde_json::json!({
            "type": "branch_started",
            "node_id": node_id.0,
            "fork_node_id": fork_node_id.0,
            "label": label,
        }),
        UiEvent::PaneContent { node_id, name, title, markdown, pane_type, entry } => serde_json::json!({
            "type": "pane_content",
            "node_id": node_id.0,
            "name": name,
            "title": title,
            "markdown": markdown,
            "pane_type": pane_type,
            "entry": entry,
        }),
        UiEvent::TurnStats { node_id, input_tokens, output_tokens, cost_usd } => serde_json::json!({
            "type": "turn_stats",
            "node_id": node_id.0,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "cost_usd": cost_usd,
        }),
        UiEvent::JudgeScoreReady {
            node_id, config_name, query, judge_model, scores, dimensions,
            backtracking_rationale, wall_secs, rework_count, judge_timestamp,
        } => serde_json::json!({
            "type": "judge_score_ready",
            "node_id": node_id.0,
            "config_name": config_name,
            "query": query,
            "judge_model": judge_model,
            "scores": scores,
            "dimensions": dimensions,
            "backtracking_rationale": backtracking_rationale,
            "wall_secs": wall_secs,
            "rework_count": rework_count,
            "judge_timestamp": judge_timestamp,
        }),
        UiEvent::JudgeError { node_id, config_name, error } => serde_json::json!({
            "type": "judge_error",
            "node_id": node_id.0,
            "config_name": config_name,
            "error": error,
        }),
        UiEvent::AwaitingApiKey { node_id } => serde_json::json!({
            "type": "awaiting_api_key",
            "node_id": node_id.0,
        }),
    }
}

impl serde::Serialize for UiEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ui_event_wire_value(self).serialize(serializer)
    }
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
            UiEvent::BranchStarted { node_id, .. } => f.debug_tuple("BranchStarted").field(node_id).finish(),
            UiEvent::PaneContent { node_id, .. } => f.debug_tuple("PaneContent").field(node_id).finish(),
            UiEvent::TurnStats { node_id, .. } => f.debug_tuple("TurnStats").field(node_id).finish(),
            UiEvent::JudgeScoreReady { node_id, .. } => f.debug_tuple("JudgeScoreReady").field(node_id).finish(),
            UiEvent::JudgeError { node_id, .. } => f.debug_tuple("JudgeError").field(node_id).finish(),
            UiEvent::AwaitingApiKey { node_id } => f.debug_tuple("AwaitingApiKey").field(node_id).finish(),
        }
    }
}

// ---------------------------------------------------------------------------
// Adaptive Card emission helper
// ---------------------------------------------------------------------------

/// Emit an Adaptive Card as a structured session log entry.
///
/// The XML fragment format is `<log:adaptive-card node-id="N" parent-id="P">…JSON…</log:adaptive-card>`.
/// The [`crate::session_bundle_reader::SessionBundleReader`] maps this fragment back to
/// `UiEvent::AdaptiveCard` during replay.
pub fn emit_adaptive_card(
    log: &dyn crate::session_log::SessionLog,
    id_gen: &IdGen,
    parent_id: NodeId,
    card_json: &str,
) {
    let node_id = id_gen.next();
    let xml_fragment = format!(
        "<log:adaptive-card node-id=\"{node_id}\" parent-id=\"{parent_id}\">{card_json}</log:adaptive-card>"
    );
    log.record(crate::session_log::LogEvent::Custom {
        xml_fragment: &xml_fragment,
        attachments: &[],
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session_log::{LogEvent, SessionLog};

    struct CaptureSink(std::sync::Mutex<Vec<String>>);
    impl SessionLog for CaptureSink {
        fn record(&self, event: LogEvent<'_>) {
            if let LogEvent::Custom { xml_fragment, .. } = event {
                self.0.lock().unwrap().push(xml_fragment.to_string());
            }
        }
    }

    #[test]
    fn emit_adaptive_card_roundtrip() {
        let sink = CaptureSink(std::sync::Mutex::new(vec![]));
        let id_gen = IdGen::new();
        let parent_id = id_gen.next();
        let card_json = r#"{"type":"AdaptiveCard","version":"1.3","body":[]}"#;
        emit_adaptive_card(&sink, &id_gen, parent_id, card_json);

        let frags = sink.0.lock().unwrap();
        assert_eq!(frags.len(), 1);
        let frag = &frags[0];
        assert!(frag.contains("log:adaptive-card"), "fragment: {frag}");
        assert!(frag.contains(card_json), "fragment: {frag}");
        assert!(frag.contains(&format!("parent-id=\"{parent_id}\"")), "fragment: {frag}");
    }
}
