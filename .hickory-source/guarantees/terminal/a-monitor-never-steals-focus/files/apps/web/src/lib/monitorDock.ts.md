<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/monitorDock.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Monitor Never Steals Focus](../../../../../contract.md)

Captured SHA-256: `1198d492be7776ce3145e3935325d59a1dad6656bf163c2e81caf491a667319f`.

<book:entry path="apps/web/src/lib/monitorDock.ts">
// The monitor dock's one decision: how loud to be.
//
// Dev servers, watchers, and log tails are long-running support processes.
// They are always saying something, so they must never claim attention the
// way a task does — a queue that fills with "webpack recompiled" is a queue
// people stop reading. But a dev server that has actually fallen over is
// worth noticing, so the dock goes amber and waits to be looked at, rather
// than interrupting.
//
// Protects docs/guarantees/terminal/a-monitor-never-steals-focus.md

import type { TerminalSession } from "../api/types";

export type DockTone = "quiet" | "amber";

/** Only the sessions that belong in the dock. */
export function monitors(sessions: readonly TerminalSession[]): TerminalSession[] {
  return sessions.filter((s) => s.monitor);
}

/**
 * How the dock should look. Amber when any monitor has died — failed, or
 * exited at all: a dev server that ended cleanly still ended, and the thing
 * you were relying on is not there any more.
 */
export function dockTone(sessions: readonly TerminalSession[]): DockTone {
  const dead = monitors(sessions).some(
    (s) => s.state === "failed" || s.state === "finished",
  );
  return dead ? "amber" : "quiet";
}
</book:entry>

</book:doc>
