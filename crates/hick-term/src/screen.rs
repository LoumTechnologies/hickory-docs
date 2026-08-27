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
