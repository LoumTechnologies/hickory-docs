//! What the shell says it is about to run.
//!
//! A terminal that writes the document has to know the **typed line**, and
//! the bytes on their way to the PTY are not it: readline editing, history
//! recall and tab completion all mean the keystrokes and the command differ.
//! So the shell is asked, through the same generated-startup-file mechanism
//! that already reports the working directory — see `shell_integration`, and
//! `docs/specs/freeform/a-terminal-that-writes-the-document.md` for the two
//! mechanisms that were measured and rejected first.
//!
//! The wire format is one OSC sequence:
//!
//! ```text
//! ESC ] 633 ; hickory-cmd ; <number> ; <percent-encoded line> BEL
//! ```
//!
//! `<number>` is the shell's own history counter, and it is the load-bearing
//! half. A shell can be configured to keep a line out of its history —
//! `HISTCONTROL=ignorespace` and `ignoredups` are common defaults — and bash
//! then reports the PREVIOUS line, because `PS0` runs in a subshell and
//! cannot remember what it last sent. A number that did not move therefore
//! means *this input was not recorded*, which is a suspension rather than a
//! command, and never a reason to write the previous line down twice.

/// One line the shell reported, before it ran it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedCommand {
    /// The shell's history counter at the moment it was reported.
    pub number: u64,
    /// The line as the shell assembled it, decoded.
    pub text: String,
}

/// The command named by an OSC 633 payload, if that is what this is.
pub fn command_from_osc633(payload: &[u8]) -> Option<TypedCommand> {
    let text = std::str::from_utf8(payload).ok()?;
    let rest = text.strip_prefix("633;hickory-cmd;")?;
    let (number, encoded) = rest.split_once(';')?;
    Some(TypedCommand {
        number: number.parse().ok()?,
        text: percent_decode(encoded),
    })
}

/// Percent-decoding, byte for byte.
///
/// The same encoding the working-directory hook uses, chosen so a command
/// carrying newlines, quotes or an accent survives an OSC payload without a
/// second escaping scheme to get wrong. A malformed escape is left as
/// written: this is somebody's command, and mangling it silently is worse
/// than carrying a `%`.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok();
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_command_round_trips() {
        let parsed = command_from_osc633(b"633;hickory-cmd;42;ls%20-la").unwrap();
        assert_eq!(parsed.number, 42);
        assert_eq!(parsed.text, "ls -la");
    }

    #[test]
    fn a_multi_line_command_keeps_its_newlines() {
        // A heredoc is one command with real newlines in it, and the encoding
        // exists so that survives an OSC payload rather than truncating it.
        let parsed =
            command_from_osc633(b"633;hickory-cmd;7;cat%20%3C%3C%27EOT%27%0Ahi%0AEOT").unwrap();
        assert_eq!(parsed.text, "cat <<'EOT'\nhi\nEOT");
    }

    #[test]
    fn quotes_and_accents_survive() {
        let parsed =
            command_from_osc633("633;hickory-cmd;1;echo%20%22caf%C3%A9%22".as_bytes()).unwrap();
        assert_eq!(parsed.text, "echo \"café\"");
    }

    #[test]
    fn anything_that_is_not_ours_is_not_a_command() {
        // OSC 7 is the working directory and shares the scanner.
        assert!(command_from_osc633(b"7;file:///home/u").is_none());
        assert!(command_from_osc633(b"633;something-else;1;x").is_none());
        // No number is not a command: without it there is no way to tell a
        // line the shell recorded from one it declined to.
        assert!(command_from_osc633(b"633;hickory-cmd;ls").is_none());
        assert!(command_from_osc633(b"633;hickory-cmd;not-a-number;ls").is_none());
    }

    #[test]
    fn an_empty_command_is_still_a_report() {
        // The shell said something ran and it decoded to nothing. Deciding
        // what that means belongs to the anchor, not to the parser.
        let parsed = command_from_osc633(b"633;hickory-cmd;9;").unwrap();
        assert_eq!(parsed.text, "");
        assert_eq!(parsed.number, 9);
    }
}
