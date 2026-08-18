# A Session Appears Where It Is Working

Given terminal sessions and an open folder, when the folder tree is shown, then
each session appears as a row at the directory it is working in; clicking that
row shows the session's terminal; a collapsed directory says how many sessions
are hidden inside it; and a session working outside the folder does not appear
at all.

The tree already answers *what is in this folder*. This makes it answer *what
is running in it*, in the same place, because those are the same question asked
about a directory twice — and the process you have forgotten about is the one
worth finding.

Four properties hold it up:

1. **The working directory is the shell's, not ours, when the shell says.**
   Sessions report where they are through **OSC 7**, the escape sequence every
   terminal emulator uses to track a shell's directory. It is the only portable
   way: reading `/proc/<pid>/cwd` does not exist on macOS or Windows. A shell
   that does not emit OSC 7 leaves the session at the directory it was started
   in — stale after a `cd`, never wrong about where it began — and
   `SessionSummary::cwd_is_live` says which of the two a reader is looking at,
   because the two are not equally trustworthy.
2. **A session is never hidden by the shape of the listing.** It is shown at
   the deepest directory the listing contains, which is its own working
   directory when that is listed and its nearest listed ancestor when the
   directory was truncated away. A collapsed directory carries a count instead,
   so shutting a folder cannot conceal what is running in it.
3. **"Inside the folder" is decided on path segments, not string prefixes.**
   `/w-other` begins with `/w` and is not in it. Getting this wrong shows
   somebody else's work as if it were yours.
4. **The scanner observes the byte stream and never consumes it.** OSC parsing
   runs beside the screen model, so output reaches the terminal unchanged.

## Boundary

**These are the sessions this app owns.** A shell somebody started in iTerm,
whose working directory happens to be in the folder, does not appear — and
should not, because "click it to go there" has nowhere to go. Enumerating every
process on the machine by working directory is a different feature with a
different answer on each platform, and it is not this one.

**A sequence split across reads still arrives.** The PTY hands over whatever
was in the buffer, so the OSC scanner is a state machine rather than a search.
An unterminated sequence is abandoned at a bounded size rather than growing.

---

Last LLM verification:
- Date: 2026-08-18
- Reviewer: Claude (Opus 5)
- Result: verified (implemented and reviewed in the same change)
- Evidence: `crates/hick-term/src/screen.rs` — the `Osc` state machine in
  `Screen::scan_osc`, `cwd_from_osc7` (host ignored deliberately),
  `percent_decode`, and `MAX_OSC`. `crates/hick-term/src/session.rs` —
  `SessionSummary::cwd` now prefers the live directory, with `cwd_is_live`
  reporting which source it came from, both read under one screen lock.
  `apps/web/src/shell/FolderTreePane.tsx` — `relativeCwd`, `placeSessions`,
  `sessionsUnder`, `directoryPaths`, and `SessionRow`; wired in
  `apps/web/src/views/WorkspaceView.tsx`, whose `showTerminal` is what a click
  calls.
- Test coverage: `crates/hick-term/src/screen.rs::osc_tests` (9 tests) — both
  terminator spellings, a sequence split across three reads, a later `cd`
  replacing an earlier one, percent-encoded spaces and accents, a silent shell
  leaving it unknown, OSC 0/2 ignored, an unterminated sequence staying
  bounded, and output still reaching the screen around a sequence.
  `apps/web/src/shell/FolderTreePane.test.tsx` — placement at its own
  directory, at the root, climbing to the nearest listed ancestor, exclusion of
  sessions outside the folder, the shared-prefix sibling, trailing slashes,
  several sessions in one directory, subtree counting, `relativeCwd`, and three
  render tests covering the click, the collapsed-directory count, and a session
  elsewhere not appearing.
- Caveats — what LLM review could NOT establish:
  - **No shell was actually driven.** Every OSC test feeds bytes directly. That
    zsh on macOS and bash on a given Linux distribution really emit OSC 7 —
    and that they emit it at the moments assumed — is untested here, and it is
    the assumption the whole feature rests on.
  - **Nothing was seen in a running window.** The rows typecheck and their
    tests pass under jsdom; how a folder with twenty sessions in it reads, and
    whether the count badge is noticed, is unverified.
  - **The row shows the session's title, not the running command.** For a shell
    that sets its title to the foreground command these coincide; for one that
    does not, the row says "bash" while `cargo test` is what is running. The
    title is already carried in `SessionSummary` and is not derived from OSC
    0/2 here.
