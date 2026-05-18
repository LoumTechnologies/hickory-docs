//! Ratatui-based terminal UI for the agent.
//!
//! Consumes [`AgentEvent`]s from the core and renders them in a scrollable
//! output area with styled text. User input is captured via crossterm key
//! events and sent back as [`AgentCommand`]s.

use std::io::{self, BufWriter, Write as _};
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
use hick_agent_script_first::tools::ScriptResult;

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

/// Which field of a credential we are currently collecting.
enum CredentialStage {
    /// Prompting for the environment variable name.
    EnvVarName,
    /// Prompting for the credential value; holds the env var name already entered.
    Value(String),
}

/// Pending MCP credential prompt — awaiting user input for env var name then value.
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
    /// .hick session file writer (None if log could not be opened).
    log: Option<BufWriter<std::fs::File>>,
    /// Display path shown in the header (just the filename).
    log_display: String,
    /// Buffered LLM response from ResponseComplete, not yet written.
    pending_response: Option<String>,
    /// Buffered script code + lang from ScriptStarted, not yet written.
    pending_action: Option<(String, String)>,
    /// Counter for action/command source attributes.
    session_action_index: usize,
    /// True when the current busy turn is a `!` direct command (not an LLM turn).
    in_direct_command: bool,
}

impl App {
    fn new(provider_info: String, log_path: Option<PathBuf>) -> Self {
        let (log, log_display) = match log_path {
            Some(path) => {
                let display = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("agent.hick")
                    .to_string();
                match std::fs::File::create(&path) {
                    Ok(f) => {
                        let mut w = BufWriter::new(f);
                        let now = chrono::Local::now();
                        let _ = writeln!(w, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
                        let _ = writeln!(
                            w,
                            r#"<hick:session xmlns:hick="http://www.hickorydocs.com/1.0" start="{}">"#,
                            now.to_rfc3339()
                        );
                        let _ = w.flush();
                        (Some(w), display)
                    }
                    Err(_) => (None, display),
                }
            }
            None => (None, String::new()),
        };
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
            log,
            log_display,
            pending_response: None,
            pending_action: None,
            session_action_index: 0,
            in_direct_command: false,
        }
    }

    /// Write a `hick:command` node for a `!`-prefixed direct command.
    fn log_command(&mut self, cmd: &str) {
        if let Some(ref mut w) = self.log {
            let now = chrono::Local::now();
            let _ = writeln!(
                w,
                r#"  <hick:command time="{}">{}</hick:command>"#,
                now.to_rfc3339(),
                xml_escape(cmd)
            );
            let _ = w.flush();
        }
    }

    /// Write a `hick:observation` node for a direct command result.
    fn log_command_observation(&mut self, result: &ScriptResult) {
        let source_id = format!("command-{}", self.session_action_index);
        self.session_action_index += 1;
        if let Some(ref mut w) = self.log {
            let now = chrono::Local::now();
            let exit_attr = result
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".into());
            let combined = format!("{}{}", result.stdout, result.stderr);
            let _ = writeln!(
                w,
                r#"  <hick:observation source="{}" exit="{}" time="{}">{}</hick:observation>"#,
                source_id,
                exit_attr,
                now.to_rfc3339(),
                xml_escape(&combined)
            );
            let _ = w.flush();
        }
    }

    /// Write a `hick:user` node for the given message text.
    fn log_user(&mut self, text: &str) {
        if let Some(ref mut w) = self.log {
            let now = chrono::Local::now();
            let _ = writeln!(
                w,
                r#"  <hick:user time="{}">{}</hick:user>"#,
                now.to_rfc3339(),
                xml_escape(text)
            );
            let _ = w.flush();
        }
    }

    /// Write `hick:assistant` + `hick:action` + `hick:observation` for a turn that
    /// executed a script.  Consumes `pending_response` and `pending_action`.
    fn log_turn_with_action(&mut self, result: &ScriptResult) {
        if self.log.is_none() {
            self.pending_response.take();
            self.pending_action.take();
            return;
        }
        let now = chrono::Local::now();
        let response = self.pending_response.take().unwrap_or_default();
        let (code, lang) = self.pending_action.take().unwrap_or_else(|| (String::new(), "sh".into()));
        let action_id = format!("action-{}", self.session_action_index);
        self.session_action_index += 1;

        if let Some(ref mut w) = self.log {
            let _ = writeln!(w, r#"  <hick:assistant time="{}">"#, now.to_rfc3339());
            if !response.trim().is_empty() {
                let _ = writeln!(w, "{}", xml_escape(response.trim()));
            }
            let _ = writeln!(w, r#"    <hick:action lang="{}">"#, lang);
            let _ = w.write_all(code.as_bytes());
            if !code.ends_with('\n') {
                let _ = w.write_all(b"\n");
            }
            let _ = writeln!(w, "    </hick:action>");
            let _ = writeln!(w, "  </hick:assistant>");

            let exit_attr = result
                .exit_code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".into());
            let combined = format!("{}{}", result.stdout, result.stderr);
            let _ = writeln!(
                w,
                r#"  <hick:observation source="{}" exit="{}" time="{}">{}</hick:observation>"#,
                action_id,
                exit_attr,
                now.to_rfc3339(),
                xml_escape(&combined)
            );
            let _ = w.flush();
        }
    }

    /// Write `hick:assistant` for a final-answer turn (no script executed).
    /// Consumes `pending_response`.
    fn log_turn_done(&mut self) {
        let response = match self.pending_response.take() {
            Some(r) if !r.trim().is_empty() => r,
            _ => return,
        };
        self.pending_action.take();
        if let Some(ref mut w) = self.log {
            let now = chrono::Local::now();
            let _ = writeln!(
                w,
                r#"  <hick:assistant time="{}">{}</hick:assistant>"#,
                now.to_rfc3339(),
                xml_escape(response.trim())
            );
            let _ = w.flush();
        }
    }

    /// Write an informational hick: node (not replayed, just for the transcript).
    fn log_info_node(&mut self, tag: &str, attrs: &[(&str, &str)], content: &str) {
        if let Some(ref mut w) = self.log {
            let now = chrono::Local::now();
            let attr_str = attrs
                .iter()
                .map(|(k, v)| format!(r#" {}="{}""#, k, xml_escape(v)))
                .collect::<String>();
            if content.is_empty() {
                let _ = writeln!(
                    w,
                    r#"  <hick:{} time="{}"{}/>"#,
                    tag,
                    now.to_rfc3339(),
                    attr_str
                );
            } else {
                let _ = writeln!(
                    w,
                    r#"  <hick:{} time="{}"{}>{}</hick:{}>"#,
                    tag,
                    now.to_rfc3339(),
                    attr_str,
                    xml_escape(content),
                    tag
                );
            }
            let _ = w.flush();
        }
    }

    fn push_line(&mut self, kind: LineKind, text: &str) {
        self.lines.push(StyledLine {
            kind,
            text: text.to_string(),
        });
        if self.auto_scroll {
            self.scroll = usize::MAX; // clamped during render
        }
    }

    fn flush_partial(&mut self) {
        if !self.current_partial.is_empty() {
            let text = std::mem::take(&mut self.current_partial);
            // Split into individual lines to preserve line structure
            for line in text.split('\n') {
                self.push_line(LineKind::Assistant, line);
            }
        }
    }

    fn append_token(&mut self, text: &str) {
        // Remove thinking indicator on first token
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

        // Flush complete lines (everything before the last newline)
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
            AgentEvent::ResponseComplete(response) => {
                // Buffer the full response; written to session file after script result arrives
                // (or on Done if no script is executed this turn).
                self.pending_response = Some(response);
            }
            AgentEvent::ScriptStarted { code, lang } => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25b6} Running script...");
                self.pending_action = Some((code, lang));
            }
            AgentEvent::ScriptFinished(result) => {
                // Replace "Running script..." indicator
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
                // Write session nodes: direct commands use hick:observation only
                // (hick:command was already written in handle_key); LLM turns write
                // hick:assistant + hick:action + hick:observation.
                if self.in_direct_command {
                    self.log_command_observation(&result);
                } else {
                    self.log_turn_with_action(&result);
                }
            }
            AgentEvent::Done(_) => {
                self.flush_partial();
                // Write any buffered response that wasn't paired with a script.
                if !self.in_direct_command {
                    self.log_turn_done();
                }
                self.in_direct_command = false;
                self.push_line(LineKind::Separator, "");
                self.busy = false;
            }
            AgentEvent::Interrupted => {
                self.flush_partial();
                self.pending_response.take();
                self.pending_action.take();
                self.in_direct_command = false;
                self.push_line(LineKind::Info, "\u{25cf} Interrupted");
                self.log_info_node("interrupted", &[], "");
                self.busy = false;
            }
            AgentEvent::Error(msg) => {
                self.flush_partial();
                self.pending_response.take();
                self.pending_action.take();
                self.in_direct_command = false;
                self.push_line(LineKind::Error, &msg);
                self.log_info_node("error", &[], &msg);
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
                self.log_info_node("security-violation", &[], &description);
            }
            AgentEvent::Reprompt { reason, attempt } => {
                self.flush_partial();
                self.push_line(
                    LineKind::Info,
                    &format!("\u{26a0} Re-prompting (attempt {attempt}): {reason}"),
                );
                self.log_info_node("reprompt", &[("attempt", &attempt.to_string())], &reason);
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
                                // Ask for the value next.
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

        // Handle pending network approval prompts first
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

                    // `! <cmd>` — run directly in the WASM container, no LLM call.
                    if msg.starts_with('!') {
                        let shell_cmd = msg[1..].trim().to_string();
                        if shell_cmd.is_empty() {
                            self.push_line(LineKind::Info, "Usage: ! <shell command>");
                            return;
                        }
                        self.push_line(LineKind::User, &format!("> {msg}"));
                        self.push_line(LineKind::Separator, "");
                        self.log_command(&shell_cmd);
                        self.in_direct_command = true;
                        cmd_tx.send(AgentCommand::DirectCommand(shell_cmd)).ok();
                        self.busy = true;
                        self.auto_scroll = true;
                        return;
                    }

                    self.push_line(LineKind::User, &format!("> {msg}"));
                    self.push_line(LineKind::Separator, "");
                    self.log_user(&msg);
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
                // auto_scroll re-enabled during render if at bottom
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

                // Simple arg parsing: everything before any --flag is the spec.
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
        let mut header_spans = vec![
            Span::styled(
                "hick-agent",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.provider_info, Style::default().fg(Color::DarkGray)),
        ];
        if !self.log_display.is_empty() {
            header_spans.push(Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)));
            header_spans.push(Span::styled(
                format!("log: {}", self.log_display),
                Style::default().fg(Color::DarkGray),
            ));
        }
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

        // Append the partial (still-streaming) line
        if !self.current_partial.is_empty() {
            text_lines.push(Line::styled(
                self.current_partial.clone(),
                Style::default().fg(Color::White),
            ));
        }

        // Inner height = block height minus 2 for the borders
        let visible_height = chunks[1].height.saturating_sub(2) as usize;
        // Inner width = block width minus 2 for the borders
        let inner_width = chunks[1].width.saturating_sub(2) as usize;

        // Count total visual rows accounting for line wrapping — Paragraph::scroll uses
        // visual row offsets, so max_scroll must be in the same units.
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

        // Input area
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
        } else if let Some(ref pa) = self.pending_approval {
            if pa.choosing_suggestion {
                let opts: Vec<String> = pa
                    .suggestions
                    .iter()
                    .enumerate()
                    .map(|(i, s)| format!("{}: {}", i + 1, s.describe()))
                    .collect();
                format!("\u{26a0} Always Allow rule \u{2014} pick a number: {}", opts.join("  "))
            } else {
                format!(
                    "\u{26a0} Allow {}? \u{2014} y = once, a = always (with rule), n = block",
                    pa.host
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

        // Cursor position (only visible when idle)
        if !self.busy {
            let cursor_x = chunks[2].x + 1 + 2 + self.input_cursor as u16; // border + "> "
            let cursor_y = chunks[2].y + 1; // top border
            frame.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

// ── /mcp helpers ───────────────────────────────────────────────────

/// Parse `/mcp install` arguments from a raw string.
///
/// Returns `(spec, name_override, trust)`. Flags `--name` and `--trust` are
/// stripped from the spec string.
fn parse_mcp_install_args(raw: &str) -> (&str, Option<&str>, McpServerTrust) {
    let mut name: Option<&str> = None;
    let mut trust = McpServerTrust::Prompt;
    let mut spec_end = raw.len();

    // Scan for --name and --trust flags.
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
            // Skip a byte safely (stay on char boundaries).
            i += bytes[i].leading_ones().max(1) as usize;
        }
    }

    let spec = raw[..spec_end].trim().trim_matches('"');
    (spec, name, trust)
}

/// Background task that drives a `/mcp install` request inside the TUI.
async fn run_mcp_install_task(
    spec: McpInstallSpec,
    server_name: String,
    trust: McpServerTrust,
    project_dir: std::path::PathBuf,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    cred_req_tx: mpsc::UnboundedSender<McpCredentialRequest>,
) {
    // Package install step (skipped — no WASM executor available in TUI slash command).
    event_tx
        .send(AgentEvent::McpInstallProgress(
            "Skipping WASM package install (server must be available on PATH).".into(),
        ))
        .ok();

    // Verify connection.
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

    // Write config.
    if let Err(e) = McpInstaller::write_config(&spec, &server_name, trust, &project_dir) {
        event_tx
            .send(AgentEvent::Error(format!("[mcp] Config write failed: {e}")))
            .ok();
        event_tx
            .send(AgentEvent::McpInstallComplete { tools: vec![] })
            .ok();
        return;
    }

    // Credential prompt via TUI overlay.
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

// ── XML helpers ────────────────────────────────────────────────────

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── Public entry point ─────────────────────────────────────────────

/// Run the ratatui TUI. Blocks until the user quits.
///
/// `log_path` — if provided, an XML session log is written there for the
/// full duration of the session. The path is shown in the header bar.
///
/// This function takes ownership of the event/command channels, the interrupt
/// signal, and an optional network approval channel. It should be called from
/// a blocking context (e.g. `tokio::task::spawn_blocking`).
pub fn run_tui(
    mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    cmd_tx: mpsc::UnboundedSender<AgentCommand>,
    interrupt: Arc<Notify>,
    provider_info: String,
    mut approval_rx: mpsc::UnboundedReceiver<NetworkApprovalRequest>,
    project_dir: PathBuf,
    log_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(provider_info, log_path);
    // Channel for credential requests from spawned install tasks.
    let (cred_req_tx, mut cred_req_rx) = mpsc::unbounded_channel::<McpCredentialRequest>();

    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| app.render(f))?;

            // Poll crossterm events (~60 fps)
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(
                        key,
                        &cmd_tx,
                        &interrupt,
                        &event_tx,
                        &project_dir,
                        &cred_req_tx,
                    );
                    if app.should_quit {
                        break;
                    }
                }
            }

            // Drain all pending agent events
            while let Ok(agent_event) = event_rx.try_recv() {
                app.handle_agent_event(agent_event);
            }

            // Check for network approval requests (only accept one at a time)
            if app.pending_approval.is_none() {
                if let Ok(approval) = approval_rx.try_recv() {
                    app.push_line(
                        LineKind::Info,
                        &format!(
                            "[network] Request to {} \u{2014} allow? (y/a/n)",
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

    // Close the session file
    if let Some(ref mut w) = app.log {
        let now = chrono::Local::now();
        let _ = writeln!(w, r#"  <hick:session-end time="{}"/>"#, now.to_rfc3339());
        let _ = writeln!(w, "</hick:session>");
        let _ = w.flush();
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
