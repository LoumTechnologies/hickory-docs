<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/revealLine.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bca672428db0"></a>
## `apps/web/src/lib/revealLine.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Plain File Has The Same Language Server A Document Has](../../../../../guarantees/editor-intelligence/a-plain-file-has-the-same-language-server/contract.md), [A Test Runs From The Line It Is Written On](../../../../../guarantees/execution/a-test-runs-from-the-line-it-is-written-on/contract.md), [Find And Replace Is Exhaustive, And Refuses What It Would Undo](../../../../../guarantees/search/find-and-replace-is-exhaustive/contract.md).

<a id="f-bca672428db0-REVEAL_LINE_EVENT"></a>
### `REVEAL_LINE_EVENT`

"Open that file, at that line."
The awkward part is timing: the request is made before the pane that will
answer it exists. A find hit opens a file, and the editor for it is built a
tick or two later — so an event alone would be shouted into an empty room.
So the request is REMEMBERED, keyed by path, and a pane claims it when it
mounts as well as when the event arrives. A claim consumes it, which means
opening the same file again later does not silently jump to a line somebody
asked about two minutes ago.

<storycode:copy id="f-bca672428db0-REVEAL_LINE_EVENT">
// "Open that file, at that line."
//
// The awkward part is timing: the request is made before the pane that will
// answer it exists. A find hit opens a file, and the editor for it is built a
// tick or two later — so an event alone would be shouted into an empty room.
//
// So the request is REMEMBERED, keyed by path, and a pane claims it when it
// mounts as well as when the event arrives. A claim consumes it, which means
// opening the same file again later does not silently jump to a line somebody
// asked about two minutes ago.

export const REVEAL_LINE_EVENT = "hickory:reveal-line";

</storycode:copy>
<a id="f-bca672428db0-pending"></a>
### `pending`

Outstanding requests, by path. At most one per file: the newest wins,
because it is the one somebody just clicked.

<storycode:copy id="f-bca672428db0-pending">
/** Outstanding requests, by path. At most one per file: the newest wins,
 * because it is the one somebody just clicked. */
const pending = new Map<string, number>();

</storycode:copy>
<a id="f-bca672428db0-revealLine"></a>
### `revealLine`

Ask for `path` to be shown with the caret on `line` (1-based).

<storycode:copy id="f-bca672428db0-revealLine">
/** Ask for `path` to be shown with the caret on `line` (1-based). */
export function revealLine(path: string, line: number): void {
  pending.set(path, Math.max(1, Math.floor(line)));
  window.dispatchEvent(new CustomEvent(REVEAL_LINE_EVENT, { detail: path }));
}

</storycode:copy>
<a id="f-bca672428db0-claimReveal"></a>
### `claimReveal`

Take the request for `path`, if there is one.
Consuming rather than peeking: a pane that scrolls to the line has answered
the question, and re-answering it on the next re-render would fight the
reader for the cursor.

<storycode:copy id="f-bca672428db0-claimReveal">
/**
 * Take the request for `path`, if there is one.
 *
 * Consuming rather than peeking: a pane that scrolls to the line has answered
 * the question, and re-answering it on the next re-render would fight the
 * reader for the cursor.
 */
export function claimReveal(path: string): number | null {
  const line = pending.get(path);
  if (line === undefined) return null;
  pending.delete(path);
  return line;
}

</storycode:copy>
<a id="f-bca672428db0-onRevealLine"></a>
### `onRevealLine`

Listen for requests. The handler is called with the path asked for; it
still has to `claimReveal` to take it. Returns the unsubscribe.

<storycode:copy id="f-bca672428db0-onRevealLine">
/** Listen for requests. The handler is called with the path asked for; it
 * still has to `claimReveal` to take it. Returns the unsubscribe. */
export function onRevealLine(handler: (path: string) => void): () => void {
  const listener = (event: Event) => {
    const path = (event as CustomEvent).detail;
    if (typeof path === "string") handler(path);
  };
  window.addEventListener(REVEAL_LINE_EVENT, listener);
  return () => window.removeEventListener(REVEAL_LINE_EVENT, listener);
}

</storycode:copy>
<a id="f-bca672428db0-OPEN_LOCATION_EVENT"></a>
### `OPEN_LOCATION_EVENT`

"Open that file, at that line" — from somewhere that cannot open a tab.
A definition can land in another file, and the editor that asked has no
way to open one: tabs belong to the workspace. So the request is announced,
the workspace opens (or raises) the tab, and the reveal above does the
rest. Paths are workspace-relative, as everywhere else; `line` is 1-based.

<storycode:copy id="f-bca672428db0-OPEN_LOCATION_EVENT">
// "Open that file, at that line" — from somewhere that cannot open a tab.
//
// A definition can land in another file, and the editor that asked has no
// way to open one: tabs belong to the workspace. So the request is announced,
// the workspace opens (or raises) the tab, and the reveal above does the
// rest. Paths are workspace-relative, as everywhere else; `line` is 1-based.

export const OPEN_LOCATION_EVENT = "hickory:open-location";

</storycode:copy>
<a id="f-bca672428db0-OpenLocation"></a>
### `OpenLocation`

<storycode:copy id="f-bca672428db0-OpenLocation">
export interface OpenLocation {
  path: string;
  line: number;
}

</storycode:copy>
<a id="f-bca672428db0-openLocation"></a>
### `openLocation`

Ask the workspace to open `path` and put the caret on `line`.

<storycode:copy id="f-bca672428db0-openLocation">
/** Ask the workspace to open `path` and put the caret on `line`. */
export function openLocation(path: string, line: number): void {
  window.dispatchEvent(
    new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT, { detail: { path, line } }),
  );
}

</storycode:copy>
<a id="f-bca672428db0-onOpenLocation"></a>
### `onOpenLocation`

The workspace's side. Returns the unsubscribe.

<storycode:copy id="f-bca672428db0-onOpenLocation">
/** The workspace's side. Returns the unsubscribe. */
export function onOpenLocation(handler: (location: OpenLocation) => void): () => void {
  const listener = (event: Event) => {
    const detail = (event as CustomEvent<OpenLocation>).detail;
    if (detail && typeof detail.path === "string") handler(detail);
  };
  window.addEventListener(OPEN_LOCATION_EVENT, listener);
  return () => window.removeEventListener(OPEN_LOCATION_EVENT, listener);
}

</storycode:copy>
<a id="f-bca672428db0-pathOfDocUri"></a>
### `pathOfDocUri`

A `hick:///<path>` URI's path, or null for any other scheme.

<storycode:copy id="f-bca672428db0-pathOfDocUri">
/** A `hick:///<path>` URI's path, or null for any other scheme. */
export function pathOfDocUri(uri: string): string | null {
  if (!uri.startsWith("hick:///")) return null;
  try { return decodeURIComponent(new URL(uri).pathname.replace(/^\//,"")); } catch { return null; }
}

</storycode:copy>
<a id="f-bca672428db0-SHOW_TERMINAL_EVENT"></a>
### `SHOW_TERMINAL_EVENT`

"Show that terminal" — from a pane that started a session and cannot open
a tab. A test run, say.

<storycode:copy id="f-bca672428db0-SHOW_TERMINAL_EVENT">
// "Show that terminal" — from a pane that started a session and cannot open
// a tab. A test run, say.

export const SHOW_TERMINAL_EVENT = "hickory:show-terminal";

</storycode:copy>
<a id="f-bca672428db0-showTerminalRequest"></a>
### `showTerminalRequest`

<storycode:copy id="f-bca672428db0-showTerminalRequest">
export function showTerminalRequest(id: string, title: string): void {
  window.dispatchEvent(new CustomEvent(SHOW_TERMINAL_EVENT, { detail: { id, title } }));
}

</storycode:copy>
<a id="f-bca672428db0-onShowTerminal"></a>
### `onShowTerminal`

<storycode:copy id="f-bca672428db0-onShowTerminal">
export function onShowTerminal(handler: (session: { id: string; title: string }) => void): () => void {
  const listener = (event: Event) => {
    const detail = (event as CustomEvent<{ id: string; title: string }>).detail;
    if (detail && typeof detail.id === "string") handler(detail);
  };
  window.addEventListener(SHOW_TERMINAL_EVENT, listener);
  return () => window.removeEventListener(SHOW_TERMINAL_EVENT, listener);
}

</storycode:copy>
<a id="f-bca672428db0-resetReveals"></a>
### `resetReveals`

Test seam: forget every outstanding request.

<storycode:copy id="f-bca672428db0-resetReveals">
/** Test seam: forget every outstanding request. */
export function resetReveals(): void {
  pending.clear();
}
</storycode:copy>

### Assemble `apps/web/src/lib/revealLine.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/revealLine.ts"><storycode:paste select="#f-bca672428db0-REVEAL_LINE_EVENT"/><storycode:paste select="#f-bca672428db0-pending"/><storycode:paste select="#f-bca672428db0-revealLine"/><storycode:paste select="#f-bca672428db0-claimReveal"/><storycode:paste select="#f-bca672428db0-onRevealLine"/><storycode:paste select="#f-bca672428db0-OPEN_LOCATION_EVENT"/><storycode:paste select="#f-bca672428db0-OpenLocation"/><storycode:paste select="#f-bca672428db0-openLocation"/><storycode:paste select="#f-bca672428db0-onOpenLocation"/><storycode:paste select="#f-bca672428db0-pathOfDocUri"/><storycode:paste select="#f-bca672428db0-SHOW_TERMINAL_EVENT"/><storycode:paste select="#f-bca672428db0-showTerminalRequest"/><storycode:paste select="#f-bca672428db0-onShowTerminal"/><storycode:paste select="#f-bca672428db0-resetReveals"/></storycode:file>

</storycode:doc>
