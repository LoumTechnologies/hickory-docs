# A Watched Cell Shows What The Terminal Showed

Given a `hick:exec` cell that has run — or is running now, with its
transcript arriving on the run channel — when its output appears under the
cell in the Document view, then it is drawn by a terminal emulator rather
than printed as text, so what the reader sees is what the command actually
drew:

- **escape sequences are applied, not shown.** `\x1b[32m` is green; it is
  never four visible characters;
- **a carriage return moves the cursor.** A build that rewrites one progress
  line is one line, not one line per rewrite;
- **stdout and stderr are not told apart**, because they were not told apart
  on the screen the command ran on: two streams, one terminal, interleaved in
  time — an order the executor already fixed when it stamped each chunk;
- **the only bytes that are not the program's are the app's own decorations**
  — a `$ ` prompt when a cell has more than one command, and a `[exit n]`
  line when the exit was non-zero — each wrapped in its own SGR and each
  resetting first, so a command that left the screen bold red cannot bleed
  into them.

This is the **watching** binding of the one terminal component described in
`docs/specs/freeform/a-terminal-that-writes-the-document.md` — the third row
of its table: *input none, writes nothing*. That is a property of the code,
not a setting: the component holds no socket, registers no `onData` handler,
and has nothing anywhere in it that could send a byte to a process. A person
may click it and type; the keystrokes have nowhere to go. `disableStdin` is
belt to that braces, not the mechanism.

It carries no claim about *what* is watched. The transcript still arrives
only when the run finishes (`LocalState::start_run` replays every cell's
events onto the run channel after `run_doc` returns), so "live" here means a
cell appending as its events arrive, not a byte-by-byte stream from a running
process.

---

Last LLM verification:
- Date: 2026-08-26
- Reviewer: Claude (Opus 5)
- Result: verified, by running it
- Evidence:
  - `apps/web/src/lib/watchStream.ts` decides which bytes reach the emulator
    (`watchBytes`, `showsCommands`) and is pure, so the decisions above are
    testable without an emulator at all.
  - `apps/web/src/terminal/WatchingTerminal.tsx` is the component. It opens
    xterm with `convertEol` (the executor normalizes captured `\r\n` to `\n`
    on every platform in `normalize_captured_newlines`, so the emulator is
    the single place a line feed becomes a new line) and appends only the
    events it has not written yet.
  - `apps/web/src/components/CellPanel.tsx` mounts it where the `<pre>`-based
    `Transcript` card used to be. That card and the playback-timing helpers
    that drove it (`segmentsAt`, `finalSegments`, `transcriptDuration`) are
    deleted rather than left behind — an emulator applies the escapes they
    existed to sidestep.
- Test coverage:
  - `apps/web/src/lib/watchStream.test.ts` — pass-through of a real
    colour-and-`\r` stream byte-for-byte, out/err not distinguished, the
    prompt rule, the non-zero-exit rule, resets before every decoration, and
    the `from` offset a live run appends through.
  - `apps/web/src/terminal/WatchingTerminal.test.tsx` — mounts the **real**
    xterm parser under jsdom and asserts on the rows it drew: `\x1b[32mok` is
    the text `ok` inside a `.xterm-fg-2` span (the escape became an
    attribute), `Compiling 12/40\rCompiling 40/40` is one row, a live run
    appends instead of redrawing, a shorter event list starts over, and 60
    lines of output stop at the 20-row cap. Two jsdom gaps xterm reaches
    through (`matchMedia`, `HTMLCanvasElement.getContext`) are stubbed in
    `apps/web/src/test-setup.ts`, with the reason each stub is honest written
    beside it.
- Verified in a real browser (Chromium via Playwright, against
  `VITE_MOCK=1 npm run dev`, 2026-08-26), because jsdom does no layout and
  half of this is layout:
  - a transcript carrying `\x1b[32m   Compiling`, a `\r` progress line, and
    `\x1b[1;31merror` rendered as green `rgb(78, 154, 6)`, one row reading
    `   Building [40] 40/40`, and bold `rgb(239, 41, 41)`. No escape byte
    survived anywhere in the element's text.
  - **A bug this found, which no unit test could have.** The panel is a
    CodeMirror block widget, so its width comes from `.cm-content`, a flex
    item with `min-width: auto` inside a horizontally scrolling
    `.cm-scroller` — as wide as its widest child. An in-flow emulator
    measured that column, set itself that wide, widened the column by doing
    so, and was measured again: the terminal was watched climbing past
    25000px, growing every frame. The `<pre>` it replaced never hit this
    because `word-break` meant it never exceeded its column. The fix is the
    pattern `.terminal-pane` already uses for the same reason — the emulator
    is absolutely positioned, so it contributes nothing to its parent's
    content width and the measurement has a fixed point. The component then
    hands the wrapper a height, since an out-of-flow box leaves it none.
    Re-measured after the fix: stable at 485×103 over ten samples, with
    `.cm-content` back at its natural 594px.
- Caveat requiring review:
  - **Nothing here was watched appending during a real run.** The run channel
    replays a cell's events only after `run_doc` returns, so the live path
    was exercised by re-rendering with a longer event list (in tests) and not
    by a genuinely streaming run. Whether output should stream *while* a cell
    runs is a change to `LocalState::start_run`, not to this component, and
    is not part of this guarantee.
  - **The 20-row cap is a number, not a finding.** It is what stops a cell
    taking a document over; nobody has yet read a long build in it and said
    whether that is the right height.
  - **Colours are xterm's defaults, not the app's theme.** The pane inherits
    the app's mono font and a transparent background, but the 16 ANSI colours
    are whatever xterm ships. Against `--term-bg` in both themes that has
    been looked at once, in one browser, by one reviewer.
  - **Only the `watching` binding exists.** The *ephemeral* binding is the
    existing `TerminalPane`; the *persistent* one — the terminal that writes
    the document, with its suspension rules — is not built, and none of the
    secret-scan or `tcgetpgrp` machinery in the spec is present. Nothing in
    this guarantee should be read as a claim about them.
