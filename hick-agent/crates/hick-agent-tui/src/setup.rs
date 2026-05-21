//! First-run API key setup screen — shown when no keys are configured.
//!
//! Renders a full-screen ratatui form with three optional provider fields.
//! On submit it returns a [`SetupResult`] with whatever the user typed.
//! A concurrent async path (browser modal) can also set the `done` flag;
//! in that case the TUI polls and exits cleanly.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Frame;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;

/// Keys collected by the setup screen.
#[derive(Debug, Clone, Default)]
pub struct SetupResult {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub xai: Option<String>,
}

impl SetupResult {
    pub fn has_any(&self) -> bool {
        [&self.anthropic, &self.openai, &self.xai]
            .iter()
            .any(|k| k.as_deref().is_some_and(|s| !s.is_empty()))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Anthropic,
    OpenAi,
    Xai,
}

const FIELDS: [Field; 3] = [Field::Anthropic, Field::OpenAi, Field::Xai];

struct SetupApp {
    anthropic: String,
    openai: String,
    xai: String,
    active: Field,
    cursor: usize,
    submitted: bool,
    error: Option<String>,
}

impl SetupApp {
    fn new() -> Self {
        Self {
            anthropic: String::new(),
            openai: String::new(),
            xai: String::new(),
            active: Field::Anthropic,
            cursor: 0,
            submitted: false,
            error: None,
        }
    }

    fn active_value(&self) -> &str {
        match self.active {
            Field::Anthropic => &self.anthropic,
            Field::OpenAi => &self.openai,
            Field::Xai => &self.xai,
        }
    }

    fn active_value_mut(&mut self) -> &mut String {
        match self.active {
            Field::Anthropic => &mut self.anthropic,
            Field::OpenAi => &mut self.openai,
            Field::Xai => &mut self.xai,
        }
    }

    fn next_field(&mut self) {
        let idx = FIELDS.iter().position(|f| *f == self.active).unwrap_or(0);
        self.active = FIELDS[(idx + 1) % FIELDS.len()];
        self.cursor = self.active_value().len();
        self.error = None;
    }

    fn prev_field(&mut self) {
        let idx = FIELDS.iter().position(|f| *f == self.active).unwrap_or(0);
        self.active = FIELDS[(idx + FIELDS.len() - 1) % FIELDS.len()];
        self.cursor = self.active_value().len();
        self.error = None;
    }

    fn insert_char(&mut self, ch: char) {
        if !ch.is_control() {
            let cursor = self.cursor;
            let val = self.active_value_mut();
            let byte_idx = val
                .char_indices()
                .nth(cursor)
                .map(|(i, _)| i)
                .unwrap_or(val.len());
            val.insert(byte_idx, ch);
            self.cursor += 1;
        }
    }

    fn delete_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let cursor = self.cursor;
        let val = self.active_value_mut();
        let byte_idx = val
            .char_indices()
            .nth(cursor - 1)
            .map(|(i, _)| i)
            .unwrap_or(0);
        val.remove(byte_idx);
        self.cursor -= 1;
    }

    fn try_submit(&mut self) {
        if !self.anthropic.trim().is_empty()
            || !self.openai.trim().is_empty()
            || !self.xai.trim().is_empty()
        {
            self.submitted = true;
        } else {
            self.error = Some("Enter at least one API key to continue.".to_string());
        }
    }

    fn result(&self) -> SetupResult {
        let clean = |s: &str| {
            let t = s.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        };
        SetupResult {
            anthropic: clean(&self.anthropic),
            openai: clean(&self.openai),
            xai: clean(&self.xai),
        }
    }

    fn render(&self, f: &mut Frame) {
        let area = f.area();
        f.render_widget(Clear, area);

        // Background
        let bg = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(bg, area);

        let outer_chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

        // ── Header ──────────────────────────────────────────────────────────
        let header = Paragraph::new(Line::from(vec![
            Span::styled(" Hick Agent — First Run Setup ", Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)),
        ]))
        .alignment(Alignment::Center);
        f.render_widget(header, outer_chunks[0]);

        let inner_chunks = Layout::vertical([
            Constraint::Length(6),  // security callout
            Constraint::Length(1),  // spacer
            Constraint::Length(3),  // anthropic field
            Constraint::Length(3),  // openai field
            Constraint::Length(3),  // xai field
            Constraint::Length(1),  // spacer
            Constraint::Length(3),  // submit hint
            Constraint::Min(0),     // padding
        ])
        .margin(1)
        .split(outer_chunks[1]);

        // ── Security callout ─────────────────────────────────────────────────
        let security_text = vec![
            Line::from(vec![
                Span::styled("  Your keys never leave your machine.  ", Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("  "),
                Span::styled("Secrets scanner", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" — Hick detects and redacts credentials before they"),
            ]),
            Line::from("  ever reach the LLM's context window. You can audit the"),
            Line::from("  scanner rules in .hick/agent-security.toml at any time."),
        ];
        let security_block = Paragraph::new(security_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title(Span::styled(" Security ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))))
            .wrap(Wrap { trim: false });
        f.render_widget(security_block, inner_chunks[0]);

        // ── Key fields ───────────────────────────────────────────────────────
        let field_specs: &[(Field, &str, &str)] = &[
            (Field::Anthropic, "Anthropic (ANTHROPIC_API_KEY)", "sk-ant-..."),
            (Field::OpenAi,    "OpenAI    (OPENAI_API_KEY)",    "sk-..."),
            (Field::Xai,       "xAI       (XAI_API_KEY)",       "xai-..."),
        ];
        for (i, (fld, label, placeholder)) in field_specs.iter().enumerate() {
            let chunk = inner_chunks[2 + i];
            let is_active = self.active == *fld;
            let value = match fld {
                Field::Anthropic => &self.anthropic,
                Field::OpenAi => &self.openai,
                Field::Xai => &self.xai,
            };
            let display = if value.is_empty() {
                Span::styled(placeholder.to_string(), Style::default().fg(Color::DarkGray))
            } else {
                // Mask all but last 4 chars
                let visible = value.len().saturating_sub(4);
                let masked = "•".repeat(visible) + &value[visible..];
                Span::styled(masked, Style::default().fg(Color::White))
            };
            let border_style = if is_active {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            let title_style = if is_active {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .title(Span::styled(format!(" {label} "), title_style));
            let para = Paragraph::new(Line::from(vec![Span::raw(" "), display])).block(block);
            f.render_widget(para, chunk);
        }

        // ── Submit hint ──────────────────────────────────────────────────────
        let hint_style = Style::default().fg(Color::DarkGray);
        let error_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
        let hint_text = if let Some(ref err) = self.error {
            vec![
                Line::from(Span::styled(err.as_str(), error_style)),
                Line::from(Span::styled("Tab/↑↓ to switch field  •  Enter to confirm  •  Ctrl-C to quit", hint_style)),
            ]
        } else {
            vec![
                Line::from(Span::styled("Tab/↑↓ to switch field  •  Enter to confirm  •  Ctrl-C to quit", hint_style)),
                Line::from(Span::styled(
                    "Keys are saved to ~/.hick/keys.toml and loaded automatically on next launch.",
                    hint_style,
                )),
            ]
        };
        let hints = Paragraph::new(hint_text).alignment(Alignment::Center);
        f.render_widget(hints, inner_chunks[6]);

        // ── Status bar ────────────────────────────────────────────────────────
        let status = Paragraph::new(
            Span::styled(
                " Enter one or more API keys — at least one provider is required ",
                Style::default().fg(Color::DarkGray),
            )
        )
        .alignment(Alignment::Center);
        f.render_widget(status, outer_chunks[2]);
    }
}

/// Run the blocking ratatui setup screen.
///
/// Returns the collected [`SetupResult`] once the user submits or `done_flag` is
/// set by the concurrent browser path.  The shared `result_out` is also written
/// so the async side can read it after `done_flag` fires.
pub fn run_setup_tui(
    done_flag: Arc<AtomicBool>,
    result_out: Arc<Mutex<Option<SetupResult>>>,
) -> anyhow::Result<SetupResult> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = SetupApp::new();
    let result = loop {
        terminal.draw(|f| app.render(f))?;

        // Check if browser path completed setup.
        if done_flag.load(Ordering::Acquire) {
            break app.result();
        }

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        // User quit — return empty result; caller will exit.
                        break SetupResult::default();
                    }
                    KeyCode::Tab | KeyCode::Down => app.next_field(),
                    KeyCode::BackTab | KeyCode::Up => app.prev_field(),
                    KeyCode::Enter => app.try_submit(),
                    KeyCode::Backspace => app.delete_back(),
                    KeyCode::Char(ch) => app.insert_char(ch),
                    _ => {}
                }
            }
        }

        if app.submitted {
            break app.result();
        }
        // Re-check external done in case browser finished while we polled.
        if done_flag.load(Ordering::Acquire) {
            break app.result();
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Write result for the async side.
    *result_out.lock().unwrap() = Some(result.clone());
    done_flag.store(true, Ordering::Release);

    Ok(result)
}
