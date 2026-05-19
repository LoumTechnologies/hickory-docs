//! Ratatui-based terminal UI for the agent.
//!
//! Consumes [`AgentEvent`]s from the core and renders them in a scrollable
//! output area with styled text. User input is captured via crossterm key
//! events and sent back as [`AgentCommand`]s.

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

use hick_agent_script_first::events::{AgentCommand, AgentEvent};
use hick_agent_sdk::mcp_install::{
    McpCredentialRequest, McpInstallSpec, McpInstaller, infer_name, list_servers, parse_spec,
    remove_server,
};
use hick_agent_sdk::mcp_config::McpServerTrust;
use hick_agent_sdk::network_filter::{ApprovalResponse, NetworkApprovalRequest, NetworkVerdict};
use hick_policy::PolicySuggestion;

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

// ── Application state ──────────────────────────────────────────────

/// Pending network approval: holds suggestions for "Always Allow" and the
/// channel back to the filter.
struct PendingApproval {
    host: String,
    suggestions: Vec<PolicySuggestion>,
    response_tx: oneshot::Sender<ApprovalResponse>,
    /// When `true`, the user typed 'a' and is now selecting a suggestion index.
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
    lines: Vec<StyledLine>,
    current_partial: String,
    input: String,
    input_cursor: usize,
    scroll: usize,
    auto_scroll: bool,
    busy: bool,
    thinking_shown: bool,
    provider_info: String,
    should_quit: bool,
    /// Pending network approval prompt — awaiting y/n/a from the user.
    pending_approval: Option<PendingApproval>,
    /// Pending MCP credential prompt — awaiting env-var name / value from the user.
    pending_mcp_credential: Option<PendingMcpCredential>,
}

impl App {
    fn new(provider_info: String) -> Self {
        Self {
            lines: Vec::new(),
            current_partial: String::new(),
            input: String::new(),
            input_cursor: 0,
            scroll: 0,
            auto_scroll: true,
            busy: false,
            thinking_shown: false,
            provider_info,
            should_quit: false,
            pending_approval: None,
            pending_mcp_credential: None,
        }
    }

    fn push_line(&mut self, kind: LineKind, text: &str) {
        self.lines.push(StyledLine {
            kind,
            text: text.to_string(),
        });
        if self.auto_scroll {
            self.scroll = usize::MAX;
        }
    }

    fn flush_partial(&mut self) {
        if !self.current_partial.is_empty() {
            let text = std::mem::take(&mut self.current_partial);
            for line in text.split('\n') {
                self.push_line(LineKind::Assistant, line);
            }
        }
    }

    fn append_token(&mut self, text: &str) {
        if self.thinking_shown {
            if self
                .lines
                .last()
                .is_some_and(|l| l.text == "\u{25cf} Thinking...")
            {
                self.lines.pop();
            }
            self.thinking_shown = false;
        }

        self.current_partial.push_str(text);

        while let Some(pos) = self.current_partial.find('\n') {
            let line = self.current_partial[..pos].to_string();
            self.lines.push(StyledLine {
                kind: LineKind::Assistant,
                text: line,
            });
            self.current_partial = self.current_partial[pos + 1..].to_string();
        }

        if self.auto_scroll {
            self.scroll = usize::MAX;
        }
    }

    // ── Event handlers ─────────────────────────────────────────────

    fn handle_agent_event(&mut self, event: AgentEvent) {
        match event {
            AgentEvent::Info(text) => self.push_line(LineKind::Info, &text),
            AgentEvent::Thinking => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25cf} Thinking...");
                self.thinking_shown = true;
            }
            AgentEvent::Token(text) => self.append_token(&text),
            AgentEvent::ResponseComplete(_) => {}
            AgentEvent::ScriptStarted { .. } => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25b6} Running script...");
            }
            AgentEvent::ScriptFinished(result) => {
                if self
                    .lines
                    .last()
                    .is_some_and(|l| l.text == "\u{25b6} Running script...")
                {
                    self.lines.pop();
                }
                if !result.stdout.is_empty() {
                    self.push_line(
                        LineKind::Separator,
                        "\u{2500}\u{2500}\u{2500} output \u{2500}\u{2500}\u{2500}",
                    );
                    for line in result.stdout.lines() {
                        self.push_line(LineKind::Output, line);
                    }
                    self.push_line(
                        LineKind::Separator,
                        "\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    );
                }
                if !result.stderr.is_empty() {
                    for line in result.stderr.lines() {
                        self.push_line(LineKind::Error, line);
                    }
                }
                let exit_ok = result.exit_code == Some(0);
                if !exit_ok {
                    self.push_line(
                        LineKind::Error,
                        &format!(
                            "exit: {}",
                            result.exit_code.map_or("unknown".into(), |c| c.to_string())
                        ),
                    );
                }
            }
            AgentEvent::Done(_) => {
                self.flush_partial();
                self.push_line(LineKind::Separator, "");
                self.busy = false;
            }
            AgentEvent::Interrupted => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25cf} Interrupted");
                self.busy = false;
            }
            AgentEvent::Error(msg) => {
                self.flush_partial();
                self.push_line(LineKind::Error, &msg);
                self.busy = false;
            }
            AgentEvent::DataClassification {
                source,
                classification,
            } => {
                self.push_line(
                    LineKind::Info,
                    &format!("Classification [{source}]: {classification}"),
                );
            }
            AgentEvent::SecurityViolation { description } => {
                self.push_line(LineKind::Error, &format!("SECURITY: {description}"));
            }
            AgentEvent::Reprompt { reason, attempt } => {
                self.flush_partial();
                self.push_line(
                    LineKind::Info,
                    &format!("\u{26a0} Re-prompting (attempt {attempt}): {reason}"),
                );
            }
            AgentEvent::McpInstallProgress(msg) => {
                self.push_line(LineKind::Info, &format!("[mcp] {msg}"));
            }
            AgentEvent::McpInstallComplete { tools } => {
                self.push_line(LineKind::Info, &format!("[mcp] Installed {} tool(s):", tools.len()));
                for t in &tools {
                    self.push_line(LineKind::Info, &format!("[mcp]   • {t}"));
                }
                self.busy = false;
            }
        }
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        cmd_tx: &mpsc::UnboundedSender<AgentCommand>,
        interrupt: &Arc<Notify>,
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
        project_dir: &std::path::Path,
        cred_req_tx: &mpsc::UnboundedSender<McpCredentialRequest>,
    ) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        // Handle pending MCP credential prompt first.
        if let Some(ref pending) = self.pending_mcp_credential {
            let stage_label = match &pending.stage {
                CredentialStage::EnvVarName => "env var name",
                CredentialStage::Value(_) => "credential value",
            };
            let _ = stage_label; // used in render only
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    if let Some(p) = self.pending_mcp_credential.take() {
                        p.response_tx.send(None).ok();
                    }
                    self.push_line(LineKind::Info, "[mcp] Credential entry cancelled.");
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
                                self.push_line(LineKind::Info, "[mcp] No credential — skipped.");
                                self.busy = false;
                            } else {
                                let env_var = input.clone();
                                self.push_line(
                                    LineKind::Info,
                                    &format!("[mcp] Enter value for {env_var} (blank to skip):"),
                                );
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
                                self.push_line(LineKind::Info, "[mcp] No credential — skipped.");
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
                    if self.input_cursor > 0 {
                        self.input_cursor -= 1;
                    }
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
                .is_some_and(|p| p.choosing_suggestion);

            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    self.pending_approval.take();
                    if self.busy {
                        interrupt.notify_one();
                    } else {
                        cmd_tx.send(AgentCommand::Quit).ok();
                        self.should_quit = true;
                    }
                }
                (_, KeyCode::Char('y')) | (_, KeyCode::Char('Y')) if !choosing => {
                    if let Some(p) = self.pending_approval.take() {
                        p.response_tx
                            .send(ApprovalResponse {
                                verdict: NetworkVerdict::Allow,
                                always_allow: None,
                            })
                            .ok();
                        self.push_line(LineKind::Info, "[network] Allowed (this request only)");
                    }
                }
                (_, KeyCode::Char('a')) | (_, KeyCode::Char('A')) if !choosing => {
                    let descs: Vec<String> = self
                        .pending_approval
                        .as_ref()
                        .map(|p| {
                            p.suggestions
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
                        self.push_line(
                            LineKind::Info,
                            "[network] Choose a rule to always allow (type a number):",
                        );
                        for d in descs {
                            self.push_line(LineKind::Info, &d);
                        }
                    }
                }
                (_, KeyCode::Char(c)) if choosing && c.is_ascii_digit() => {
                    let idx = c.to_digit(10).unwrap_or(0) as usize;
                    if let Some(p) = self.pending_approval.take() {
                        if idx >= 1 && idx <= p.suggestions.len() {
                            let suggestion = p.suggestions.into_iter().nth(idx - 1).unwrap();
                            let desc = suggestion.describe();
                            p.response_tx
                                .send(ApprovalResponse {
                                    verdict: NetworkVerdict::Allow,
                                    always_allow: Some(suggestion),
                                })
                                .ok();
                            self.push_line(
                                LineKind::Info,
                                &format!("[network] Always allow: {desc}"),
                            );
                        } else {
                            self.push_line(LineKind::Info, "[network] Invalid selection");
                            // Put it back without choosing mode
                            // (approval is dropped; fall through to block-safe path)
                            p.response_tx
                                .send(ApprovalResponse {
                                    verdict: NetworkVerdict::Block,
                                    always_allow: None,
                                })
                                .ok();
                            self.push_line(LineKind::Info, "[network] Blocked (invalid selection)");
                        }
                    }
                }
                (_, KeyCode::Char('n')) | (_, KeyCode::Char('N')) if !choosing => {
                    if let Some(p) = self.pending_approval.take() {
                        p.response_tx
                            .send(ApprovalResponse {
                                verdict: NetworkVerdict::Block,
                                always_allow: None,
                            })
                            .ok();
                        self.push_line(LineKind::Info, "[network] Blocked");
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
                    cmd_tx.send(AgentCommand::Quit).ok();
                    self.should_quit = true;
                }
            }
            (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => {
                cmd_tx.send(AgentCommand::Quit).ok();
                self.should_quit = true;
            }
            (_, KeyCode::Enter) => {
                if !self.busy && !self.input.is_empty() {
                    let msg = std::mem::take(&mut self.input);
                    self.input_cursor = 0;

                    if msg.eq_ignore_ascii_case("quit") || msg.eq_ignore_ascii_case("exit") {
                        cmd_tx.send(AgentCommand::Quit).ok();
                        self.should_quit = true;
                        return;
                    }

                    // `/mcp <sub>` — MCP server management slash commands.
                    if msg.starts_with("/mcp ") || msg == "/mcp" {
                        self.handle_mcp_slash(
                            msg[4..].trim(),
                            event_tx,
                            project_dir,
                            cred_req_tx,
                        );
                        return;
                    }

                    if msg.starts_with('!') {
                        let shell_cmd = msg[1..].trim().to_string();
                        if shell_cmd.is_empty() {
                            self.push_line(LineKind::Info, "Usage: ! <shell command>");
                            return;
                        }
                        self.push_line(LineKind::User, &format!("> {msg}"));
                        self.push_line(LineKind::Separator, "");
                        cmd_tx.send(AgentCommand::DirectCommand(shell_cmd)).ok();
                        self.busy = true;
                        self.auto_scroll = true;
                        return;
                    }

                    self.push_line(LineKind::User, &format!("> {msg}"));
                    self.push_line(LineKind::Separator, "");
                    cmd_tx.send(AgentCommand::Message(msg)).ok();
                    self.busy = true;
                    self.auto_scroll = true;
                }
            }
            (_, KeyCode::Backspace) => {
                if !self.busy && self.input_cursor > 0 {
                    self.input_cursor -= 1;
                    self.input.remove(self.input_cursor);
                }
            }
            (_, KeyCode::Left) => {
                if self.input_cursor > 0 {
                    self.input_cursor -= 1;
                }
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
                self.scroll += 1;
            }
            (_, KeyCode::PageUp) => {
                self.auto_scroll = false;
                self.scroll = self.scroll.saturating_sub(20);
            }
            (_, KeyCode::PageDown) => {
                self.scroll += 20;
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
        event_tx: &mpsc::UnboundedSender<AgentEvent>,
        project_dir: &std::path::Path,
        cred_req_tx: &mpsc::UnboundedSender<McpCredentialRequest>,
    ) {
        if self.busy {
            self.push_line(LineKind::Info, "[mcp] Command rejected — agent is busy.");
            return;
        }

        let (sub, arg) = match rest.split_once(' ') {
            Some((s, a)) => (s.trim(), a.trim()),
            None => (rest.trim(), ""),
        };

        match sub {
            "list" => {
                match list_servers(project_dir) {
                    Ok(lines) => {
                        for l in lines {
                            self.push_line(LineKind::Info, &l);
                        }
                    }
                    Err(e) => self.push_line(LineKind::Error, &format!("[mcp] {e}")),
                }
            }
            "remove" => {
                let name = arg.trim();
                if name.is_empty() {
                    self.push_line(LineKind::Info, "Usage: /mcp remove <name>");
                    return;
                }
                match remove_server(name, project_dir) {
                    Ok(()) => self.push_line(LineKind::Info, &format!("[mcp] Removed '{name}'.")),
                    Err(e) => self.push_line(LineKind::Error, &format!("[mcp] {e}")),
                }
            }
            "install" => {
                if arg.is_empty() {
                    self.push_line(
                        LineKind::Info,
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

                self.push_line(
                    LineKind::Info,
                    &format!("[mcp] Installing '{server_name}'..."),
                );
                self.busy = true;

                let event_tx = event_tx.clone();
                let cred_req_tx = cred_req_tx.clone();
                let project_dir = project_dir.to_path_buf();

                tokio::runtime::Handle::current().spawn(async move {
                    run_mcp_install_task(
                        spec,
                        server_name,
                        trust,
                        project_dir,
                        event_tx,
                        cred_req_tx,
                    )
                    .await;
                });
            }
            _ => {
                self.push_line(
                    LineKind::Info,
                    "Usage: /mcp install <spec> | /mcp list | /mcp remove <name>",
                );
            }
        }
    }

    // ── Rendering ──────────────────────────────────────────────────

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(1),    // output
            Constraint::Length(3), // input
        ])
        .split(area);

        // Header bar
        let header_spans = vec![
            Span::styled(
                "hick-basic",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.provider_info, Style::default().fg(Color::DarkGray)),
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

        if !self.current_partial.is_empty() {
            text_lines.push(Line::styled(
                self.current_partial.clone(),
                Style::default().fg(Color::White),
            ));
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
                    p.host
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
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    cred_req_tx: mpsc::UnboundedSender<McpCredentialRequest>,
) {
    event_tx
        .send(AgentEvent::McpInstallProgress(
            "Skipping WASM package install (server must be available on PATH).".into(),
        ))
        .ok();

    event_tx
        .send(AgentEvent::McpInstallProgress("Verifying server connection...".into()))
        .ok();
    let tools = match McpInstaller::verify_connection(&spec).await {
        Ok(t) => t,
        Err(e) => {
            event_tx
                .send(AgentEvent::Error(format!("[mcp] Connection failed: {e}")))
                .ok();
            event_tx
                .send(AgentEvent::McpInstallComplete { tools: vec![] })
                .ok();
            return;
        }
    };

    if let Err(e) = McpInstaller::write_config(&spec, &server_name, trust, &project_dir) {
        event_tx
            .send(AgentEvent::Error(format!("[mcp] Config write failed: {e}")))
            .ok();
        event_tx
            .send(AgentEvent::McpInstallComplete { tools: vec![] })
            .ok();
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
            McpInstaller::write_credentials(
                &server_name,
                vec![(env_var, value)],
                &project_dir,
            )
            .ok();
        }
    }

    let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
    event_tx
        .send(AgentEvent::McpInstallComplete { tools: tool_names })
        .ok();
}

// ── Public entry point ─────────────────────────────────────────────

/// Run the ratatui TUI. Blocks until the user quits.
///
/// Identical to the hick-agent TUI but with "hick-basic" in the header bar.
pub fn run_tui(
    mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    cmd_tx: mpsc::UnboundedSender<AgentCommand>,
    interrupt: Arc<Notify>,
    provider_info: String,
    mut approval_rx: mpsc::UnboundedReceiver<NetworkApprovalRequest>,
    project_dir: PathBuf,
) -> anyhow::Result<()> {
    use std::io;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(provider_info);
    let (cred_req_tx, mut cred_req_rx) = mpsc::unbounded_channel::<McpCredentialRequest>();

    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| app.render(f))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key, &cmd_tx, &interrupt, &event_tx, &project_dir, &cred_req_tx);
                    if app.should_quit {
                        break;
                    }
                }
            }

            while let Ok(agent_event) = event_rx.try_recv() {
                app.handle_agent_event(agent_event);
            }

            if app.pending_approval.is_none() {
                if let Ok(approval) = approval_rx.try_recv() {
                    app.push_line(
                        LineKind::Info,
                        &format!(
                            "[network] Request to {} \u{2014} y = once, a = always, n = block",
                            approval.host
                        ),
                    );
                    app.pending_approval = Some(PendingApproval {
                        host: approval.host,
                        suggestions: approval.suggestions,
                        response_tx: approval.response_tx,
                        choosing_suggestion: false,
                    });
                }
            }

            // Check for MCP credential requests from install tasks.
            if app.pending_mcp_credential.is_none() {
                if let Ok(req) = cred_req_rx.try_recv() {
                    app.push_line(
                        LineKind::Info,
                        &format!(
                            "[mcp] Credential needed for '{}' — enter env var name (blank to skip):",
                            req.server_name
                        ),
                    );
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
