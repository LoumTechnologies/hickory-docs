// Terminals, shown where they are running.
//
// There is one tree in this window, and it is the folder. Terminals used to
// have a second one — a pane listing every session grouped by project — which
// meant two places to look for "what is going on", each with its own idea of
// how things are arranged, and neither of them the place you were already
// looking. A terminal has a working directory; the tree already draws
// directories; the honest thing is to put the terminal on its directory.
//
// They are visible child nodes beside files, not decorations on a directory
// row. That is the first concrete instance of the workspace tree: things with
// a location and an action belong at that location even when they are not
// files. The pure placement remains in FolderTreePane; this module owns the
// stable urgency order and the summary a collapsed ancestor draws.

/** The states a session reports, worst first — this order IS the ranking. */
export const TERMINAL_STATES = [
  "needs-you",
  "failed",
  "working",
  "finished",
  "idle",
] as const;

export type TerminalState = (typeof TERMINAL_STATES)[number];

/** An unknown state sorts last rather than first: a build we do not
 * understand must not outrank a question somebody is waiting on. */
function rank(state: string): number {
  const i = (TERMINAL_STATES as readonly string[]).indexOf(state);
  return i === -1 ? TERMINAL_STATES.length : i;
}

/** Sessions in the order their nodes should sit in a directory: most urgent first,
 * then by title so the order is stable while nothing changes state. */
export function rankSessions<T extends { state: string; title: string }>(
  sessions: readonly T[],
): T[] {
  return [...sessions].sort(
    (a, b) => rank(a.state) - rank(b.state) || a.title.localeCompare(b.title),
  );
}

/** What the row's summary should say when the directory is collapsed and its
 * terminals are hidden inside it. */
export function hiddenSummary(count: number, urgent: number): string {
  if (count === 0) return "";
  const plural = count === 1 ? "terminal" : "terminals";
  if (urgent === 0) return `${count} ${plural} running in here`;
  return `${count} ${plural} running in here, ${urgent} needing you`;
}

/** How many of these want something from a person. */
export function urgentCount(sessions: readonly { state: string }[]): number {
  return sessions.filter((s) => s.state === "needs-you" || s.state === "failed").length;
}
