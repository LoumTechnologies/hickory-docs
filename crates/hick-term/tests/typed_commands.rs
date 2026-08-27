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
//! Skipped loudly without bash. zsh is **not** covered: it was not installed
//! on the machine this was written on, so its `preexec` hook ships written
//! and unverified, and pretending otherwise is what a green suite that tests
//! nothing looks like.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::command::TypedCommand;
use hick_term::config::TermConfig;
use hick_term::session::{Session, SessionSpec};

fn have_bash() -> Option<String> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join("bash"))
        .find(|candidate| candidate.is_file())
        .map(|path| path.to_string_lossy().into_owned())
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
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    let mut shell = start(&bash);
    shell.type_line("echo one\n");
    // A pipeline is ONE typed line. A `DEBUG` trap reports it as two, and a
    // cell holding `head -1` on its own does not reproduce.
    shell.type_line("echo a | tr a b\n");
    // A loop is one line too; a DEBUG trap reports one record per iteration.
    shell.type_line("for i in 1 2; do echo $i; done\n");

    let lines: Vec<String> = shell.reported().into_iter().map(|c| c.text).collect();
    assert_eq!(
        lines,
        vec![
            "echo one".to_string(),
            "echo a | tr a b".to_string(),
            "for i in 1 2; do echo $i; done".to_string(),
        ],
        "the shell did not report the typed lines"
    );
}

#[test]
fn nothing_is_reported_before_anything_is_typed() {
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    // The failure mode of the `PROMPT_COMMAND` + `history 1` design: at the
    // first prompt it reports the last line of the user's ~/.bash_history —
    // a command they never typed in this session, written into a document
    // before they touched the keyboard.
    let mut shell = start(&bash);
    assert_eq!(
        shell.reported(),
        Vec::new(),
        "a command was reported before anything was typed"
    );
}

#[test]
fn an_empty_line_reports_nothing_and_a_repeat_reports_twice() {
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    let mut shell = start(&bash);
    shell.type_line("\n");
    shell.type_line("\n");
    shell.type_line("echo twice\n");
    let lines: Vec<String> = shell.reported().into_iter().map(|c| c.text).collect();
    assert_eq!(lines, vec!["echo twice".to_string()]);
}

#[test]
fn the_number_moves_only_when_the_shell_recorded_the_line() {
    let Some(bash) = have_bash() else {
        eprintln!("SKIPPED: no bash on this machine");
        return;
    };
    let mut shell = start(&bash);
    // `ignorespace` is half of the very common `HISTCONTROL=ignoreboth`, and
    // it keeps a space-prefixed line out of history. bash's PS0 runs in a
    // subshell and cannot remember what it last sent, so `history 1` then
    // reports the PREVIOUS line — which would record a command that did not
    // run. The number is what tells them apart.
    shell.type_line("HISTCONTROL=ignorespace\n");
    shell.type_line("echo recorded\n");
    shell.type_line(" echo hidden\n");

    let reported = shell.reported();
    let recorded = reported
        .iter()
        .position(|c| c.text == "echo recorded")
        .expect("the plain command was reported");
    // Whatever came after it either repeated the previous line — with the
    // previous NUMBER, which is how the anchor knows to suspend instead of
    // writing it down — or was not reported at all. What must never happen
    // is a NEW number carrying the stale text.
    for later in &reported[recorded + 1..] {
        assert!(
            !(later.text == "echo recorded" && later.number != reported[recorded].number),
            "a stale line arrived with a fresh number, which would be recorded as a \
             command that never ran: {later:?}"
        );
        // And the hidden line must not have been reported at all.
        assert_ne!(
            later.text, "echo hidden",
            "a line the shell hid was reported"
        );
    }
}
