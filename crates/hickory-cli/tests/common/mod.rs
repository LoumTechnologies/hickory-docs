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
