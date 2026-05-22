//! First-run API key setup screen — shown when no keys are configured.
//!
//! A single smart input field accepts any supported API key and auto-detects
//! the provider from the key prefix. Unknown prefixes fall back to a manual
//! Tab-cycled provider selector. Multiple keys (one per provider) can be
//! added before submitting.

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

// ---------------------------------------------------------------------------
// Provider detection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProviderKind {
    Anthropic,
    OpenAI,
    XAI,
    OpenRouter,
}

const ALL_PROVIDERS: [ProviderKind; 4] = [
    ProviderKind::Anthropic,
    ProviderKind::OpenAI,
    ProviderKind::XAI,
    ProviderKind::OpenRouter,
];

impl ProviderKind {
    fn display(self) -> &'static str {
        match self {
            ProviderKind::Anthropic => "Anthropic Claude",
            ProviderKind::OpenAI => "OpenAI",
            ProviderKind::XAI => "xAI / Grok",
            ProviderKind::OpenRouter => "OpenRouter",
        }
    }
}

/// Infer provider from a key's prefix, or `None` if unrecognised.
pub(crate) fn detect_from_key(key: &str) -> Option<ProviderKind> {
    let k = key.trim();
    if k.starts_with("sk-ant-") {
        Some(ProviderKind::Anthropic)
    } else if k.starts_with("sk-or-") {
        Some(ProviderKind::OpenRouter)
    } else if k.starts_with("xai-") {
        Some(ProviderKind::XAI)
    } else if k.starts_with("sk-") {
        Some(ProviderKind::OpenAI)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Public result type
// ---------------------------------------------------------------------------

/// Keys collected by the setup screen.
#[derive(Debug, Clone, Default)]
pub struct SetupResult {
    pub anthropic: Option<String>,
    pub openai: Option<String>,
    pub xai: Option<String>,
    pub openrouter: Option<String>,
}

impl SetupResult {
    pub fn has_any(&self) -> bool {
        [&self.anthropic, &self.openai, &self.xai, &self.openrouter]
            .iter()
            .any(|k| k.as_deref().is_some_and(|s| !s.is_empty()))
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct SetupApp {
    /// Raw key text currently in the input field.
    input: String,
    cursor: usize,
    /// Confirmed (provider, key) pairs — at most one per provider; later
    /// additions replace the earlier one for the same provider.
    configured: Vec<(ProviderKind, String)>,
    /// Used when the prefix is unrecognised so the user can pick a provider
    /// with Tab.
    manual_provider: ProviderKind,
    submitted: bool,
    error: Option<String>,
}

impl SetupApp {
    fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            configured: Vec::new(),
            manual_provider: ProviderKind::Anthropic,
            submitted: false,
            error: None,
        }
    }

    /// Auto-detect or fall back to the manual selection.
    fn effective_provider(&self) -> Option<ProviderKind> {
        if self.input.trim().is_empty() {
            None
        } else {
            Some(detect_from_key(&self.input).unwrap_or(self.manual_provider))
        }
    }

    fn insert_char(&mut self, ch: char) {
        if !ch.is_control() {
            let cursor = self.cursor;
            let byte_idx = self
                .input
                .char_indices()
                .nth(cursor)
                .map(|(i, _)| i)
                .unwrap_or(self.input.len());
            self.input.insert(byte_idx, ch);
            self.cursor += 1;
            self.error = None;
        }
    }

    fn delete_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let cursor = self.cursor;
        let byte_idx = self
            .input
            .char_indices()
            .nth(cursor - 1)
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.remove(byte_idx);
        self.cursor -= 1;
        self.error = None;
    }

    fn cycle_manual_provider(&mut self) {
        // Only meaningful when auto-detection fails.
        if detect_from_key(&self.input).is_none() {
            let idx = ALL_PROVIDERS
                .iter()
                .position(|p| *p == self.manual_provider)
                .unwrap_or(0);
            self.manual_provider = ALL_PROVIDERS[(idx + 1) % ALL_PROVIDERS.len()];
        }
    }

    /// Add the current input as a confirmed key, replacing any existing key
    /// for the same provider.
    fn add_key(&mut self) {
        let key = self.input.trim().to_string();
        if key.is_empty() {
            return;
        }
        let provider = detect_from_key(&key).unwrap_or(self.manual_provider);
        // Replace if already exists for this provider.
        if let Some(pos) = self.configured.iter().position(|(p, _)| *p == provider) {
            self.configured[pos] = (provider, key);
        } else {
            self.configured.push((provider, key));
        }
        self.input.clear();
        self.cursor = 0;
        self.error = None;
    }

    /// Add key (if any in input) then submit if at least one key exists.
    fn try_submit(&mut self) {
        if !self.input.trim().is_empty() {
            self.add_key();
        }
        if self.configured.is_empty() {
            self.error = Some("Paste at least one API key to continue.".to_string());
        } else {
            self.submitted = true;
        }
    }

    fn result(&self) -> SetupResult {
        let mut r = SetupResult::default();
        for (p, k) in &self.configured {
            let key = Some(k.trim().to_string()).filter(|s| !s.is_empty());
            match p {
                ProviderKind::Anthropic  => r.anthropic = key,
                ProviderKind::OpenAI     => r.openai = key,
                ProviderKind::XAI        => r.xai = key,
                ProviderKind::OpenRouter => r.openrouter = key,
            }
        }
        r
    }

    // ── Rendering ──────────────────────────────────────────────────────────

    fn render(&self, f: &mut Frame) {
        let area = f.area();
        f.render_widget(Clear, area);

        let bg = Block::default().style(Style::default().bg(Color::Black));
        f.render_widget(bg, area);

        let outer = Layout::vertical([
            Constraint::Length(1),  // header
            Constraint::Min(0),     // body
            Constraint::Length(1),  // status bar
        ])
        .split(area);

        // ── Header ──────────────────────────────────────────────────────────
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                " Hick Agent — First Run Setup ",
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
            )))
            .alignment(Alignment::Center),
            outer[0],
        );

        // Body rows
        let configured_rows = (self.configured.len().max(1)) as u16;
        let inner = Layout::vertical([
            Constraint::Length(6),                 // security callout
            Constraint::Length(1),                 // spacer
            Constraint::Length(configured_rows + 1), // configured list header + rows
            Constraint::Length(1),                 // spacer
            Constraint::Length(3),                 // input field
            Constraint::Length(1),                 // detection / provider hint
            Constraint::Length(1),                 // spacer
            Constraint::Length(2),                 // keyboard hints
            Constraint::Min(0),                    // padding
        ])
        .margin(1)
        .split(outer[1]);

        // ── Security callout ─────────────────────────────────────────────────
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "  Your keys never leave your machine.  ",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        "Secrets scanner",
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" — Hick detects and redacts credentials before they"),
                ]),
                Line::from("  ever reach the LLM's context window. Audit rules in"),
                Line::from("  .hick/agent-security.toml at any time."),
            ])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title(Span::styled(
                        " Security ",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )),
            )
            .wrap(Wrap { trim: false }),
            inner[0],
        );

        // ── Configured keys list ─────────────────────────────────────────────
        let mut list_lines = vec![Line::from(Span::styled(
            "Configured providers:",
            Style::default().fg(Color::Gray),
        ))];
        if self.configured.is_empty() {
            list_lines.push(Line::from(Span::styled(
                "  (none yet)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            for (provider, key) in &self.configured {
                let visible = key.len().saturating_sub(4);
                let masked = format!("{}{}",
                    "•".repeat(visible.min(8)),
                    &key[visible..],
                );
                list_lines.push(Line::from(vec![
                    Span::styled("  ✓ ", Style::default().fg(Color::Green)),
                    Span::styled(
                        format!("{:<20}", provider.display()),
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(masked, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }
        f.render_widget(Paragraph::new(list_lines), inner[2]);

        // ── Input field ──────────────────────────────────────────────────────
        let detected = self.effective_provider();
        let field_title = match detected {
            Some(p) => format!(" Paste API key — {} detected ", p.display()),
            None if !self.input.trim().is_empty() => {
                " Paste API key — Tab to select provider ".to_string()
            }
            None => " Paste API key ".to_string(),
        };
        let (border_style, title_style) = match detected {
            Some(_) => (
                Style::default().fg(Color::Cyan),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ),
            None if !self.input.trim().is_empty() => (
                Style::default().fg(Color::Yellow),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            None => (
                Style::default().fg(Color::Gray),
                Style::default().fg(Color::Gray),
            ),
        };
        let display_text = if self.input.is_empty() {
            Span::styled("paste key here…", Style::default().fg(Color::DarkGray))
        } else {
            let visible = self.input.len().saturating_sub(4);
            let masked = format!("{}{}", "•".repeat(visible.min(16)), &self.input[visible..]);
            Span::styled(masked, Style::default().fg(Color::White))
        };
        f.render_widget(
            Paragraph::new(Line::from(vec![Span::raw(" "), display_text])).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .title(Span::styled(field_title, title_style)),
            ),
            inner[4],
        );

        // ── Detection / provider selector hint ───────────────────────────────
        let det_line = if !self.input.trim().is_empty() && detect_from_key(&self.input).is_none() {
            // Unknown prefix — show manual selector
            let mut spans = vec![Span::styled(
                "  Provider: ",
                Style::default().fg(Color::DarkGray),
            )];
            for p in ALL_PROVIDERS {
                if p == self.manual_provider {
                    spans.push(Span::styled(
                        format!(" {} ", p.display()),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::styled(
                        format!(" {} ", p.display()),
                        Style::default().fg(Color::DarkGray),
                    ));
                }
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled("Tab to switch", Style::default().fg(Color::DarkGray)));
            Line::from(spans)
        } else if let Some(p) = detected {
            Line::from(Span::styled(
                format!("  Detected: {}", p.display()),
                Style::default().fg(Color::Green),
            ))
        } else {
            Line::from("")
        };
        f.render_widget(Paragraph::new(det_line), inner[5]);

        // ── Keyboard hints ───────────────────────────────────────────────────
        let hint_style = Style::default().fg(Color::DarkGray);
        let error_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
        let (line1, line2) = if let Some(ref err) = self.error {
            (
                Line::from(Span::styled(err.as_str(), error_style)),
                Line::from(Span::styled(
                    "Enter = add key / confirm  •  Tab = select provider (unknown keys)  •  Ctrl-C = quit",
                    hint_style,
                )),
            )
        } else {
            (
                Line::from(Span::styled(
                    "Enter = add key or confirm  •  Tab = select provider (unknown keys)",
                    hint_style,
                )),
                Line::from(Span::styled(
                    "Keys saved to ~/.config/hick/keys.toml and loaded automatically next launch.",
                    hint_style,
                )),
            )
        };
        f.render_widget(
            Paragraph::new(vec![line1, line2]).alignment(Alignment::Center),
            inner[7],
        );

        // ── Status bar ────────────────────────────────────────────────────────
        let status_text = if self.configured.is_empty() {
            " Paste at least one API key — prefix is auto-detected "
        } else {
            " Enter on empty field to save & launch the agent "
        };
        f.render_widget(
            Paragraph::new(Span::styled(status_text, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center),
            outer[2],
        );
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Run the blocking ratatui setup screen.
///
/// Returns the collected [`SetupResult`] once the user submits or `done_flag`
/// is set by the concurrent browser path.  Also writes `result_out` so the
/// async side can read it after `done_flag` fires.
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
                        break SetupResult::default();
                    }
                    KeyCode::Tab => app.cycle_manual_provider(),
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
        if done_flag.load(Ordering::Acquire) {
            break app.result();
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    *result_out.lock().unwrap() = Some(result.clone());
    done_flag.store(true, Ordering::Release);

    Ok(result)
}
