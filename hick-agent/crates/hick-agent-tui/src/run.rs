//! Shared Ratatui TUI consuming [`UiEvent`] / emitting [`UiCommand`], backed by [`ConversationTree`].

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use hick_eval_store::{ConfigStats, StorageBackend, compute_config_stats};

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
use tokio::sync::{Notify, broadcast, mpsc, oneshot};

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

// ---------------------------------------------------------------------------
// Public multi-tab API
// ---------------------------------------------------------------------------

/// One config slot for [`run_tui`] — supplies events from one agent instance.
pub struct TabSource {
    /// Label shown in the tab bar (e.g. `"claude-opus / hick-edit"`).
    pub label: String,
    /// Events produced by this agent instance.
    pub event_rx: mpsc::UnboundedReceiver<UiEvent>,
    /// Channel to send commands back to this agent instance (or to the shared UI).
    pub cmd_tx: mpsc::UnboundedSender<UiCommand>,
}

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

/// Payload sent from the TUI to the rating handler in `main.rs` (AC #4).
pub struct RatingSubmission {
    pub config_name: String,
    /// The user query that was judged.
    pub query: String,
    pub scores: HashMap<String, f64>,
    /// RFC 3339 timestamp of the judge call this rating responds to.
    pub judge_timestamp: Option<String>,
}

/// A prune or un-prune action initiated from the TUI pruning view (TASK-49).
pub enum PruneAction {
    /// User confirmed manual pruning of `config_name`.
    Prune {
        config_name: String,
        scores: HashMap<String, f64>,
        /// Wilson CI upper bound at the moment of pruning.
        confidence: f64,
    },
    /// User un-pruned `config_name`.
    Unprune { config_name: String },
}

/// Configuration forwarded from each binary crate.
pub struct TuiConfig {
    pub agent_name: String,
    pub provider_info: String,
    pub project_dir: PathBuf,
    pub interrupt: Arc<Notify>,
    pub ui_fanout_tx: broadcast::Sender<UiEvent>,
    /// URL of the local web UI, if it started successfully.
    pub web_ui_url: Option<String>,
    /// When set, the TUI enters read-only replay mode.  The value is shown in
    /// the header banner as `[REPLAY] <label>` and the input bar is hidden.
    pub replay_label: Option<String>,
    /// Receive dynamically-added tabs (mid-session mode switching, AC #6).
    pub add_tab_rx: Option<mpsc::UnboundedReceiver<TabSource>>,
    /// Channel to write human ratings to the eval store (AC #4).  None when
    /// the eval store is not configured.
    pub rating_tx: Option<mpsc::UnboundedSender<RatingSubmission>>,
    /// Eval store for the pruning view; `None` when not configured.
    pub eval_store: Option<Arc<dyn StorageBackend>>,
    /// Channel to send prune/un-prune actions to the handler in `main.rs`.
    pub prune_tx: Option<mpsc::UnboundedSender<PruneAction>>,
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

struct PendingReplay {
    turn_node_id: NodeId,
}

/// Pending human rating for a completed judged turn (AC #3 / #4 / #10).
struct PendingRating {
    config_name: String,
    query: String,
    judge_scores: HashMap<String, f64>,
    /// Ordered dimension names — positional index maps to `/rate` args.
    dimensions: Vec<String>,
    backtracking_rationale: Vec<String>,
    wall_secs: Option<f64>,
    rework_count: Option<u32>,
    judge_timestamp: String,
}

struct PaneState {
    title: String,
    markdown: String,
}

/// State for the pruning view overlay (TASK-49).
struct PruningViewState {
    stats: Vec<ConfigStats>,
    selected: usize,
    /// Ephemeral message shown at the bottom of the view.
    message: Option<String>,
}

struct App {
    tree: ConversationTree,
    fold: HashMap<NodeId, FoldOverride>,
    id_gen: IdGen,
    preorder_ids: Vec<NodeId>,
    selected_ix: usize,
    pending_turn_sep: bool,
    /// Completed turn count (for per-tab stats display).
    turn_count: u64,
    /// Accumulated approximate output token count.
    total_output_tokens: u64,
    pulse_counter: usize,
    lines: Vec<StyledLine>,
    input: String,
    input_cursor: usize,
    /// Inner width of the input box last time it was rendered.
    input_box_inner_width: usize,
    /// How many rows of the input box are visible (box height minus 2 borders).
    input_box_visible_rows: u16,
    /// Vertical scroll offset within the input box (visual rows from top).
    input_scroll: u16,
    history: Vec<String>,
    history_idx: Option<usize>,
    history_draft: String,
    scroll: usize,
    auto_scroll: bool,
    busy: bool,
    config: TuiConfig,
    should_quit: bool,
    approvals_q: VecDeque<UiNetworkApproval>,
    pending_approval: Option<PendingApproval>,
    pending_mcp_credential: Option<PendingMcpCredential>,
    pending_replay: Option<PendingReplay>,
    /// Judge scores ready for human override (AC #3).
    pending_rating: Option<PendingRating>,
    panes: HashMap<String, PaneState>,
    show_panes: bool,
    /// Active pruning view (TASK-49). `Some` when the user has opened the view.
    pruning_view: Option<PruningViewState>,
}

impl App {
    fn new(config: TuiConfig) -> Self {
        Self {
            tree: ConversationTree::new(),
            fold: HashMap::new(),
            // Start well above bridge IDs (which start at 1) so locally-injected
            // separator nodes never collide with tree nodes emitted by the agent.
            id_gen: IdGen::with_start(1 << 32),
            preorder_ids: Vec::new(),
            selected_ix: 0,
            pending_turn_sep: false,
            turn_count: 0,
            total_output_tokens: 0,
            pulse_counter: 0,
            lines: Vec::new(),
            input: String::new(),
            input_cursor: 0,
            input_box_inner_width: 80,
            input_box_visible_rows: 1,
            input_scroll: 0,
            history: Vec::new(),
            history_idx: None,
            history_draft: String::new(),
            scroll: 0,
            auto_scroll: true,
            busy: false,
            config,
            should_quit: false,
            approvals_q: VecDeque::new(),
            pending_approval: None,
            pending_mcp_credential: None,
            pending_replay: None,
            pending_rating: None,
            panes: HashMap::new(),
            show_panes: false,
            pruning_view: None,
        }
    }

    fn misc_info(&mut self, text: impl Into<String>) {
        let node_id = self.id_gen.next();
        self.ingest(&UiEvent::Info {
            node_id,
            parent_id: None,
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
                parent_id: None,
                text: "─────────────────────────────────".into(),
            });
            self.pending_turn_sep = false;
        }

        match ev {
            UiEvent::PaneContent { name, title, markdown, .. } => {
                if title.is_empty() && markdown.is_empty() {
                    self.panes.remove(name);
                } else {
                    self.panes.insert(name.clone(), PaneState {
                        title: title.clone(),
                        markdown: markdown.clone(),
                    });
                }
                return;
            }
            UiEvent::TurnStats { output_tokens, .. } => {
                self.total_output_tokens += output_tokens;
                return; // not a tree node
            }
            UiEvent::JudgeScoreReady {
                config_name,
                query,
                judge_model,
                scores,
                dimensions,
                backtracking_rationale,
                wall_secs,
                rework_count,
                judge_timestamp,
                ..
            } => {
                // Show summary in conversation view.
                let score_text = dimensions
                    .iter()
                    .filter_map(|d| scores.get(d).map(|s| format!("{d} {s:.1}")))
                    .collect::<Vec<_>>()
                    .join(" | ");
                let mut info = format!("[judge/{judge_model}] {score_text}");
                if let Some(ws) = wall_secs {
                    info.push_str(&format!("  Time: {ws:.0}s"));
                }
                if let Some(rc) = rework_count {
                    info.push_str(&format!("  Rework: {rc}"));
                }
                self.misc_info(info);
                if !backtracking_rationale.is_empty() {
                    for rationale in backtracking_rationale.iter() {
                        self.misc_info(format!("  backtrack: {rationale}"));
                    }
                }
                self.pending_rating = Some(PendingRating {
                    config_name: config_name.clone(),
                    query: query.clone(),
                    judge_scores: scores.clone(),
                    dimensions: dimensions.clone(),
                    backtracking_rationale: backtracking_rationale.clone(),
                    wall_secs: *wall_secs,
                    rework_count: *rework_count,
                    judge_timestamp: judge_timestamp.clone(),
                });
                return;
            }
            UiEvent::JudgeError { config_name, error, .. } => {
                self.misc_error(format!("[judge] {config_name}: {error}"));
                return;
            }
            UiEvent::NetworkApproval(inner) => {
                self.approvals_q.push_back(inner.clone());
            }
            UiEvent::Done { .. } | UiEvent::Interrupted { .. } => {
                self.pending_turn_sep = true;
                self.busy = false;
                self.turn_count += 1;
            }
            UiEvent::Error { .. } => self.busy = false,
            UiEvent::Info { text, .. } => {
                if text.starts_with("[mcp install]") && text.contains("complete") {
                    self.busy = false;
                }
            }
            UiEvent::BranchStarted { fork_node_id, label, .. } => {
                let sep_id = self.id_gen.next();
                self.tree.apply(&UiEvent::Info {
                    node_id: sep_id,
                    parent_id: None,
                    text: format!("── {label} (from \u{23f5}#{fork_node_id}) ──"),
                });
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

    fn branchable(kind: ConversationNodeKind) -> bool {
        matches!(kind, ConversationNodeKind::UserTurn | ConversationNodeKind::Assistant)
    }

    fn has_selected_branchable(&self) -> bool {
        self.selected_node()
            .and_then(|id| self.tree.node(id))
            .is_some_and(|n| Self::branchable(n.kind))
    }

    fn bump_branchable_focus(&mut self, delta: isize) {
        if self.preorder_ids.is_empty() {
            return;
        }
        let n = self.preorder_ids.len();
        let mut ix = self.selected_ix;
        for _ in 0..n {
            ix = ((ix as isize + delta).rem_euclid(n as isize)) as usize;
            if let Some(node) = self.tree.node(self.preorder_ids[ix]) {
                if Self::branchable(node.kind) {
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
                if folded {
                    push_row_tagged(LineKind::Info, format!("{pref}{chev}{}", self.pulse_str()));
                } else {
                    match &node.payload {
                        ConversationPayload::Text(t) if !t.is_empty() => {
                            push_row_tagged(LineKind::Info, format!("{pref}{chev}[thinking]"));
                            let pad = pref.chars().count() + chev.chars().count();
                            let spacer = " ".repeat(pad + 2);
                            for ln in t.lines() {
                                out.push(StyledLine {
                                    kind: LineKind::Assistant,
                                    text: format!("{spacer}{ln}"),
                                });
                            }
                        }
                        _ => {
                            push_row_tagged(
                                LineKind::Info,
                                format!("{pref}{chev}[LLM call — no extended thinking content]"),
                            );
                        }
                    }
                }
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
                let text_lines = adaptive_card_text_lines(&js);
                if folded {
                    let preview = text_lines.first().map(|s| s.as_str()).unwrap_or("…");
                    let head: String = preview.chars().take(120).collect();
                    push_row_tagged(LineKind::Info, format!("{pref}{chev}[card] {head}"));
                } else {
                    push_row_tagged(LineKind::Info, format!("{pref}{chev}[card]"));
                    let pad = pref.chars().count() + chev.chars().count();
                    let spacer = " ".repeat(pad + 2);
                    for ln in text_lines {
                        out.push(StyledLine { kind: LineKind::Assistant, text: format!("{spacer}{ln}") });
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


    fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_idx {
            None => {
                self.history_draft = self.input.clone();
                let idx = self.history.len() - 1;
                self.history_idx = Some(idx);
                self.input = self.history[idx].clone();
            }
            Some(0) => {}
            Some(idx) => {
                let new_idx = idx - 1;
                self.history_idx = Some(new_idx);
                self.input = self.history[new_idx].clone();
            }
        }
        self.input_cursor = self.input.len();
    }

    fn history_down(&mut self) {
        match self.history_idx {
            None => return,
            Some(idx) if idx + 1 < self.history.len() => {
                let new_idx = idx + 1;
                self.history_idx = Some(new_idx);
                self.input = self.history[new_idx].clone();
            }
            Some(_) => {
                self.history_idx = None;
                self.input = std::mem::take(&mut self.history_draft);
            }
        }
        self.input_cursor = self.input.len();
    }

    fn history_push(&mut self, msg: &str) {
        if !msg.is_empty() && self.history.last().map(|s| s.as_str()) != Some(msg) {
            self.history.push(msg.to_string());
        }
        self.history_idx = None;
        self.history_draft.clear();
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        // Primary command sender (for this tab's agent + UI state).
        cmd_tx: &mpsc::UnboundedSender<UiCommand>,
        cred_req_tx: &mpsc::UnboundedSender<McpCredentialRequest>,
        // Extra senders that should receive fan-out copies of Message/DirectCommand/Cancel.
        fanout_txs: &[mpsc::UnboundedSender<UiCommand>],
    ) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // In replay mode only Ctrl+C / Ctrl+D / scroll keys are honoured.
        if self.config.replay_label.is_some() {
            let interrupt = &self.config.interrupt;
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    interrupt.notify_one();
                    cmd_tx.send(UiCommand::Quit).ok();
                    self.should_quit = true;
                }
                (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => {
                    cmd_tx.send(UiCommand::Quit).ok();
                    self.should_quit = true;
                }
                (m, KeyCode::Up) if m.contains(KeyModifiers::SHIFT) => {
                    self.bump_collapsible_focus(-1);
                }
                (m, KeyCode::Down) if m.contains(KeyModifiers::SHIFT) => {
                    self.bump_collapsible_focus(1);
                }
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
                (_, KeyCode::Char(' ')) => self.toggle_selected_fold(),
                (_, KeyCode::Enter) => self.toggle_selected_fold(),
                _ => {}
            }
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

        if self.pending_replay.is_some() {
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    self.pending_replay = None;
                    self.misc_info("[replay] Cancelled.");
                }
                (_, KeyCode::Enter) => {
                    let input = std::mem::take(&mut self.input);
                    self.input_cursor = 0;
                    let pr = self.pending_replay.take().unwrap();
                    if input.is_empty() {
                        self.misc_info("[replay] Cancelled (no config paths entered).");
                    } else {
                        let paths: Vec<String> = input
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if paths.is_empty() {
                            self.misc_info("[replay] Cancelled (no valid config paths).");
                        } else {
                            cmd_tx.send(UiCommand::ReplayTurn {
                                turn_node_id: pr.turn_node_id,
                                config_paths: paths,
                            }).ok();
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

        // ── Pruning view mode (TASK-49) ─────────────────────────────────────
        if self.pruning_view.is_some() {
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    self.pruning_view = None;
                }
                (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => {
                    self.pruning_view = None;
                }
                (_, KeyCode::Up) => {
                    if let Some(ref mut pv) = self.pruning_view {
                        pv.selected = pv.selected.saturating_sub(1);
                    }
                }
                (_, KeyCode::Down) => {
                    if let Some(ref mut pv) = self.pruning_view {
                        let max = pv.stats.len().saturating_sub(1);
                        pv.selected = (pv.selected + 1).min(max);
                    }
                }
                (_, KeyCode::Char('p')) | (_, KeyCode::Enter) => {
                    self.handle_prune_confirm();
                }
                (_, KeyCode::Char('u')) => {
                    self.handle_unprune_confirm();
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
            // Alt+Enter: insert a newline into the prompt (creates multi-line input).
            (m, KeyCode::Enter) if m.contains(KeyModifiers::ALT) => {
                if !self.busy {
                    self.input.insert(self.input_cursor, '\n');
                    self.input_cursor += 1;
                }
            }
            (_, KeyCode::Enter) => {
                if self.input.is_empty() && !self.busy {
                    self.toggle_selected_fold();
                    return;
                }
                if !self.busy && !self.input.is_empty() {
                    let msg = std::mem::take(&mut self.input);
                    self.input_cursor = 0;
                    self.history_push(&msg);

                    if msg.eq_ignore_ascii_case("quit") || msg.eq_ignore_ascii_case("exit") {
                        cmd_tx.send(UiCommand::Quit).ok();
                        self.should_quit = true;
                        return;
                    }

                    if msg.starts_with("/mcp ") || msg == "/mcp" {
                        self.handle_mcp_slash(msg[4..].trim(), cred_req_tx);
                        return;
                    }

                    if msg.starts_with("/rate ") {
                        self.handle_rate_slash(msg[6..].trim());
                        return;
                    }

                    if msg.eq_ignore_ascii_case("/accept") {
                        self.handle_accept_slash();
                        return;
                    }

                    if msg.starts_with('!') {
                        let shell_cmd = msg[1..].trim().to_string();
                        if shell_cmd.is_empty() {
                            self.misc_info("Usage: ! <shell command>");
                            return;
                        }
                        let dc = UiCommand::DirectCommand { text: shell_cmd };
                        cmd_tx.send(dc.clone()).ok();
                        for tx in fanout_txs { tx.send(dc.clone()).ok(); }
                        self.busy = true;
                        self.auto_scroll = true;
                        return;
                    }

                    let msg_cmd = UiCommand::Message { text: msg };
                    cmd_tx.send(msg_cmd.clone()).ok();
                    for tx in fanout_txs { tx.send(msg_cmd.clone()).ok(); }
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
            // Ctrl+Backspace: terminals send either Ctrl+Backspace or Ctrl+H (\x08).
            (m, KeyCode::Backspace) if m.contains(KeyModifiers::CONTROL) => {
                if !self.busy && self.input_cursor > 0 {
                    let new_pos = prev_word_start(&self.input, self.input_cursor);
                    self.input.drain(new_pos..self.input_cursor);
                    self.input_cursor = new_pos;
                }
            }
            (m, KeyCode::Char('h')) if m.contains(KeyModifiers::CONTROL) => {
                if !self.busy && self.input_cursor > 0 {
                    let new_pos = prev_word_start(&self.input, self.input_cursor);
                    self.input.drain(new_pos..self.input_cursor);
                    self.input_cursor = new_pos;
                }
            }
            (_, KeyCode::Backspace) => {
                if !self.busy && self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                }
            }
            (m, KeyCode::Left) if m.contains(KeyModifiers::CONTROL) => {
                self.input_cursor = prev_word_start(&self.input, self.input_cursor);
            }
            (_, KeyCode::Left) => {
                self.input_cursor = self.input_cursor.saturating_sub(1);
            }
            (m, KeyCode::Right) if m.contains(KeyModifiers::CONTROL) => {
                self.input_cursor = next_word_end(&self.input, self.input_cursor);
            }
            (_, KeyCode::Right) => {
                if self.input_cursor < self.input.len() {
                    self.input_cursor += 1;
                }
            }
            (m, KeyCode::Home) if m.contains(KeyModifiers::CONTROL) => self.input_cursor = 0,
            (_, KeyCode::Home) => {
                let bw = self.input_box_inner_width;
                self.input_cursor = visual_row_home(&self.input, self.input_cursor, bw);
            }
            (m, KeyCode::End) if m.contains(KeyModifiers::CONTROL) => {
                self.input_cursor = self.input.len();
            }
            (_, KeyCode::End) => {
                let bw = self.input_box_inner_width;
                self.input_cursor = visual_row_end(&self.input, self.input_cursor, bw);
            }
            (m, KeyCode::Delete) if m.contains(KeyModifiers::CONTROL) => {
                if !self.busy && self.input_cursor < self.input.len() {
                    let end = next_word_end(&self.input, self.input_cursor);
                    self.input.drain(self.input_cursor..end);
                }
            }
            (_, KeyCode::Delete) => {
                if !self.busy && self.input_cursor < self.input.len() {
                    self.input.remove(self.input_cursor);
                }
            }
            (_, KeyCode::Up) => {
                if !self.busy {
                    let bw = self.input_box_inner_width;
                    if let Some(new_pos) = move_cursor_visual_up(&self.input, self.input_cursor, bw) {
                        self.input_cursor = new_pos;
                    } else {
                        self.history_up();
                    }
                } else {
                    self.auto_scroll = false;
                    self.scroll = self.scroll.saturating_sub(1);
                }
            }
            (_, KeyCode::Down) => {
                if !self.busy {
                    let bw = self.input_box_inner_width;
                    if let Some(new_pos) = move_cursor_visual_down(&self.input, self.input_cursor, bw) {
                        self.input_cursor = new_pos;
                    } else {
                        self.history_down();
                    }
                } else {
                    self.scroll = self.scroll.saturating_add(1);
                }
            }
            (_, KeyCode::PageUp) => {
                if !self.busy {
                    let bw = self.input_box_inner_width;
                    let page = self.input_box_visible_rows.max(1);
                    self.input_cursor = move_cursor_visual_page_up(&self.input, self.input_cursor, bw, page);
                } else {
                    self.auto_scroll = false;
                    self.scroll = self.scroll.saturating_sub(20);
                }
            }
            (_, KeyCode::PageDown) => {
                if !self.busy {
                    let bw = self.input_box_inner_width;
                    let page = self.input_box_visible_rows.max(1);
                    self.input_cursor = move_cursor_visual_page_down(&self.input, self.input_cursor, bw, page);
                } else {
                    self.scroll = self.scroll.saturating_add(20);
                }
            }
            // Tab/Shift+Tab: cycle through history (Tab=older, BackTab=newer).
            // Falls back to pane toggle when there is no history at all.
            (_, KeyCode::BackTab) => {
                if !self.busy {
                    self.history_down();
                }
            }
            (_, KeyCode::Tab) => {
                if !self.busy {
                    if !self.history.is_empty() || self.history_idx.is_some() {
                        self.history_up();
                    } else if !self.panes.is_empty() {
                        self.show_panes = !self.show_panes;
                    }
                }
            }
            (_, KeyCode::Char('b')) if self.input.is_empty() && !self.busy => {
                if let Some(sel) = self.selected_node() {
                    if let Some(node) = self.tree.node(sel) {
                        if Self::branchable(node.kind) {
                            cmd_tx.send(UiCommand::RewindTo { node_id: sel }).ok();
                        }
                    }
                }
            }
            (_, KeyCode::Char('r')) if self.input.is_empty() && !self.busy => {
                if let Some(sel) = self.selected_node() {
                    if let Some(node) = self.tree.node(sel) {
                        if node.kind == ConversationNodeKind::UserTurn {
                            self.pending_replay = Some(PendingReplay { turn_node_id: sel });
                            self.misc_info(
                                "[replay] Enter config path(s) to replay against (comma-separated):",
                            );
                        }
                    }
                }
            }
            (_, KeyCode::Char('P')) if self.input.is_empty() && !self.busy => {
                self.open_pruning_view();
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

    fn handle_rate_slash(&mut self, args: &str) {
        let Some(ref pending) = self.pending_rating else {
            self.misc_info("[rate] No pending judge scores to rate.");
            return;
        };
        let raw: Vec<&str> = args.split_whitespace().collect();
        if raw.len() != pending.dimensions.len() {
            self.misc_info(format!(
                "[rate] Expected {} scores ({}), got {}. Example: /rate {}",
                pending.dimensions.len(),
                pending.dimensions.join(" "),
                raw.len(),
                (0..pending.dimensions.len()).map(|_| "8").collect::<Vec<_>>().join(" "),
            ));
            return;
        }
        let mut scores = HashMap::new();
        for (dim, raw_score) in pending.dimensions.iter().zip(raw.iter()) {
            match raw_score.parse::<f64>() {
                Ok(s) => { scores.insert(dim.clone(), s.clamp(1.0, 10.0)); }
                Err(_) => {
                    self.misc_error(format!("[rate] Invalid score '{raw_score}' for {dim} — expected a number."));
                    return;
                }
            }
        }
        self.submit_rating(scores);
    }

    fn handle_accept_slash(&mut self) {
        let Some(ref pending) = self.pending_rating else {
            self.misc_info("[rate] No pending judge scores to accept.");
            return;
        };
        let scores = pending.judge_scores.clone();
        self.submit_rating(scores);
    }

    // ── Pruning view helpers (TASK-49) ─────────────────────────────────

    fn open_pruning_view(&mut self) {
        let Some(ref store) = self.config.eval_store else {
            self.misc_info("[prune] Eval store not configured.");
            return;
        };
        match store.read_all() {
            Err(e) => self.misc_error(format!("[prune] Failed to load eval store: {e}")),
            Ok(records) => {
                let stats = compute_config_stats(&records);
                if stats.is_empty() {
                    self.misc_info("[prune] No scored configs yet — nothing to show.");
                    return;
                }
                self.pruning_view = Some(PruningViewState {
                    stats,
                    selected: 0,
                    message: None,
                });
            }
        }
    }

    fn handle_prune_confirm(&mut self) {
        let Some(ref mut pv) = self.pruning_view else { return };
        let Some(stat) = pv.stats.get(pv.selected) else { return };

        if stat.is_pruned {
            pv.message = Some(format!("{} is already pruned — press u to un-prune.", stat.config_name));
            return;
        }
        if !stat.is_candidate {
            pv.message = Some(format!(
                "{} is not a pruning candidate (CI upper bound {:.2} >= best win rate).",
                stat.config_name, stat.ci_high
            ));
            return;
        }

        let config_name = stat.config_name.clone();
        let ci_high = stat.ci_high;
        // Build a summary of scores for the record.
        let scores: HashMap<String, f64> = {
            let mut m = HashMap::new();
            m.insert("win_rate".to_string(), stat.win_rate);
            m.insert("combined_score".to_string(), stat.combined_score);
            m
        };

        if let Some(ref tx) = self.config.prune_tx {
            if tx.send(PruneAction::Prune {
                config_name: config_name.clone(),
                scores,
                confidence: ci_high,
            }).is_err() {
                if let Some(ref mut pv2) = self.pruning_view {
                    pv2.message = Some("[prune] Channel disconnected.".to_string());
                }
                return;
            }
        }

        // Reflect the pruning immediately in the local stats so the view
        // updates without a full reload.
        if let Some(ref mut pv2) = self.pruning_view {
            if let Some(s) = pv2.stats.get_mut(pv2.selected) {
                s.is_pruned = true;
                s.is_candidate = false;
            }
            pv2.message = Some(format!("Pruned {config_name}."));
        }
        self.misc_info(format!("[prune] {config_name} pruned."));
    }

    fn handle_unprune_confirm(&mut self) {
        let Some(ref mut pv) = self.pruning_view else { return };
        let Some(stat) = pv.stats.get(pv.selected) else { return };

        if !stat.is_pruned {
            pv.message = Some(format!("{} is not pruned.", stat.config_name));
            return;
        }

        let config_name = stat.config_name.clone();
        if let Some(ref tx) = self.config.prune_tx {
            if tx.send(PruneAction::Unprune { config_name: config_name.clone() }).is_err() {
                if let Some(ref mut pv2) = self.pruning_view {
                    pv2.message = Some("[prune] Channel disconnected.".to_string());
                }
                return;
            }
        }

        if let Some(ref mut pv2) = self.pruning_view {
            if let Some(s) = pv2.stats.get_mut(pv2.selected) {
                s.is_pruned = false;
            }
            pv2.message = Some(format!("Un-pruned {config_name}."));
        }
        self.misc_info(format!("[prune] {config_name} un-pruned."));
    }

    fn submit_rating(&mut self, scores: HashMap<String, f64>) {
        let Some(pending) = self.pending_rating.take() else { return };
        let Some(ref tx) = self.config.rating_tx else {
            self.misc_info("[rate] Rating store not configured.");
            return;
        };
        let submission = RatingSubmission {
            config_name: pending.config_name.clone(),
            query: pending.query.clone(),
            scores,
            judge_timestamp: Some(pending.judge_timestamp.clone()),
        };
        if tx.send(submission).is_err() {
            self.misc_error("[rate] Rating channel disconnected.");
        } else {
            self.misc_info(format!("[rate] Rating submitted for {}.", pending.config_name));
        }
    }

    // ── Rendering ──────────────────────────────────────────────────

    fn render_pruning_view(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        use ratatui::layout::Constraint;
        use ratatui::text::Text;

        let pv = match &self.pruning_view {
            Some(pv) => pv,
            None => return,
        };

        let chunks = ratatui::layout::Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(1),    // list
            Constraint::Length(3), // status/help bar
        ])
        .split(area);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "Pruning view",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  ↑↓ navigate  p/Enter prune  u un-prune  Esc quit",
                Style::default().fg(Color::DarkGray),
            ),
        ]));
        frame.render_widget(header, chunks[0]);

        // Config list
        let mut rows: Vec<Line> = Vec::new();
        for (i, stat) in pv.stats.iter().enumerate() {
            let selected = i == pv.selected;
            let rank = format!("{:>2}. ", i + 1);
            let ci = format!(
                "[{:.2}–{:.2}]",
                stat.ci_low, stat.ci_high
            );
            let status = if stat.is_pruned {
                " [PRUNED]"
            } else if stat.is_candidate {
                " [CANDIDATE]"
            } else {
                ""
            };
            let label = format!(
                "{}{:<30}  score {:.1}  win {:.0}%  CI {}  n={}{}",
                rank,
                stat.config_name,
                stat.combined_score,
                stat.win_rate * 100.0,
                ci,
                stat.sample_count,
                status,
            );
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else if stat.is_pruned {
                Style::default().fg(Color::DarkGray)
            } else if stat.is_candidate {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            rows.push(Line::styled(label, style));
        }

        let list_widget = Paragraph::new(Text::from(rows))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(list_widget, chunks[1]);

        // Status / message bar
        let status_text = pv
            .message
            .as_deref()
            .unwrap_or("Yellow = candidate  Gray = pruned  p = prune selected  u = un-prune");
        let status_widget = Paragraph::new(status_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::DarkGray)),
            );
        frame.render_widget(status_widget, chunks[2]);
    }

    fn render_in(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Pruning view takes over the full area when active (TASK-49).
        if self.pruning_view.is_some() {
            self.render_pruning_view(frame, area);
            return;
        }

        self.rebuild_screen();
        self.pump_network_prompt();
        let area = area;

        let replay_mode = self.config.replay_label.is_some();
        // Dynamic input box height: grows with the prompt content.
        let box_inner_width = area.width.saturating_sub(2) as usize;
        self.input_box_inner_width = box_inner_width;
        let input_visual_rows = if replay_mode || self.busy
            || self.pending_approval.is_some()
            || self.pending_mcp_credential.is_some()
            || self.pending_replay.is_some()
            || self.pending_rating.is_some()
        {
            1u16
        } else {
            visual_input_rows(&self.input, box_inner_width)
        };
        let max_input_height: u16 = (area.height * 3 / 4).max(3);
        let input_height: u16 = if replay_mode {
            0
        } else {
            (input_visual_rows + 2).min(max_input_height)
        };
        self.input_box_visible_rows = input_height.saturating_sub(2);

        let constraints = if replay_mode {
            vec![Constraint::Length(1), Constraint::Min(1)]
        } else {
            vec![Constraint::Length(1), Constraint::Min(1), Constraint::Length(input_height)]
        };
        let chunks = Layout::vertical(constraints).split(area);

        // Header bar
        let agent = self.config.agent_name.clone();
        let provider_info = self.config.provider_info.clone();
        let mut header_spans = if let Some(ref label) = self.config.replay_label {
            vec![
                Span::styled(
                    format!("[REPLAY] {label}"),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    agent,
                    Style::default().fg(Color::DarkGray),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    agent,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
                Span::styled(provider_info, Style::default().fg(Color::DarkGray)),
            ]
        };
        if let Some(ref url) = self.config.web_ui_url {
            header_spans.push(Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)));
            header_spans.push(Span::styled(
                format!("web \u{25b8} {url}"),
                Style::default().fg(Color::Green),
            ));
        }
        header_spans.push(Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)));
        if !self.panes.is_empty() {
            header_spans.push(Span::styled(
                format!("panes({}) Tab  ", self.panes.len()),
                Style::default().fg(Color::Yellow),
            ));
        }
        header_spans.push(Span::styled(
            "\u{2191}\u{2193}: branch-nav  b: branch  Shift+\u{2191}\u{2193}: fold-nav  Space: expand",
            Style::default().fg(Color::DarkGray),
        ));
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

        // Split body area horizontally if pane sidebar is active.
        let body_area = chunks[1];
        let (conv_area, pane_area_opt) = if self.show_panes && !self.panes.is_empty() {
            let cols = Layout::horizontal([Constraint::Min(40), Constraint::Length(40)]).split(body_area);
            (cols[0], Some(cols[1]))
        } else {
            (body_area, None)
        };

        if let Some(pane_area) = pane_area_opt {
            let mut pane_lines: Vec<Line> = Vec::new();
            for (name, state) in &self.panes {
                pane_lines.push(Line::styled(
                    format!("── {} ──", if state.title.is_empty() { name } else { &state.title }),
                    Style::default().fg(Color::Yellow),
                ));
                for ln in state.markdown.lines().take(200) {
                    pane_lines.push(Line::styled(ln.to_string(), Style::default().fg(Color::White)));
                }
                pane_lines.push(Line::from(""));
            }
            let pane_widget = Paragraph::new(Text::from(pane_lines))
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
            frame.render_widget(pane_widget, pane_area);
        }

        let visible_height = conv_area.height.saturating_sub(2) as usize;
        let inner_width = conv_area.width.saturating_sub(2) as usize;

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
        frame.render_widget(output, conv_area);

        if !replay_mode {
            let input_display = if let Some(ref pr) = self.pending_rating {
                let score_hint = pr.dimensions.iter()
                    .filter_map(|d| pr.judge_scores.get(d).map(|s| format!("{d} {s:.0}")))
                    .collect::<Vec<_>>()
                    .join(" | ");
                let mut line = format!("[judge] {score_hint}");
                if let Some(ws) = pr.wall_secs {
                    line.push_str(&format!("  Time: {ws:.0}s"));
                }
                if let Some(rc) = pr.rework_count {
                    line.push_str(&format!("  Rework: {rc}"));
                }
                if self.input.is_empty() {
                    format!("{line}  \u{2502}  /accept or /rate {} (then Enter)", pr.dimensions.iter().map(|_| "N").collect::<Vec<_>>().join(" "))
                } else {
                    format!("{line}  \u{2502}  > {}", self.input)
                }
            } else if let Some(ref _pr) = self.pending_replay {
                format!(
                    "[replay] Config path(s), comma-separated (blank=cancel): {}",
                    self.input
                )
            } else if let Some(ref pm) = self.pending_mcp_credential {
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
            } else if self.input.is_empty() && self.has_selected_branchable() {
                let replay_hint = self
                    .selected_node()
                    .and_then(|id| self.tree.node(id))
                    .is_some_and(|n| n.kind == ConversationNodeKind::UserTurn);
                if replay_hint {
                    "[b] branch  [r] replay with alternate config  [P] pruning view  (type to send a message)".to_string()
                } else {
                    "[b] branch from here  [P] pruning view  (type to send a message instead)".to_string()
                }
            } else {
                self.input.clone()
            };

            let input_style = if self.pending_rating.is_some() {
                Style::default().fg(Color::Yellow)
            } else if self.pending_replay.is_some() {
                Style::default().fg(Color::Green)
            } else if self.pending_mcp_credential.is_some() {
                Style::default().fg(Color::Magenta)
            } else if self.pending_approval.is_some() {
                Style::default().fg(Color::Yellow)
            } else if self.busy {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let input_widget = Paragraph::new(input_display)
                .style(input_style)
                .wrap(Wrap { trim: false })
                .scroll((self.input_scroll, 0))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                );
            frame.render_widget(input_widget, chunks[2]);

            if !self.busy {
                let bw = chunks[2].width.saturating_sub(2) as usize;
                let (vcol, vrow) = visual_cursor_pos(&self.input, self.input_cursor, bw);
                // Keep cursor visible by adjusting input_scroll.
                let visible = self.input_box_visible_rows;
                if vrow < self.input_scroll {
                    self.input_scroll = vrow;
                } else if visible > 0 && vrow >= self.input_scroll + visible {
                    self.input_scroll = vrow.saturating_sub(visible - 1);
                }
                let cursor_x = chunks[2].x + 1 + vcol;
                let cursor_y = chunks[2].y + 1 + vrow.saturating_sub(self.input_scroll);
                frame.set_cursor_position((cursor_x, cursor_y));
            }
        }
    }
}

// ── Visual layout helpers ───────────────────────────────────────────
//
// The input is displayed left-justified with no prefix.  All helpers work in
// terms of (visual-row, visual-col) where wrapping happens exactly at
// `box_inner_width` — matching ratatui's Wrap { trim: false } behaviour for
// ASCII text (one byte ≈ one column).  Logical lines are delimited by '\n'.

/// Returns the (col, row) visual cursor position for a given byte offset.
fn visual_cursor_pos(input: &str, cursor: usize, bw: usize) -> (u16, u16) {
    if bw == 0 {
        return (0, 0);
    }
    let mut row = 0u16;
    let mut col = 0u16;
    let mut pos = 0usize;

    for line in input.split('\n') {
        for byte_off in 0..=line.len() {
            if pos + byte_off == cursor {
                return (col, row);
            }
            if byte_off < line.len() {
                col += 1;
                if col as usize >= bw {
                    col = 0;
                    row += 1;
                }
            }
        }
        pos += line.len() + 1; // +1 for '\n'
        row += 1;
        col = 0;
    }
    (col, row)
}

/// Total number of visual rows the input occupies (without borders).
fn visual_input_rows(input: &str, bw: usize) -> u16 {
    if bw == 0 {
        return 1;
    }
    let mut rows = 0u16;
    for line in input.split('\n') {
        rows += if line.is_empty() { 1 } else { line.len().div_ceil(bw) as u16 };
    }
    rows.max(1)
}

/// Given a target (row, preferred_col) returns the byte offset whose visual
/// position is as close as possible; clamps to the end of the target row when
/// `preferred_col` is past it.
fn byte_at_visual_row_col(input: &str, target_row: u16, preferred_col: u16, bw: usize) -> usize {
    if bw == 0 {
        return 0;
    }
    let mut row = 0u16;
    let mut col = 0u16;
    let mut pos = 0usize;
    let mut last = 0usize;
    let mut found = false;

    for line in input.split('\n') {
        for byte_off in 0..=line.len() {
            if row == target_row {
                found = true;
                last = pos + byte_off;
                if col >= preferred_col {
                    return pos + byte_off;
                }
            } else if row > target_row {
                return last;
            }
            if byte_off < line.len() {
                col += 1;
                if col as usize >= bw {
                    col = 0;
                    row += 1;
                }
            }
        }
        if row == target_row && found {
            return last;
        }
        pos += line.len() + 1;
        row += 1;
        col = 0;
    }
    if found { last } else { input.len() }
}

/// Byte offset at the start of the current visual row (Home key).
fn visual_row_home(input: &str, cursor: usize, bw: usize) -> usize {
    let (_, vrow) = visual_cursor_pos(input, cursor, bw);
    byte_at_visual_row_col(input, vrow, 0, bw)
}

/// Byte offset at the end of the current visual row (End key).
/// For soft-wrap rows this is the first byte of the next row (cursor sits at
/// column 0 of the next visual row — the only way to be "after" the last
/// visible character in a terminal).  For logical-line endings the cursor
/// stops before the '\n'.
fn visual_row_end(input: &str, cursor: usize, bw: usize) -> usize {
    let (_, vrow) = visual_cursor_pos(input, cursor, bw);
    let total = visual_input_rows(input, bw);
    if vrow + 1 >= total {
        return input.len();
    }
    let next_start = byte_at_visual_row_col(input, vrow + 1, 0, bw);
    // If the next visual row begins right after a '\n', stop before the '\n'.
    if next_start > 0 && input.as_bytes().get(next_start - 1) == Some(&b'\n') {
        next_start - 1
    } else {
        next_start
    }
}

/// Move cursor up one visual row; returns `None` on the first row.
fn move_cursor_visual_up(input: &str, cursor: usize, bw: usize) -> Option<usize> {
    let (vcol, vrow) = visual_cursor_pos(input, cursor, bw);
    if vrow == 0 {
        return None;
    }
    Some(byte_at_visual_row_col(input, vrow - 1, vcol, bw))
}

/// Move cursor down one visual row; returns `None` on the last row.
fn move_cursor_visual_down(input: &str, cursor: usize, bw: usize) -> Option<usize> {
    let (vcol, vrow) = visual_cursor_pos(input, cursor, bw);
    if vrow + 1 >= visual_input_rows(input, bw) {
        return None;
    }
    Some(byte_at_visual_row_col(input, vrow + 1, vcol, bw))
}

/// Move cursor up by `page_rows` visual rows (clamps to row 0).
fn move_cursor_visual_page_up(input: &str, cursor: usize, bw: usize, page_rows: u16) -> usize {
    let (vcol, vrow) = visual_cursor_pos(input, cursor, bw);
    let target = vrow.saturating_sub(page_rows);
    byte_at_visual_row_col(input, target, vcol, bw)
}

/// Move cursor down by `page_rows` visual rows (clamps to last row).
fn move_cursor_visual_page_down(input: &str, cursor: usize, bw: usize, page_rows: u16) -> usize {
    let (vcol, vrow) = visual_cursor_pos(input, cursor, bw);
    let last_row = visual_input_rows(input, bw).saturating_sub(1);
    let target = (vrow + page_rows).min(last_row);
    byte_at_visual_row_col(input, target, vcol, bw)
}

// ── Word-boundary helpers (byte-indexed, ASCII whitespace) ─────────

fn prev_word_start(s: &str, pos: usize) -> usize {
    let bytes = s.as_bytes();
    let mut i = pos;
    while i > 0 && bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    while i > 0 && !bytes[i - 1].is_ascii_whitespace() {
        i -= 1;
    }
    i
}

fn next_word_end(s: &str, pos: usize) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let mut i = pos;
    while i < len && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    while i < len && !bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
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
    ui_tx: broadcast::Sender<UiEvent>,
    cred_req_tx: mpsc::UnboundedSender<McpCredentialRequest>,
) {
    let ids = IdGen::new();
    let send_info = |text: String| {
        let _ = ui_tx.send(UiEvent::Info {
            node_id: ids.next(),
            parent_id: None,
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

// ── AdaptiveCard text extraction ───────────────────────────────────

fn adaptive_card_text_lines(json: &str) -> Vec<String> {
    let Ok(val) = serde_json::from_str::<serde_json::Value>(json) else {
        return vec![json.chars().take(120).collect()];
    };
    let mut out = Vec::new();
    extract_card_text(&val, &mut out);
    if out.is_empty() {
        out.push(json.chars().take(120).collect());
    }
    out
}

fn extract_card_text(val: &serde_json::Value, out: &mut Vec<String>) {
    match val {
        serde_json::Value::Object(map) => {
            let ty = map.get("type").and_then(|v| v.as_str()).unwrap_or("");
            match ty {
                "TextBlock" | "RichTextBlock" => {
                    if let Some(text) = map.get("text").and_then(|v| v.as_str()) {
                        out.push(text.to_string());
                    }
                }
                "FactSet" => {
                    if let Some(facts) = map.get("facts").and_then(|v| v.as_array()) {
                        for fact in facts {
                            let title = fact.get("title").and_then(|v| v.as_str()).unwrap_or("");
                            let value = fact.get("value").and_then(|v: &serde_json::Value| v.as_str()).unwrap_or("");
                            out.push(format!("{title}: {value}"));
                        }
                    }
                }
                _ => {
                    for v in map.values() {
                        extract_card_text(v, out);
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                extract_card_text(v, out);
            }
        }
        _ => {}
    }
}

// ── Public entry point ─────────────────────────────────────────────

/// Blocking Ratatui session — multi-tab capable.
///
/// `tabs` must have at least one entry. When it has exactly one entry the UI is
/// identical to the previous single-tab behaviour. With multiple entries a tab
/// bar appears at the top; `[` / `]` switch tabs and messages are fanned out to
/// all active agents simultaneously.
///
/// Dynamic tabs can be added mid-session via `config.add_tab_rx`.
pub fn run_tui(
    tabs: Vec<TabSource>,
    config: TuiConfig,
) -> anyhow::Result<()> {
    assert!(!tabs.is_empty(), "run_tui requires at least one TabSource");
    use std::io;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Decompose tabs into parallel vecs — event receivers, command senders, labels, Apps.
    let mut event_rxs: Vec<mpsc::UnboundedReceiver<UiEvent>> = Vec::new();
    let mut cmd_txs: Vec<mpsc::UnboundedSender<UiCommand>> = Vec::new();
    let mut tab_labels: Vec<String> = Vec::new();
    let mut apps: Vec<App> = Vec::new();
    let mut add_tab_rx = config.add_tab_rx;

    for tab in tabs {
        event_rxs.push(tab.event_rx);
        cmd_txs.push(tab.cmd_tx);
        tab_labels.push(tab.label);
    }

    // Create a shared TuiConfig (without the add_tab_rx) for the apps.
    let shared_config = TuiConfig {
        agent_name: config.agent_name.clone(),
        provider_info: config.provider_info.clone(),
        project_dir: config.project_dir.clone(),
        interrupt: Arc::clone(&config.interrupt),
        ui_fanout_tx: config.ui_fanout_tx.clone(),
        web_ui_url: config.web_ui_url.clone(),
        replay_label: config.replay_label.clone(),
        add_tab_rx: None,
        rating_tx: config.rating_tx.clone(),
        eval_store: config.eval_store.clone(),
        prune_tx: config.prune_tx.clone(),
    };
    for _ in 0..event_rxs.len() {
        apps.push(App::new(shared_config.clone()));
    }

    let mut active_tab: usize = 0;
    let (cred_req_tx, mut cred_req_rx) = mpsc::unbounded_channel::<McpCredentialRequest>();

    let multi = apps.len() > 1;
    // Tab bar height: 1 row for labels + 1 divider row when multi-tab.
    let tab_bar_height: u16 = if multi { 2 } else { 0 };

    let result = (|| -> anyhow::Result<()> {
        loop {
            // Dynamically add new tabs if requested (AC #6).
            if let Some(ref mut rx) = add_tab_rx {
                while let Ok(new_tab) = rx.try_recv() {
                    event_rxs.push(new_tab.event_rx);
                    cmd_txs.push(new_tab.cmd_tx);
                    tab_labels.push(new_tab.label);
                    apps.push(App::new(shared_config.clone()));
                }
            }

            let n_tabs = apps.len();
            let is_multi = n_tabs > 1;

            // Increment pulse on the active app.
            apps[active_tab].pulse_counter = apps[active_tab].pulse_counter.wrapping_add(1);

            terminal.draw(|f| {
                let full_area = f.area();
                if is_multi {
                    // Render tab bar at top.
                    let (bar_area, body_area) = split_tab_bar(full_area, tab_bar_height);
                    render_tab_bar(f, bar_area, &tab_labels, &apps, active_tab);
                    apps[active_tab].render_in(f, body_area);
                } else {
                    apps[0].render_in(f, full_area);
                }
            })?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    // Tab switching keys (multi-tab only).
                    if is_multi && key.kind == KeyEventKind::Press {
                        match (key.modifiers, key.code) {
                            (_, KeyCode::Char('[')) if apps[active_tab].input.is_empty() => {
                                active_tab = active_tab.saturating_sub(1);
                                continue;
                            }
                            (_, KeyCode::Char(']')) if apps[active_tab].input.is_empty() => {
                                active_tab = (active_tab + 1).min(n_tabs - 1);
                                continue;
                            }
                            _ => {}
                        }
                    }

                    // Fan-out senders: all cmd_txs except the active tab's.
                    let fanout: Vec<mpsc::UnboundedSender<UiCommand>> = cmd_txs
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != active_tab)
                        .map(|(_, tx)| tx.clone())
                        .collect();

                    apps[active_tab].handle_key(key, &cmd_txs[active_tab], &cred_req_tx, &fanout);
                    if apps[active_tab].should_quit {
                        break;
                    }
                }
            }

            // Drain events for every tab.
            for (i, rx) in event_rxs.iter_mut().enumerate() {
                while let Ok(ev) = rx.try_recv() {
                    apps[i].ingest(&ev);
                }
            }

            // MCP credential prompts only apply to the active tab.
            if apps[active_tab].pending_mcp_credential.is_none() {
                if let Ok(req) = cred_req_rx.try_recv() {
                    apps[active_tab].misc_info(format!(
                        "[mcp] Credential needed for '{}' — enter env var name (blank to skip):",
                        req.server_name
                    ));
                    apps[active_tab].pending_mcp_credential = Some(PendingMcpCredential {
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

// ── Tab bar helpers ────────────────────────────────────────────────

/// Split `area` into (tab_bar, body) vertically.
fn split_tab_bar(area: ratatui::layout::Rect, bar_height: u16) -> (ratatui::layout::Rect, ratatui::layout::Rect) {
    use ratatui::layout::Rect;
    let bar = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: bar_height.min(area.height),
    };
    let body_y = area.y.saturating_add(bar_height);
    let body = Rect {
        x: area.x,
        y: body_y,
        width: area.width,
        height: area.height.saturating_sub(bar_height),
    };
    (bar, body)
}

/// Render the tab bar labels with active tab highlighted.
fn render_tab_bar(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    labels: &[String],
    apps: &[App],
    active: usize,
) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (label, app)) in labels.iter().zip(apps.iter()).enumerate() {
        let display = if label.is_empty() {
            format!(" Tab {} ", i + 1)
        } else {
            format!(" {} ", label)
        };
        // Append per-tab stats if any turns have completed.
        let stats = if app.turn_count > 0 {
            format!(
                "[{}t ~{}k]",
                app.turn_count,
                app.total_output_tokens / 1000
            )
        } else {
            String::new()
        };
        let full_label = format!("{}{}", display, stats);
        let style = if i == active {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if app.busy {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(full_label, style));
        if i + 1 < labels.len() {
            spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
        }
    }
    // Hint for navigation.
    spans.push(Span::styled(
        "  [/] switch tab",
        Style::default().fg(Color::DarkGray),
    ));

    let tab_line = ratatui::widgets::Paragraph::new(ratatui::text::Line::from(spans));
    frame.render_widget(tab_line, area);
}

// Implement Clone for TuiConfig so we can clone it for each App.
impl Clone for TuiConfig {
    fn clone(&self) -> Self {
        Self {
            agent_name: self.agent_name.clone(),
            provider_info: self.provider_info.clone(),
            project_dir: self.project_dir.clone(),
            interrupt: Arc::clone(&self.interrupt),
            ui_fanout_tx: self.ui_fanout_tx.clone(),
            web_ui_url: self.web_ui_url.clone(),
            replay_label: self.replay_label.clone(),
            add_tab_rx: None, // not cloneable; only used in the outer run_tui
            rating_tx: self.rating_tx.clone(),
            eval_store: self.eval_store.clone(),
            prune_tx: self.prune_tx.clone(),
        }
    }
}
