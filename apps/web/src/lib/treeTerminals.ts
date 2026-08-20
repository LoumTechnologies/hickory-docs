// Terminals, shown where they are running.
//
// There is one tree in this window, and it is the folder. Terminals used to
// have a second one — a pane listing every session grouped by project — which
// meant two places to look for "what is going on", each with its own idea of
// how things are arranged, and neither of them the place you were already
// looking. A terminal has a working directory; the tree already draws
// directories; the honest thing is to put the terminal on its directory.
//
// They are ICONS on the directory's own row rather than rows of their own,
// and that is the whole design. A row per session pushes the folder's
// contents down and makes a busy project's tree mostly not-files, which
// inverts what the tree is for. An icon rides a row that already exists and
// costs no vertical space at all.
//
// Two rules make the icons readable rather than decorative:
//
//  - **Order is urgency, not recency.** A session that is asking a question
//    comes first, wherever it started. Sorting by start time would make the
//    one thing worth acting on wander.
//  - **A session that needs you does not hide.** Most icons appear on hover,
//    because a tree covered in glyphs is a tree nobody reads. "Needs you" and
//    "failed" stay visible, because an affordance that only appears when the
//    pointer is already there cannot tell you to go there.
//
// Pure: what a session looks like is CSS, and where they are placed is
// `placeSessions` in the tree pane. This is only the ranking.

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

/**
 * Whether this session's icon stays visible when its row is not hovered.
 *
 * The test is deliberately "does this want something from a person", not
 * "is this busy": a build churning away needs nothing, and marking it would
 * spend the reader's attention on the state that least deserves it.
 */
export function alwaysVisible(state: string): boolean {
  return state === "needs-you" || state === "failed";
}

/** Sessions in the order their icons should sit on a row: most urgent first,
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
  return sessions.filter((s) => alwaysVisible(s.state)).length;
}
