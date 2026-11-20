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

Captured debugger positions retain their source bytes and stop/frame lifetime through asynchronous tab opening and final editor claim. A line-only consumer cannot claim this stronger request without source checks. Unknown mapping identity opens text without a precise reveal; loading disk after a stop never manufactures an expected identity. These guards authorize navigation, not writes or claims about executed code.

A reveal may carry a zero-based UTF-16 column as well as its one-based line. Keep both in the same one-shot record so opening a tab before its editor mounts cannot lose the symbol column. Legacy line-only consumers still claim the line and consume the request; the simple editor claims the richer position. Neither replaying a stale request nor opening an editor grants write authority.

<storycode:copy id="f-bca672428db0-pending">
/** Outstanding requests, by path. At most one per file: the newest wins,
 * because it is the one somebody just clicked. */
type RevealGuard = {
  source: string;
  current: () => boolean;
  accepts?: (editor: HTMLElement) => boolean;
  onRevealed?: () => void;
};
const pending = new Map<string, {line: number; character?: number; guard?: RevealGuard}>();

</storycode:copy>
<a id="f-bca672428db0-revealLine"></a>
### `revealLine`

Ask for `path` to be shown with the caret on `line` (1-based).

<storycode:copy id="f-bca672428db0-revealLine">
/** Ask for `path` to be shown with the caret on `line` (1-based). */
export function revealLine(path: string, line: number, character?: number, guard?: RevealGuard): void {
  if(guard&&(!guard.current()||!Number.isInteger(line)||line<1||character!==undefined&&(!Number.isInteger(character)||character<0)))return;
  pending.set(path, { line: Math.max(1, Math.floor(line)), ...(character === undefined ? {} : { character: Math.max(0, Math.floor(character)) }),...(guard?{guard}:{}) });
  window.dispatchEvent(new CustomEvent(REVEAL_LINE_EVENT, { detail: path }));
}

</storycode:copy>
### Foreground tab activation

A verified source request can open a tab before that tab's editor can claim
the position. Bind the claim to the first committed activation of that tab.
If the reader leaves and returns, the new focus object represents a new
choice, even when it carries the same tab.

<storycode:copy id="source-tab-activation">
export interface SourceRevealTab {
  kind: string;
  target: string;
  docId?: string;
}

interface SourceRevealFocus { tab?: SourceRevealTab }

/** Return a poll that is done once it has delivered or retired the request. */
export function afterSourceTabActivation(
  target: SourceRevealTab,
  focused: () => SourceRevealFocus,
  current: () => boolean,
  deliver: (stillFocused: () => boolean) => void,
): () => boolean {
  const origin = focused();
  let done = false;
  return () => {
    if (done) return true;
    if (!current()) {
      done = true;
      return true;
    }
    const focus = focused(), tab = focus.tab;
    if (!tab || tab.kind !== target.kind || tab.docId !== target.docId ||
        (target.kind !== "document" && tab.target !== target.target)) {
      done = focus !== origin;
      return done;
    }
    done = true;
    deliver(() => current() && focused() === focus);
    return true;
  };
}

</storycode:copy>

<a id="f-bca672428db0-claimReveal"></a>
### `claimReveal`

Take the request for `path`, if there is one.
Consuming rather than peeking: a pane that scrolls to the line has answered
the question, and re-answering it on the next re-render would fight the
reader for the cursor.

Tabs can hold separate editors for the same path. Leave a guarded reveal
available until its intended editor claims it. A claimant reports completion
after setting the caret so history only advances for a delivered visit.

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
  if(line.guard)return null;
  return line.line;
}
/** Consume the same one-shot request while retaining its optional UTF-16 column. */
export function claimRevealPosition(path: string, source?: string, editor?: HTMLElement): {line: number; character?: number; onRevealed?: () => void} | null {
  const location = pending.get(path);
  if (!location) return null;
  const {line, character, guard} = location;
  if (guard && !guard.current()) { pending.delete(path); return null; }
  if (guard?.accepts && (!editor || !guard.accepts(editor))) return null;
  pending.delete(path);
  if (guard && source !== guard.source) return null;
  const onRevealed = guard?.onRevealed;
  return {line, ...(character === undefined ? {} : {character}), ...(onRevealed ? {onRevealed} : {})};
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

A provider result can carry an ordinary identifier alongside its source lease.
The host uses the identifier to check an inferred cross-file coordinate after
reading the destination. Explicit source and debugger captures carry their own
authority.

<storycode:copy id="f-bca672428db0-OpenLocation">
export interface OpenLocation {
  path: string;
  line: number;
  character?: number;
  recordHistory?: boolean;
  expectedSource?: string;
  originCurrent?: () => boolean;
  providerName?: string;
  capture?:{hash:string|null;current:()=>boolean;sourceReference?:number|null;positionKnown?:boolean};
}

</storycode:copy>
<a id="f-bca672428db0-openLocation"></a>
### `openLocation`

Ask the workspace to open `path` and put the caret on `line`.

Diagram paths resolve relative to their containing document. Decode before checking schemes and absolute paths, and reject parent traversal beyond the workspace before asking the existing file opener to show a tab.

<storycode:copy id="f-bca672428db0-openLocation">
/** Ask the workspace to open `path` and put the caret on `line`. */
export function diagramFilePath(href: string, source: string): string | null {
  let path: string;
  try { path = decodeURIComponent(href); } catch { return null; }
  if (!path || /[:\\\r\n]/.test(path) || path.startsWith("/")) return null;
  const parts = source.split("/").slice(0, -1);
  for (const part of path.split("/")) {
    if (part === "..") { if (!parts.length) return null; parts.pop(); }
    else if (part && part !== ".") parts.push(part);
  }
  return parts.length ? parts.join("/") : null;
}
export function openLocation(path: string, line: number, character?: number, recordHistory = true, expectedSource?: string): void {
  window.dispatchEvent(
    new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT, { detail: { path, line, ...(character === undefined ? {} : { character }), ...(recordHistory ? {} : { recordHistory: false }), ...(expectedSource === undefined ? {} : { expectedSource }) } }),
  );
}

</storycode:copy>
<a id="f-bca672428db0-onOpenLocation"></a>
### `onOpenLocation`

The workspace's side. Returns the unsubscribe.

<storycode:copy id="f-bca672428db0-onOpenLocation">
/** Carry the source lease and any exact provider name until the host checks the target. */
export function openProviderLocation(path: string, line: number, character: number, originCurrent: () => boolean, providerName?: string): void {
  if (!originCurrent()) return;
  window.dispatchEvent(new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT, {
    detail: {path, line, character, originCurrent, ...(providerName === undefined ? {} : {providerName})},
  }));
}

/** Unknown captured source identities open text without asserting any line position. */
export function openCapturedLocation(path:string,line:number|null,hash:string|null,current:()=>boolean,sourceReference?:number|null){
 const positionKnown=line!==null&&Number.isInteger(line)&&line>=1;
 window.dispatchEvent(new CustomEvent<OpenLocation>(OPEN_LOCATION_EVENT,{detail:{path,line:positionKnown?line!:1,character:0,recordHistory:false,capture:{hash:positionKnown&&!(sourceReference&&sourceReference>0)&&hash&&/^[0-9a-f]{64}$/.test(hash)?hash:null,current,positionKnown,...(sourceReference===undefined?{}:{sourceReference})}}}));
}

/** The workspace's side. Returns the unsubscribe. */
export function onOpenLocation(handler: (location: OpenLocation) => void,supportsCaptured=false): () => void {
  const listener = (event: Event) => {
    const detail = (event as CustomEvent<OpenLocation>).detail;
    if (detail && typeof detail.path === "string"&&(!detail.capture||supportsCaptured)) handler(detail);
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

<storycode:file path="apps/web/src/lib/revealLine.ts"><storycode:paste select="#f-bca672428db0-REVEAL_LINE_EVENT"/><storycode:paste select="#f-bca672428db0-pending"/><storycode:paste select="#f-bca672428db0-revealLine"/><storycode:paste select="#source-tab-activation"/><storycode:paste select="#f-bca672428db0-claimReveal"/><storycode:paste select="#f-bca672428db0-onRevealLine"/><storycode:paste select="#f-bca672428db0-OPEN_LOCATION_EVENT"/><storycode:paste select="#f-bca672428db0-OpenLocation"/><storycode:paste select="#f-bca672428db0-openLocation"/><storycode:paste select="#f-bca672428db0-onOpenLocation"/><storycode:paste select="#f-bca672428db0-pathOfDocUri"/><storycode:paste select="#f-bca672428db0-SHOW_TERMINAL_EVENT"/><storycode:paste select="#f-bca672428db0-showTerminalRequest"/><storycode:paste select="#f-bca672428db0-onShowTerminal"/><storycode:paste select="#f-bca672428db0-resetReveals"/></storycode:file>

</storycode:doc>
