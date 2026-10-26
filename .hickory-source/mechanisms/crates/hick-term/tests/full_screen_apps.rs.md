<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-term/tests/full_screen_apps.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-31c9ccf64945"></a>
## `crates/hick-term/tests/full_screen_apps.rs`

Related contracts: [Launch the artifact built from the selected compiled project](../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Full-Screen Program Gets A Real Terminal](../../../../guarantees/terminal/a-full-screen-program-gets-a-real-terminal/contract.md), [A Terminal Outlives Its Pane](../../../../guarantees/terminal/a-terminal-outlives-its-pane/contract.md).

[Read the test `installed` beside its requirement](../../../../guarantees/terminal/a-terminal-outlives-its-pane/contract.md#f-31c9ccf64945-installed).
<a id="f-31c9ccf64945-skip"></a>
### `skip`

Say what was not tested, and why. Never skip in silence.

<storycode:copy id="f-31c9ccf64945-skip">
/// Say what was not tested, and why. Never skip in silence.
fn skip(program: &str) {
    eprintln!("SKIPPED: {program} is not installed on this machine");
}

</storycode:copy>
<a id="f-31c9ccf64945-open"></a>
### `open`

<storycode:copy id="f-31c9ccf64945-open">
fn open(terminals: &Terminals, dir: &std::path::Path, argv: &[&str]) -> Arc<Session> {
    terminals
        .open(SessionSpec {
            title: argv[0].to_string(),
            cwd: dir.to_path_buf(),
            argv: argv.iter().map(|a| a.to_string()).collect(),
            monitor: false,
        })
        .expect("the program starts")
}

</storycode:copy>
<a id="f-31c9ccf64945-wait_for"></a>
### `wait_for`

Wait until the drawn screen satisfies `done`, or give up. Full-screen
programs paint in bursts, so this polls rather than sleeping once.

<storycode:copy id="f-31c9ccf64945-wait_for">
/// Wait until the drawn screen satisfies `done`, or give up. Full-screen
/// programs paint in bursts, so this polls rather than sleeping once.
fn wait_for(session: &Session, timeout: Duration, done: impl Fn(&str) -> bool) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        let screen = session.screen_text();
        if done(&screen) || Instant::now() > deadline {
            return screen;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

</storycode:copy>
<a id="f-31c9ccf64945-vim_draws_its_editor_and_leaves_the_alternate_screen_on_exit"></a>
### `vim_draws_its_editor_and_leaves_the_alternate_screen_on_exit`

<storycode:copy id="f-31c9ccf64945-vim_draws_its_editor_and_leaves_the_alternate_screen_on_exit">
#[test]
fn vim_draws_its_editor_and_leaves_the_alternate_screen_on_exit() {
    if !installed("vim") {
        skip("vim");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("poem.txt"), "the hickory stands\n").unwrap();

    let terminals = Terminals::new(TermConfig::default());
    // -u NONE: someone's own vimrc must not decide whether this passes.
    let session = open(
        &terminals,
        dir.path(),
        &["vim", "-u", "NONE", "-n", "poem.txt"],
    );

    let screen = wait_for(&session, Duration::from_secs(10), |s| {
        s.contains("the hickory stands")
    });
    assert!(
        screen.contains("the hickory stands"),
        "vim never drew the file. Screen was:\n{screen}"
    );
    assert!(
        session.alternate_screen(),
        "vim should be on the alternate screen"
    );

    // Resizing must reach the child: vim redraws to the new width.
    session.resize(40, 100).expect("resize reaches the pty");
    let after = wait_for(&session, Duration::from_secs(5), |s| {
        s.contains("the hickory stands")
    });
    assert!(
        after.contains("the hickory stands"),
        "after resize:\n{after}"
    );

    session.write(b":q!\r").unwrap();
    let left = wait_for(&session, Duration::from_secs(5), |_| {
        !session.alternate_screen()
    });
    assert!(
        !session.alternate_screen(),
        "quitting vim should restore the normal screen. Screen was:\n{left}"
    );

    terminals.close(&session.id).unwrap();
}

</storycode:copy>
<a id="f-31c9ccf64945-htop_paints_a_full_screen_of_meters"></a>
### `htop_paints_a_full_screen_of_meters`

<storycode:copy id="f-31c9ccf64945-htop_paints_a_full_screen_of_meters">
#[test]
fn htop_paints_a_full_screen_of_meters() {
    if !installed("htop") {
        skip("htop");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let terminals = Terminals::new(TermConfig::default());
    let session = open(&terminals, dir.path(), &["htop"]);

    // htop's header is the same in every locale we care about: the load
    // average line, and the Tasks counter.
    let screen = wait_for(&session, Duration::from_secs(10), |s| {
        s.contains("Load average") || s.contains("Tasks")
    });
    assert!(
        screen.contains("Load average") || screen.contains("Tasks"),
        "htop never painted. Screen was:\n{screen}"
    );
    assert!(session.alternate_screen(), "htop takes the whole screen");

    session.write(b"q").unwrap();
    terminals.close(&session.id).unwrap();
}

</storycode:copy>
<a id="f-31c9ccf64945-less_pages_a_file_and_answers_its_keys"></a>
### `less_pages_a_file_and_answers_its_keys`

<storycode:copy id="f-31c9ccf64945-less_pages_a_file_and_answers_its_keys">
#[test]
fn less_pages_a_file_and_answers_its_keys() {
    if !installed("less") {
        skip("less");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let lines: String = (1..=200).map(|i| format!("line {i}\n")).collect();
    std::fs::write(dir.path().join("long.txt"), lines).unwrap();

    let terminals = Terminals::new(TermConfig::default());
    let session = open(&terminals, dir.path(), &["less", "long.txt"]);

    let first = wait_for(&session, Duration::from_secs(10), |s| s.contains("line 1"));
    assert!(
        first.contains("line 1"),
        "less never drew page one:\n{first}"
    );

    // G jumps to the end: a key the program reads in raw mode, which only
    // works if the PTY is giving it one.
    session.write(b"G").unwrap();
    let last = wait_for(&session, Duration::from_secs(5), |s| s.contains("line 200"));
    assert!(
        last.contains("line 200"),
        "less did not answer its key. Screen was:\n{last}"
    );

    session.write(b"q").unwrap();
    terminals.close(&session.id).unwrap();
}

</storycode:copy>
<a id="f-31c9ccf64945-a_coding_agent_draws_its_interface"></a>
### `a_coding_agent_draws_its_interface`

The case this whole feature exists for: someone else's coding agent,
running in a plain terminal, drawing its own interface.

<storycode:copy id="f-31c9ccf64945-a_coding_agent_draws_its_interface">
/// The case this whole feature exists for: someone else's coding agent,
/// running in a plain terminal, drawing its own interface.
#[test]
fn a_coding_agent_draws_its_interface() {
    if !installed("claude") {
        skip("claude");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let terminals = Terminals::new(TermConfig::default());
    // --help rather than a session: no API key, no network, no prompt left
    // running — the claim here is about the terminal, not about the agent.
    let session = open(&terminals, dir.path(), &["claude", "--help"]);

    // The claim is about the SCROLLBACK, not the viewport: help longer than
    // 24 rows has already scrolled off the screen model by the time it ends,
    // which is exactly right — the client scrolls back through the replay.
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut transcript = String::new();
    while Instant::now() < deadline {
        transcript = String::from_utf8_lossy(&session.attach().0).to_string();
        if transcript.contains("Usage:") {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(
        transcript.contains("Usage:"),
        "the agent printed nothing a terminal could show. Transcript was:\n{transcript}"
    );
    assert!(
        !session.screen_text().trim().is_empty(),
        "the visible screen should hold the tail of the output"
    );

    terminals.close(&session.id).unwrap();
}
</storycode:copy>

### Assemble `crates/hick-term/tests/full_screen_apps.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/tests/full_screen_apps.rs"><storycode:paste select="#f-31c9ccf64945-installed"/><storycode:paste select="#f-31c9ccf64945-skip"/><storycode:paste select="#f-31c9ccf64945-open"/><storycode:paste select="#f-31c9ccf64945-wait_for"/><storycode:paste select="#f-31c9ccf64945-vim_draws_its_editor_and_leaves_the_alternate_screen_on_exit"/><storycode:paste select="#f-31c9ccf64945-htop_paints_a_full_screen_of_meters"/><storycode:paste select="#f-31c9ccf64945-less_pages_a_file_and_answers_its_keys"/><storycode:paste select="#f-31c9ccf64945-a_coding_agent_draws_its_interface"/></storycode:file>

</storycode:doc>
