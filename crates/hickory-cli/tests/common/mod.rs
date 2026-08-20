//! Cell commands, in the shell the cell actually gets.
//!
//! Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, deliberately
//! — `LocalExecutor::shell` says a Windows user writing `dir` should not need
//! MSYS installed. A test that embeds POSIX shell in a cell therefore does not
//! test what it claims on Windows, so these tests used to be `#![cfg(unix)]`
//! and the behaviour went unverified there entirely.
//!
//! The forms below were measured against a real `cmd.exe` on Windows 11
//! (2026-08-20). See `crates/hick-literate/tests/common/mod.rs` for the same
//! helpers on the pipeline side; they are duplicated rather than shared
//! because an integration test cannot reach across crates without a
//! dev-dependency that exists only for this.
#![allow(dead_code)]

/// A cell that prints each of `lines`, one per line, and exits 0.
///
/// The cmd form is `echo one& echo two`. Two things about it were measured
/// rather than guessed: the `&` must have no space before it (cmd's `echo`
/// prints everything up to the separator, trailing space included), and there
/// is no cmd builtin that can print without a trailing newline — `<nul set /p=`
/// is the usual trick, and it exits **1**, which fails the cell outright.
///
/// So on Windows a cell's output ends `\r\n` and there is nothing the document
/// can do about it. That is why `hickory_executor::normalize_captured_newlines`
/// exists: the bytes are recorded as `\n`, and the `<hick:expect match="exact">`
/// written in this file — which is LF, being text in git — means the same
/// thing here as it does under `sh`.
/// docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md
/// The separator is per-shell too, not just the command: cmd sequences with
/// `&`, which in `sh` would put the first command in the background instead.
pub fn echo_lines(lines: &[&str]) -> String {
    if cfg!(windows) {
        lines
            .iter()
            .map(|l| format!("echo {l}"))
            .collect::<Vec<_>>()
            .join("& ")
    } else {
        // Literal backslash-n: this string is written into a `.hick` document,
        // where it is the shell that interprets it, not Rust.
        format!("printf '{}\\n'", lines.join("\\n"))
    }
}

/// A cell that keeps running until something stops it.
///
/// Not `waitfor` and not `ping`: `waitfor` needs an object in the global
/// `\BaseNamedObjects` namespace and fails immediately inside an AppContainer,
/// and ping needs a network a confined cell may not have. Either fails for the
/// wrong reason, and a timeout test whose cell dies instantly passes without
/// reaching the timeout it exists to check.
pub fn runs_until_killed() -> String {
    if cfg!(windows) {
        "for /L %i in (1,1,2000000000) do @rem".to_string()
    } else {
        "sleep 30".to_string()
    }
}

/// A cell that takes roughly `seconds` and then succeeds, printing `text`.
///
/// `ping -n` is the cheapest sleep cmd has that still exits 0: `timeout /t`
/// needs a console it does not have under redirected stdin, and `waitfor`
/// exits 1. `ping -n N` sends N pings a second apart, so it waits about N-1
/// seconds.
pub fn sleeps_then_echoes(seconds: u32, text: &str) -> String {
    if cfg!(windows) {
        format!("ping -n {} 127.0.0.1 >nul & echo {text}", seconds + 1)
    } else {
        format!("sleep {seconds}; echo {text}")
    }
}

/// A cell whose output is different every single time it runs.
///
/// `date +%s%N` is a GNU-coreutils spelling twice over: `date` in cmd is a
/// builtin that tries to *set* the clock, and given an argument it fails —
/// deterministically, with the same message every run. A test that needs a
/// volatile output would then be handed a perfectly stable one and would pass
/// while proving nothing. `%TIME%` is re-expanded by cmd on every run and
/// carries centiseconds; `%RANDOM%` is there so two runs inside the same
/// centisecond still differ.
pub fn changes_every_run() -> String {
    if cfg!(windows) {
        "echo %TIME% %RANDOM%".to_string()
    } else {
        "date +%s%N".to_string()
    }
}

/// A cell that creates `path`, as evidence it ran.
///
/// Named for what it is rather than `create_marker`, which already exists in
/// `hick-literate`'s own test helpers — two modules with a same-named helper
/// merged cleanly once and then did not compile.
///
/// A FILE rather than a marker in the output, because a transcript records the
/// command as well as its result: a cell that echoed a marker would put that
/// marker in the document whether or not it ever ran, and the assertion would
/// be reading its own fixture back. It also survives a bug class that stdout
/// does not — a cell that silently runs nothing still produces a plausible
/// empty transcript, but it cannot create a file.
pub fn create_side_effect_file(path: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("type nul > \"{}\"", path.display())
    } else {
        format!("touch '{}'", path.display())
    }
}

/// A program that is installed wherever a cell can run at all.
///
/// The shell itself: if this is missing, no cell could run regardless, so it
/// is the only thing safe to assert is present on someone else's machine.
pub fn always_installed_program() -> &'static str {
    if cfg!(windows) { "cmd" } else { "sh" }
}
