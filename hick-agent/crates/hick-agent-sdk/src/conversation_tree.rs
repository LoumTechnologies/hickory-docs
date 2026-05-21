//! In-memory conversation tree built incrementally from [`UiEvent`](crate::UiEvent) records.

use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::ui_event::{NodeId, UiEvent};

/// High-level kind for a [`ConversationNode`] (render hint).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationNodeKind {
    Session,
    UserTurn,
    Thinking,
    Assistant,
    Action,
    SubAgent,
    McpCall,
    McpResponse,
    Security,
    NetworkApproval,
    AdaptiveCard,
    Info,
    Error,
    Done,
    Interrupted,
}

/// Opaque per-node payload distilled from [`UiEvent`] application.
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationPayload {
    Empty,
    Session {
        model: Option<String>,
        sandbox_image: Option<String>,
        provider_info: Option<String>,
    },
    Text(String),
    Action {
        lang: String,
        code: String,
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
    },
    SubAgent { label: String },
    Mcp {
        server: String,
        method: String,
        is_response: bool,
    },
    Security { description: String },
    NetworkApproval {
        host: String,
        suggestion_count: usize,
    },
    AdaptiveCardJson(String),
}

/// One vertex in the ordered conversation DAG (tree under each root).
#[derive(Debug, Clone, PartialEq)]
pub struct ConversationNode {
    pub id: NodeId,
    pub parent_id: Option<NodeId>,
    pub kind: ConversationNodeKind,
    pub timestamp: DateTime<Utc>,
    pub collapsed_by_default: bool,
    pub payload: ConversationPayload,
}

/// Append-only conversation tree — all frontends render from this structure.
#[derive(Debug, Default, Clone)]
pub struct ConversationTree {
    nodes: HashMap<NodeId, ConversationNode>,
    /// Children grouped under each parent key (`None` = forest roots).
    children: HashMap<Option<NodeId>, Vec<NodeId>>,
}

impl ConversationTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node(&self, id: NodeId) -> Option<&ConversationNode> {
        self.nodes.get(&id)
    }

    pub fn roots(&self) -> &[NodeId] {
        self.children.get(&None).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn children_of(&self, parent: Option<NodeId>) -> &[NodeId] {
        self.children.get(&parent).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn children_ids(&self, parent: NodeId) -> &[NodeId] {
        self.children_of(Some(parent))
    }

    /// Apply one UI event — creates or updates nodes while preserving child insertion order.
    pub fn apply(&mut self, event: &UiEvent) {
        let now = Utc::now();
        match event {
            UiEvent::SessionStarted {
                node_id,
                model,
                sandbox_image,
                provider_info,
            } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Session,
                    false,
                    ConversationPayload::Session {
                        model: model.clone(),
                        sandbox_image: sandbox_image.clone(),
                        provider_info: provider_info.clone(),
                    },
                    now,
                );
            }
            UiEvent::UserTurn {
                node_id,
                parent_id,
                text,
            } => {
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::UserTurn,
                    false,
                    ConversationPayload::Text(text.clone()),
                    now,
                );
            }
            UiEvent::Thinking {
                node_id,
                parent_id,
                text,
            } => {
                let payload = match text {
                    Some(t) if !t.is_empty() => ConversationPayload::Text(t.clone()),
                    _ => ConversationPayload::Empty,
                };
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::Thinking,
                    true,
                    payload,
                    now,
                );
            }
            UiEvent::Token { node_id, text } => {
                self.append_text(*node_id, text, ConversationNodeKind::Assistant, now);
            }
            UiEvent::AssistantDone { node_id, text } => {
                self.set_full_text(*node_id, text, ConversationNodeKind::Assistant, false, now);
            }
            UiEvent::ActionStart {
                node_id,
                parent_id,
                lang,
                code,
                collapsed_by_default,
            } => {
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::Action,
                    *collapsed_by_default,
                    ConversationPayload::Action {
                        lang: lang.clone(),
                        code: code.clone(),
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                    },
                    now,
                );
            }
            UiEvent::ActionEnd {
                node_id,
                stdout,
                stderr,
                exit_code,
                collapsed_by_default,
            } => {
                self.patch_action_end(*node_id, stdout, stderr, *exit_code, *collapsed_by_default, now);
            }
            UiEvent::SubAgentSpawned {
                node_id,
                parent_id,
                label,
            } => {
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::SubAgent,
                    false,
                    ConversationPayload::SubAgent {
                        label: label.clone(),
                    },
                    now,
                );
            }
            UiEvent::SubAgentDone { node_id } => {
                self.touch_timestamp(*node_id, now);
            }
            UiEvent::McpCall {
                node_id,
                parent_id,
                server,
                method,
                collapsed_by_default,
            } => {
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::McpCall,
                    *collapsed_by_default,
                    ConversationPayload::Mcp {
                        server: server.clone(),
                        method: method.clone(),
                        is_response: false,
                    },
                    now,
                );
            }
            UiEvent::McpResponse {
                node_id,
                server,
                method,
                collapsed_by_default,
            } => {
                // Parent unknown from event alone — attach as root unless node exists (caller should emit parent later).
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::McpResponse,
                    *collapsed_by_default,
                    ConversationPayload::Mcp {
                        server: server.clone(),
                        method: method.clone(),
                        is_response: true,
                    },
                    now,
                );
            }
            UiEvent::SecurityViolation {
                node_id,
                description,
            } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Security,
                    false,
                    ConversationPayload::Security {
                        description: description.clone(),
                    },
                    now,
                );
            }
            UiEvent::NetworkApproval(inner) => {
                self.insert_node(
                    inner.node_id,
                    None,
                    ConversationNodeKind::NetworkApproval,
                    false,
                    ConversationPayload::NetworkApproval {
                        host: inner.host.clone(),
                        suggestion_count: inner.suggestions.len(),
                    },
                    now,
                );
            }
            UiEvent::AdaptiveCard {
                node_id,
                parent_id,
                json,
            } => {
                self.insert_node(
                    *node_id,
                    *parent_id,
                    ConversationNodeKind::AdaptiveCard,
                    false,
                    ConversationPayload::AdaptiveCardJson(json.clone()),
                    now,
                );
            }
            UiEvent::Info { node_id, text } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Info,
                    false,
                    ConversationPayload::Text(text.clone()),
                    now,
                );
            }
            UiEvent::Error { node_id, text } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Error,
                    false,
                    ConversationPayload::Text(text.clone()),
                    now,
                );
            }
            UiEvent::Done { node_id } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Done,
                    false,
                    ConversationPayload::Empty,
                    now,
                );
            }
            UiEvent::Interrupted { node_id } => {
                self.insert_node(
                    *node_id,
                    None,
                    ConversationNodeKind::Interrupted,
                    false,
                    ConversationPayload::Empty,
                    now,
                );
            }
            // Branch topology is handled by the agent core; tree ignores this event for now.
            UiEvent::BranchStarted { .. } => {}
            // PaneContent is a side-channel; TUI/web UI maintain their own pane map.
            UiEvent::PaneContent { .. } => {}
            // TurnStats is a UI-only signal; not a conversation node.
            UiEvent::TurnStats { .. } => {}
            // Judge events are TUI-only signals; not conversation nodes.
            UiEvent::JudgeScoreReady { .. } | UiEvent::JudgeError { .. } => {}
            // AwaitingApiKey is a UI-only signal; not a conversation node.
            UiEvent::AwaitingApiKey { .. } => {}
        }
    }

    fn insert_node(
        &mut self,
        id: NodeId,
        parent_id: Option<NodeId>,
        kind: ConversationNodeKind,
        collapsed_by_default: bool,
        payload: ConversationPayload,
        timestamp: DateTime<Utc>,
    ) {
        let node = ConversationNode {
            id,
            parent_id,
            kind,
            timestamp,
            collapsed_by_default,
            payload,
        };
        self.nodes.insert(id, node);
        self.children.entry(parent_id).or_default().push(id);
    }

    fn append_text(
        &mut self,
        id: NodeId,
        delta: &str,
        kind: ConversationNodeKind,
        timestamp: DateTime<Utc>,
    ) {
        if let Some(node) = self.nodes.get_mut(&id) {
            match &mut node.payload {
                ConversationPayload::Text(t) => t.push_str(delta),
                ConversationPayload::Empty => {
                    node.payload = ConversationPayload::Text(delta.to_string());
                }
                _ => {
                    node.payload = ConversationPayload::Text(delta.to_string());
                }
            }
            node.kind = kind;
            node.timestamp = timestamp;
            return;
        }
        self.insert_node(
            id,
            None,
            kind,
            false,
            ConversationPayload::Text(delta.to_string()),
            timestamp,
        );
    }

    fn set_full_text(
        &mut self,
        id: NodeId,
        text: &str,
        kind: ConversationNodeKind,
        collapsed_by_default: bool,
        timestamp: DateTime<Utc>,
    ) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.payload = ConversationPayload::Text(text.to_string());
            node.kind = kind;
            node.collapsed_by_default = collapsed_by_default;
            node.timestamp = timestamp;
            return;
        }
        self.insert_node(
            id,
            None,
            kind,
            collapsed_by_default,
            ConversationPayload::Text(text.to_string()),
            timestamp,
        );
    }

    fn patch_action_end(
        &mut self,
        id: NodeId,
        stdout: &str,
        stderr: &str,
        exit_code: Option<i32>,
        collapsed_by_default: bool,
        timestamp: DateTime<Utc>,
    ) {
        if let Some(node) = self.nodes.get_mut(&id) {
            if let ConversationPayload::Action {
                lang: _,
                code: _,
                stdout: so,
                stderr: se,
                exit_code: ec,
            } = &mut node.payload
            {
                so.clear();
                so.push_str(stdout);
                se.clear();
                se.push_str(stderr);
                *ec = exit_code;
            } else {
                node.payload = ConversationPayload::Action {
                    lang: String::new(),
                    code: String::new(),
                    stdout: stdout.to_string(),
                    stderr: stderr.to_string(),
                    exit_code,
                };
            }
            node.collapsed_by_default = collapsed_by_default;
            node.kind = ConversationNodeKind::Action;
            node.timestamp = timestamp;
            return;
        }
        self.insert_node(
            id,
            None,
            ConversationNodeKind::Action,
            collapsed_by_default,
            ConversationPayload::Action {
                lang: String::new(),
                code: String::new(),
                stdout: stdout.to_string(),
                stderr: stderr.to_string(),
                exit_code,
            },
            timestamp,
        );
    }

    fn touch_timestamp(&mut self, id: NodeId, timestamp: DateTime<Utc>) {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.timestamp = timestamp;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_event::IdGen;

    #[test]
    fn ordered_children_under_parent() {
        let mut tree = ConversationTree::new();
        let ids = IdGen::new();

        let root = ids.next();
        let a = ids.next();
        let b = ids.next();

        tree.apply(&UiEvent::SessionStarted {
            node_id: root,
            model: Some("m".into()),
            sandbox_image: None,
            provider_info: None,
        });
        tree.apply(&UiEvent::UserTurn {
            node_id: a,
            parent_id: Some(root),
            text: "hello".into(),
        });
        tree.apply(&UiEvent::UserTurn {
            node_id: b,
            parent_id: Some(root),
            text: "again".into(),
        });

        let kids = tree.children_ids(root);
        assert_eq!(kids, &[a, b]);
    }

    #[test]
    fn action_collapsed_by_default_flag_on_events() {
        let mut tree = ConversationTree::new();
        let ids = IdGen::new();
        let n = ids.next();
        tree.apply(&UiEvent::ActionStart {
            node_id: n,
            parent_id: None,
            lang: "py".into(),
            code: "print(1)".into(),
            collapsed_by_default: true,
        });
        assert!(tree.node(n).unwrap().collapsed_by_default);
        tree.apply(&UiEvent::ActionEnd {
            node_id: n,
            stdout: "1\n".into(),
            stderr: "".into(),
            exit_code: Some(0),
            collapsed_by_default: true,
        });
        assert!(tree.node(n).unwrap().collapsed_by_default);
    }

    #[test]
    fn tokens_append_to_same_assistant_node() {
        let mut tree = ConversationTree::new();
        let ids = IdGen::new();
        let ast = ids.next();
        tree.apply(&UiEvent::Token {
            node_id: ast,
            text: "hel".into(),
        });
        tree.apply(&UiEvent::Token {
            node_id: ast,
            text: "lo".into(),
        });
        assert_eq!(
            tree.node(ast).unwrap().payload,
            ConversationPayload::Text("hello".into())
        );
    }
}
