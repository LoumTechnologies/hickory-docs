//! Terminal settings, read from the user's machine and validated at startup.
//!
//! Typed, parsed once, and never read again at the point of use — a bad
//! `HICKORY_TERM_SCROLLBACK` has to fail with a message naming the variable
//! while someone is still looking at the terminal they typed it in, not at
//! the moment a session happens to overflow its buffer on a laptop nobody can
//! debug for them. See `.instructions/config-and-environments.md`.

use anyhow::{Result, bail};

/// How this machine wants its terminals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermConfig {
    /// The program a session starts when it is given no command of its own.
    pub shell: String,
    /// Lines of scrollback kept per session, server-side, so a closed pane
    /// can be reopened onto what it was showing.
    pub scrollback_lines: usize,
}

/// The shell to fall back to when the environment names none. Not a guess
/// about what the user likes — a guess about what exists.
#[cfg(windows)]
const FALLBACK_SHELL: &str = "cmd.exe";
#[cfg(not(windows))]
const FALLBACK_SHELL: &str = "/bin/sh";

const DEFAULT_SCROLLBACK_LINES: usize = 10_000;
/// Above this a single idle session can hold tens of megabytes of text the
/// user will never scroll to. Refused loudly rather than silently clamped:
/// silently ignoring a number someone typed is how they conclude the setting
/// does nothing.
const MAX_SCROLLBACK_LINES: usize = 1_000_000;

impl Default for TermConfig {
    fn default() -> Self {
        TermConfig {
            shell: FALLBACK_SHELL.to_string(),
            scrollback_lines: DEFAULT_SCROLLBACK_LINES,
        }
    }
}

impl TermConfig {
    /// Read `HICKORY_SHELL` and `HICKORY_TERM_SCROLLBACK`.
    ///
    /// Both are optional and the defaults work on a machine with nothing set,
    /// which is the machine most downloads land on: `HICKORY_SHELL` falls back
    /// to `$SHELL`, then to the platform's.
    pub fn from_env() -> Result<Self> {
        let shell = match std::env::var("HICKORY_SHELL") {
            Ok(s) if !s.trim().is_empty() => s,
            _ => match std::env::var("SHELL") {
                Ok(s) if !s.trim().is_empty() => s,
                _ => FALLBACK_SHELL.to_string(),
            },
        };
        let scrollback_lines = match std::env::var("HICKORY_TERM_SCROLLBACK") {
            Err(_) => DEFAULT_SCROLLBACK_LINES,
            Ok(raw) if raw.trim().is_empty() => DEFAULT_SCROLLBACK_LINES,
            Ok(raw) => parse_scrollback(raw.trim())?,
        };
        Ok(TermConfig {
            shell,
            scrollback_lines,
        })
    }
}

fn parse_scrollback(raw: &str) -> Result<usize> {
    let Ok(lines) = raw.parse::<usize>() else {
        bail!(
            "HICKORY_TERM_SCROLLBACK must be a whole number of lines, but was '{raw}' \
             (for example: HICKORY_TERM_SCROLLBACK=10000). Unset it to keep the default \
             of {DEFAULT_SCROLLBACK_LINES}."
        );
    };
    if lines == 0 {
        bail!(
            "HICKORY_TERM_SCROLLBACK=0 would leave a reopened terminal blank. Use a \
             positive number of lines (the default is {DEFAULT_SCROLLBACK_LINES}), or \
             unset the variable."
        );
    }
    if lines > MAX_SCROLLBACK_LINES {
        bail!(
            "HICKORY_TERM_SCROLLBACK={lines} is more scrollback than a session can hold \
             without eating memory; the most this build accepts is {MAX_SCROLLBACK_LINES}."
        );
    }
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_machine_with_nothing_set_still_gets_a_working_terminal() {
        let config = TermConfig::default();
        assert!(!config.shell.is_empty());
        assert_eq!(config.scrollback_lines, DEFAULT_SCROLLBACK_LINES);
    }

    #[test]
    fn scrollback_must_be_a_number_and_says_so_by_name() {
        let err = parse_scrollback("lots").unwrap_err().to_string();
        assert!(err.contains("HICKORY_TERM_SCROLLBACK"), "{err}");
        assert!(err.contains("lots"), "{err}");
    }

    #[test]
    fn zero_and_absurd_scrollback_are_refused_rather_than_clamped() {
        assert!(parse_scrollback("0").is_err());
        assert!(parse_scrollback(&(MAX_SCROLLBACK_LINES + 1).to_string()).is_err());
        assert_eq!(parse_scrollback("500").unwrap(), 500);
    }
}
