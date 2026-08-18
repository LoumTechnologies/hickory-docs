//! Guessing that a stranger's program is waiting for an answer.
//!
//! `hickory-agent` never needs this: its protocol declares a question, with
//! the choices it will accept, and the attention card renders them. This
//! module is for the other case — someone else's coding agent running in a
//! plain shell, where the only evidence is what is on the screen.
//!
//! It is therefore **best-effort, and stays quarantined here**. Two rules
//! keep a guess from doing damage:
//!
//! - a guessed prompt may raise a session to *needs you*, which only ever
//!   costs you a glance;
//! - a guessed prompt may never be auto-answered by turbo, which would cost
//!   you an answer you did not give. See
//!   `docs/guarantees/terminal/turbo-never-answers-a-prompt-it-did-not-parse.md`.
//!
//! There are two shapes, and they need different windows onto the screen.
//!
//! **A line-oriented question** — `Delete build/? [y/N]` — is only live while
//! it is the last thing printed. Once it is answered the answer and its
//! output follow, so looking at the last non-empty line alone is what stops a
//! long-answered `[y/N]` from pinning a session at *needs you* forever.
//!
//! **A drawn menu** — the `❯ 1. Yes` list that Claude Code, Codex and their
//! siblings put up — is not the last line: below it sit a blank row and a
//! footer like `Enter to confirm · Esc to cancel`. It is also redrawn rather
//! than scrolled, so it leaves the screen the moment it is answered. That one
//! is looked for across the last few lines.

/// Substrings that, at the end of a session's output, mean a program has
/// stopped to ask. Lowercased before matching.
const ASKING: &[&str] = &[
    "[y/n]",
    "(y/n)",
    "[yes/no]",
    "(yes/no)",
    "? (y)",
    "press enter to continue",
    "do you want to proceed",
    "do you want to continue",
    "overwrite?",
];

/// Footers a drawn menu puts under its choices.
const FOOTERS: &[&str] = &[
    "enter to confirm",
    "to confirm · esc",
    "press enter to select",
    "esc to cancel",
];

/// How far up to look for a drawn menu. Enough for the choices plus a footer
/// and a blank line, not so far that ordinary output resembles one.
const MENU_WINDOW: usize = 8;

/// The question the visible screen is asking, if it is asking one.
///
/// Give it the screen, not the scrollback: what matters is what a person
/// would see if they looked now. The text handed back is what the attention
/// card shows — for a line-oriented question that is the line; for a drawn
/// menu it is the choices and their footer, because the choices ARE the
/// question.
pub fn question_in(screen: &str) -> Option<String> {
    let live: Vec<&str> = screen
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect();
    let last = live.last()?;
    if ASKING
        .iter()
        .any(|needle| last.to_lowercase().contains(needle))
    {
        return Some(last.trim().to_string());
    }

    let from = live.len().saturating_sub(MENU_WINDOW);
    let tail = &live[from..];
    let first_choice = tail
        .iter()
        .position(|line| is_choice(&line.to_lowercase()))?;
    let has_footer = tail.iter().any(|line| {
        let line = line.to_lowercase();
        FOOTERS.iter().any(|f| line.contains(f))
    });
    if !has_footer {
        return None;
    }
    Some(
        tail[first_choice..]
            .iter()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// Whether the visible screen looks like an unanswered question.
pub fn looks_like_a_question(screen: &str) -> bool {
    question_in(screen).is_some()
}

/// A selectable option in a drawn menu: a marker or a number, then a label.
/// `❯ 1. Yes, I trust this folder` and `  2. No, exit` both count.
fn is_choice(line: &str) -> bool {
    let line = line.trim_start();
    let rest = line
        .strip_prefix('❯')
        .or_else(|| line.strip_prefix('>'))
        .unwrap_or(line)
        .trim_start();
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_digit() {
        return false;
    }
    // "1." or "1)" — and something after it, so a bare number in a table is
    // not a choice.
    matches!(chars.next(), Some('.') | Some(')')) && chars.as_str().trim().len() > 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What Claude Code actually draws on its first run in a new folder,
    /// captured from a real PTY session (tests/full_screen_apps.rs is the
    /// harness that produced it). The exact case this feature exists for.
    const CODING_AGENT_TRUST_PROMPT: &str = "\
────────────────────────────────────────────────────
 Accessing workspace:

 /tmp/.tmprwHQ9S

 Quick safety check: Is this a project you created or one you trust?

 Security guide

 ❯ 1. Yes, I trust this folder
   2. No, exit

 Enter to confirm · Esc to cancel";

    #[test]
    fn a_coding_agents_drawn_menu_reads_as_asking() {
        assert!(looks_like_a_question(CODING_AGENT_TRUST_PROMPT));
    }

    #[test]
    fn the_question_it_reports_is_the_choices_not_the_footer() {
        // The card shows this. "Enter to confirm · Esc to cancel" on its own
        // would tell a person nothing about what they are confirming.
        let question = question_in(CODING_AGENT_TRUST_PROMPT).unwrap();
        assert!(
            question.starts_with("❯ 1. Yes, I trust this folder"),
            "{question}"
        );
        assert!(question.contains("2. No, exit"), "{question}");
    }

    #[test]
    fn a_line_question_reports_itself() {
        assert_eq!(
            question_in("Delete build/? [y/N] ").unwrap(),
            "Delete build/? [y/N]"
        );
    }

    #[test]
    fn a_yes_no_question_on_the_last_line_reads_as_asking() {
        assert!(looks_like_a_question("Delete build/? [y/N] "));
        assert!(looks_like_a_question("Overwrite?"));
        assert!(looks_like_a_question("Press ENTER to continue"));
    }

    #[test]
    fn an_answered_question_further_up_does_not_hold_the_session_hostage() {
        assert!(!looks_like_a_question(
            "Delete build/? [y/N] y\nDeleted.\n$ "
        ));
    }

    #[test]
    fn choices_without_a_footer_are_just_a_list() {
        // A numbered list in ordinary output — release notes, a table of
        // contents — must not put a session in the queue.
        assert!(!looks_like_a_question(
            "Changes:\n 1. faster weaves\n 2. fewer bugs\n"
        ));
    }

    #[test]
    fn a_footer_without_choices_is_not_a_menu_either() {
        assert!(!looks_like_a_question("some prose\nesc to cancel"));
    }

    #[test]
    fn ordinary_output_is_not_a_question() {
        assert!(!looks_like_a_question("Compiling hick-term v0.1.0\n"));
        assert!(!looks_like_a_question(""));
    }

    #[test]
    fn a_bare_number_is_not_a_choice() {
        assert!(!is_choice("  42"));
        assert!(!is_choice("1."));
        assert!(is_choice("❯ 1. Yes, I trust this folder"));
        assert!(is_choice("  2) No, exit"));
    }
}
