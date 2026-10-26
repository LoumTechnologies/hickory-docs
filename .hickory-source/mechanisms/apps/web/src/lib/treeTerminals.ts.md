<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/treeTerminals.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cc4a9831efa5"></a>
## `apps/web/src/lib/treeTerminals.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cc4a9831efa5-TERMINAL_STATES"></a>
### `TERMINAL_STATES`

Terminals, shown where they are running.
There is one tree in this window, and it is the folder. Terminals used to
have a second one — a pane listing every session grouped by project — which
meant two places to look for "what is going on", each with its own idea of
how things are arranged, and neither of them the place you were already
looking. A terminal has a working directory; the tree already draws
directories; the honest thing is to put the terminal on its directory.
They are visible child nodes beside files, not decorations on a directory
row. That is the first concrete instance of the workspace tree: things with
a location and an action belong at that location even when they are not
files. The pure placement remains in FolderTreePane; this module owns the
stable urgency order and the summary a collapsed ancestor draws.
The states a session reports, worst first — this order IS the ranking.

<storycode:copy id="f-cc4a9831efa5-TERMINAL_STATES">
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

</storycode:copy>
<a id="f-cc4a9831efa5-TerminalState"></a>
### `TerminalState`

<storycode:copy id="f-cc4a9831efa5-TerminalState">
export type TerminalState = (typeof TERMINAL_STATES)[number];

</storycode:copy>
<a id="f-cc4a9831efa5-rank"></a>
### `rank`

An unknown state sorts last rather than first: a build we do not
understand must not outrank a question somebody is waiting on.

<storycode:copy id="f-cc4a9831efa5-rank">
/** An unknown state sorts last rather than first: a build we do not
 * understand must not outrank a question somebody is waiting on. */
function rank(state: string): number {
  const i = (TERMINAL_STATES as readonly string[]).indexOf(state);
  return i === -1 ? TERMINAL_STATES.length : i;
}

</storycode:copy>
<a id="f-cc4a9831efa5-rankSessions"></a>
### `rankSessions`

Sessions in the order their nodes should sit in a directory: most urgent first,
then by title so the order is stable while nothing changes state.

<storycode:copy id="f-cc4a9831efa5-rankSessions">
/** Sessions in the order their nodes should sit in a directory: most urgent first,
 * then by title so the order is stable while nothing changes state. */
export function rankSessions<T extends { state: string; title: string }>(
  sessions: readonly T[],
): T[] {
  return [...sessions].sort(
    (a, b) => rank(a.state) - rank(b.state) || a.title.localeCompare(b.title),
  );
}

</storycode:copy>
<a id="f-cc4a9831efa5-hiddenSummary"></a>
### `hiddenSummary`

What the row's summary should say when the directory is collapsed and its
terminals are hidden inside it.

<storycode:copy id="f-cc4a9831efa5-hiddenSummary">
/** What the row's summary should say when the directory is collapsed and its
 * terminals are hidden inside it. */
export function hiddenSummary(count: number, urgent: number): string {
  if (count === 0) return "";
  const plural = count === 1 ? "terminal" : "terminals";
  if (urgent === 0) return `${count} ${plural} running in here`;
  return `${count} ${plural} running in here, ${urgent} needing you`;
}

</storycode:copy>
<a id="f-cc4a9831efa5-urgentCount"></a>
### `urgentCount`

How many of these want something from a person.

<storycode:copy id="f-cc4a9831efa5-urgentCount">
/** How many of these want something from a person. */
export function urgentCount(sessions: readonly { state: string }[]): number {
  return sessions.filter((s) => s.state === "needs-you" || s.state === "failed").length;
}
</storycode:copy>

### Assemble `apps/web/src/lib/treeTerminals.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/treeTerminals.ts"><storycode:paste select="#f-cc4a9831efa5-TERMINAL_STATES"/><storycode:paste select="#f-cc4a9831efa5-TerminalState"/><storycode:paste select="#f-cc4a9831efa5-rank"/><storycode:paste select="#f-cc4a9831efa5-rankSessions"/><storycode:paste select="#f-cc4a9831efa5-hiddenSummary"/><storycode:paste select="#f-cc4a9831efa5-urgentCount"/></storycode:file>

</storycode:doc>
