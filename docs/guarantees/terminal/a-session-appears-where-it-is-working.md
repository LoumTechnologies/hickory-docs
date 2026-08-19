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

   **No shell on macOS says it on its own, so the shell is told.** Both system
   shells carry Apple's `update_terminal_cwd` hook and both gate it on
   `TERM_PROGRAM`: `/etc/zshrc` sources `/etc/zshrc_$TERM_PROGRAM`, and only
   `/etc/zshrc_Apple_Terminal` exists. Measured on macOS 15.7.7, `zsh -i` emits
   **zero** OSC 7 sequences and `bash -l -i` zero, so every session's row sat at
   its start directory for the life of the session.

   [`shell_integration`](../../../crates/hick-term/src/shell_integration.rs)
   closes that: a generated startup file that **sources the user's own
   configuration first** and then adds one hook — `ZDOTDIR` for zsh, `--rcfile`
   for bash. Nothing the user owns is edited and nothing persists; the scripts
   live in a temporary directory that goes when the session does, and
   `HICKORY_SHELL_INTEGRATION=0` turns it off. A shell that cannot be integrated
   is still a working shell reporting its start directory, which is the
   behaviour that was already shipping.

   **Claiming to be Terminal.app was rejected, and not only for being a lie.**
   That file is mostly *not* OSC 7 — it is Terminal.app's session save/restore
   machinery, which splits the user's shell history into per-session files keyed
   on `$TERM_SESSION_ID`. A terminal launched from another terminal inherits
   that id, so every session in this app would share one history file; launched
   from the Dock it would not. Instead the app now names itself —
   `TERM_PROGRAM=HickoryDocs` — which is true, stops
   `/etc/zshrc_$TERM_PROGRAM` matching somebody else's file, and removes the
   worst property the old behaviour had: **OSC 7 used to work or not depending
   on how the app was launched**, because `TERM_PROGRAM=Apple_Terminal` was
   inherited from whatever started it.
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
- Result: verified; the OSC 7 assumption was **measured on Apple hardware**, the
  macOS answer turned out to be that no shell says it, and the shell is now told
  — see the caveats
- Evidence: `crates/hick-term/src/shell_integration.rs` — the generated
  `ZDOTDIR` trampoline (all four zsh startup files forwarded, because setting
  `ZDOTDIR` moves every one of them and a dropped `.zshenv` is a dropped
  `PATH`), the bash `--rcfile`, and the byte-wise percent-encoding both hooks
  emit. Applied in `session.rs::spawn`, and only when the session has no `argv`
  of its own — a session running somebody's command is not one whose startup we
  rewrite. `TermConfig::integrate_shell` reads
  `HICKORY_SHELL_INTEGRATION`, validated at boot like the rest.
  `crates/hick-term/src/screen.rs` — the `Osc` state machine in
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
  - **A shell is now driven, macOS's answer was "no", and it is now told.**
    Measured 2026-08-18
    on a MacBook Pro (MacBookPro15,1, Intel Core i7), macOS 15.7.7, by running
    each shell in a real PTY, `cd`-ing twice, and counting OSC 7 sequences in
    what came back:

    | shell | `TERM_PROGRAM` unset (what this app sends) | `TERM_PROGRAM=Apple_Terminal` |
    |---|---|---|
    | `/bin/zsh -i` | **0** | 3 — `file://host/Users/loumtech`, `…/tmp`, `…/usr/share` |
    | `/bin/bash -i` | 0 | 0 — `bash -i` never reads `/etc/bashrc` on macOS |
    | `/bin/bash -l -i` | **0** | 3 |

    The hook is in `/etc/zshrc_Apple_Terminal` and `/etc/bashrc_Apple_Terminal`,
    reached only through `[ -r "/etc/zshrc_$TERM_PROGRAM" ]`. What it emits is
    exactly what `cwd_from_osc7` parses — `printf '\e]7;%s\a' "file://$HOST$url_path"`
    — so the scanner is right and the shells are silent. Linux distributions were
    not measured; `vte.sh` on a GNOME-derived setup is the usual source there,
    and it is not in evidence here.
  - **The scanner is now proven against a real shell rather than a fixture.**
    `crates/hick-term/tests/a_real_shell_reports_its_directory.rs` drives a real
    zsh through a real PTY: one test makes it emit OSC 7 and asserts the session
    follows to the new directory with `cwd_is_live` true; the other lets a silent
    shell `cd` and asserts the session stays at its started-in directory with
    `cwd_is_live` false. Both pass on macOS 15.7.7.
  - **Shell integration is covered by real shells, and found a bug in its own
    hook.** `crates/hick-term/tests/a_real_shell_reports_its_directory.rs` (6
    tests) drives zsh and bash through real PTYs: an integrated shell follows a
    `cd` with nobody typing an escape sequence, the user's own `.zshrc` still
    runs, the opt-out opts out, a silent shell still admits it is not live, and
    a directory named `dir with café` round-trips. That last one exists because
    it failed: `printf '%d' "'é"` in bash reports −61 rather than 195 — it
    sign-extends — so the first version emitted `%FFFFFFFFFFFFFFC3` and a
    directory with an accent arrived as nonsense. zsh's printf does not do this,
    so only one of the two hooks needs the masking, and only a real shell in a
    real directory catches it.
  - **Only zsh and bash are integrated.** `sh` is deliberately not, because a
    shell invoked as `sh` is in POSIX mode and reads a different file, and
    guessing which is how a startup file gets sourced twice. fish and
    PowerShell are not handled at all. Those sessions report their start
    directory, honestly, as before. A `bash -l` session is also not covered:
    `--rcfile` is ignored by a login shell.
  - **Windows is untested here.** The integration is Unix-shaped, and nothing
    has run on Windows.
  - **Nothing was seen in a running window.** The rows typecheck and their
    tests pass under jsdom; how a folder with twenty sessions in it reads, and
    whether the count badge is noticed, is unverified.
  - **The row shows the session's title, not the running command.** For a shell
    that sets its title to the foreground command these coincide; for one that
    does not, the row says "bash" while `cargo test` is what is running. The
    title is already carried in `SessionSummary` and is not derived from OSC
    0/2 here.
