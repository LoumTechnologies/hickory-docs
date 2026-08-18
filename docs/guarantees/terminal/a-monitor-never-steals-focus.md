# A Monitor Never Steals Focus

Given a session opened with `monitor: true` — a dev server, a test watcher, a
log tail — when it prints, restarts, or dies, then it never enters the
attention queue, never opens a tab by itself, and never takes focus from
whatever pane is in front. It appears in the dock, where it shows its name and
its last line, and the dock turns amber when any monitor has stopped running.

Opening a monitor is always the person's move: the dock's chips are buttons
they press.

Monitors are also left out of the project rows in the terminals list, so a dev
server does not make every project look permanently busy.

This is a real exclusion, not a convention: a queue that fills up with
"recompiled in 240 ms" is a queue people stop reading, and once that happens
the blocked agent it also contains is lost.

Amber rather than red, and without motion — something you were relying on has
stopped, which is worth seeing when you look down, not worth interrupting for.

---

Last LLM verification:
- Date: 2026-08-17
- Reviewer: Claude (Opus 5)
- Result: verified
- Evidence: `hick_term::registry::Terminals::attention` filters `monitor`
  sessions out before ranking, so the exclusion holds at the source of the
  queue rather than in any one view. `apps/web/src/lib/monitorDock.ts` holds
  the dock's only decision (`dockTone`, amber when a monitor is finished or
  failed); `apps/web/src/lib/sessionGroups.ts::groupSessions` skips monitors
  when grouping. `apps/web/src/terminal/MonitorDock.tsx` renders the strip and
  calls `onOpen` only from a click. Nothing in `WorkspaceView`'s terminal
  wiring opens a tab or moves focus on its own — `openTerminal` runs from the
  menu action, `showTerminal` from a click.
- Test coverage: `hick_term::registry::tests::
  a_monitor_never_enters_the_queue_however_loudly_it_fails`;
  `crates/hickory-cli/tests/serve_terminals.rs::a_monitor_stays_out_of_the_queue`
  (a monitor that exits non-zero, over the wire);
  `apps/web/src/lib/monitorDock.test.ts` (tone, and that an ordinary failed
  session does not colour the dock);
  `apps/web/src/lib/sessionGroups.test.ts` (monitors excluded from groups).
- Caveat for LLM review: "never takes focus" is a claim about the absence of
  code — there is no test that would fail if a future change called
  `showTerminal` from the poll. Re-read `WorkspaceView`'s terminal section
  when reviewing this guarantee.
