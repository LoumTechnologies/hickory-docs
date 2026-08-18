//! A real shell, in a real PTY, and what it does or does not say about `cd`.
//!
//! Protects docs/guarantees/terminal/a-session-appears-where-it-is-working.md
//!
//! Every other OSC 7 test feeds bytes straight into the scanner, which proves
//! the parser and nothing about the shells people have. That gap is what this
//! closes, and the answer it found is not the comfortable one: **macOS's default
//! zsh emits OSC 7 only when `TERM_PROGRAM=Apple_Terminal`**, because the hook
//! lives in `/etc/zshrc_Apple_Terminal`, sourced from `/etc/zshrc` by that name.
//! A terminal that answers truthfully about what it is gets nothing.
//!
//! So there are two tests, and both matter:
//!
//! - the shell that *does* emit it is followed correctly, end to end, through a
//!   PTY rather than through a byte fixture; and
//! - the shell that does not is reported as not-live rather than guessed at,
//!   which is the property a reader of the folder tree depends on.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::session::Session;
use hick_term::{SessionSpec, TermConfig, Terminals};

fn shell_at(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

fn open(terminals: &Terminals, dir: &std::path::Path, argv: &[&str]) -> Arc<Session> {
    terminals
        .open(SessionSpec {
            title: argv[0].to_string(),
            cwd: dir.to_path_buf(),
            argv: argv.iter().map(|a| a.to_string()).collect(),
            monitor: false,
        })
        .expect("the shell starts")
}

/// Poll the summary until `done`, or give up. A shell prints its prompt when it
/// is ready, not when we ask.
fn wait_for(
    session: &Session,
    timeout: Duration,
    done: impl Fn(&hick_term::SessionSummary) -> bool,
) -> hick_term::SessionSummary {
    let deadline = Instant::now() + timeout;
    loop {
        let summary = session.summary();
        if done(&summary) || Instant::now() > deadline {
            return summary;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// A shell that emits OSC 7 is followed. Driven through a PTY, so this is the
/// shell speaking and not a fixture.
///
/// `zsh -ic` with the hook forced on is how the sequence is provoked without
/// depending on `TERM_PROGRAM` reaching the child: the point being tested is the
/// scanner's behaviour against a real shell's real bytes.
#[test]
fn a_shell_that_emits_osc_7_moves_the_session() {
    if !shell_at("/bin/zsh") && !shell_at("/usr/bin/zsh") {
        eprintln!("no zsh on this machine — skipped");
        return;
    }
    let zsh = if shell_at("/bin/zsh") {
        "/bin/zsh"
    } else {
        "/usr/bin/zsh"
    };

    let start = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    // Canonicalised because macOS hands out `/var/folders/...` paths that are
    // really `/private/var/...`, and the shell reports what it resolved to.
    let target = elsewhere.path().canonicalize().unwrap();

    let terminals = Terminals::new(TermConfig::default());
    let session = open(&terminals, start.path(), &[zsh, "-f", "-i"]);

    let before = wait_for(&session, Duration::from_secs(5), |s| !s.cwd.is_empty());
    assert!(
        !before.cwd_is_live,
        "nothing has been emitted yet, so the directory cannot be live: {before:?}"
    );

    // The hook macOS keeps for Terminal.app, written out by hand: `cd` and then
    // announce where we landed.
    session
        .write(b"osc7() { printf '\\033]7;file://%s%s\\a' \"$HOST\" \"$PWD\" }\n")
        .unwrap();
    session
        .write(format!("cd {} && osc7\n", target.display()).as_bytes())
        .unwrap();

    let after = wait_for(&session, Duration::from_secs(10), |s| s.cwd_is_live);
    assert!(
        after.cwd_is_live,
        "the shell announced its directory and the session did not follow: {after:?}"
    );
    assert_eq!(
        std::path::Path::new(&after.cwd),
        target.as_path(),
        "followed to the wrong directory"
    );

    session.kill().ok();
}

/// A shell that says nothing leaves the session where it started, and says so.
///
/// This is the macOS default, not an edge case: with a truthful `TERM_PROGRAM`,
/// `/etc/zshrc` never sources the hook, so the folder tree shows the
/// started-in directory for the life of the session. `cwd_is_live` being false
/// is the whole reason a reader is not misled by it.
#[test]
fn a_silent_shell_stays_where_it_started_and_admits_it() {
    if !shell_at("/bin/sh") {
        eprintln!("no /bin/sh — skipped");
        return;
    }
    let start = tempfile::tempdir().unwrap();
    // Not canonicalised: the fallback is the directory the session was *told* to
    // start in, verbatim, which is the honest thing for it to report.
    let started_in = start.path().to_path_buf();

    let terminals = Terminals::new(TermConfig::default());
    let session = open(&terminals, start.path(), &["/bin/sh", "-i"]);

    // Somewhere real, and definitely not where it started.
    session.write(b"cd /usr/share\n").unwrap();
    // Give it longer than it could possibly need, so this is "it never came"
    // rather than "we did not wait".
    let summary = wait_for(&session, Duration::from_secs(3), |s| s.cwd_is_live);

    assert!(
        !summary.cwd_is_live,
        "a shell that emits no OSC 7 must not be reported as live: {summary:?}"
    );
    assert_eq!(
        std::path::Path::new(&summary.cwd),
        started_in.as_path(),
        "the fallback must be the directory the session started in"
    );

    session.kill().ok();
}
