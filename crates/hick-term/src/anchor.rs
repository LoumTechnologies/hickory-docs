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
    /// The line was typed with a leading space, which every shell with a
    /// history treats as "do not record this".
    ///
    /// Honoured here rather than left to the shell, because the shells do not
    /// agree and hick must. Measured 2026-08-27: bash with
    /// `HISTCONTROL=ignorespace` never reports the line at all (so the
    /// *next* one arrives stale and [`Suspension::NotInHistory`] catches it),
    /// while zsh's `preexec` reports it in full — so without this rule the
    /// same keystrokes would be kept out of a bash document and written into
    /// a zsh one.
    HiddenByLeadingSpace,
}

/// The keystrokes are going to a program that is not the shell.
///
/// **Not a [`Suspension`], and modelling it as one was the first draft's
/// mistake.** The other two stop recording until a person resumes, because
/// after them the shell holds state the document does not describe. This one
/// is temporary and heals itself — the spec's own words are "the terminal is
/// yours; the document resumes when it exits". There is nothing to resume,
/// because nothing was suspended: a REPL's keystrokes never reach the shell,
/// so the shell reports nothing and there is nothing to refuse.
///
/// It exists only to be **said**. Without it, a person typing into `python3`
/// inside an anchored terminal watches their lines not appear and has to
/// guess why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignInput {
    /// What is holding the terminal, when the platform will say.
    pub program: Option<String>,
    /// Whether it took over the screen (`ESC [ ? 1049 h`). This decides only
    /// how the sentence READS — never whether to record.
    pub full_screen: bool,
}

impl ForeignInput {
    pub fn message(&self) -> String {
        let who = self
            .program
            .clone()
            .unwrap_or_else(|| "a program".to_string());
        let what = if self.full_screen {
            "is not a shell command"
        } else {
            "is reading these keys itself"
        };
        format!(
            "not recording — {who} {what}\n   the terminal is yours; the document resumes when \
             it exits"
        )
    }
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
            Suspension::HiddenByLeadingSpace => "recording paused — this line starts with a \
                 space, which means do not record\n   the shell ran it; the document did not \
                 record it"
                .to_string(),
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
        // The leading-space convention, applied by hick rather than left to
        // whichever shell this is. The shells disagree about whether such a
        // line is even reported, and a person's "do not record this" must not
        // depend on that.
        if text.starts_with(|c: char| c.is_whitespace()) {
            return self.suspend(Suspension::HiddenByLeadingSpace);
        }
        if looks_like_a_secret(text) {
            return self.suspend(Suspension::LooksLikeSecret);
        }
        Decision::Record(text.to_string())
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
    fn a_foreign_program_is_said_rather_than_suspended() {
        // The distinction the first draft got wrong. A secret or a hidden
        // line leaves the shell holding state the document cannot describe,
        // so recording stops until a person resumes. A program reading the
        // keys does not: when it exits the shell reports again and the cell
        // carries on, so this is a sentence rather than a state.
        let full = ForeignInput {
            program: Some("less".into()),
            full_screen: true,
        };
        assert!(
            full.message().starts_with("not recording"),
            "{}",
            full.message()
        );
        assert!(
            full.message().contains("is not a shell command"),
            "{}",
            full.message()
        );

        let repl = ForeignInput {
            program: Some("python3".into()),
            full_screen: false,
        };
        assert!(
            repl.message().contains("reading these keys itself"),
            "{}",
            repl.message()
        );
        // A platform that will not name the program still says the useful
        // half rather than nothing.
        let unknown = ForeignInput {
            program: None,
            full_screen: false,
        };
        assert!(
            unknown.message().contains("a program"),
            "{}",
            unknown.message()
        );
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
    fn a_leading_space_means_do_not_record_in_every_shell() {
        // Measured 2026-08-27, and the reason this rule is here rather than
        // left to the shell: bash with `HISTCONTROL=ignorespace` never
        // reports such a line, while zsh's `preexec` reports it in full. Same
        // keystrokes, opposite outcomes, unless hick decides.
        let mut recording = Recording::new();
        recording.observe(1, "echo before");
        assert_eq!(
            recording.observe(2, " echo hidden"),
            Decision::Suspend(Suspension::HiddenByLeadingSpace)
        );
        // Sticky, like every other suspension: a hole is worse than a stop.
        assert_eq!(recording.observe(3, "echo after"), Decision::Ignore);
    }

    #[test]
    fn a_repeated_number_after_a_hidden_line_is_already_suspended() {
        // zsh's HISTCMD is the slot a line WOULD take, and a hidden line
        // takes it and gives it back — so the next genuine command reuses the
        // number. That looked like a stale report and would have suspended a
        // perfectly good line, if the leading-space rule had not already
        // stopped recording one line earlier.
        let mut recording = Recording::new();
        recording.observe(6, "setopt HIST_IGNORE_SPACE");
        assert_eq!(
            recording.observe(7, " echo hidden"),
            Decision::Suspend(Suspension::HiddenByLeadingSpace)
        );
        assert_eq!(recording.observe(7, "echo after"), Decision::Ignore);
    }

    #[test]
    fn the_scanner_reads_a_line_of_output_the_same_way_as_a_line_of_input() {
        // `hick run` uses this on every cell's recorded output, because a
        // command that PRINTS a token has always been committed with the
        // document. What it does about it is different — a warning, not a
        // refusal, because declining to record would break the weave on a
        // false positive — but the shapes it looks for are the same ones.
        assert!(looks_like_a_secret(
            "token is ghp_0123456789abcdefghijklmnopqrstuvwx"
        ));
        assert!(looks_like_a_secret("AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI"));
        assert!(!looks_like_a_secret("   Compiling hick-lang v0.1.0"));
        assert!(!looks_like_a_secret("test result: ok. 12 passed; 0 failed"));
    }

    #[test]
    fn an_empty_line_is_neither_recorded_nor_a_suspension() {
        let mut recording = Recording::new();
        assert_eq!(recording.observe(1, "   "), Decision::Ignore);
        assert!(recording.suspended().is_none());
    }
}
