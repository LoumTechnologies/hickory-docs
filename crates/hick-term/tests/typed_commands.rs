//! What the shell reports it is running, through a real PTY.
//!
//! Protects docs/specs/freeform/a-terminal-that-writes-the-document.md
//!
//! The mechanism is the anchored terminal's foundation: everything the
//! document eventually holds comes through here, so a line that arrives
//! wrong arrives wrong in somebody's git repository. Two obvious
//! implementations were measured and rejected before this one (a `DEBUG`
//! trap, and `PROMPT_COMMAND` with `history 1`) — the module docs say why.
//!
//! Both hooked shells are covered, and they are **not** the same mechanism:
//! bash uses `PS0` and reports whatever `history 1` holds, while zsh's
//! `preexec` receives the typed line directly. Every case below runs against
//! both, because "it works in bash" was exactly the assumption that turned
//! out to hide a difference (see `a_leading_space_is_not_recorded_by_either`).
//! Skipped loudly for a shell that is not installed.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::command::TypedCommand;
use hick_term::config::TermConfig;
use hick_term::session::{Session, SessionSpec};

fn have(shell: &str) -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(shell))
        .find(|candidate| candidate.is_file())
        .map(|path| path.to_string_lossy().into_owned())
}

/// Every shell hick installs a command hook for.
///
/// A test that runs against one of two supported shells is a test that says
/// nothing about the other, and this file exists because that gap had a real
/// difference hiding in it.
const HOOKED: &[&str] = &["bash", "zsh"];

/// Run `body` against each installed hooked shell, saying which are missing.
fn for_each_shell(body: impl Fn(&str, Shell)) {
    let mut ran = 0;
    for name in HOOKED {
        match have(name) {
            Some(path) => {
                let shell = start(&path);
                // Installed is not the same as hookable. macOS ships bash
                // 3.2 as `/bin/bash` (4.0 went GPLv3 and Apple stopped
                // following), and `PS0` — the whole mechanism below — arrived
                // in 4.4. Asking the SESSION rather than re-deriving a
                // version here keeps the test and the product on one answer:
                // whatever `reports_commands()` says is what the anchor
                // endpoint will say, so this can never pass against a shell
                // a user would be refused on.
                if !shell.session.reports_commands() {
                    eprintln!(
                        "SKIPPED {name} ({path}): installed, but hick has no command hook for \
                         it — for bash that means older than 4.4, which has no PS0."
                    );
                    continue;
                }
                eprintln!("--- {name} ---");
                body(name, shell);
                ran += 1;
            }
            None => eprintln!("SKIPPED {name}: not installed on this machine"),
        }
    }
    assert!(
        ran > 0,
        "no hooked shell is installed, so this tested nothing"
    );
}

struct Shell {
    session: Arc<Session>,
    seen: tokio::sync::broadcast::Receiver<TypedCommand>,
    _dir: tempfile::TempDir,
}

fn start(shell: &str) -> Shell {
    let dir = tempfile::tempdir().expect("scratch");
    let config = TermConfig {
        shell: shell.to_string(),
        scrollback_lines: 10_000,
        integrate_shell: true,
    };
    let session = Session::spawn(
        "typed".into(),
        SessionSpec {
            title: "typed".into(),
            cwd: dir.path().to_path_buf(),
            argv: Vec::new(),
            monitor: false,
        },
        &config,
    )
    .expect("the shell starts");
    let seen = session.typed_commands();
    // Let the shell finish reading its startup files before typing at it.
    std::thread::sleep(Duration::from_millis(600));
    Shell {
        session,
        seen,
        _dir: dir,
    }
}

impl Shell {
    fn type_line(&self, line: &str) {
        self.session.write(line.as_bytes()).expect("writes");
        std::thread::sleep(Duration::from_millis(350));
    }

    /// Everything reported so far, in order.
    fn reported(&mut self) -> Vec<TypedCommand> {
        let mut out = Vec::new();
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            match self.seen.try_recv() {
                Ok(command) => out.push(command),
                Err(_) => std::thread::sleep(Duration::from_millis(25)),
            }
        }
        out
    }
}

#[test]
fn a_shell_reports_the_line_that_was_typed_not_the_commands_it_ran() {
    for_each_shell(|shell, mut session| {
        session.type_line("echo one\n");
        // A pipeline is ONE typed line. bash's `DEBUG` trap reports it as
        // two, and a cell holding `tr a b` on its own does not reproduce.
        session.type_line("echo a | tr a b\n");
        // A loop is one line too; a DEBUG trap reports one per iteration.
        session.type_line("for i in 1 2; do echo $i; done\n");

        let lines: Vec<String> = session.reported().into_iter().map(|c| c.text).collect();
        assert_eq!(
            lines,
            vec![
                "echo one".to_string(),
                "echo a | tr a b".to_string(),
                "for i in 1 2; do echo $i; done".to_string(),
            ],
            "{shell} did not report the typed lines"
        );
    });
}

#[test]
fn nothing_is_reported_before_anything_is_typed() {
    // The failure mode of the `PROMPT_COMMAND` + `history 1` design: at the
    // first prompt it reports the last line of the user's own history file —
    // a command they never typed in this session, written into a document
    // before they touched the keyboard.
    for_each_shell(|shell, mut session| {
        assert_eq!(
            session.reported(),
            Vec::new(),
            "{shell} reported a command before anything was typed"
        );
    });
}

#[test]
fn an_empty_line_reports_nothing() {
    for_each_shell(|shell, mut session| {
        session.type_line("\n");
        session.type_line("\n");
        session.type_line("echo twice\n");
        let lines: Vec<String> = session.reported().into_iter().map(|c| c.text).collect();
        assert_eq!(lines, vec!["echo twice".to_string()], "{shell}");
    });
}

#[test]
fn a_leading_space_is_not_recorded_by_either_shell() {
    // **The difference measuring found.** The two shells disagree about this
    // line completely: bash with `HISTCONTROL=ignorespace` never reports it
    // (and the NEXT line then arrives stale, carrying a repeated number),
    // while zsh's `preexec` reports it in full and the next line reuses its
    // history slot. Same keystrokes, opposite raw behaviour.
    //
    // So what is asserted here is the property that must hold either way:
    // the hidden line never arrives as a fresh, recordable report. Which of
    // the two suspensions it becomes is `hick_term::anchor`'s business, and
    // both stop at the same point.
    for_each_shell(|shell, mut session| {
        session.type_line("echo recorded\n");
        session.type_line(" echo hidden\n");
        session.type_line("echo after\n");

        let reported = session.reported();
        let recorded = reported
            .iter()
            .position(|c| c.text == "echo recorded")
            .unwrap_or_else(|| panic!("{shell} never reported the plain command: {reported:?}"));

        for later in &reported[recorded + 1..] {
            // Never a stale line with a fresh number: that would be recorded
            // as a command that did not run.
            assert!(
                !(later.text == "echo recorded" && later.number != reported[recorded].number),
                "{shell} sent a stale line with a fresh number: {later:?}"
            );
            // And if the hidden line arrives at all, it arrives WITH its
            // leading space — which is what the anchor suspends on.
            if later.text.contains("hidden") {
                assert!(
                    later.text.starts_with(' '),
                    "{shell} reported the hidden line with its space stripped, so nothing \
                     downstream can tell it was hidden: {later:?}"
                );
            }
        }
    });
}
