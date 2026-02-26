//! Ratatui-based terminal UI for the agent.
//!
//! Consumes [`AgentEvent`]s from the core and renders them in a scrollable
//! output area with styled text. User input is captured via crossterm key
//! events and sent back as [`AgentCommand`]s.

use std::io;
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

use crate::events::{AgentCommand, AgentEvent};
use crate::network_filter::{NetworkApprovalRequest, NetworkVerdict};

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
            AgentEvent::ResponseComplete(_) => {
                // Tokens already handled inline; nothing to do here.
            }
            AgentEvent::ScriptStarted { .. } => {
                self.flush_partial();
                self.push_line(LineKind::Info, "\u{25b6} Running script...");
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
                if result.exit_code != Some(0) {
                    self.push_line(
                        LineKind::Error,
                        &format!(
                            "exit: {}",
                            result
                                .exit_code
                                .map_or("unknown".into(), |c| c.to_string())
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

        // Handle pending network approval prompts first
        if self.pending_approval.is_some() {
            match (key.modifiers, key.code) {
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) => {
                    // Drop the approval sender (filter fails open)
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
                _ => {} // Ignore other keys during approval
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

    // ── Rendering ──────────────────────────────────────────────────

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Length(1), // header
            Constraint::Min(1),   // output
            Constraint::Length(3), // input
        ])
        .split(area);

        // Header bar
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                "hick-agent",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" \u{2502} ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.provider_info, Style::default().fg(Color::DarkGray)),
        ]));
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

        let total_lines = text_lines.len();
        // Inner height = block height minus 2 for the borders
        let visible_height = chunks[1].height.saturating_sub(2) as usize;

        let max_scroll = total_lines.saturating_sub(visible_height);
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

        let input_widget = Paragraph::new(input_display)
            .style(input_style)
            .block(
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

// ── Public entry point ─────────────────────────────────────────────

/// Run the ratatui TUI. Blocks until the user quits.
///
/// This function takes ownership of the event/command channels, the interrupt
/// signal, and an optional network approval channel. It should be called from
/// a blocking context (e.g. `tokio::task::spawn_blocking`).
pub fn run_tui(
    mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
    cmd_tx: mpsc::UnboundedSender<AgentCommand>,
    interrupt: Arc<Notify>,
    provider_info: String,
    mut approval_rx: mpsc::UnboundedReceiver<NetworkApprovalRequest>,
) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(provider_info);

    let result = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| app.render(f))?;

            // Poll crossterm events (~60 fps)
            if event::poll(Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    app.handle_key(key, &cmd_tx, &interrupt);
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
                        &format!("[network] Request to {} \u{2014} allow? (y/n)", approval.host),
                    );
                    app.pending_approval = Some(approval.response_tx);
                }
            }
        }
        Ok(())
    })();

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
