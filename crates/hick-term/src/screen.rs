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

/// One session's output, bounded.
pub struct Screen {
    parser: vt100::Parser,
    /// Raw bytes as the PTY produced them, trimmed at line boundaries so a
    /// replay never begins in the middle of an escape sequence's line.
    raw: Vec<u8>,
    max_lines: usize,
}

impl Screen {
    pub fn new(rows: u16, cols: u16, max_lines: usize) -> Self {
        Screen {
            parser: vt100::Parser::new(rows, cols, max_lines),
            raw: Vec::new(),
            max_lines,
        }
    }

    /// Take bytes from the PTY.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.process(bytes);
        self.raw.extend_from_slice(bytes);
        self.trim();
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
