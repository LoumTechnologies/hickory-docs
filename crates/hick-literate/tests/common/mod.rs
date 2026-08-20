//! Cell commands, in the shell the cell actually gets.
//!
//! Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, deliberately:
//! `LocalExecutor::shell` says a Windows user writing `dir` should not need
//! MSYS installed for it to run. A test that embeds POSIX shell in an
//! `<hick:exec>` block therefore does not test what it claims on Windows — it
//! tests whether Git for Windows happens to be on PATH. And when it is, the
//! test still fails under the sandbox: msys binaries need a section object in
//! the global `\BaseNamedObjects` namespace that an AppContainer denies.
//!
//! Worse than failing, some passed. `touch /etc/x && echo WROTE` fails on
//! Windows whatever the sandbox does, so an escape test was green for a run in
//! which nothing was confined. Writing each command in the shell that will run
//! it is what keeps a green result meaning something.
//!
//! Every form below was measured against a real `cmd.exe` on Windows 11
//! (2026-08-20) rather than assumed. The ones that look odd are the ones that
//! had to be.
#![allow(dead_code)]

/// A path as this platform's shell spells it.
///
/// `cmd` is inconsistent about it rather than tolerant: `echo x>out/f.txt`
/// works, `type out/f.txt` fails with "The syntax of the command is
/// incorrect". Converting everywhere avoids depending on which side of that
/// line a command falls.
pub fn path(p: &str) -> String {
    if cfg!(windows) {
        p.replace('/', "\\")
    } else {
        p.to_string()
    }
}

/// Write `text` to `path`, creating or truncating it.
///
/// `echo` on both, NOT `printf`/`set /p`. `<nul set /p=x>f` is the usual way
/// to write without a trailing newline in cmd, and it does two things that
/// break tests: it exits **1**, which stops any `&&` chain after it, and it
/// leaves a trailing space in the file. So the written bytes differ per shell
/// by a line ending, and assertions compare trimmed content — the guarantee is
/// which bytes land in which file, not how the line ends.
pub fn write(text: &str, to: &str) -> String {
    format!("echo {text}>{}", path(to))
}

/// Print a file's contents.
pub fn show(file: &str) -> String {
    if cfg!(windows) {
        format!("type {}", path(file))
    } else {
        format!("cat {file}")
    }
}

/// Create a directory and any parents it needs.
pub fn make_dir(dir: &str) -> String {
    if cfg!(windows) {
        // cmd's mkdir creates intermediate directories when command
        // extensions are on, which is the default.
        format!("mkdir {}", path(dir))
    } else {
        format!("mkdir -p {dir}")
    }
}

/// Fail unless `file` is absent — the cell asserts the absence itself, so a
/// leak fails the exec rather than being reported as a passing read.
pub fn require_absent(file: &str) -> String {
    if cfg!(windows) {
        format!("if exist {} exit 1", path(file))
    } else {
        format!("test ! -f {file}")
    }
}

/// Join cell commands so a failure stops the rest, as `&&` does.
pub fn and(parts: &[String]) -> String {
    parts.join(" && ")
}

/// A cell that keeps running until something stops it.
///
/// Not `waitfor` and not `ping`. `waitfor /t 30 x` needs an object in the
/// global `\BaseNamedObjects` namespace, which fails outright inside an
/// AppContainer ("Cannot wait for the specified signal"), and pinging loopback
/// needs the network a confined cell is denied. Either fails instantly for the
/// wrong reason, and a timeout test whose cell dies immediately passes without
/// ever reaching the timeout. A `for /L` burns CPU but depends on nothing
/// outside cmd itself.
pub fn runs_until_killed() -> String {
    if cfg!(windows) {
        "for /L %i in (1,1,2000000000) do @rem".to_string()
    } else {
        "sleep 30".to_string()
    }
}

/// A cell that takes appreciably longer than a few tens of milliseconds and
/// then succeeds, printing `echo`.
///
/// Here the executor is unsandboxed, so loopback `ping` is available and is
/// the cheapest sub-second sleep cmd has that still exits 0 — `timeout /t`
/// needs a console it does not have under redirected stdin.
pub fn sleeps_then_echoes(text: &str) -> String {
    if cfg!(windows) {
        format!("ping -n 2 127.0.0.1 >nul & echo {text}")
    } else {
        format!("sleep 0.3; echo {text}")
    }
}

/// Create an empty file, as a marker that a cell ran.
pub fn create_marker(at: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("type nul > {}", at.display())
    } else {
        format!("touch {}", at.display())
    }
}

/// Append one mark to a file that counts executions.
pub fn append_mark(mark: &str, to: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("echo {mark}>>{}", to.display())
    } else {
        format!("printf {mark} >> {}", to.display())
    }
}

/// The marks in a counter file, with the line endings the shell added removed.
///
/// `printf x >>` writes one byte; cmd's `echo x>>` writes the byte and a CRLF,
/// and the no-newline forms in cmd exit 1 and would break the cell. What the
/// test means is "how many times did this run, and in what order", so compare
/// the marks rather than the bytes.
pub fn marks(content: &str) -> String {
    content.chars().filter(|c| !c.is_whitespace()).collect()
}
