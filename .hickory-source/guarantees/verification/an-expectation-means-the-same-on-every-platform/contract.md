<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# An Expectation Means The Same Thing On Every Platform

For: Reviewers deciding what an output, recording or prediction actually establishes.

Given a document whose `<hick:exec>` cell carries an `<hick:expect>`, when the
document is run on any supported platform, then the cell's captured output is
recorded with `\n` line endings — a `\r\n` emitted by the shell, the program,
or the container becomes `\n` at capture, before it reaches the transcript
entry, the returned stdout, the woven `.md`, or a `.hick-cache` recording. A
document that passes on macOS therefore passes on Windows, and a recording made
on one serves a run on the other.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-82e96cc48a54"></a>
## `crates/hickory-cli/tests/common/mod.rs`

Related contracts: [An Expectation Means The Same Thing On Every Platform](contract.md).

<a id="f-82e96cc48a54-echo_lines"></a>
### `echo_lines`

Cell commands, in the shell the cell actually gets.
Cells run through `sh -c` on Unix and `cmd.exe /C` on Windows, deliberately
— `LocalExecutor::shell` says a Windows user writing `dir` should not need
MSYS installed. A test that embeds POSIX shell in a cell therefore does not
test what it claims on Windows, so these tests used to be `#![cfg(unix)]`
and the behaviour went unverified there entirely.
The forms below were measured against a real `cmd.exe` on Windows 11
(2026-08-20). See `crates/hick-literate/tests/common/mod.rs` for the same
helpers on the pipeline side; they are duplicated rather than shared
because an integration test cannot reach across crates without a
dev-dependency that exists only for this.

<storycode:copy id="f-82e96cc48a54-echo_lines">
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
        // Literal backslash-n: this string is written into a `.md` document,
        // where it is the shell that interprets it, not Rust.
        format!("printf '{}\\n'", lines.join("\\n"))
    }
}

</storycode:copy>
<a id="f-82e96cc48a54-runs_until_killed"></a>
### `runs_until_killed`

A cell that keeps running until something stops it.
Not `waitfor` and not `ping`: `waitfor` needs an object in the global
`\BaseNamedObjects` namespace and fails immediately inside an AppContainer,
and ping needs a network a confined cell may not have. Either fails for the
wrong reason, and a timeout test whose cell dies instantly passes without
reaching the timeout it exists to check.

<storycode:copy id="f-82e96cc48a54-runs_until_killed">
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

</storycode:copy>
<a id="f-82e96cc48a54-sleeps_then_echoes"></a>
### `sleeps_then_echoes`

A cell that takes roughly `seconds` and then succeeds, printing `text`.
`ping -n` is the cheapest sleep cmd has that still exits 0: `timeout /t`
needs a console it does not have under redirected stdin, and `waitfor`
exits 1. `ping -n N` sends N pings a second apart, so it waits about N-1
seconds.

<storycode:copy id="f-82e96cc48a54-sleeps_then_echoes">
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

</storycode:copy>
<a id="f-82e96cc48a54-changes_every_run"></a>
### `changes_every_run`

A cell whose output is different every single time it runs.
`date +%s%N` is a GNU-coreutils spelling twice over: `date` in cmd is a
builtin that tries to *set* the clock, and given an argument it fails —
deterministically, with the same message every run. A test that needs a
volatile output would then be handed a perfectly stable one and would pass
while proving nothing. `%TIME%` is re-expanded by cmd on every run and
carries centiseconds; `%RANDOM%` is there so two runs inside the same
centisecond still differ.

<storycode:copy id="f-82e96cc48a54-changes_every_run">
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

</storycode:copy>
<a id="f-82e96cc48a54-create_side_effect_file"></a>
### `create_side_effect_file`

A cell that creates `path`, as evidence it ran.
Named for what it is rather than `create_marker`, which already exists in
`hick-literate`'s own test helpers — two modules with a same-named helper
merged cleanly once and then did not compile.
A FILE rather than a marker in the output, because a transcript records the
command as well as its result: a cell that echoed a marker would put that
marker in the document whether or not it ever ran, and the assertion would
be reading its own fixture back. It also survives a bug class that stdout
does not — a cell that silently runs nothing still produces a plausible
empty transcript, but it cannot create a file.

<storycode:copy id="f-82e96cc48a54-create_side_effect_file">
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

</storycode:copy>
<a id="f-82e96cc48a54-always_installed_program"></a>
### `always_installed_program`

A program that is installed wherever a cell can run at all.
The shell itself: if this is missing, no cell could run regardless, so it
is the only thing safe to assert is present on someone else's machine.

<storycode:copy id="f-82e96cc48a54-always_installed_program">
/// A program that is installed wherever a cell can run at all.
///
/// The shell itself: if this is missing, no cell could run regardless, so it
/// is the only thing safe to assert is present on someone else's machine.
pub fn always_installed_program() -> &'static str {
    if cfg!(windows) { "cmd" } else { "sh" }
}
</storycode:copy>

### Assemble `crates/hickory-cli/tests/common/mod.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-cli/tests/common/mod.rs"><storycode:paste select="#f-82e96cc48a54-echo_lines"/><storycode:paste select="#f-82e96cc48a54-runs_until_killed"/><storycode:paste select="#f-82e96cc48a54-sleeps_then_echoes"/><storycode:paste select="#f-82e96cc48a54-changes_every_run"/><storycode:paste select="#f-82e96cc48a54-create_side_effect_file"/><storycode:paste select="#f-82e96cc48a54-always_installed_program"/></storycode:file>

<a id="output-eaa988f0d416"></a>
## `crates/hickory-executor-canopy/src/frame.rs`

Related contracts: [Only `hickory-executor-canopy` Knows Cloud Canopy's API](../../execution/canopy-api-isolated-to-one-crate/contract.md), [An Expectation Means The Same Thing On Every Platform](contract.md).

<a id="f-eaa988f0d416-SENTINEL_START"></a>
### `SENTINEL_START`

The guest pty protocol, mirrored from cloud-canopy's reference client
(`crates/canopy-cli/src/guest.rs` at the vendored proto's commit).
The guest hands each channel connection to a shell on a **pty**, so
everything written is echoed back interleaved with output. The protocol
therefore sends the whole script as one base64-encoded line — the echo is
exactly one predictable line — and brackets the real output in sentinels:
\```text
echo __CANOPY_S__; echo <b64 script> | base64 -d | /bin/sh; echo __CANOPY_E__$?
\```
Capture starts after a line that is *nothing but* the start sentinel
(the echoed command also contains it, but never alone on a line) and ends
at the end sentinel, whose trailing digits are the script's exit code.
ANSI escapes are stripped first: the guest shell brackets its prompt with
bracketed-paste sequences, so the sentinel arrives as
`\x1b[?2004l__CANOPY_S__` and a raw comparison never matches.

<storycode:copy id="f-eaa988f0d416-SENTINEL_START">
//! The guest pty protocol, mirrored from cloud-canopy's reference client
//! (`crates/canopy-cli/src/guest.rs` at the vendored proto's commit).
//!
//! The guest hands each channel connection to a shell on a **pty**, so
//! everything written is echoed back interleaved with output. The protocol
//! therefore sends the whole script as one base64-encoded line — the echo is
//! exactly one predictable line — and brackets the real output in sentinels:
//!
//! ```text
//! echo __CANOPY_S__; echo <b64 script> | base64 -d | /bin/sh; echo __CANOPY_E__$?
//! ```
//!
//! Capture starts after a line that is *nothing but* the start sentinel
//! (the echoed command also contains it, but never alone on a line) and ends
//! at the end sentinel, whose trailing digits are the script's exit code.
//! ANSI escapes are stripped first: the guest shell brackets its prompt with
//! bracketed-paste sequences, so the sentinel arrives as
//! `\x1b[?2004l__CANOPY_S__` and a raw comparison never matches.

use base64::Engine as _;

/// Line that opens captured output. Never printed alone by anything else.
pub const SENTINEL_START: &str = "__CANOPY_S__";
</storycode:copy>
<a id="f-eaa988f0d416-SENTINEL_END"></a>
### `SENTINEL_END`

Prefix of the line that closes captured output; the exit code follows.

<storycode:copy id="f-eaa988f0d416-SENTINEL_END">
/// Prefix of the line that closes captured output; the exit code follows.
pub const SENTINEL_END: &str = "__CANOPY_E__";
</storycode:copy>
<a id="f-eaa988f0d416-READY_BANNER"></a>
### `READY_BANNER`

Printed by the guest agent when a connection is handed to a shell.

<storycode:copy id="f-eaa988f0d416-READY_BANNER">
/// Printed by the guest agent when a connection is handed to a shell.
pub const READY_BANNER: &str = "canopy-guest ready";

</storycode:copy>
<a id="f-eaa988f0d416-encode_script"></a>
### `encode_script`

Base64 for scripts on the wire (standard alphabet, padded — must match
what the guest's `base64 -d` accepts).

<storycode:copy id="f-eaa988f0d416-encode_script">
/// Base64 for scripts on the wire (standard alphabet, padded — must match
/// what the guest's `base64 -d` accepts).
pub fn encode_script(script: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(script)
}

</storycode:copy>
<a id="f-eaa988f0d416-decode_script"></a>
### `decode_script`

Decode the base64 produced by [`encode_script`] (used by tests and the
in-crate mock guest).

<storycode:copy id="f-eaa988f0d416-decode_script">
/// Decode the base64 produced by [`encode_script`] (used by tests and the
/// in-crate mock guest).
pub fn decode_script(encoded: &str) -> anyhow::Result<String> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded.trim())?;
    Ok(String::from_utf8(bytes)?)
}

</storycode:copy>
<a id="f-eaa988f0d416-exec_line"></a>
### `exec_line`

The single line sent to the guest shell to run `script`.
`/bin/sh` by absolute path: a minimal guest image has no `sh` on PATH
(the guest agent execs its shell by store path), and the bare
"command not found" would look like the script was wrong rather than the
interpreter being unreachable.

<storycode:copy id="f-eaa988f0d416-exec_line">
/// The single line sent to the guest shell to run `script`.
///
/// `/bin/sh` by absolute path: a minimal guest image has no `sh` on PATH
/// (the guest agent execs its shell by store path), and the bare
/// "command not found" would look like the script was wrong rather than the
/// interpreter being unreachable.
pub fn exec_line(script: &str) -> String {
    format!(
        "echo {SENTINEL_START}; echo {} | base64 -d | /bin/sh; echo {SENTINEL_END}$?\n",
        encode_script(script)
    )
}

</storycode:copy>
<a id="f-eaa988f0d416-shell_quote"></a>
### `shell_quote`

Quote `s` as a single shell word (single quotes, `'` → `'\''`).

<storycode:copy id="f-eaa988f0d416-shell_quote">
/// Quote `s` as a single shell word (single quotes, `'` → `'\''`).
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

</storycode:copy>
<a id="f-eaa988f0d416-strip_ansi"></a>
### `strip_ansi`

Remove ANSI escape sequences and carriage returns.
Everything talking to the guest sees a pty, so anything that colourises
or manages a prompt when attached to a terminal does so here.
Dropping `\r` outright is *stricter* than the rule the other executors
follow (`hickory_executor::normalize_captured_newlines` rewrites `\r\n` and
leaves a lone `\r` alone), and that is deliberate rather than an oversight:
a pty ends every line with `\r\n` and uses a bare `\r` to move the cursor,
so neither carries meaning once the stream has been flattened into recorded
text. This is what already satisfies
`docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md`
for the canopy executor.

<storycode:copy id="f-eaa988f0d416-strip_ansi">
/// Remove ANSI escape sequences and carriage returns.
///
/// Everything talking to the guest sees a pty, so anything that colourises
/// or manages a prompt when attached to a terminal does so here.
///
/// Dropping `\r` outright is *stricter* than the rule the other executors
/// follow (`hickory_executor::normalize_captured_newlines` rewrites `\r\n` and
/// leaves a lone `\r` alone), and that is deliberate rather than an oversight:
/// a pty ends every line with `\r\n` and uses a bare `\r` to move the cursor,
/// so neither carries meaning once the stream has been flattened into recorded
/// text. This is what already satisfies
/// `docs/guarantees/verification/an-expectation-means-the-same-on-every-platform.md`
/// for the canopy executor.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            // A CSI sequence is ESC '[', parameter/intermediate bytes, then a
            // final byte in @-~. The '[' is itself in that range, so consume
            // it before scanning or every sequence "ends" immediately.
            if chars.next() == Some('[') {
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            }
        } else if c != '\r' {
            out.push(c);
        }
    }
    out
}

</storycode:copy>
<a id="f-eaa988f0d416-FrameItem"></a>
### `FrameItem`

One parsed item from the guest byte stream.

<storycode:copy id="f-eaa988f0d416-FrameItem">
/// One parsed item from the guest byte stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameItem {
    /// A captured output line (between the sentinels), escape-stripped,
    /// including its trailing newline.
    Line(String),
    /// The end sentinel arrived; the script exited with this code. Nothing
    /// further is captured.
    Done(i32),
}

</storycode:copy>
<a id="f-eaa988f0d416-FrameParser"></a>
### `FrameParser`

Incremental parser for the sentinel-framed guest stream.
Feed raw bytes as they arrive (chunk boundaries are arbitrary — a
sentinel may span two gRPC chunks); complete lines come back as
[`FrameItem`]s.

<storycode:copy id="f-eaa988f0d416-FrameParser">
/// Incremental parser for the sentinel-framed guest stream.
///
/// Feed raw bytes as they arrive (chunk boundaries are arbitrary — a
/// sentinel may span two gRPC chunks); complete lines come back as
/// [`FrameItem`]s.
#[derive(Debug, Default)]
pub struct FrameParser {
    buf: Vec<u8>,
    started: bool,
    done: bool,
}

</storycode:copy>
<a id="f-eaa988f0d416-impl-FrameParser"></a>
### `impl-FrameParser`

<storycode:copy id="f-eaa988f0d416-impl-FrameParser">
impl FrameParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether any bytes seen so far contain the guest ready banner.
    /// (Banner detection ignores line framing: it can share a line with
    /// escape sequences or arrive split across chunks.)
    pub fn saw_ready_banner(&self, earlier_text: &str) -> bool {
        earlier_text.contains(READY_BANNER)
    }

    /// Consume a chunk, returning the items completed by it.
    pub fn push(&mut self, data: &[u8]) -> Vec<FrameItem> {
        let mut items = Vec::new();
        if self.done {
            return items;
        }
        self.buf.extend_from_slice(data);
        while let Some(nl) = self.buf.iter().position(|&b| b == b'\n') {
            let line_bytes: Vec<u8> = self.buf.drain(..=nl).collect();
            let raw = String::from_utf8_lossy(&line_bytes);
            let stripped = strip_ansi(&raw);

            if self.started {
                if let Some(at) = stripped.find(SENTINEL_END) {
                    // Output not ending in a newline shares its last line
                    // with the end sentinel (the guest shell's `echo` starts
                    // wherever the cursor is). Keep that partial line.
                    if at > 0 {
                        items.push(FrameItem::Line(stripped[..at].to_string()));
                    }
                    let code: i32 = stripped[at + SENTINEL_END.len()..]
                        .trim()
                        .chars()
                        .take_while(|c| c.is_ascii_digit())
                        .collect::<String>()
                        .parse()
                        .unwrap_or(0);
                    self.done = true;
                    items.push(FrameItem::Done(code));
                    return items;
                }
                items.push(FrameItem::Line(stripped));
                continue;
            }
            // Our own echoed command also contains both sentinels, so capture
            // only starts after a line that is nothing BUT the start sentinel.
            if stripped.trim() == SENTINEL_START {
                self.started = true;
            }
        }
        items
    }

    /// Text accumulated but not yet terminated by a newline (diagnostics).
    pub fn pending(&self) -> String {
        strip_ansi(&String::from_utf8_lossy(&self.buf))
    }
}

</storycode:copy>
<a id="f-eaa988f0d416-tests"></a>
### `tests`

<storycode:copy id="f-eaa988f0d416-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_base64_round_trips() {
        let script = "printf '%s' \"multi\nline $VAR 'quoted'\" | wc -c\n";
        assert_eq!(decode_script(&encode_script(script)).unwrap(), script);
    }

    #[test]
    fn exec_line_is_one_line_embedding_the_encoded_script() {
        let line = exec_line("echo hi");
        assert_eq!(line.matches('\n').count(), 1);
        assert!(line.ends_with('\n'));
        let b64 = line
            .split("echo ")
            .nth(2)
            .unwrap()
            .split(" | base64 -d")
            .next()
            .unwrap();
        assert_eq!(decode_script(b64).unwrap(), "echo hi");
    }

    #[test]
    fn sentinel_recognised_through_terminal_escapes() {
        assert_eq!(
            strip_ansi("\u{1b}[?2004l__CANOPY_S__\r\n").trim(),
            "__CANOPY_S__"
        );
        assert_eq!(strip_ansi("\u{1b}[32mok\u{1b}[0m").trim(), "ok");
    }

    #[test]
    fn echoed_command_line_does_not_start_capture_or_finish() {
        let mut p = FrameParser::new();
        // The pty echoes the command we sent; it contains both sentinels but
        // must be ignored entirely.
        let echoed = format!(
            "echo {SENTINEL_START}; echo QUJD | base64 -d | /bin/sh; echo {SENTINEL_END}$?\r\n"
        );
        assert_eq!(p.push(echoed.as_bytes()), vec![]);
        assert_eq!(p.push(format!("{SENTINEL_START}\r\n").as_bytes()), vec![]);
        assert_eq!(
            p.push(b"hello\r\n"),
            vec![FrameItem::Line("hello\n".into())]
        );
        assert_eq!(
            p.push(format!("{SENTINEL_END}0\r\n").as_bytes()),
            vec![FrameItem::Done(0)]
        );
    }

    #[test]
    fn frames_survive_arbitrary_chunk_boundaries() {
        let full =
            format!("\u{1b}[?2004l{SENTINEL_START}\r\nline one\r\nline two\r\n{SENTINEL_END}7\r\n");
        let bytes = full.as_bytes();
        // Feed one byte at a time — the cruellest chunking.
        let mut p = FrameParser::new();
        let mut items = Vec::new();
        for b in bytes {
            items.extend(p.push(std::slice::from_ref(b)));
        }
        assert_eq!(
            items,
            vec![
                FrameItem::Line("line one\n".into()),
                FrameItem::Line("line two\n".into()),
                FrameItem::Done(7),
            ]
        );
    }

    #[test]
    fn nothing_is_captured_after_done() {
        let mut p = FrameParser::new();
        p.push(format!("{SENTINEL_START}\n{SENTINEL_END}0\n").as_bytes());
        assert_eq!(p.push(b"stray shell prompt\n"), vec![]);
    }

    #[test]
    fn partial_last_line_before_end_sentinel_is_kept() {
        let mut p = FrameParser::new();
        p.push(format!("{SENTINEL_START}\r\n").as_bytes());
        assert_eq!(
            p.push(format!("no-newline{SENTINEL_END}5\r\n").as_bytes()),
            vec![FrameItem::Line("no-newline".into()), FrameItem::Done(5)]
        );
    }

    #[test]
    fn exit_code_digits_are_parsed_and_non_digits_ignored() {
        let mut p = FrameParser::new();
        p.push(format!("{SENTINEL_START}\n").as_bytes());
        let items = p.push(format!("{SENTINEL_END}42\u{1b}[0m\r\n").as_bytes());
        assert_eq!(items, vec![FrameItem::Done(42)]);
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
        assert_eq!(shell_quote("plain"), "'plain'");
    }
}
</storycode:copy>

### Assemble `crates/hickory-executor-canopy/src/frame.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor-canopy/src/frame.rs"><storycode:paste select="#f-eaa988f0d416-SENTINEL_START"/><storycode:paste select="#f-eaa988f0d416-SENTINEL_END"/><storycode:paste select="#f-eaa988f0d416-READY_BANNER"/><storycode:paste select="#f-eaa988f0d416-encode_script"/><storycode:paste select="#f-eaa988f0d416-decode_script"/><storycode:paste select="#f-eaa988f0d416-exec_line"/><storycode:paste select="#f-eaa988f0d416-shell_quote"/><storycode:paste select="#f-eaa988f0d416-strip_ansi"/><storycode:paste select="#f-eaa988f0d416-FrameItem"/><storycode:paste select="#f-eaa988f0d416-FrameParser"/><storycode:paste select="#f-eaa988f0d416-impl-FrameParser"/><storycode:paste select="#f-eaa988f0d416-tests"/></storycode:file>

<a id="output-b12473f62cbf"></a>
## `crates/hickory-executor/src/capture.rs`

Related contracts: [An Expectation Means The Same Thing On Every Platform](contract.md).

<a id="f-b12473f62cbf-normalize_captured_newlines"></a>
### `normalize_captured_newlines`

What a cell's captured output is *recorded* as.
# The rule
**Captured cell output is recorded with `\n` line endings on every
platform.** A `\r\n` pair in a cell's stdout or stderr becomes `\n` before
it reaches a transcript entry, a returned `String`, a woven document, or a
recording on disk.
# Why it is here and not in the comparison
A cell runs through `sh -c` on Unix and `cmd.exe /C` on Windows
(`LocalExecutor::shell`), and cmd's `echo` emits CRLF — there is no way to
ask it not to. `<hick:expect>` bodies, meanwhile, are text inside a `.md`
file in git, so they are LF. Without this, a document that passes on macOS
reports drift on Windows for a line-ending reason rather than a real one,
and `match="exact"` is unusable there for any output containing a newline.
Measured on a real Windows machine with the shipped binary, not inferred:
the run reports `expectation FAILED: outputs differ only in trailing
whitespace or final newline` (issue #18).
Normalising *here* rather than in `hick_literate::expect` keeps `exact`
meaning exact. The comparison is still byte-for-byte; what changed is that
the bytes it compares against are portable. The alternative — comparing
loosely — would make `exact` quietly mean "exact modulo whatever we decided
not to look at", and would leave the transcript, the woven `.md`, and the
`.hick-cache` recording still platform-specific, so a recording made on
Windows would still fail to serve a run on Linux.
# Binary output
Rewriting bytes is wrong for a cell that emits binary — and this path
cannot carry binary in the first place. [`Executor::execute`] returns
`String` and [`ExecTranscriptEntry::output`] is a `String`: captured output
is decoded with `String::from_utf8_lossy`, so any byte that is not valid
UTF-8 has already been replaced with U+FFFD before newline normalisation is
reached. Nothing that CRLF→LF would damage survives capture intact anyway.
Bytes that must stay bytes have their own route and never pass through
here: a cell writes them to a file, and they leave the container through
[`Executor::extract_volume`], which is `Vec<u8>` end to end.
The residual case is real and worth naming: valid UTF-8 text in which CRLF
is the *subject* — a cell asserting that some tool emits DOS line endings.
That claim can no longer be made about stdout directly; write the output to
a file and have the cell report the bytes (`od -c`, `xxd`), which is a
sturdier test of it regardless.
# Lone `\r`
Left alone, deliberately. cmd emits CRLF, so CRLF is the whole problem, and
a bare `\r` is load-bearing everywhere else it appears: a progress bar or a
spinner repaints one line with it, and rewriting those to `\n` would turn a
single line of woven output into hundreds. The narrowest rule that fixes
the reported problem is the one that stays.
[`Executor::execute`]: crate::Executor::execute
[`Executor::extract_volume`]: crate::Executor::extract_volume
[`ExecTranscriptEntry::output`]: crate::ExecTranscriptEntry::output

<storycode:copy id="f-b12473f62cbf-normalize_captured_newlines">
//! What a cell's captured output is *recorded* as.
//!
//! # The rule
//!
//! **Captured cell output is recorded with `\n` line endings on every
//! platform.** A `\r\n` pair in a cell's stdout or stderr becomes `\n` before
//! it reaches a transcript entry, a returned `String`, a woven document, or a
//! recording on disk.
//!
//! # Why it is here and not in the comparison
//!
//! A cell runs through `sh -c` on Unix and `cmd.exe /C` on Windows
//! (`LocalExecutor::shell`), and cmd's `echo` emits CRLF — there is no way to
//! ask it not to. `<hick:expect>` bodies, meanwhile, are text inside a `.md`
//! file in git, so they are LF. Without this, a document that passes on macOS
//! reports drift on Windows for a line-ending reason rather than a real one,
//! and `match="exact"` is unusable there for any output containing a newline.
//! Measured on a real Windows machine with the shipped binary, not inferred:
//! the run reports `expectation FAILED: outputs differ only in trailing
//! whitespace or final newline` (issue #18).
//!
//! Normalising *here* rather than in `hick_literate::expect` keeps `exact`
//! meaning exact. The comparison is still byte-for-byte; what changed is that
//! the bytes it compares against are portable. The alternative — comparing
//! loosely — would make `exact` quietly mean "exact modulo whatever we decided
//! not to look at", and would leave the transcript, the woven `.md`, and the
//! `.hick-cache` recording still platform-specific, so a recording made on
//! Windows would still fail to serve a run on Linux.
//!
//! # Binary output
//!
//! Rewriting bytes is wrong for a cell that emits binary — and this path
//! cannot carry binary in the first place. [`Executor::execute`] returns
//! `String` and [`ExecTranscriptEntry::output`] is a `String`: captured output
//! is decoded with `String::from_utf8_lossy`, so any byte that is not valid
//! UTF-8 has already been replaced with U+FFFD before newline normalisation is
//! reached. Nothing that CRLF→LF would damage survives capture intact anyway.
//!
//! Bytes that must stay bytes have their own route and never pass through
//! here: a cell writes them to a file, and they leave the container through
//! [`Executor::extract_volume`], which is `Vec<u8>` end to end.
//!
//! The residual case is real and worth naming: valid UTF-8 text in which CRLF
//! is the *subject* — a cell asserting that some tool emits DOS line endings.
//! That claim can no longer be made about stdout directly; write the output to
//! a file and have the cell report the bytes (`od -c`, `xxd`), which is a
//! sturdier test of it regardless.
//!
//! # Lone `\r`
//!
//! Left alone, deliberately. cmd emits CRLF, so CRLF is the whole problem, and
//! a bare `\r` is load-bearing everywhere else it appears: a progress bar or a
//! spinner repaints one line with it, and rewriting those to `\n` would turn a
//! single line of woven output into hundreds. The narrowest rule that fixes
//! the reported problem is the one that stays.
//!
//! [`Executor::execute`]: crate::Executor::execute
//! [`Executor::extract_volume`]: crate::Executor::extract_volume
//! [`ExecTranscriptEntry::output`]: crate::ExecTranscriptEntry::output

use std::borrow::Cow;

/// Record captured text the way every platform records it: `\r\n` → `\n`.
///
/// Borrows when there is nothing to change, which is every capture on Unix.
pub fn normalize_captured_newlines(text: &str) -> Cow<'_, str> {
    if text.contains("\r\n") {
        Cow::Owned(text.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(text)
    }
}

</storycode:copy>
<a id="f-b12473f62cbf-CapturedStream"></a>
### `CapturedStream`

[`normalize_captured_newlines`] for output that arrives in chunks.
`LocalExecutor` streams stdout and stderr in 8 KiB reads and timestamps
each chunk as its own transcript event, so a `\r\n` can straddle two of
them. Normalising each chunk on its own would miss exactly those pairs —
rarely, and only on outputs of the wrong length, which is the worst kind of
bug to have. This holds a trailing `\r` back until the next chunk says what
it was.

<storycode:copy id="f-b12473f62cbf-CapturedStream">
/// [`normalize_captured_newlines`] for output that arrives in chunks.
///
/// `LocalExecutor` streams stdout and stderr in 8 KiB reads and timestamps
/// each chunk as its own transcript event, so a `\r\n` can straddle two of
/// them. Normalising each chunk on its own would miss exactly those pairs —
/// rarely, and only on outputs of the wrong length, which is the worst kind of
/// bug to have. This holds a trailing `\r` back until the next chunk says what
/// it was.
#[derive(Debug, Default)]
pub struct CapturedStream {
    pending_cr: bool,
}

</storycode:copy>
<a id="f-b12473f62cbf-impl-CapturedStream"></a>
### `impl-CapturedStream`

<storycode:copy id="f-b12473f62cbf-impl-CapturedStream">
impl CapturedStream {
    pub fn new() -> Self {
        Self::default()
    }

    /// Decode and normalise one chunk of captured bytes.
    pub fn push(&mut self, chunk: &[u8]) -> String {
        let mut bytes = Vec::with_capacity(chunk.len() + 1);
        if std::mem::take(&mut self.pending_cr) {
            bytes.push(b'\r');
        }
        bytes.extend_from_slice(chunk);
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
            self.pending_cr = true;
        }
        normalize_captured_newlines(&String::from_utf8_lossy(&bytes)).into_owned()
    }

    /// Emit anything held back once the stream has ended — a `\r` that turned
    /// out to be the last byte of the output rather than half of a `\r\n`.
    pub fn finish(&mut self) -> String {
        if std::mem::take(&mut self.pending_cr) {
            "\r".to_string()
        } else {
            String::new()
        }
    }
}

</storycode:copy>
<a id="f-b12473f62cbf-tests"></a>
### `tests`

<storycode:copy id="f-b12473f62cbf-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crlf_becomes_lf() {
        assert_eq!(normalize_captured_newlines("one\r\ntwo\r\n"), "one\ntwo\n");
    }

    #[test]
    fn lf_only_output_is_untouched_and_not_copied() {
        assert!(matches!(
            normalize_captured_newlines("one\ntwo\n"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn a_lone_carriage_return_is_left_alone() {
        // A repainted progress line stays one line.
        assert_eq!(
            normalize_captured_newlines("10%\r50%\r100%\n"),
            "10%\r50%\r100%\n"
        );
    }

    #[test]
    fn a_crlf_split_across_chunks_is_still_normalised() {
        let mut s = CapturedStream::new();
        let first = s.push(b"one\r");
        let second = s.push(b"\ntwo\r\n");
        assert_eq!(format!("{first}{second}{}", s.finish()), "one\ntwo\n");
    }

    #[test]
    fn a_carriage_return_at_the_very_end_survives() {
        let mut s = CapturedStream::new();
        let first = s.push(b"done\r");
        assert_eq!(format!("{first}{}", s.finish()), "done\r");
    }

    #[test]
    fn an_empty_chunk_does_not_release_a_held_back_cr() {
        let mut s = CapturedStream::new();
        assert_eq!(s.push(b"x\r"), "x");
        assert_eq!(s.push(b""), "");
        assert_eq!(s.push(b"\ny"), "\ny");
        assert_eq!(s.finish(), "");
    }
}
</storycode:copy>

### Assemble `crates/hickory-executor/src/capture.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hickory-executor/src/capture.rs"><storycode:paste select="#f-b12473f62cbf-normalize_captured_newlines"/><storycode:paste select="#f-b12473f62cbf-CapturedStream"/><storycode:paste select="#f-b12473f62cbf-impl-CapturedStream"/><storycode:paste select="#f-b12473f62cbf-tests"/></storycode:file>

## Implementation review

### Implementation

- [crates/hick-literate/src/expect.rs](../../../mechanisms/crates/hick-literate/src/expect.rs.md#output-162ca50044aa)
- [crates/hickory-executor-canopy/src/frame.rs](contract.md#output-eaa988f0d416)
- [crates/hickory-executor-docker/src/lib.rs](../../../mechanisms/crates/hickory-executor-docker/src/lib.rs.md#output-772796e266eb)
- [crates/hickory-executor/src/capture.rs](contract.md#output-b12473f62cbf)
- [crates/hickory-executor/src/lib.rs](../../../mechanisms/crates/hickory-executor/src/lib.rs.md#output-87ce480a6eab)

### Tests and fixtures

- [crates/hickory-cli/tests/common/mod.rs](contract.md#output-82e96cc48a54)
- [crates/hickory-cli/tests/test_command_tests.rs](../../../mechanisms/crates/hickory-cli/tests/test_command_tests.rs.md#output-87d6e5478294)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
