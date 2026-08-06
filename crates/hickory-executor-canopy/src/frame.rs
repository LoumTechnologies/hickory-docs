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
/// Prefix of the line that closes captured output; the exit code follows.
pub const SENTINEL_END: &str = "__CANOPY_E__";
/// Printed by the guest agent when a connection is handed to a shell.
pub const READY_BANNER: &str = "canopy-guest ready";

/// Base64 for scripts on the wire (standard alphabet, padded — must match
/// what the guest's `base64 -d` accepts).
pub fn encode_script(script: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(script)
}

/// Decode the base64 produced by [`encode_script`] (used by tests and the
/// in-crate mock guest).
pub fn decode_script(encoded: &str) -> anyhow::Result<String> {
    let bytes = base64::engine::general_purpose::STANDARD.decode(encoded.trim())?;
    Ok(String::from_utf8(bytes)?)
}

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

/// Quote `s` as a single shell word (single quotes, `'` → `'\''`).
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Remove ANSI escape sequences and carriage returns.
///
/// Everything talking to the guest sees a pty, so anything that colourises
/// or manages a prompt when attached to a terminal does so here.
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
