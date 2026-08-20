# A Full-Screen Program Gets A Real Terminal

Given a session running a program that takes over the screen — an editor, a
process monitor, a pager, another vendor's coding agent — when it runs in one
of these terminals, then it gets everything it needs to draw itself:

- a PTY, so it enters raw mode and reads single keypresses rather than lines;
- `TERM=xterm-256color`, which is what the client actually renders with;
- a size it can ask for, updated when the pane changes (`POST
  /api/terminals/:id/resize` resizes the PTY **and** the server's screen
  model, so the two never disagree about where the last line is);
- the alternate screen, entered and left cleanly.

The bytes are passed through untouched in both directions. Nothing here
rewrites, filters, or line-buffers what a program draws — the client's
emulator and the server's screen model parse the same stream, which is why a
peek at a session nobody is watching (the folded row's preview) shows the same
thing its pane would.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified against real programs
- Evidence: `hick_term::session::Session::spawn` opens a real PTY through
  `portable-pty` and sets `TERM`; `resize` drives both the PTY and
  `Screen::resize`. The server keeps a `vt100` model of the same byte stream
  the client receives (`hick_term::screen::Screen`), so asserting on that
  model is asserting on what an emulator would draw from those bytes.
- Test coverage: `crates/hick-term/tests/full_screen_apps.rs` runs the real
  programs, skipping any that are not installed — and *saying so*, which it
  did not until 2026-08-20. The probe was `sh -c "command -v x"`, which does
  not spawn at all on a machine without `sh`; `unwrap_or(false)` read that as
  "not installed", so on Windows all four tests returned before their first
  assertion and printed nothing. That is a green suite covering nothing. The
  probe now walks `PATH` directly (honouring `PATHEXT`) and every skip names
  the program it wanted:
  - **vim** draws the file, stays on the alternate screen, redraws after a
    resize, and restores the normal screen on `:q!`;
  - **htop** paints its meters full-screen;
  - **less** pages a 200-line file and answers `G` — a key only a program in
    raw mode on a real PTY receives;
  - **claude** (Claude Code) prints its help into the scrollback, with the
    tail on the visible screen.
- Additional manual check: a full interactive `claude` session in a 100×30
  PTY drew its first-run trust prompt correctly — box-drawing rule, the `❯`
  selection marker, the numbered choices, and the `Enter to confirm · Esc to
  cancel` footer all rendered. That capture is the fixture behind
  `hick_term::prompt::tests::CODING_AGENT_TRUST_PROMPT`, which is how a drawn
  menu came to be recognised as a question at all (see
  `turbo-never-answers-a-prompt-it-did-not-parse.md` for what may and may not
  then be done about it).
- Caveat for LLM review: the assertions are on the server's screen model, not
  on pixels in the app. What is NOT covered is xterm.js's own rendering —
  fonts, colours, and the pane's CSS — which still wants a human eye.
