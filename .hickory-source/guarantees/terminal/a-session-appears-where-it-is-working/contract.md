<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Session Appears Where It Is Working

For: Authors running interactive tools alongside documents without losing session state.

Given terminal sessions and an open folder, when the workspace tree is shown,
then each session appears as a visible child node of the directory it is working
in; clicking that node shows the session's terminal; a collapsed directory says how many sessions
are hidden inside it; and a session working outside the folder does not appear
at all.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)
- [Requirements 2](requirements/02.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/terminal/a-session-appears-where-it-is-working.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />
- [Requirement detail](requirements/02.md)
<storycode:include file="requirements/02.md" />

## Implementation

<a id="output-1b1b0d3cafa9"></a>
## `apps/web/src/lib/treeTerminals.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Session Appears Where It Is Working](contract.md).

<a id="f-1b1b0d3cafa9-s"></a>
### `s`

<storycode:copy id="f-1b1b0d3cafa9-s">
import { describe, expect, it } from "vitest";
import {
  hiddenSummary,
  rankSessions,
  urgentCount,
} from "./treeTerminals";

const s = (title: string, state: string) => ({ title, state });

</storycode:copy>
<a id="f-1b1b0d3cafa9-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-1b1b0d3cafa9-expression_statement">
// Guarantee: docs/guarantees/terminal/a-session-appears-where-it-is-working.md
describe("which terminal node comes first", () => {
  it("ranks by urgency, not by when it started", () => {
    // Sorting by start time would make the one thing worth acting on wander
    // as other sessions come and go.
    const order = rankSessions([
      s("idle-one", "idle"),
      s("build", "working"),
      s("prompt", "needs-you"),
      s("broken", "failed"),
    ]).map((x) => x.title);
    expect(order).toEqual(["prompt", "broken", "build", "idle-one"]);
  });

  it("is stable while nothing changes state", () => {
    const order = rankSessions([s("zeta", "working"), s("alpha", "working")]);
    expect(order.map((x) => x.title)).toEqual(["alpha", "zeta"]);
  });

  it("sorts a state it does not know LAST", () => {
    // A build we cannot interpret must not outrank a question somebody is
    // waiting on.
    const order = rankSessions([s("mystery", "sideways"), s("prompt", "needs-you")]);
    expect(order.map((x) => x.title)).toEqual(["prompt", "mystery"]);
  });
});

</storycode:copy>
<a id="f-1b1b0d3cafa9-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1b1b0d3cafa9-expression_statement-2">
describe("which terminal nodes need attention", () => {
  it("counts the urgent ones", () => {
    expect(urgentCount([s("a", "needs-you"), s("b", "working"), s("c", "failed")])).toBe(2);
  });
});

</storycode:copy>
<a id="f-1b1b0d3cafa9-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1b1b0d3cafa9-expression_statement-3">
describe("what a folded directory says", () => {
  it("says nothing when nothing is running", () => {
    expect(hiddenSummary(0, 0)).toBe("");
  });

  it("counts, and gets the plural right", () => {
    expect(hiddenSummary(1, 0)).toBe("1 terminal running in here");
    expect(hiddenSummary(3, 0)).toBe("3 terminals running in here");
  });

  it("says when one of them is waiting on you", () => {
    expect(hiddenSummary(3, 1)).toBe("3 terminals running in here, 1 needing you");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/treeTerminals.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/treeTerminals.test.ts"><storycode:paste select="#f-1b1b0d3cafa9-s"/><storycode:paste select="#f-1b1b0d3cafa9-expression_statement"/><storycode:paste select="#f-1b1b0d3cafa9-expression_statement-2"/><storycode:paste select="#f-1b1b0d3cafa9-expression_statement-3"/></storycode:file>

<a id="output-ca8d87824737"></a>
## `crates/hick-term/src/screen.rs`

Related contracts: [A Session Appears Where It Is Working](contract.md).

<a id="f-ca8d87824737-MAX_OSC"></a>
### `MAX_OSC`

What a session has said, kept where the session is.
Two things, for two different readers:
- a **replay buffer** of raw bytes, so closing a pane and opening it again
shows the terminal you left rather than an empty one. It is raw because
the client renders with a real terminal emulator; anything we
pretty-printed here would have to be un-pretty-printed there.
- a **screen model**, parsed headlessly, so a folded row can show the last
line a session printed without anyone opening its pane. This is the only
reason the server parses escape sequences at all.
The most an OSC payload may accumulate before we stop believing it is one.
A sequence with no terminator would otherwise grow without bound on any
program that emits a stray `ESC ]`.

<storycode:copy id="f-ca8d87824737-MAX_OSC">
//! What a session has said, kept where the session is.
//!
//! Two things, for two different readers:
//!
//! - a **replay buffer** of raw bytes, so closing a pane and opening it again
//!   shows the terminal you left rather than an empty one. It is raw because
//!   the client renders with a real terminal emulator; anything we
//!   pretty-printed here would have to be un-pretty-printed there.
//! - a **screen model**, parsed headlessly, so a folded row can show the last
//!   line a session printed without anyone opening its pane. This is the only
//!   reason the server parses escape sequences at all.

/// The most an OSC payload may accumulate before we stop believing it is one.
///
/// A sequence with no terminator would otherwise grow without bound on any
/// program that emits a stray `ESC ]`.
const MAX_OSC: usize = 4096;

</storycode:copy>
<a id="f-ca8d87824737-Osc"></a>
### `Osc`

Where the OSC scanner is between bytes, because a sequence can be split
across two reads from the PTY.

<storycode:copy id="f-ca8d87824737-Osc">
/// Where the OSC scanner is between bytes, because a sequence can be split
/// across two reads from the PTY.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Osc {
    /// Ordinary output.
    Idle,
    /// Just saw `ESC`.
    Escape,
    /// Inside `ESC ] … `, collecting the payload.
    Collecting,
    /// Inside a payload and just saw `ESC`, which may begin the `ESC \`
    /// terminator.
    CollectingEscape,
}

</storycode:copy>
<a id="f-ca8d87824737-Screen"></a>
### `Screen`

<storycode:copy id="f-ca8d87824737-Screen">
/// One session's output, bounded.
pub struct Screen {
    parser: vt100::Parser,
    /// Raw bytes as the PTY produced them, trimmed at line boundaries so a
    /// replay never begins in the middle of an escape sequence's line.
    raw: Vec<u8>,
    max_lines: usize,
    /// The directory the shell last said it was in, from OSC 7.
    ///
    /// This is how every terminal emulator learns a shell's working directory,
    /// and it is the only portable way: the alternative is reading
    /// `/proc/<pid>/cwd`, which does not exist on macOS or Windows. A shell
    /// that does not emit OSC 7 leaves this `None`, and the session falls back
    /// to the directory it was started in — stale after a `cd`, but never
    /// wrong about where it began.
    cwd: Option<String>,
    osc_state: Osc,
    osc_payload: Vec<u8>,
    /// Commands the shell has reported and nobody has collected yet.
    typed: Vec<crate::command::TypedCommand>,
}

</storycode:copy>
<a id="f-ca8d87824737-impl-Screen"></a>
### `impl-Screen`

<storycode:copy id="f-ca8d87824737-impl-Screen">
impl Screen {
    pub fn new(rows: u16, cols: u16, max_lines: usize) -> Self {
        Screen {
            parser: vt100::Parser::new(rows, cols, max_lines),
            raw: Vec::new(),
            max_lines,
            cwd: None,
            osc_state: Osc::Idle,
            osc_payload: Vec::new(),
            typed: Vec::new(),
        }
    }

    /// Take bytes from the PTY.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.scan_osc(bytes);
        self.parser.process(bytes);
        self.raw.extend_from_slice(bytes);
        self.trim();
    }

    /// The directory the shell says it is in, if it says.
    pub fn cwd(&self) -> Option<&str> {
        self.cwd.as_deref()
    }

    /// Watch the byte stream for OSC sequences, in a state machine rather than
    /// a search, because a sequence arrives split across reads as often as not.
    fn scan_osc(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            match self.osc_state {
                Osc::Idle => {
                    if byte == 0x1b {
                        self.osc_state = Osc::Escape;
                    }
                }
                Osc::Escape => {
                    self.osc_state = if byte == b']' {
                        self.osc_payload.clear();
                        Osc::Collecting
                    } else if byte == 0x1b {
                        Osc::Escape
                    } else {
                        Osc::Idle
                    };
                }
                Osc::Collecting => match byte {
                    // BEL terminates, as most shells spell it.
                    0x07 => {
                        self.finish_osc();
                    }
                    0x1b => self.osc_state = Osc::CollectingEscape,
                    _ => {
                        if self.osc_payload.len() < MAX_OSC {
                            self.osc_payload.push(byte);
                        } else {
                            // Not a sequence anybody meant. Give up on it
                            // rather than growing.
                            self.osc_state = Osc::Idle;
                            self.osc_payload.clear();
                        }
                    }
                },
                Osc::CollectingEscape => {
                    if byte == b'\\' {
                        // `ESC \` — the other spelling of the terminator.
                        self.finish_osc();
                    } else {
                        // The ESC belonged to the payload after all.
                        self.osc_payload.push(0x1b);
                        self.osc_state = Osc::Collecting;
                        if byte == 0x1b {
                            self.osc_state = Osc::CollectingEscape;
                        } else if self.osc_payload.len() < MAX_OSC {
                            self.osc_payload.push(byte);
                        }
                    }
                }
            }
        }
    }

    fn finish_osc(&mut self) {
        let payload = std::mem::take(&mut self.osc_payload);
        self.osc_state = Osc::Idle;
        if let Some(cwd) = cwd_from_osc7(&payload) {
            self.cwd = Some(cwd);
        }
        if let Some(command) = crate::command::command_from_osc633(&payload) {
            // Queued rather than acted on: this runs under the screen lock in
            // the PTY reader, and what to DO with a typed command depends on
            // whether the session is anchored to a document — which is not
            // something a screen model should know.
            self.typed.push(command);
        }
    }

    /// Take the commands the shell has reported since this was last called.
    ///
    /// Draining rather than accumulating: an unanchored session reports
    /// commands too (the hook is always installed, like OSC 7), and keeping
    /// them would be a growing record of a terminal that promised to write
    /// nothing anywhere.
    pub fn take_typed(&mut self) -> Vec<crate::command::TypedCommand> {
        std::mem::take(&mut self.typed)
    }

    /// The bytes a newly-attached client should be sent before live output.
    pub fn replay(&self) -> &[u8] {
        &self.raw
    }

    /// The whole visible screen as text, the way a terminal would show it.
    ///
    /// Not what the client renders — that is xterm.js, from the raw bytes.
    /// This is for looking at a session without attaching to it: the folded
    /// row's preview is built from it, and a test can assert what a
    /// full-screen program actually drew.
    pub fn contents(&self) -> String {
        self.parser.screen().contents()
    }

    /// The last line with anything on it — what a folded session row shows.
    ///
    /// Empty when the session has printed nothing, which the caller shows as
    /// nothing rather than as a blank line pretending to be output.
    pub fn preview(&self) -> String {
        self.parser
            .screen()
            .contents()
            .lines()
            .rev()
            .map(str::trim_end)
            .find(|line| !line.is_empty())
            .unwrap_or_default()
            .to_string()
    }

    /// Whether the program took over the whole screen (vim, htop, a coding
    /// agent's UI). Alt-screen programs are the ones a naive line-oriented
    /// buffer gets wrong, so it is worth being able to say.
    pub fn alternate_screen(&self) -> bool {
        self.parser.screen().alternate_screen()
    }

    /// Follow the pane's new size. The child is told separately, by the PTY.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        self.parser.set_size(rows, cols);
    }

    /// Drop whole leading lines until the buffer is within its bound.
    fn trim(&mut self) {
        let lines = self.raw.iter().filter(|&&b| b == b'\n').count();
        if lines <= self.max_lines {
            return;
        }
        let mut to_drop = lines - self.max_lines;
        let mut cut = 0;
        for (i, &byte) in self.raw.iter().enumerate() {
            if byte == b'\n' {
                to_drop -= 1;
                if to_drop == 0 {
                    cut = i + 1;
                    break;
                }
            }
        }
        self.raw.drain(..cut);
    }
}

</storycode:copy>
<a id="f-ca8d87824737-tests"></a>
### `tests`

<storycode:copy id="f-ca8d87824737-tests">
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reattaching_client_replays_what_the_session_said() {
        let mut screen = Screen::new(24, 80, 100);
        screen.feed(b"hello\r\n");
        screen.feed(b"world\r\n");
        assert_eq!(screen.replay(), b"hello\r\nworld\r\n");
    }

    #[test]
    fn the_preview_is_the_last_line_with_anything_on_it() {
        let mut screen = Screen::new(24, 80, 100);
        screen.feed(b"building\r\ndone\r\n");
        assert_eq!(screen.preview(), "done");
    }

    #[test]
    fn a_session_that_has_printed_nothing_previews_as_nothing() {
        assert_eq!(Screen::new(24, 80, 100).preview(), "");
    }

    #[test]
    fn scrollback_is_bounded_and_cut_at_line_boundaries() {
        let mut screen = Screen::new(24, 80, 3);
        for i in 0..10 {
            screen.feed(format!("line {i}\n").as_bytes());
        }
        let replay = String::from_utf8(screen.replay().to_vec()).unwrap();
        assert_eq!(replay, "line 7\nline 8\nline 9\n");
    }

    #[test]
    fn escape_sequences_shape_the_preview_rather_than_appearing_in_it() {
        let mut screen = Screen::new(24, 80, 100);
        // Bold, some text, reset — the preview is what a person would read.
        screen.feed(b"\x1b[1mcompiling\x1b[0m\r\n");
        assert_eq!(screen.preview(), "compiling");
    }
}

</storycode:copy>
<a id="f-ca8d87824737-cwd_from_osc7"></a>
### `cwd_from_osc7`

The directory named by an OSC 7 payload, if that is what this is.
The payload is `7;file://HOST/PATH`. The host is whatever the shell thinks
the machine is called and is deliberately ignored: there is no remote
session here, and a hostname that disagrees with `uname` (containers,
`hostname` changes, `localhost`) is not a reason to throw away the path.

<storycode:copy id="f-ca8d87824737-cwd_from_osc7">
/// The directory named by an OSC 7 payload, if that is what this is.
///
/// The payload is `7;file://HOST/PATH`. The host is whatever the shell thinks
/// the machine is called and is deliberately ignored: there is no remote
/// session here, and a hostname that disagrees with `uname` (containers,
/// `hostname` changes, `localhost`) is not a reason to throw away the path.
fn cwd_from_osc7(payload: &[u8]) -> Option<String> {
    let text = std::str::from_utf8(payload).ok()?;
    let rest = text.strip_prefix("7;")?;
    let after_scheme = rest.strip_prefix("file://")?;
    // Everything from the first `/` on: `hostname/path` or, when the shell
    // omits the host, `/path` already.
    let path = match after_scheme.find('/') {
        Some(slash) => &after_scheme[slash..],
        None => return None,
    };
    let decoded = percent_decode(path);
    (!decoded.is_empty()).then_some(decoded)
}

</storycode:copy>
<a id="f-ca8d87824737-percent_decode"></a>
### `percent_decode`

Percent-decoding, for the spaces and accents in real directory names.
A malformed escape is left as written rather than dropped: this is
somebody's path, and mangling it silently is worse than carrying a `%`.

<storycode:copy id="f-ca8d87824737-percent_decode">
/// Percent-decoding, for the spaces and accents in real directory names.
///
/// A malformed escape is left as written rather than dropped: this is
/// somebody's path, and mangling it silently is worse than carrying a `%`.
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

</storycode:copy>
<a id="f-ca8d87824737-osc_tests"></a>
### `osc_tests`

<storycode:copy id="f-ca8d87824737-osc_tests">
#[cfg(test)]
mod osc_tests {
    use super::*;

    fn fed(chunks: &[&[u8]]) -> Screen {
        let mut screen = Screen::new(24, 80, 100);
        for chunk in chunks {
            screen.feed(chunk);
        }
        screen
    }

    #[test]
    fn a_shell_reporting_its_directory_is_believed() {
        let screen = fed(&[b"\x1b]7;file://host/home/nate/notes\x07$ "]);
        assert_eq!(screen.cwd(), Some("/home/nate/notes"));
    }

    #[test]
    fn the_string_terminator_spelling_works_too() {
        // `ESC \` rather than BEL. Both are in the wild.
        let screen = fed(&[b"\x1b]7;file://host/tmp/x\x1b\\"]);
        assert_eq!(screen.cwd(), Some("/tmp/x"));
    }

    #[test]
    fn a_sequence_split_across_reads_still_arrives() {
        // The PTY hands over whatever happened to be in the buffer, so this is
        // the common case rather than an edge one.
        let screen = fed(&[b"\x1b]7;file://ho", b"st/home/na", b"te\x07"]);
        assert_eq!(screen.cwd(), Some("/home/nate"));
    }

    #[test]
    fn a_later_cd_replaces_an_earlier_one() {
        let screen = fed(&[
            b"\x1b]7;file://host/first\x07",
            b"some output\r\n",
            b"\x1b]7;file://host/second\x07",
        ]);
        assert_eq!(screen.cwd(), Some("/second"));
    }

    #[test]
    fn spaces_and_accents_survive() {
        let screen = fed(&[b"\x1b]7;file://host/home/nate/My%20Notes/caf%C3%A9\x07"]);
        assert_eq!(screen.cwd(), Some("/home/nate/My Notes/café"));
    }

    #[test]
    fn a_shell_that_says_nothing_leaves_it_unknown() {
        // Falling back to the directory the session started in is the caller's
        // job; the screen does not invent one.
        let screen = fed(&[b"$ ls\r\nfile.txt\r\n"]);
        assert_eq!(screen.cwd(), None);
    }

    #[test]
    fn other_osc_sequences_are_ignored() {
        // OSC 0 and 2 set the window title, and are far more common than 7.
        let screen = fed(&[b"\x1b]0;vim notes.md\x07\x1b]2;bash\x07"]);
        assert_eq!(screen.cwd(), None);
    }

    #[test]
    fn an_unterminated_sequence_cannot_grow_without_bound() {
        let mut screen = Screen::new(24, 80, 100);
        screen.feed(b"\x1b]7;file://host/");
        for _ in 0..200 {
            screen.feed(&[b'x'; 64]);
        }
        assert!(screen.osc_payload.len() <= MAX_OSC);
        assert_eq!(screen.cwd(), None);
    }

    #[test]
    fn output_still_reaches_the_screen_around_a_sequence() {
        // The scanner must observe the stream, never consume it.
        let screen = fed(&[b"before\r\n\x1b]7;file://host/tmp\x07after\r\n"]);
        assert!(screen.contents().contains("before"));
        assert!(screen.contents().contains("after"));
        assert_eq!(screen.cwd(), Some("/tmp"));
    }
}
</storycode:copy>

### Assemble `crates/hick-term/src/screen.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/src/screen.rs"><storycode:paste select="#f-ca8d87824737-MAX_OSC"/><storycode:paste select="#f-ca8d87824737-Osc"/><storycode:paste select="#f-ca8d87824737-Screen"/><storycode:paste select="#f-ca8d87824737-impl-Screen"/><storycode:paste select="#f-ca8d87824737-tests"/><storycode:paste select="#f-ca8d87824737-cwd_from_osc7"/><storycode:paste select="#f-ca8d87824737-percent_decode"/><storycode:paste select="#f-ca8d87824737-osc_tests"/></storycode:file>

<a id="output-cfc07fe1b62f"></a>
## `crates/hick-term/tests/a_real_shell_reports_its_directory.rs`

Related contracts: [A Session Appears Where It Is Working](contract.md).

<a id="f-cfc07fe1b62f-shell_at"></a>
### `shell_at`

A real shell, in a real PTY, and what it does or does not say about `cd`.
Protects docs/guarantees/terminal/a-session-appears-where-it-is-working.md
Every other OSC 7 test feeds bytes straight into the scanner, which proves
the parser and nothing about the shells people have. That gap is what this
closes, and the answer it found is not the comfortable one: **macOS's default
zsh emits OSC 7 only when `TERM_PROGRAM=Apple_Terminal`**, because the hook
lives in `/etc/zshrc_Apple_Terminal`, sourced from `/etc/zshrc` by that name.
A terminal that answers truthfully about what it is gets nothing.
So there are two tests, and both matter:
- the shell that *does* emit it is followed correctly, end to end, through a
PTY rather than through a byte fixture; and
- the shell that does not is reported as not-live rather than guessed at,
which is the property a reader of the folder tree depends on.
**Windows is entirely the second case, by design.** `Integration::install`
recognises zsh and bash and nothing else, so `cmd.exe` is handed no hook,
never emits OSC 7, and every session there reports the directory it was
told to start in with `cwd_is_live` false. The tests that drive a zsh hook
skip there — loudly, naming the shell they wanted — and the two that assert
the honest fallback run on both platforms, because that fallback is the
whole of the Windows behaviour rather than an edge of it.

<storycode:copy id="f-cfc07fe1b62f-shell_at">
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
//!
//! **Windows is entirely the second case, by design.** `Integration::install`
//! recognises zsh and bash and nothing else, so `cmd.exe` is handed no hook,
//! never emits OSC 7, and every session there reports the directory it was
//! told to start in with `cwd_is_live` false. The tests that drive a zsh hook
//! skip there — loudly, naming the shell they wanted — and the two that assert
//! the honest fallback run on both platforms, because that fallback is the
//! whole of the Windows behaviour rather than an edge of it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use hick_term::session::Session;
use hick_term::{SessionSpec, TermConfig, Terminals};

fn shell_at(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

</storycode:copy>
<a id="f-cfc07fe1b62f-zsh_path"></a>
### `zsh_path`

<storycode:copy id="f-cfc07fe1b62f-zsh_path">
/// Wherever zsh lives on this machine.
fn zsh_path() -> Option<&'static str> {
    ["/bin/zsh", "/usr/bin/zsh", "/opt/homebrew/bin/zsh"]
        .into_iter()
        .find(|path| shell_at(path))
}

</storycode:copy>
<a id="f-cfc07fe1b62f-silent_shell"></a>
### `silent_shell`

A shell that emits no OSC 7 and is not going to be taught to.
`/bin/sh` qualifies because `shell_integration` deliberately refuses to
guess what a shell invoked as `sh` reads at startup. `cmd.exe` qualifies
because there is no hook for it at all — which makes it the Windows case
this test is about, not a stand-in for one.

<storycode:copy id="f-cfc07fe1b62f-silent_shell">
/// A shell that emits no OSC 7 and is not going to be taught to.
///
/// `/bin/sh` qualifies because `shell_integration` deliberately refuses to
/// guess what a shell invoked as `sh` reads at startup. `cmd.exe` qualifies
/// because there is no hook for it at all — which makes it the Windows case
/// this test is about, not a stand-in for one.
fn silent_shell() -> Option<&'static str> {
    if cfg!(windows) {
        Some("cmd.exe")
    } else {
        shell_at("/bin/sh").then_some("/bin/sh")
    }
}

</storycode:copy>
<a id="f-cfc07fe1b62f-cd_elsewhere"></a>
### `cd_elsewhere`

`cd` to somewhere that exists on this platform, entered as a user would.
CR, not LF: Enter is a carriage return, a Windows console reads a line when
it sees one, and a Unix tty translates it to the newline the shell wants.

<storycode:copy id="f-cfc07fe1b62f-cd_elsewhere">
/// `cd` to somewhere that exists on this platform, entered as a user would.
///
/// CR, not LF: Enter is a carriage return, a Windows console reads a line when
/// it sees one, and a Unix tty translates it to the newline the shell wants.
fn cd_elsewhere() -> &'static [u8] {
    if cfg!(windows) {
        b"cd C:\\Windows\r"
    } else {
        b"cd /usr/share\r"
    }
}

</storycode:copy>
<a id="f-cfc07fe1b62f-open"></a>
### `open`

<storycode:copy id="f-cfc07fe1b62f-open">
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

</storycode:copy>
<a id="f-cfc07fe1b62f-wait_for"></a>
### `wait_for`

Poll the summary until `done`, or give up. A shell prints its prompt when it
is ready, not when we ask.

<storycode:copy id="f-cfc07fe1b62f-wait_for">
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

</storycode:copy>
<a id="f-cfc07fe1b62f-a_shell_that_emits_osc_7_moves_the_session"></a>
### `a_shell_that_emits_osc_7_moves_the_session`

A shell that emits OSC 7 is followed. Driven through a PTY, so this is the
shell speaking and not a fixture.
`zsh -ic` with the hook forced on is how the sequence is provoked without
depending on `TERM_PROGRAM` reaching the child: the point being tested is the
scanner's behaviour against a real shell's real bytes.

<storycode:copy id="f-cfc07fe1b62f-a_shell_that_emits_osc_7_moves_the_session">
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

</storycode:copy>
<a id="f-cfc07fe1b62f-a_silent_shell_stays_where_it_started_and_admits_it"></a>
### `a_silent_shell_stays_where_it_started_and_admits_it`

A shell that says nothing leaves the session where it started, and says so.
This is the macOS default, not an edge case: with a truthful `TERM_PROGRAM`,
`/etc/zshrc` never sources the hook, so the folder tree shows the
started-in directory for the life of the session. `cwd_is_live` being false
is the whole reason a reader is not misled by it.

<storycode:copy id="f-cfc07fe1b62f-a_silent_shell_stays_where_it_started_and_admits_it">
/// A shell that says nothing leaves the session where it started, and says so.
///
/// This is the macOS default, not an edge case: with a truthful `TERM_PROGRAM`,
/// `/etc/zshrc` never sources the hook, so the folder tree shows the
/// started-in directory for the life of the session. `cwd_is_live` being false
/// is the whole reason a reader is not misled by it.
#[test]
fn a_silent_shell_stays_where_it_started_and_admits_it() {
    let Some(shell) = silent_shell() else {
        eprintln!("no /bin/sh — skipped");
        return;
    };
    let start = tempfile::tempdir().unwrap();
    // Not canonicalised: the fallback is the directory the session was *told* to
    // start in, verbatim, which is the honest thing for it to report.
    let started_in = start.path().to_path_buf();

    let terminals = Terminals::new(TermConfig::default());
    let argv: Vec<&str> = if cfg!(windows) {
        vec![shell]
    } else {
        vec![shell, "-i"]
    };
    let session = open(&terminals, start.path(), &argv);

    // Somewhere real, and definitely not where it started.
    session.write(cd_elsewhere()).unwrap();
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

</storycode:copy>
<a id="f-cfc07fe1b62f-an_integrated_shell_follows_a_cd_with_no_help"></a>
### `an_integrated_shell_follows_a_cd_with_no_help`

The point of the whole thing: a plain shell, started the way the app starts
it, follows a `cd` with nobody having typed an escape sequence.
This is the test that would have failed before shell integration existed —
on macOS it is the *only* way this can pass, because no shell there emits
OSC 7 on its own.

<storycode:copy id="f-cfc07fe1b62f-an_integrated_shell_follows_a_cd_with_no_help">
/// The point of the whole thing: a plain shell, started the way the app starts
/// it, follows a `cd` with nobody having typed an escape sequence.
///
/// This is the test that would have failed before shell integration existed —
/// on macOS it is the *only* way this can pass, because no shell there emits
/// OSC 7 on its own.
#[test]
fn an_integrated_shell_follows_a_cd_with_no_help() {
    let Some(zsh) = zsh_path() else {
        eprintln!("no zsh on this machine — skipped");
        return;
    };

    let start = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let target = elsewhere.path().canonicalize().unwrap();

    // The shell is started with no argv of its own, which is what a session
    // opened from the app looks like, and what turns integration on.
    let config = TermConfig {
        shell: zsh.to_string(),
        ..TermConfig::default()
    };
    assert!(
        config.integrate_shell,
        "integration is meant to be the default"
    );
    let terminals = Terminals::new(config);
    let session = terminals
        .open(SessionSpec {
            title: "shell".to_string(),
            cwd: start.path().to_path_buf(),
            argv: Vec::new(),
            monitor: false,
        })
        .expect("the shell starts");

    session
        .write(format!("cd {}\n", target.display()).as_bytes())
        .unwrap();

    // Waiting for `cwd_is_live` alone races the shell: it reports at its FIRST
    // prompt, which is before the `cd` has been read, so the first live value
    // is the directory it started in. The thing being tested is that it follows
    // — so wait for where it should have followed to.
    let after = wait_for(&session, Duration::from_secs(20), |s| {
        s.cwd_is_live && std::path::Path::new(&s.cwd) == target.as_path()
    });
    assert!(
        after.cwd_is_live,
        "an integrated shell did not report its directory at all: {after:?}"
    );
    assert_eq!(
        std::path::Path::new(&after.cwd),
        target.as_path(),
        "reported a directory, but never followed the cd"
    );

    session.kill().ok();
}

</storycode:copy>
<a id="f-cfc07fe1b62f-integration_does_not_replace_the_users_own_zshrc"></a>
### `integration_does_not_replace_the_users_own_zshrc`

The user's own configuration still runs, which is the part that makes this
acceptable at all. A generated `.zshrc` that quietly replaced theirs would
be a worse bug than the one it fixes.

<storycode:copy id="f-cfc07fe1b62f-integration_does_not_replace_the_users_own_zshrc">
/// The user's own configuration still runs, which is the part that makes this
/// acceptable at all. A generated `.zshrc` that quietly replaced theirs would
/// be a worse bug than the one it fixes.
#[test]
fn integration_does_not_replace_the_users_own_zshrc() {
    let Some(zsh) = zsh_path() else {
        eprintln!("no zsh on this machine — skipped");
        return;
    };

    // A home of their own, with a `.zshrc` that leaves a mark.
    let home = tempfile::tempdir().unwrap();
    std::fs::write(
        home.path().join(".zshrc"),
        "export HICKORY_USER_RC_RAN=indeed\n",
    )
    .unwrap();

    let work = tempfile::tempdir().unwrap();
    let terminals = Terminals::new(TermConfig {
        shell: zsh.to_string(),
        ..TermConfig::default()
    });
    // `ZDOTDIR` is what the integration forwards to, and setting it here is
    // how this test stands in for a user who has one.
    let previous = std::env::var("ZDOTDIR").ok();
    // SAFETY: single-threaded test setup, restored below.
    unsafe { std::env::set_var("ZDOTDIR", home.path()) };
    let session = terminals
        .open(SessionSpec {
            title: "shell".to_string(),
            cwd: work.path().to_path_buf(),
            argv: Vec::new(),
            monitor: false,
        })
        .expect("the shell starts");
    match previous {
        Some(value) => unsafe { std::env::set_var("ZDOTDIR", value) },
        None => unsafe { std::env::remove_var("ZDOTDIR") },
    }

    session
        .write(b"printf 'RC=%s\\n' \"$HICKORY_USER_RC_RAN\"\n")
        .unwrap();

    let screen = {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let text = session.screen_text();
            if text.contains("RC=indeed") || Instant::now() > deadline {
                break text;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    };
    assert!(
        screen.contains("RC=indeed"),
        "the user's own .zshrc did not run:\n{screen}"
    );

    session.kill().ok();
}

</storycode:copy>
<a id="f-cfc07fe1b62f-an_integrated_shell_reports_a_directory_that_needs_escaping"></a>
### `an_integrated_shell_reports_a_directory_that_needs_escaping`

Every shell we integrate with, into a directory whose name needs escaping.
The percent-encoding is where these two hooks differ, and the difference was
a real bug: `printf '%d' "'é"` in bash reports -61 rather than 195, so the
unmasked form emitted `%FFFFFFFFFFFFFFC3` and a directory with an accent in
its name arrived as nonsense. zsh's printf does not sign-extend. Only a real
shell in a real directory catches that, which is why this is here and not a
unit test.

<storycode:copy id="f-cfc07fe1b62f-an_integrated_shell_reports_a_directory_that_needs_escaping">
/// Every shell we integrate with, into a directory whose name needs escaping.
///
/// The percent-encoding is where these two hooks differ, and the difference was
/// a real bug: `printf '%d' "'é"` in bash reports -61 rather than 195, so the
/// unmasked form emitted `%FFFFFFFFFFFFFFC3` and a directory with an accent in
/// its name arrived as nonsense. zsh's printf does not sign-extend. Only a real
/// shell in a real directory catches that, which is why this is here and not a
/// unit test.
#[test]
fn an_integrated_shell_reports_a_directory_that_needs_escaping() {
    let shells: Vec<&str> = ["/bin/zsh", "/usr/bin/zsh", "/bin/bash", "/usr/bin/bash"]
        .into_iter()
        .filter(|path| shell_at(path))
        .collect();
    if shells.is_empty() {
        // On Unix an empty list is a broken machine and must fail. On Windows
        // it is the truth: there is no shell here we integrate with, so there
        // is no percent-encoding to get wrong. The Windows half of the same
        // question — an awkward directory survives the spawn and is reported
        // verbatim — is asserted by the test below, which is the strongest
        // claim available on a platform with no hook.
        if cfg!(windows) {
            eprintln!("SKIPPED: nothing on this platform emits OSC 7 to get wrong");
            return;
        }
        panic!("no zsh or bash on this machine at all");
    }

    for shell in shells {
        let start = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();
        // A space and a non-ASCII character, which is the whole point.
        let awkward = elsewhere.path().join("dir with café");
        std::fs::create_dir(&awkward).unwrap();
        let target = awkward.canonicalize().unwrap();

        let terminals = Terminals::new(TermConfig {
            shell: shell.to_string(),
            ..TermConfig::default()
        });
        let session = terminals
            .open(SessionSpec {
                title: "shell".to_string(),
                cwd: start.path().to_path_buf(),
                argv: Vec::new(),
                monitor: false,
            })
            .expect("the shell starts");

        session
            .write(format!("cd '{}'\n", target.display()).as_bytes())
            .unwrap();

        let after = wait_for(&session, Duration::from_secs(20), |s| {
            s.cwd_is_live && std::path::Path::new(&s.cwd) == target.as_path()
        });
        assert_eq!(
            std::path::Path::new(&after.cwd),
            target.as_path(),
            "{shell} did not report a directory with a space and an accent \
             correctly (live={}): {after:?}",
            after.cwd_is_live
        );
        session.kill().ok();
    }
}

</storycode:copy>
<a id="f-cfc07fe1b62f-a_session_with_no_hook_reports_an_awkward_directory_verbatim_and_not_live"></a>
### `a_session_with_no_hook_reports_an_awkward_directory_verbatim_and_not_live`

Windows: an awkward directory survives the spawn, and the session says
plainly that it is not following the shell.
There is no OSC 7 hook for `cmd.exe`, so the reported directory is the one
the session was told to start in for the whole of its life. That makes two
things worth asserting and nothing else: the path arrives intact through
the ConPTY spawn even with a space and an accent in it, and `cwd_is_live`
is false rather than a guess. A reader of the folder tree is entitled to
know which of those two it is looking at.

<storycode:copy id="f-cfc07fe1b62f-a_session_with_no_hook_reports_an_awkward_directory_verbatim_and_not_live">
/// Windows: an awkward directory survives the spawn, and the session says
/// plainly that it is not following the shell.
///
/// There is no OSC 7 hook for `cmd.exe`, so the reported directory is the one
/// the session was told to start in for the whole of its life. That makes two
/// things worth asserting and nothing else: the path arrives intact through
/// the ConPTY spawn even with a space and an accent in it, and `cwd_is_live`
/// is false rather than a guess. A reader of the folder tree is entitled to
/// know which of those two it is looking at.
#[cfg(windows)]
#[test]
fn a_session_with_no_hook_reports_an_awkward_directory_verbatim_and_not_live() {
    let elsewhere = tempfile::tempdir().unwrap();
    let awkward = elsewhere.path().join("dir with café");
    std::fs::create_dir(&awkward).unwrap();

    let terminals = Terminals::new(TermConfig::default());
    let session = terminals
        .open(SessionSpec {
            title: "shell".to_string(),
            cwd: awkward.clone(),
            argv: Vec::new(),
            monitor: false,
        })
        .expect("the shell starts");

    // Long enough that "it never came" is the finding, not "we did not wait".
    let summary = wait_for(&session, Duration::from_secs(5), |s| s.cwd_is_live);
    assert!(
        !summary.cwd_is_live,
        "nothing here emits OSC 7, so no session may be reported as live: {summary:?}"
    );
    assert_eq!(
        std::path::Path::new(&summary.cwd),
        awkward.as_path(),
        "the directory the session was started in came back changed"
    );
    session.kill().ok();
}

</storycode:copy>
<a id="f-cfc07fe1b62f-integration_can_be_turned_off"></a>
### `integration_can_be_turned_off`

<storycode:copy id="f-cfc07fe1b62f-integration_can_be_turned_off">
/// The opt-out actually opts out.
#[test]
fn integration_can_be_turned_off() {
    let Some(zsh) = zsh_path() else {
        eprintln!("no zsh on this machine — skipped");
        return;
    };
    let start = tempfile::tempdir().unwrap();
    let terminals = Terminals::new(TermConfig {
        shell: zsh.to_string(),
        integrate_shell: false,
        ..TermConfig::default()
    });
    let session = terminals
        .open(SessionSpec {
            title: "shell".to_string(),
            cwd: start.path().to_path_buf(),
            argv: Vec::new(),
            monitor: false,
        })
        .expect("the shell starts");

    session.write(cd_elsewhere()).unwrap();
    let summary = wait_for(&session, Duration::from_secs(3), |s| s.cwd_is_live);
    assert!(
        !summary.cwd_is_live,
        "integration was off and the shell still reported: {summary:?}"
    );
    assert_eq!(
        std::path::Path::new(&summary.cwd),
        start.path(),
        "the fallback is still the directory it started in"
    );
    session.kill().ok();
}
</storycode:copy>

### Assemble `crates/hick-term/tests/a_real_shell_reports_its_directory.rs`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="crates/hick-term/tests/a_real_shell_reports_its_directory.rs"><storycode:paste select="#f-cfc07fe1b62f-shell_at"/><storycode:paste select="#f-cfc07fe1b62f-zsh_path"/><storycode:paste select="#f-cfc07fe1b62f-silent_shell"/><storycode:paste select="#f-cfc07fe1b62f-cd_elsewhere"/><storycode:paste select="#f-cfc07fe1b62f-open"/><storycode:paste select="#f-cfc07fe1b62f-wait_for"/><storycode:paste select="#f-cfc07fe1b62f-a_shell_that_emits_osc_7_moves_the_session"/><storycode:paste select="#f-cfc07fe1b62f-a_silent_shell_stays_where_it_started_and_admits_it"/><storycode:paste select="#f-cfc07fe1b62f-an_integrated_shell_follows_a_cd_with_no_help"/><storycode:paste select="#f-cfc07fe1b62f-integration_does_not_replace_the_users_own_zshrc"/><storycode:paste select="#f-cfc07fe1b62f-an_integrated_shell_reports_a_directory_that_needs_escaping"/><storycode:paste select="#f-cfc07fe1b62f-a_session_with_no_hook_reports_an_awkward_directory_verbatim_and_not_live"/><storycode:paste select="#f-cfc07fe1b62f-integration_can_be_turned_off"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/shell/FolderTreePane.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-9c0af3061e25)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [crates/hick-term/src/screen.rs](contract.md#output-ca8d87824737)
- [crates/hick-term/src/session.rs](../../../mechanisms/crates/hick-term/src/session.rs.md#output-5705d2b8acc0)
- [crates/hick-term/src/shell_integration.rs](../../../mechanisms/crates/hick-term/src/shell_integration.rs.md#output-3465af886d7a)

### Tests and fixtures

- [apps/web/src/lib/treeTerminals.test.ts](contract.md#output-1b1b0d3cafa9)
- [apps/web/src/shell/FolderTreePane.test.tsx](../../../mechanisms/apps/web/src/shell/FolderTreePane.tsx.md#output-508e32e299fd)
- [crates/hick-term/tests/a_real_shell_reports_its_directory.rs](contract.md#output-cfc07fe1b62f)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
