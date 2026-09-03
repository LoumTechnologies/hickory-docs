// Where execution is paused, when that is not the file being debugged.
//
// A plain file's debugger steps into `src/lib.rs`; the pane for `src/main.rs`
// owns the session and cannot draw a line in a file it does not hold. So the
// owner publishes "paused at this file, this line" here, the workspace opens
// that file's tab, and that tab's pane draws the paused line as if the
// session were its own — which, for reading, it is.
//
// One value, not a map: a person is stopped in one place at a time. The
// owner is recorded so that only the session that published a position can
// clear it, and a neighbour's pane finishing its own run does not wipe the
// arrow out from under a session that is still paused.

import { useSyncExternalStore } from "react";

export interface PausedElsewhere {
  /** Root-relative path of the file execution is paused in. */
  path: string;
  /** 0-based line in that file. */
  line: number;
  /** Who published it — the debugging pane's own path — so only it clears it. */
  owner: string;
}

let current: PausedElsewhere | null = null;
const listeners = new Set<() => void>();

function emit() {
  for (const listener of listeners) listener();
}

/** Publish where `owner`'s session is paused, or clear it with `null`. */
export function publishPausedElsewhere(
  owner: string,
  at: { path: string; line: number } | null,
): void {
  if (at === null) {
    if (current?.owner !== owner) return;
    current = null;
  } else {
    if (current && current.owner === owner && current.path === at.path && current.line === at.line) {
      return;
    }
    current = { ...at, owner };
  }
  emit();
}

export function pausedElsewhere(): PausedElsewhere | null {
  return current;
}

export function onPausedElsewhere(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** The current position, as React state. */
export function usePausedElsewhere(): PausedElsewhere | null {
  return useSyncExternalStore(onPausedElsewhere, pausedElsewhere, pausedElsewhere);
}

/** Test seam. */
export function resetPausedElsewhere(): void {
  current = null;
  emit();
}
