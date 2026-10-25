<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# docs/guarantees/terminal/a-terminal-outlives-its-pane.md

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

This is preserved output from the original capture. Current requirement dispositions are in the integrated guarantee pages.


Captured SHA-256: `6d881865c56c98e09c7821811d9e9ba4d61c300ffcf0baa81763096d744d6141`.

<book:entry path="docs/guarantees/terminal/a-terminal-outlives-its-pane.md">
# A Terminal Outlives Its Pane

Given a terminal session started through `POST /api/terminals`, when its tab
is closed — or the window is rearranged, or nobody is attached at all — then
the process keeps running, its output keeps accumulating server-side, and a
client that attaches later is sent everything the session has already said
before it is sent anything new.

A session is the unit, not the pane. The pane is a view of it: mounting
attaches a socket, unmounting closes that socket, and neither act touches the
process. Ending a session is a separate, explicit `DELETE /api/terminals/:id`.

This is what makes it safe to close a tab during a long build, which is in
turn what makes several sessions bearable at once — if closing a pane could
kill work, every pane would have to stay open, and the window would be full
of terminals nobody is reading.

The scrollback is bounded by `HICKORY_TERM_SCROLLBACK` (lines, default
10 000) and trimmed at line boundaries, so a replay never begins mid-line.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_term::session::Session` owns the PTY, the scrollback
  (`hick_term::screen::Screen`), and a `tokio::sync::broadcast` channel; the
  reader lives on its own thread (`Session::read_forever`) and writes into the
  screen whether or not anyone is subscribed. `Session::attach` takes the
  replay and the subscription under one lock, so the seam between "what was
  said" and "what is being said" drops and duplicates nothing.
  `crates/hickory-cli/src/serve/terminal.rs::pump` sends the replay as the
  first binary frame. Nothing in `apps/web/src/terminal/TerminalPane.tsx`
  calls close/delete — its cleanup closes the socket only.
- Test coverage:
  `crates/hickory-cli/tests/serve_terminals.rs::
  a_client_that_arrives_after_the_output_still_sees_it` produces output with
  no client attached, connects afterwards, and asserts the bytes arrive;
  `hick_term::registry::tests::
  a_session_replays_what_it_said_to_a_client_that_arrives_late` makes the same
  claim at the crate boundary. `hick_term::screen::tests::
  scrollback_is_bounded_and_cut_at_line_boundaries` covers the bound.
</book:entry>

</book:doc>
