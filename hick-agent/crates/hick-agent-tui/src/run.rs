//! Shared Ratatui TUI consuming [`UiEvent`] / emitting [`UiCommand`], backed by [`ConversationTree`].

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use tokio::sync::{Notify, mpsc, oneshot};

use hick_agent_sdk::mcp_install::{
    McpCredentialRequest, McpInstallSpec, McpInstaller, infer_name, list_servers, parse_spec,
    remove_server,
};
use hick_agent_sdk::mcp_config::McpServerTrust;
use hick_agent_sdk::network_filter::{ApprovalResponse, NetworkVerdict};
use hick_agent_sdk::{
    ConversationNode, ConversationNodeKind, ConversationPayload, ConversationTree, IdGen, NodeId,
    UiCommand, UiEvent, UiNetworkApproval,
};

// ── Styled output model ────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
enum LineKind {
    Info,
    User,
    Assistant,
    Output,
    Error,
    Separator,
}

struct StyledLine {
    kind: LineKind,
    text: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FoldOverride {
    Expanded,
    Collapsed,
}

/// Configuration forwarded from each binary crate.
#[derive(Clone)]
pub struct TuiConfig {
    pub agent_name: String,
    pub provider_info: String,
    pub project_dir: PathBuf,
    pub interrupt: Arc<Notify>,
    pub ui_fanout_tx: mpsc::UnboundedSender<UiEvent>,
}

// ── Application state ──────────────────────────────────────────────

/// Pending network approval; wraps [`UiNetworkApproval`] so clones share completion state.
struct PendingApproval {
    inner: UiNetworkApproval,
    choosing_suggestion: bool,
}

enum CredentialStage {
    EnvVarName,
    Value(String),
}

struct PendingMcpCredential {
    server_name: String,
    stage: CredentialStage,
    response_tx: oneshot::Sender<Option<(String, String)>>,
}

struct App {
    tree: ConversationTree,
    fold: HashMap<NodeId, FoldOverride>,
    id_gen: IdGen,
    preorder_ids: Vec<NodeId>,
    selected_ix: usize,
    pending_turn_sep: bool,
    pulse_counter: usize,
    lines: Vec<StyledLine>,
    input: String,
    input_cursor: usize,
    scroll: usize,
    auto_scroll: bool,
    busy: bool,
    config: TuiConfig,
    should_quit: bool,
    approvals_q: VecDeque<UiNetworkApproval>,
    pending_approval: Option<PendingApproval>,
    pending_mcp_credential: Option<PendingMcpCredential>,
}

impl App {
    fn new(config: TuiConfig) -> Self {
        Self {
            tree: ConversationTree::new(),
            fold: HashMap::new(),
            id_gen: IdGen::new(),
            preorder_ids: Vec::new(),
            selected_ix: 0,
            pending_turn_sep: false,
            pulse_counter: 0,
            lines: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            scroll: 0,
            auto_scroll: true,
            busy: false,
            config,
            should_quit: false,
            approvals_q: VecDeque::new(),
            pending_approval: None,
            pending_mcp_credential: None,
        }
    }

    fn misc_info(&mut self, text: impl Into<String>) {
        let node_id = self.id_gen.next();
        self.ingest(&UiEvent::Info {
            node_id,
            text: text.into(),
        });
    }

    fn misc_error(&mut self, text: impl Into<String>) {
        let node_id = self.id_gen.next();
        self.ingest(&UiEvent::Error {
            node_id,
            text: text.into(),
        });
    }

    fn ingest(&mut self, ev: &UiEvent) {
        if matches!(ev, UiEvent::UserTurn { .. }) && self.pending_turn_sep {
            let node_id = self.id_gen.next();
            self.tree.apply(&UiEvent::Info {
                node_id,
                text: "─────────────────────────────────".into(),
            });
            self.pending_turn_sep = false;
        }

        match ev {
            UiEvent::NetworkApproval(inner) => {
                self.approvals_q.push_back(inner.clone());
            }
            UiEvent::Done { .. } | UiEvent::Interrupted { .. } => {
                self.pending_turn_sep = true;
                self.busy = false;
            }
            UiEvent::Error { .. } => self.busy = false,
            UiEvent::Info { text, .. } => {
                if text.starts_with("[mcp install]") && text.contains("complete") {
                    self.busy = false;
                }
            }
            _ => {}
        }

        self.tree.apply(ev);
        self.refresh_preorder();
    }

    fn refresh_preorder(&mut self) {
        self.preorder_ids.clear();

        fn walk(tree: &ConversationTree, id: NodeId, out: &mut Vec<NodeId>) {
            out.push(id);
            for &c in tree.children_ids(id) {
                walk(tree, c, out);
            }
        }
        for &r in self.tree.roots() {
            walk(&self.tree, r, &mut self.preorder_ids);
        }
        if !self.preorder_ids.is_empty() && self.selected_ix >= self.preorder_ids.len() {
            self.selected_ix = self.preorder_ids.len() - 1;
        }
    }

    fn selected_node(&self) -> Option<NodeId> {
        self.preorder_ids.get(self.selected_ix).copied()
    }

    fn graph_prefix(depth: usize, sib_ix: usize, sib_n: usize) -> String {
        let mut out = String::new();
        for _ in 0..depth.saturating_sub(1) {
            out.push_str("\u{2502} ");
        }
        if depth > 0 {
            out.push_str(if sib_ix + 1 == sib_n {
                "\u{2514}\u{2500} "
            } else {
                "\u{251c}\u{2500} "
            });
        }
        out
    }

    fn collapsed(&self, id: NodeId, node: &ConversationNode) -> bool {
        let flag = node.collapsed_by_default;
        let kind = node.kind;
        let base = match kind {
            ConversationNodeKind::Thinking => true,
            ConversationNodeKind::Action
            | ConversationNodeKind::McpCall
            | ConversationNodeKind::McpResponse => flag,
            _ => false,
        };
        match self.fold.get(&id) {
            Some(FoldOverride::Expanded) => false,
            Some(FoldOverride::Collapsed) => true,
            None => base,
        }
    }

    fn collapsible(kind: ConversationNodeKind) -> bool {
        matches!(
            kind,
            ConversationNodeKind::Thinking
                | ConversationNodeKind::Assistant
                | ConversationNodeKind::Action
                | ConversationNodeKind::McpCall
                | ConversationNodeKind::McpResponse
                | ConversationNodeKind::AdaptiveCard
        )
    }

    fn pulse_str(&self) -> String {
        const FRAMES: &[&str] = &[
            "\u{25cf}\u{25cf}\u{25cf}",
            "\u{25cb}\u{25cf}\u{25cf}",
            "\u{25cf}\u{25cb}\u{25cf}",
            "\u{25cf}\u{25cf}\u{25cb}",
        ];
        FRAMES[self.pulse_counter % FRAMES.len()].to_string()
    }

    fn rebuild_screen(&mut self) {
        let mut rows: Vec<StyledLine> = Vec::new();

        let roots: Vec<NodeId> = self.tree.roots().to_vec();
        let total = roots.len().max(1);
        for (i, rid) in roots.into_iter().enumerate() {
            self.visit_emit(rid, 0, i, total, &mut rows);
        }
        self.lines = rows;
        if self.auto_scroll {
            self.scroll = usize::MAX;
        }
    }

    fn bump_collapsible_focus(&mut self, delta: isize) {
        if self.preorder_ids.is_empty() {
            return;
        }
        let n = self.preorder_ids.len();
        let mut ix = self.selected_ix;
        for _ in 0..n {
            ix = ((ix as isize + delta).rem_euclid(n as isize)) as usize;
            if let Some(node) = self.tree.node(self.preorder_ids[ix]) {
                let k = node.kind;
                if Self::collapsible(k) {
                    self.selected_ix = ix;
                    return;
                }
            }
        }
    }

    fn toggle_selected_fold(&mut self) {
        let Some(sel) = self.selected_node() else {
            return;
        };
        let Some(node_ref) = self.tree.node(sel) else {
            return;
        };
        if !Self::collapsible(node_ref.kind) {
            return;
        }
        let collapsed_now = self.collapsed(sel, node_ref);
        self.fold.insert(
            sel,
            if collapsed_now {
                FoldOverride::Expanded
            } else {
                FoldOverride::Collapsed
            },
        );
    }

    fn pump_network_prompt(&mut self) {
        if self.pending_approval.is_none()
            && let Some(inner) = self.approvals_q.pop_front()
        {
            self.pending_approval = Some(PendingApproval {
                inner,
                choosing_suggestion: false,
            });
        }
    }

    #[allow(clippy::too_many_lines)]
    fn visit_emit(&self, id: NodeId, depth: usize, sib_ix: usize, sib_n: usize, out: &mut Vec<StyledLine>) {
        let Some(node) = self.tree.node(id).cloned() else {
            return;
        };
        let pref = Self::graph_prefix(depth, sib_ix, sib_n);
        let folded = self.collapsed(id, &node);
        let collapsible_flag = Self::collapsible(node.kind);
        let chev = if !collapsible_flag {
            ""
        } else if folded {
            "\u{25b8} "
        } else {
            "\u{25be} "
        };
        let sel_suffix = || {
            if Some(id) == self.selected_node() {
                " ◀"
            } else {
                ""
            }
        };

        let mut push_row_tagged = |kind: LineKind, text: String| {
            out.push(StyledLine {
                kind,
                text: format!("{}{}", text, sel_suffix()),
            });
        };

        match node.kind {
            ConversationNodeKind::Session => {
                if let ConversationPayload::Session {
                    model,
                    sandbox_image,
                    provider_info,
                } = node.payload
                {
                    let mut bits = Vec::new();
                    if let Some(m) = model {
                        bits.push(format!("model {m}"));
                    }
                    if let Some(si) = sandbox_image {
                        bits.push(format!("sandbox {si}"));
                    }
                    if let Some(pi) = provider_info {
                        bits.push(pi);
                    }
                    push_row_tagged(
                        LineKind::Info,
                        format!("{pref}\u{25cb} Session — {}", bits.join(" | ")),
                    );
                }
            }
            ConversationNodeKind::UserTurn => {
                let text = match &node.payload {
                    ConversationPayload::Text(t) => t.clone(),
                    _ => String::new(),
                };
                push_row_tagged(LineKind::User, format!("{pref}{chev}>{text}"));
            }
            ConversationNodeKind::Thinking => {
                push_row_tagged(LineKind::Info, format!("{pref}{chev}{}", self.pulse_str()));
            }
            ConversationNodeKind::Assistant => {
                let body = match &node.payload {
                    ConversationPayload::Text(t) => t.clone(),
                    _ => String::new(),
                };
                if folded {
                    if body.trim().is_empty() {
                        push_row_tagged(
                            LineKind::Assistant,
                            format!("{pref}{chev}{}", self.pulse_str()),
                        );
                    } else {
                        let peek: String = body.lines().next().unwrap_or("").chars().take(160).collect();
                        push_row_tagged(LineKind::Assistant, format!("{pref}{chev}{peek}"));
                    }
                } else if body.trim().is_empty() {
                    push_row_tagged(
                        LineKind::Assistant,
                        format!("{pref}{chev}\u{25cf} thinking {}", self.pulse_str()),
                    );
                } else {
                    push_row_tagged(LineKind::Assistant, format!("{pref}{chev}"));
                    let pad = pref.chars().count() + chev.chars().count();
                    let spacer = " ".repeat(pad);
                    for ln in body.lines() {
                        out.push(StyledLine {
                            kind: LineKind::Assistant,
                            text: format!("{spacer}{ln}"),
                        });
                    }
                }
            }
            ConversationNodeKind::Action => {
                if let ConversationPayload::Action {
                    lang,
                    code,
                    stdout,
                    stderr,
                    exit_code,
                } = node.payload
                {
                    if folded {
                        let head =
                            format!("[{lang}] {}", code.lines().next().unwrap_or(""))
                                .chars()
                                .take(180)
                                .collect::<String>();
                        push_row_tagged(LineKind::Info, format!("{pref}{chev}{head}"));
                    } else {
                        push_row_tagged(
                            LineKind::Info,
                            format!("{pref}{chev}Script [{lang}]"),
                        );
                        let pad = pref.chars().count() + chev.chars().count();
                        let spacer = " ".repeat(pad + 2);
                        for ln in code.lines() {
                            out.push(StyledLine {
                                kind: LineKind::Assistant,
                                text: format!("{spacer}{ln}"),
                            });
                        }
                        out.push(StyledLine {
                            kind: LineKind::Separator,
                            text: format!(
                                "{}output",
                                " ".repeat(pref.chars().count())
                            ),
                        });
                        let out_spacer = " ".repeat(pad + 2);
                        if !stdout.trim().is_empty() {
                            for ln in stdout.lines() {
                                out.push(StyledLine {
                                    kind: LineKind::Output,
                                    text: format!("{out_spacer}{ln}"),
                                });
                            }
                        }
                        if !stderr.trim().is_empty() {
                            for ln in stderr.lines() {
                                out.push(StyledLine {
                                    kind: LineKind::Error,
                                    text: format!("{out_spacer}{ln}"),
                                });
                            }
                        }
                        let exit_txt = exit_code.map_or_else(|| "?".into(), |c| c.to_string());
                        out.push(StyledLine {
                            kind: if exit_code != Some(0) {
                                LineKind::Error
                            } else {
                                LineKind::Info
                            },
                            text: format!(
                                "{}exit {}",
                                " ".repeat(pref.chars().count()),
                                exit_txt
                            ),
                        });
                    }
                }
            }
            ConversationNodeKind::SubAgent => {
                let label = match &node.payload {
                    ConversationPayload::SubAgent { label } => label.clone(),
                    _ => "sub-agent".into(),
                };
                push_row_tagged(LineKind::Info, format!("{pref}{chev}\u{25cb} {label}"));
            }
            ConversationNodeKind::McpCall | ConversationNodeKind::McpResponse => {
                if let ConversationPayload::Mcp {
                    server,
                    method,
                    is_response,
                } = node.payload
                {
                    let tag = if is_response {
                        format!("<- {server}/{method}")
                    } else {
                        format!("-> {server}/{method}")
                    };
                    push_row_tagged(LineKind::Info, format!("{pref}{chev}{tag}"));
                }
            }
            ConversationNodeKind::Security => {
                let text = match &node.payload {
                    ConversationPayload::Security { description } => description.clone(),
                    _ => String::new(),
                };
                push_row_tagged(
                    LineKind::Error,
                    format!("{pref}{chev}\u{26a0} SECURITY: {text}"),
                );
            }
            ConversationNodeKind::NetworkApproval => {
                let host = match &node.payload {
                    ConversationPayload::NetworkApproval { host, .. } => host.clone(),
                    _ => "unknown".into(),
                };
                push_row_tagged(LineKind::Info, format!("{pref}{chev}[network] {host}"));
            }
            ConversationNodeKind::AdaptiveCard => {
                let js = match &node.payload {
                    ConversationPayload::AdaptiveCardJson(j) => j.clone(),
                    _ => "{}".into(),
                };
                if folded {
                    let head: String = js.chars().take(120).collect();
                    push_row_tagged(
                        LineKind::Info,
                        format!("{pref}{chev}AdaptiveCard {head}\u{2026}"),
                    );
                } else {
                    push_row_tagged(LineKind::Info, format!("{pref}{chev}AdaptiveCard"));
                    let pad = pref.chars().count() + chev.chars().count();
                    let spacer = " ".repeat(pad + 2);
                    for ln in js.lines().take(64) {
                        out.push(StyledLine {
                            kind: LineKind::Info,
                            text: format!("{spacer}{ln}"),
                        });
                    }
                }
            }
            ConversationNodeKind::Info => {
                let text = match &node.payload {
                    ConversationPayload::Text(t) => t.clone(),
                    _ => String::new(),
                };
                push_row_tagged(LineKind::Info, format!("{pref}{chev}{text}"));
            }
            ConversationNodeKind::Error => {
                let text = match &node.payload {
                    ConversationPayload::Text(t) => t.clone(),
                    _ => String::new(),
                };
                push_row_tagged(LineKind::Error, format!("{pref}{chev}{text}"));
            }
            ConversationNodeKind::Done => {
                push_row_tagged(LineKind::Separator, format!("{pref}{chev}\u{25cb} Done"));
            }
            ConversationNodeKind::Interrupted => {
                push_row_tagged(LineKind::Info, format!("{pref}{chev}\u{25cf} Interrupted"));
            }
        }

        let children = self.tree.children_ids(id).to_vec();
        let n = children.len();
        for (ci, cid) in children.into_iter().enumerate() {
            self.visit_emit(cid, depth + 1, ci, n, out);
        }
    }


    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        cmd_tx: &mpsc::UnboundedSender<UiCommand>,
        cred_req_tx: &mpsc::UnboundedSender<McpCredentialRequest>,
    ) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        let interrupt = &self.config.interrupt;

        if let Some(ref _pending) = self.pending_mcp_credential {
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    if let Some(p) = self.pending_mcp_credential.take() {
                        p.response_tx.send(None).ok();
                    }
                    self.misc_info("[mcp] Credential entry cancelled.");
                    self.busy = false;
                }
                (_, KeyCode::Enter) => {
                    let input = std::mem::take(&mut self.input);
                    self.input_cursor = 0;
                    let pending = self.pending_mcp_credential.take().unwrap();
                    match pending.stage {
                        CredentialStage::EnvVarName => {
                            if input.is_empty() {
                                pending.response_tx.send(None).ok();
                                self.misc_info("[mcp] No credential — skipped.");
                                self.busy = false;
                            } else {
                                let env_var = input.clone();
                                self.misc_info(format!(
                                    "[mcp] Enter value for {env_var} (blank to skip):"
                                ));
                                self.pending_mcp_credential = Some(PendingMcpCredential {
                                    server_name: pending.server_name,
                                    stage: CredentialStage::Value(env_var),
                                    response_tx: pending.response_tx,
                                });
                            }
                        }
                        CredentialStage::Value(env_var) => {
                            if input.is_empty() {
                                pending.response_tx.send(None).ok();
                                self.misc_info("[mcp] No credential — skipped.");
                            } else {
                                pending.response_tx.send(Some((env_var, input))).ok();
                            }
                            self.busy = false;
                        }
                    }
                }
                (_, KeyCode::Backspace) => {
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                        self.input.remove(self.input_cursor);
                    }
                }
                (_, KeyCode::Left) => {
                    self.input_cursor = self.input_cursor.saturating_sub(1);
                }
                (_, KeyCode::Right) => {
                    if self.input_cursor < self.input.len() {
                        self.input_cursor += 1;
                    }
                }
                (_, KeyCode::Home) => self.input_cursor = 0,
                (_, KeyCode::End) => self.input_cursor = self.input.len(),
                (_, KeyCode::Char(c)) => {
                    self.input.insert(self.input_cursor, c);
                    self.input_cursor += 1;
                }
                _ => {}
            }
            return;
        }

        if self.pending_approval.is_some() {
            let choosing = self
                .pending_approval
                .as_ref()
                .map(|p| p.choosing_suggestion)
                .unwrap_or(false);

            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    drop(self.pending_approval.take());
                    if self.busy {
                        interrupt.notify_one();
                    } else {
                        cmd_tx.send(UiCommand::Quit).ok();
                        self.should_quit = true;
                    }
                }
                (_, KeyCode::Char('y')) | (_, KeyCode::Char('Y')) if !choosing => {
                    if let Some(p) = self.pending_approval.take() {
                        let _ = p.inner.respond(ApprovalResponse {
                            verdict: NetworkVerdict::Allow,
                            always_allow: None,
                        });
                        self.misc_info("[network] Allowed (this request only)");
                    }
                }
                (_, KeyCode::Char('a')) | (_, KeyCode::Char('A')) if !choosing => {
                    let descs: Vec<String> = self
                        .pending_approval
                        .as_ref()
                        .map(|p| {
                            p.inner
                                .suggestions
                                .iter()
                                .enumerate()
                                .map(|(i, s)| format!("  [{}] {}", i + 1, s.describe()))
                                .collect()
                        })
                        .unwrap_or_default();
                    if let Some(ref mut p) = self.pending_approval {
                        p.choosing_suggestion = true;
                    }
                    if !descs.is_empty() {
                        self.misc_info("[network] Choose a rule to always allow (type a number):");
                        for d in descs {
                            self.misc_info(d);
                        }
                    }
                }
                (_, KeyCode::Char(c)) if choosing && c.is_ascii_digit() => {
                    let idx = c.to_digit(10).unwrap_or(0) as usize;
                    if let Some(p) = self.pending_approval.take() {
                        let n = p.inner.suggestions.len();
                        if idx >= 1 && idx <= n {
                            let suggestion = p.inner.suggestions[idx - 1].clone();
                            let desc = suggestion.describe();
                            let _ = p.inner.respond(ApprovalResponse {
                                verdict: NetworkVerdict::Allow,
                                always_allow: Some(suggestion),
                            });
                            self.misc_info(format!("[network] Always allow: {desc}"));
                        } else {
                            self.misc_info("[network] Invalid selection");
                            let _ = p.inner.respond(ApprovalResponse {
                                verdict: NetworkVerdict::Block,
                                always_allow: None,
                            });
                            self.misc_info("[network] Blocked (invalid selection)");
                        }
                    }
                }
                (_, KeyCode::Char('n')) | (_, KeyCode::Char('N')) if !choosing => {
                    if let Some(p) = self.pending_approval.take() {
                        let _ = p.inner.respond(ApprovalResponse {
                            verdict: NetworkVerdict::Block,
                            always_allow: None,
                        });
                        self.misc_info("[network] Blocked");
                    }
                }
                _ => {}
            }
            return;
        }

        match (key.modifiers, key.code) {
            (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                if self.busy {
                    interrupt.notify_one();
                } else {
                    cmd_tx.send(UiCommand::Quit).ok();
                    self.should_quit = true;
                }
            }
            (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => {
                cmd_tx.send(UiCommand::Quit).ok();
                self.should_quit = true;
            }
            (_, KeyCode::Enter) => {
                if self.input.is_empty() && !self.busy {
                    self.toggle_selected_fold();
                    return;
                }
                if !self.busy && !self.input.is_empty() {
                    let msg = std::mem::take(&mut self.input);
                    self.input_cursor = 0;

                    if msg.eq_ignore_ascii_case("quit") || msg.eq_ignore_ascii_case("exit") {
                        cmd_tx.send(UiCommand::Quit).ok();
                        self.should_quit = true;
                        return;
                    }

                    if msg.starts_with("/mcp ") || msg == "/mcp" {
                        self.handle_mcp_slash(msg[4..].trim(), cred_req_tx);
                        return;
                    }

                    if msg.starts_with('!') {
                        let shell_cmd = msg[1..].trim().to_string();
                        if shell_cmd.is_empty() {
                            self.misc_info("Usage: ! <shell command>");
                            return;
                        }
                        cmd_tx
                            .send(UiCommand::DirectCommand(shell_cmd))
                            .ok();
                        self.busy = true;
                        self.auto_scroll = true;
                        return;
                    }

                    cmd_tx.send(UiCommand::Message(msg)).ok();
                    self.busy = true;
                    self.auto_scroll = true;
                }
            }
            (_, KeyCode::Char(' ')) if self.input.is_empty() && !self.busy => {
                self.toggle_selected_fold();
            }
            (_, KeyCode::Up) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.bump_collapsible_focus(-1);
            }
            (_, KeyCode::Down) if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.bump_collapsible_focus(1);
            }
            (_, KeyCode::Backspace) => {
                if !self.busy && self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                }
            }
            (_, KeyCode::Left) => {
                self.input_cursor = self.input_cursor.saturating_sub(1);
            }
            (_, KeyCode::Right) => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            (_, KeyCode::Home) => self.input_cursor = 0,
            (_, KeyCode::End) => self.input_cursor = self.input.len(),
            (_, KeyCode::Up) => {
                self.auto_scroll = false;
                self.scroll = self.scroll.saturating_sub(1);
            }
            (_, KeyCode::Down) => {
                self.scroll = self.scroll.saturating_add(1);
            }
            (_, KeyCode::PageUp) => {
                self.auto_scroll = false;
                self.scroll = self.scroll.saturating_sub(20);
            }
            (_, KeyCode::PageDown) => {
                self.scroll = self.scroll.saturating_add(20);
            }
            (_, KeyCode::Char(c)) if !self.busy => {
                self.input.insert(self.input_cursor, c);
                self.input_cursor += 1;
            }
            _ => {}
        }
    }

    fn handle_mcp_slash(
        &mut self,
        rest: &str,
        cred_req_tx: &mpsc::UnboundedSender<McpCredentialRequest>,
    ) {
        if self.busy {
            self.misc_info("[mcp] Command rejected — agent is busy.");
            return;
        }

        let project_dir = self.config.project_dir.clone();
        let (sub, arg) = match rest.split_once(' ') {
            Some((s, a)) => (s.trim(), a.trim()),
            None => (rest.trim(), ""),
        };

        match sub {
            "list" => match list_servers(&project_dir) {
                Ok(rows) => {
                    for row in rows {
                        self.misc_info(row);
                    }
                }
                Err(e) => self.misc_error(format!("[mcp] {e}")),
            },
            "remove" => {
                let name = arg.trim();
                if name.is_empty() {
                    self.misc_info("Usage: /mcp remove <name>");
                    return;
                }
                match remove_server(name, &project_dir) {
                    Ok(()) => self.misc_info(format!("[mcp] Removed '{name}'.")),
                    Err(e) => self.misc_error(format!("[mcp] {e}")),
                }
            }
            "install" => {
                if arg.is_empty() {
                    self.misc_info(
                        "Usage: /mcp install <spec> [--name <n>] [--trust trusted|untrusted|prompt]",
                    );
                    return;
                }

                let (raw_spec, name_override, trust) = parse_mcp_install_args(arg);
                let spec = parse_spec(raw_spec);
                let server_name = name_override
                    .map(|s| s.to_string())
                    .or_else(|| infer_name(&spec))
                    .unwrap_or_else(|| "mcp-server".to_string());

                self.misc_info(format!("[mcp] Installing '{server_name}'..."));
                self.busy = true;

                let ui_tx = self.config.ui_fanout_tx.clone();
                let cred = cred_req_tx.clone();
                tokio::runtime::Handle::current().spawn(async move {
                    run_mcp_install_task(spec, server_name, trust, project_dir, ui_tx, cred).await;
                });
            }
            _ => self.misc_info("Usage: /mcp install <spec> | /mcp list | /mcp remove <name>"),
        }
    }

    // ── Rendering ──────────────────────────────────────────────────

    fn render(&mut self, frame: &mut Frame) {
        self.rebuild_screen();
        self.pump_network_prompt();
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(1),    // output
            Constraint::Length(3), // input
        ])
        .split(area);

        // Header bar
        let agent = self.config.agent_name.clone();
        let provider_info = self.config.provider_info.clone();
        let header_spans = vec![
            Span::styled(
                agent,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
            Span::styled(provider_info, Style::default().fg(Color::DarkGray)),
        ];
        let header = Paragraph::new(Line::from(header_spans));
        frame.render_widget(header, chunks[0]);

        // Build output text
        let mut text_lines: Vec<Line> = Vec::new();
        for line in &self.lines {
            let style = match line.kind {
                LineKind::Info => Style::default().fg(Color::DarkGray),
                LineKind::User => Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
                LineKind::Assistant => Style::default().fg(Color::White),
                LineKind::Output => Style::default().fg(Color::Green),
                LineKind::Error => Style::default().fg(Color::Red),
                LineKind::Separator => Style::default().fg(Color::DarkGray),
            };
            text_lines.push(Line::styled(line.text.clone(), style));
        }

        let visible_height = chunks[1].height.saturating_sub(2) as usize;
        let inner_width = chunks[1].width.saturating_sub(2) as usize;

        let total_visual_rows: usize = text_lines
            .iter()
            .map(|line| {
                let char_count: usize = line.spans.iter().map(|s| s.content.chars().count()).sum();
                if inner_width == 0 {
                    1
                } else if char_count == 0 {
                    1
                } else {
                    char_count.div_ceil(inner_width)
                }
            })
            .sum::<usize>()
            .max(1);

        let max_scroll = total_visual_rows.saturating_sub(visible_height);
        if self.auto_scroll {
            self.scroll = max_scroll;
        } else {
            self.scroll = self.scroll.min(max_scroll);
            if self.scroll >= max_scroll {
                self.auto_scroll = true;
            }
        }

        let output = Paragraph::new(Text::from(text_lines))
            .scroll((self.scroll as u16, 0))
            .wrap(Wrap { trim: false })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(output, chunks[1]);

        let input_display = if let Some(ref pm) = self.pending_mcp_credential {
            match &pm.stage {
                CredentialStage::EnvVarName => format!(
                    "[mcp] {} — enter credential env var name (blank to skip): {}",
                    pm.server_name, self.input
                ),
                CredentialStage::Value(env_var) => format!(
                    "[mcp] Enter value for {} (blank to skip): {}",
                    env_var, self.input
                ),
            }
        } else if let Some(ref p) = self.pending_approval {
            if p.choosing_suggestion {
                "\u{26a0} Type a number to select a rule, or Ctrl+C to cancel".to_string()
            } else {
                format!(
                    "\u{26a0} Allow {}? \u{2014} y = once, a = always (with rule), n = block",
                    p.inner.host
                )
            }
        } else if self.busy {
            "\u{25cf} Agent is working... (Ctrl+C to interrupt)".to_string()
        } else {
            format!("> {}", self.input)
        };

        let input_style = if self.pending_mcp_credential.is_some() {
            Style::default().fg(Color::Magenta)
        } else if self.pending_approval.is_some() {
            Style::default().fg(Color::Yellow)
        } else if self.busy {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Cyan)
        };

        let input_widget = Paragraph::new(input_display).style(input_style).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(input_widget, chunks[2]);

        if !self.busy {
            let cursor_x = chunks[2].x + 1 + 2 + self.input_cursor as u16;
            let cursor_y = chunks[2].y + 1;
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

// ── /mcp helpers ───────────────────────────────────────────────────

fn parse_mcp_install_args(raw: &str) -> (&str, Option<&str>, McpServerTrust) {
    let mut name: Option<&str> = None;
    let mut trust = McpServerTrust::Prompt;
    let mut spec_end = raw.len();

    let mut i = 0;
    let bytes = raw.as_bytes();
    while i < raw.len() {
        if raw[i..].starts_with("--name ") {
            spec_end = spec_end.min(i);
            let rest = &raw[i + 7..];
            let end = rest.find(' ').unwrap_or(rest.len());
            name = Some(rest[..end].trim_matches('"'));
            i += 7 + end;
        } else if raw[i..].starts_with("--trust ") {
            spec_end = spec_end.min(i);
            let rest = &raw[i + 8..];
            let end = rest.find(' ').unwrap_or(rest.len());
            trust = match rest[..end].trim() {
                "trusted" => McpServerTrust::Trusted,
                "untrusted" => McpServerTrust::Untrusted,
                _ => McpServerTrust::Prompt,
            };
            i += 8 + end;
        } else {
            i += bytes[i].leading_ones().max(1) as usize;
        }
    }

    let spec = raw[..spec_end].trim().trim_matches('"');
    (spec, name, trust)
}

async fn run_mcp_install_task(
    spec: McpInstallSpec,
    server_name: String,
    trust: McpServerTrust,
    project_dir: std::path::PathBuf,
    ui_tx: mpsc::UnboundedSender<UiEvent>,
    cred_req_tx: mpsc::UnboundedSender<McpCredentialRequest>,
) {
    let ids = IdGen::new();
    let send_info = |text: String| {
        let _ = ui_tx.send(UiEvent::Info {
            node_id: ids.next(),
            text,
        });
    };
    let send_err = |text: String| {
        let _ = ui_tx.send(UiEvent::Error {
            node_id: ids.next(),
            text,
        });
    };

    send_info("Skipping WASM package install (server must be available on PATH).".into());
    send_info("Verifying server connection...".into());
    let tools = match McpInstaller::verify_connection(&spec).await {
        Ok(t) => t,
        Err(e) => {
            send_err(format!("[mcp] Connection failed: {e}"));
            send_info(format!(
                "[mcp install] complete — {} tool(s):",
                0_usize,
            ));
            return;
        }
    };

    if let Err(e) = McpInstaller::write_config(&spec, &server_name, trust, &project_dir) {
        send_err(format!("[mcp] Config write failed: {e}"));
        send_info(format!(
            "[mcp install] complete — {} tool(s):",
            0_usize,
        ));
        return;
    }

    let (response_tx, response_rx) = tokio::sync::oneshot::channel();
    let suggested = format!(
        "{}_API_KEY",
        server_name.to_uppercase().replace('-', "_")
    );
    if cred_req_tx
        .send(McpCredentialRequest {
            server_name: server_name.clone(),
            suggested_env_var: suggested,
            response_tx,
        })
        .is_ok()
    {
        if let Ok(Some((env_var, value))) = response_rx.await {
            McpInstaller::write_credentials(&server_name, vec![(env_var, value)], &project_dir).ok();
        }
    }

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    send_info(format!(
        "[mcp install] complete — {} tool(s): {}",
        tool_names.len(),
        tool_names.join(", ")
    ));
}

// ── Public entry point ─────────────────────────────────────────────

/// Blocking Ratatui session — consumes merged [`UiEvent`]s and emits [`UiCommand`]s upward.
pub fn run_tui(
    mut ui_rx: mpsc::UnboundedReceiver<UiEvent>,
    ui_cmd_tx: mpsc::UnboundedSender<UiCommand>,
    config: TuiConfig,
) -> anyhow::Result<()> {
    use std::io;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(config);
    let (cred_req_tx, mut cred_req_rx) = mpsc::unbounded_channel::<McpCredentialRequest>();

    let result = (|| -> anyhow::Result<()> {
        loop {
            app.pulse_counter = app.pulse_counter.wrapping_add(1);
            terminal.draw(|f| app.render(f))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key, &ui_cmd_tx, &cred_req_tx);
                    if app.should_quit {
                        break;
                    }
                }
            }

            while let Ok(ev) = ui_rx.try_recv() {
                app.ingest(&ev);
            }

            if app.pending_mcp_credential.is_none() {
                if let Ok(req) = cred_req_rx.try_recv() {
                    app.misc_info(format!(
                        "[mcp] Credential needed for '{}' — enter env var name (blank to skip):",
                        req.server_name
                    ));
                    app.pending_mcp_credential = Some(PendingMcpCredential {
                        server_name: req.server_name,
                        stage: CredentialStage::EnvVarName,
                        response_tx: req.response_tx,
                    });
                }
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
