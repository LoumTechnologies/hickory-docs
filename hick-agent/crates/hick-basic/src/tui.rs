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
use hick_agent_sdk::network_filter::{NetworkApprovalRequest, NetworkVerdict};
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
    /// Pending network approval prompt — awaiting y/n from the user.
    pending_approval: Option<oneshot::Sender<NetworkVerdict>>,
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
                    .unwrap_or("basic.hick")
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
            log,
            log_display,
            pending_response: None,
            pending_action: None,
            session_action_index: 0,
            in_direct_command: false,
        }
    }

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
            AgentEvent::ResponseComplete(response) => {
                self.pending_response = Some(response);
            }
            AgentEvent::ScriptStarted { code, lang } => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25b6} Running script...");
                self.pending_action = Some((code, lang));
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
                if self.in_direct_command {
                    self.log_command_observation(&result);
                } else {
                    self.log_turn_with_action(&result);
                }
            }
            AgentEvent::Done(_) => {
                self.flush_partial();
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
        }
    }

    fn handle_key(
        &mut self,
        key: crossterm::event::KeyEvent,
        cmd_tx: &mpsc::UnboundedSender<AgentCommand>,
        interrupt: &Arc<Notify>,
    ) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.pending_approval.is_some() {
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
                (_, KeyCode::Char('y')) | (_, KeyCode::Char('Y')) => {
                    if let Some(tx) = self.pending_approval.take() {
                        tx.send(NetworkVerdict::Allow).ok();
                        self.push_line(LineKind::Info, "[network] Allowed");
                    }
                }
                (_, KeyCode::Char('n')) | (_, KeyCode::Char('N')) => {
                    if let Some(tx) = self.pending_approval.take() {
                        tx.send(NetworkVerdict::Block).ok();
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
                "hick-basic",
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

        let input_display = if self.pending_approval.is_some() {
            "\u{26a0} Network approval required \u{2014} y to allow, n to block".to_string()
        } else if self.busy {
            "\u{25cf} Agent is working... (Ctrl+C to interrupt)".to_string()
        } else {
            format!("> {}", self.input)
        };

        let input_style = if self.pending_approval.is_some() {
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
/// Identical to the hick-agent TUI but with "hick-basic" in the header bar.
pub fn run_tui(
    mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    cmd_tx: mpsc::UnboundedSender<AgentCommand>,
    interrupt: Arc<Notify>,
    provider_info: String,
    mut approval_rx: mpsc::UnboundedReceiver<NetworkApprovalRequest>,
    log_path: Option<PathBuf>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(provider_info, log_path);

    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| app.render(f))?;

            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key, &cmd_tx, &interrupt);
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
                            "[network] Request to {} \u{2014} allow? (y/n)",
                            approval.host
                        ),
                    );
                    app.pending_approval = Some(approval.response_tx);
                }
            }
        }
        Ok(())
    })();

    if let Some(ref mut w) = app.log {
        let now = chrono::Local::now();
        let _ = writeln!(w, r#"  <hick:session-end time="{}"/>"#, now.to_rfc3339());
        let _ = writeln!(w, "</hick:session>");
        let _ = w.flush();
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
