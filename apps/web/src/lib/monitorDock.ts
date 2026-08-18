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
