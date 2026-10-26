<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain crates/hick-literate/tests/common/mod.rs

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ef1a9cf0de20"></a>
## `crates/hick-literate/tests/common/mod.rs`

<a id="f-ef1a9cf0de20-path"></a>
### `path`

Cell commands, in the shell the cell actually gets.
Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, deliberately:
`LocalExecutor::shell` says a Windows user writing `dir` should not need
MSYS installed for it to run. A test that embeds POSIX shell in an
`<hick:exec>` block therefore does not test what it claims on Windows — it
tests whether Git for Windows happens to be on PATH. And when it is, the
test still fails under the sandbox: msys binaries need a section object in
the global `\BaseNamedObjects` namespace that an AppContainer denies.
Worse than failing, some passed. `touch /etc/x && echo WROTE` fails on
Windows whatever the sandbox does, so an escape test was green for a run in
which nothing was confined. Writing each command in the shell that will run
it is what keeps a green result meaning something.
Every form below was measured against a real `cmd.exe` on Windows 11
(2026-08-20) rather than assumed. The ones that look odd are the ones that
had to be.

<storycode:copy id="f-ef1a9cf0de20-path">
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

</storycode:copy>
<a id="f-ef1a9cf0de20-write"></a>
### `write`

Write `text` to `path`, creating or truncating it.
`echo` on both, NOT `printf`/`set /p`. `<nul set /p=x>f` is the usual way
to write without a trailing newline in cmd, and it does two things that
break tests: it exits **1**, which stops any `&&` chain after it, and it
leaves a trailing space in the file. So the written bytes differ per shell
by a line ending, and assertions compare trimmed content — the guarantee is
which bytes land in which file, not how the line ends.

<storycode:copy id="f-ef1a9cf0de20-write">
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

</storycode:copy>
<a id="f-ef1a9cf0de20-show"></a>
### `show`

<storycode:copy id="f-ef1a9cf0de20-show">
/// Print a file's contents.
pub fn show(file: &str) -> String {
    if cfg!(windows) {
        format!("type {}", path(file))
    } else {
        format!("cat {file}")
    }
}

</storycode:copy>
<a id="f-ef1a9cf0de20-make_dir"></a>
### `make_dir`

Create a directory and any parents it needs.

<storycode:copy id="f-ef1a9cf0de20-make_dir">
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

</storycode:copy>
<a id="f-ef1a9cf0de20-require_absent"></a>
### `require_absent`

Fail unless `file` is absent — the cell asserts the absence itself, so a
leak fails the exec rather than being reported as a passing read.

<storycode:copy id="f-ef1a9cf0de20-require_absent">
/// Fail unless `file` is absent — the cell asserts the absence itself, so a
/// leak fails the exec rather than being reported as a passing read.
pub fn require_absent(file: &str) -> String {
    if cfg!(windows) {
        format!("if exist {} exit 1", path(file))
    } else {
        format!("test ! -f {file}")
    }
}

</storycode:copy>
<a id="f-ef1a9cf0de20-and"></a>
### `and`

Join cell commands so a failure stops the rest, as `&&` does.

<storycode:copy id="f-ef1a9cf0de20-and">
/// Join cell commands so a failure stops the rest, as `&&` does.
pub fn and(parts: &[String]) -> String {
    parts.join(" && ")
}

</storycode:copy>
<a id="f-ef1a9cf0de20-runs_until_killed"></a>
### `runs_until_killed`

A cell that keeps running until something stops it.
Not `waitfor` and not `ping`. `waitfor /t 30 x` needs an object in the
global `\BaseNamedObjects` namespace, which fails outright inside an
AppContainer ("Cannot wait for the specified signal"), and pinging loopback
needs the network a confined cell is denied. Either fails instantly for the
wrong reason, and a timeout test whose cell dies immediately passes without
ever reaching the timeout. A `for /L` burns CPU but depends on nothing
outside cmd itself.

<storycode:copy id="f-ef1a9cf0de20-runs_until_killed">
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

</storycode:copy>
<a id="f-ef1a9cf0de20-sleeps_then_echoes"></a>
### `sleeps_then_echoes`

A cell that takes appreciably longer than a few tens of milliseconds and
then succeeds, printing `echo`.
Here the executor is unsandboxed, so loopback `ping` is available and is
the cheapest sub-second sleep cmd has that still exits 0 — `timeout /t`
needs a console it does not have under redirected stdin.

<storycode:copy id="f-ef1a9cf0de20-sleeps_then_echoes">
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

</storycode:copy>
<a id="f-ef1a9cf0de20-create_marker"></a>
### `create_marker`

Create an empty file, as a marker that a cell ran.

<storycode:copy id="f-ef1a9cf0de20-create_marker">
/// Create an empty file, as a marker that a cell ran.
pub fn create_marker(at: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("type nul > {}", at.display())
    } else {
        format!("touch {}", at.display())
    }
}

</storycode:copy>
<a id="f-ef1a9cf0de20-append_mark"></a>
### `append_mark`

Append one mark to a file that counts executions.

<storycode:copy id="f-ef1a9cf0de20-append_mark">
/// Append one mark to a file that counts executions.
pub fn append_mark(mark: &str, to: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("echo {mark}>>{}", to.display())
    } else {
        format!("printf {mark} >> {}", to.display())
    }
}

</storycode:copy>
<a id="f-ef1a9cf0de20-marks"></a>
### `marks`

The marks in a counter file, with the line endings the shell added removed.
`printf x >>` writes one byte; cmd's `echo x>>` writes the byte and a CRLF,
and the no-newline forms in cmd exit 1 and would break the cell. What the
test means is "how many times did this run, and in what order", so compare
the marks rather than the bytes.

<storycode:copy id="f-ef1a9cf0de20-marks">
/// The marks in a counter file, with the line endings the shell added removed.
///
/// `printf x >>` writes one byte; cmd's `echo x>>` writes the byte and a CRLF,
/// and the no-newline forms in cmd exit 1 and would break the cell. What the
/// test means is "how many times did this run, and in what order", so compare
/// the marks rather than the bytes.
pub fn marks(content: &str) -> String {
    content.chars().filter(|c| !c.is_whitespace()).collect()
}
</storycode:copy>

### Assemble `crates/hick-literate/tests/common/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-literate/tests/common/mod.rs"><storycode:paste select="#f-ef1a9cf0de20-path"/><storycode:paste select="#f-ef1a9cf0de20-write"/><storycode:paste select="#f-ef1a9cf0de20-show"/><storycode:paste select="#f-ef1a9cf0de20-make_dir"/><storycode:paste select="#f-ef1a9cf0de20-require_absent"/><storycode:paste select="#f-ef1a9cf0de20-and"/><storycode:paste select="#f-ef1a9cf0de20-runs_until_killed"/><storycode:paste select="#f-ef1a9cf0de20-sleeps_then_echoes"/><storycode:paste select="#f-ef1a9cf0de20-create_marker"/><storycode:paste select="#f-ef1a9cf0de20-append_mark"/><storycode:paste select="#f-ef1a9cf0de20-marks"/></storycode:file>

</storycode:doc>
