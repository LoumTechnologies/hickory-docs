//! Deciding what an anchored terminal may write down.
//!
//! `docs/specs/freeform/a-terminal-that-writes-the-document.md`: a terminal
//! bound to a container in a document writes the lines you type into that
//! cell. This module is the decision and nothing else — no document, no
//! filesystem, no PTY — because the decision is where this feature is
//! subtly wrong or subtly right, and it should be possible to test it by
//! reading it.
//!
//! The invariant the whole thing rests on:
//!
//! > **An anchored cell is always a prefix of the session that reproduces.**
//!
//! Which is why every hazard here resolves to the same verb: **suspend**.
//! Not skip. A cell with line 3 of 10 removed claims a run that no longer
//! reproduces, because whatever line 3 did is missing and lines 4 onward
//! depended on it. A cell that stops at line 2 is simply true. Skipping a
//! line is worse than stopping, and that asymmetry is the entire argument.
//!
//! Nothing here can un-write anything, and that is deliberate rather than a
//! limitation. The document is a live CRDT that autosaves and syncs, so a
//! byte that reached it may already be on another machine; there is no
//! taking it back. **The scan gates the write or it does nothing worth
//! having.**

/// Why recording stopped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Suspension {
    /// The line looks like it carries a credential.
    ///
    /// Never phrased as a guarantee. A scanner catches known prefixes and
    /// high-entropy strings; it cannot catch a short password, and saying
    /// "your secrets are safe here" would be a promise nothing can keep.
    LooksLikeSecret,
    /// The shell kept this line out of its own history, so what arrived was
    /// the PREVIOUS line — see `command.rs`. Recording it would write down a
    /// command that did not run.
    NotInHistory,
    /// The keystrokes belong to a program that is not the shell, so they are
    /// not commands and recording them would be a lie.
    ///
    /// Mostly unreachable by construction: a REPL's input never reaches the
    /// shell, so the shell reports nothing and there is nothing to refuse.
    /// It exists for the case a caller detects with `tcgetpgrp` and wants
    /// said out loud, and `full_screen` (from the alternate-screen sequence)
    /// only decides how the sentence reads.
    ForeignProgram { program: String, full_screen: bool },
}

impl Suspension {
    /// What a person is told, at the moment it happens — never in a diff
    /// afterwards. Someone who does not know recording stopped will assume
    /// the cell holds what they did, and that assumption is the failure this
    /// whole design exists to prevent.
    pub fn message(&self) -> String {
        match self {
            Suspension::LooksLikeSecret => "recording paused — this line looks like a secret\n   \
                 the shell ran it; the document did not record it"
                .to_string(),
            Suspension::NotInHistory => "recording paused — your shell kept this line out of its \
                 history\n   the shell ran it; the document did not record it"
                .to_string(),
            Suspension::ForeignProgram {
                program,
                full_screen,
            } => {
                let what = if *full_screen {
                    "is not a shell command"
                } else {
                    "is reading these keys itself"
                };
                format!(
                    "recording paused — {program} {what}\n   the terminal is yours; the document \
                     resumes when it exits"
                )
            }
        }
    }
}

/// What to do with a line the shell reported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Append this line to the anchored cell.
    Record(String),
    /// Stop recording. The shell still ran it.
    Suspend(Suspension),
    /// Already suspended, or nothing to say. Write nothing, report nothing.
    Ignore,
}

/// One anchored terminal's recording state.
///
/// Deliberately tiny and deliberately sticky: once suspended it stays
/// suspended until a person resumes, because after a suspension the shell's
/// state contains something the document does not describe. A cell that
/// carried on would be claiming that state came from the lines above it.
#[derive(Debug, Default)]
pub struct Recording {
    last_number: Option<u64>,
    suspended: Option<Suspension>,
}

impl Recording {
    pub fn new() -> Self {
        Self::default()
    }

    /// Why recording is stopped, if it is.
    pub fn suspended(&self) -> Option<&Suspension> {
        self.suspended.as_ref()
    }

    /// Decide what to do with a reported line.
    pub fn observe(&mut self, number: u64, text: &str) -> Decision {
        if self.suspended.is_some() {
            // Already stopped. Saying so again on every keystroke would turn
            // the one message that matters into noise.
            self.last_number = Some(number.max(self.last_number.unwrap_or(0)));
            return Decision::Ignore;
        }
        // A counter that did not move means the shell declined to record this
        // line, and what arrived is therefore the previous one. This is
        // checked BEFORE the text is looked at, because the text is stale.
        if let Some(last) = self.last_number
            && number <= last
        {
            return self.suspend(Suspension::NotInHistory);
        }
        self.last_number = Some(number);
        if text.trim().is_empty() {
            return Decision::Ignore;
        }
        if looks_like_a_secret(text) {
            return self.suspend(Suspension::LooksLikeSecret);
        }
        Decision::Record(text.to_string())
    }

    /// Stop because the keystrokes stopped belonging to the shell.
    pub fn foreign_program(&mut self, program: &str, full_screen: bool) -> Decision {
        if self.suspended.is_some() {
            return Decision::Ignore;
        }
        self.suspend(Suspension::ForeignProgram {
            program: program.to_string(),
            full_screen,
        })
    }

    fn suspend(&mut self, why: Suspension) -> Decision {
        self.suspended = Some(why.clone());
        Decision::Suspend(why)
    }

    /// Start recording again, which always means a new cell.
    ///
    /// Returned rather than assumed: after a suspension the shell holds state
    /// the document does not describe, so continuing the same cell would
    /// claim that state came from the lines above it.
    #[must_use = "resuming starts a NEW cell; continuing the old one claims a run it cannot reproduce"]
    pub fn resume(&mut self) -> bool {
        self.suspended.take().is_some()
    }
}

/// Token prefixes that are worth stopping for.
///
/// Every one is a published, documented prefix — not a guess about what a
/// credential looks like.
const PREFIXES: &[&str] = &[
    "sk-",         // OpenAI, Anthropic
    "sk_live_",    // Stripe
    "sk_test_",    // Stripe
    "rk_live_",    // Stripe restricted
    "ghp_",        // GitHub personal access token
    "gho_",        // GitHub OAuth
    "ghu_",        // GitHub user-to-server
    "ghs_",        // GitHub server-to-server
    "ghr_",        // GitHub refresh
    "github_pat_", // GitHub fine-grained
    "glpat-",      // GitLab
    "AKIA",        // AWS access key
    "ASIA",        // AWS temporary
    "xoxb-",       // Slack bot
    "xoxp-",       // Slack user
    "xapp-",       // Slack app
    "AIza",        // Google API
    "npm_",        // npm
    "dop_v1_",     // DigitalOcean
    "-----BEGIN",  // a PEM block pasted onto a line
];

/// Names that make whatever follows them worth stopping for.
const NAMES: &[&str] = &[
    "key",
    "token",
    "secret",
    "password",
    "passwd",
    "credential",
    "auth",
];

/// Whether a line looks like it carries a credential.
///
/// **A heuristic, and only ever described as one.** "A line that looks like a
/// secret stops the recording" is true. "Your secrets are safe here" is not,
/// and this product does not say things it cannot prove — a short password is
/// not catchable by anything here.
///
/// The asymmetry decides the tuning: a false positive costs a suspension a
/// person can see and resume from, and a false negative costs a key in a git
/// repository. So a base64 blob tripping it is an acceptable price.
pub fn looks_like_a_secret(line: &str) -> bool {
    if PREFIXES.iter().any(|prefix| line.contains(prefix)) {
        return true;
    }
    for token in line.split(|c: char| c.is_whitespace()) {
        // `FOO_API_KEY=value` and `--token=value` alike: a named thing with
        // something assigned to it.
        if let Some((name, value)) = token.split_once('=') {
            let lowered = name.to_ascii_lowercase();
            if NAMES.iter().any(|n| lowered.contains(n)) && !value.trim().is_empty() {
                return true;
            }
        }
        if high_entropy(token) {
            return true;
        }
    }
    false
}

/// A long token drawn from a credential-ish alphabet with mixed character
/// classes.
///
/// The length floor is what keeps this from firing on ordinary words and
/// paths; the mixed-class requirement is what keeps it from firing on a
/// long lowercase identifier or a sha1 that is only hex.
fn high_entropy(token: &str) -> bool {
    let token = token.trim_matches(|c| c == '"' || c == '\'');
    if token.len() < 32 {
        return false;
    }
    if !token.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || c == '_' || c == '-'
    }) {
        return false;
    }
    let has_upper = token.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = token.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    has_upper && has_lower && has_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_commands_are_recorded_in_order() {
        let mut recording = Recording::new();
        assert_eq!(
            recording.observe(1, "dotnet new console -o app"),
            Decision::Record("dotnet new console -o app".into())
        );
        assert_eq!(
            recording.observe(2, "ls app"),
            Decision::Record("ls app".into())
        );
        assert!(recording.suspended().is_none());
    }

    #[test]
    fn a_secret_stops_the_recording_and_does_not_stop_the_shell() {
        let mut recording = Recording::new();
        recording.observe(1, "cd app");
        let decision = recording.observe(2, "export ANTHROPIC_API_KEY=sk-ant-api03-abcdef");
        assert_eq!(decision, Decision::Suspend(Suspension::LooksLikeSecret));
        // The message says all three things, and none of them is a promise.
        let message = Suspension::LooksLikeSecret.message();
        assert!(message.contains("looks like a secret"), "{message}");
        assert!(message.contains("the shell ran it"), "{message}");
        assert!(!message.to_lowercase().contains("safe"), "{message}");
    }

    #[test]
    fn suspension_is_sticky_because_a_prefix_is_true_and_a_hole_is_not() {
        // The whole invariant. After a suspension the shell holds state the
        // document does not describe, so the lines that follow cannot honestly
        // be appended to the same cell — they would claim that state came
        // from the lines above them.
        let mut recording = Recording::new();
        recording.observe(1, "cd app");
        recording.observe(2, "export TOKEN=ghp_aaaaaaaaaaaaaaaaaaaa");
        assert_eq!(recording.observe(3, "make"), Decision::Ignore);
        assert_eq!(recording.observe(4, "make install"), Decision::Ignore);
        assert!(recording.suspended().is_some());
    }

    #[test]
    fn a_number_that_did_not_move_suspends_rather_than_repeating_the_last_line() {
        // What `HISTCONTROL=ignorespace` does to bash: the line is kept out
        // of history and `history 1` reports the PREVIOUS one. Writing that
        // down would claim a command ran twice when the second one was
        // something else entirely.
        let mut recording = Recording::new();
        assert_eq!(
            recording.observe(10, "echo recorded"),
            Decision::Record("echo recorded".into())
        );
        assert_eq!(
            recording.observe(10, "echo recorded"),
            Decision::Suspend(Suspension::NotInHistory)
        );
    }

    #[test]
    fn resuming_says_it_is_a_new_cell() {
        let mut recording = Recording::new();
        recording.observe(1, "export KEY=sk-abcdef");
        assert!(recording.resume(), "there was a suspension to lift");
        assert_eq!(
            recording.observe(2, "make"),
            Decision::Record("make".into())
        );
        // Resuming when nothing was suspended is not a new cell.
        assert!(!recording.resume());
    }

    #[test]
    fn a_program_that_is_not_the_shell_reads_differently_when_it_is_full_screen() {
        let mut recording = Recording::new();
        let full = recording.foreign_program("less", true);
        match full {
            Decision::Suspend(s) => {
                assert!(
                    s.message().contains("is not a shell command"),
                    "{}",
                    s.message()
                )
            }
            other => panic!("{other:?}"),
        }
        let mut recording = Recording::new();
        let repl = recording.foreign_program("python3", false);
        match repl {
            Decision::Suspend(s) => assert!(
                s.message().contains("reading these keys itself"),
                "{}",
                s.message()
            ),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn the_scanner_catches_published_prefixes_and_named_assignments() {
        assert!(looks_like_a_secret("export OPENAI_API_KEY=sk-proj-xyz"));
        assert!(looks_like_a_secret(
            "aws configure set x AKIAIOSFODNN7EXAMPLE"
        ));
        assert!(looks_like_a_secret(
            "curl -H 'Authorization: ghp_0123456789abcdefghij'"
        ));
        assert!(looks_like_a_secret(
            "echo -----BEGIN OPENSSH PRIVATE KEY-----"
        ));
        assert!(looks_like_a_secret("DATABASE_PASSWORD=hunter2 ./app"));
        assert!(looks_like_a_secret("./deploy --token=abc123"));
    }

    #[test]
    fn the_scanner_leaves_ordinary_shell_alone() {
        // False positives cost a suspension somebody has to notice and clear,
        // so the common vocabulary of a build has to survive.
        for line in [
            "dotnet build --configuration Debug",
            "git commit -m 'the table is scaffolding'",
            "ls -la /usr/local/share/some-quite-long-directory-name",
            "cargo test -p hick-term -- --nocapture",
            "grep -rn 'password' src/",
            "curl -sSL https://example.com/install.sh | sh",
        ] {
            assert!(!looks_like_a_secret(line), "false positive: {line}");
        }
    }

    #[test]
    fn a_long_mixed_case_blob_trips_it_and_that_is_the_price() {
        // The spec's own words: a base64 blob tripping this is acceptable,
        // because the other error writes a key into a git repository.
        assert!(looks_like_a_secret(
            "echo dGhpcyBpcyBhIHZlcnkgbG9uZyBiYXNlNjQgYmxvYjEyMw"
        ));
        // A hex digest is not mixed-class, and hashes are everywhere.
        assert!(!looks_like_a_secret(
            "sha256sum: 4f2c1a9e8b3d6f0a5c7e9b1d3f5a7c9e1b3d5f7a9c1e3b5d7f9a1c3e5b7d9f1a"
        ));
    }

    #[test]
    fn an_empty_line_is_neither_recorded_nor_a_suspension() {
        let mut recording = Recording::new();
        assert_eq!(recording.observe(1, "   "), Decision::Ignore);
        assert!(recording.suspended().is_none());
    }
}
