<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# A Keystroke Redraws Only The Editor

For: Authors editing executable documents and the files those documents produce.

Given a document open in the workspace beside any number of other panes, when
a character is typed into it, then the only React render that keystroke causes
is inside the editor pane that received it — the workspace chrome (the other
panes, the folder tree, the rulers, the rails, the ribbon overlay, the status
bar's other fields) is not re-rendered, and nothing that was already on screen
is re-measured against the DOM.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-8f044116e767"></a>
## `apps/web/src/lib/offsets.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-8f044116e767-byteLenAt"></a>
### `byteLenAt`

The server contract (docs/specs/freeform/api.md) expresses provenance and
output-edit ranges in BYTE offsets (UTF-8); JavaScript strings index UTF-16
code units. These helpers convert between the two.
UTF-8 byte length of the code point starting at UTF-16 index `i`.

<storycode:copy id="f-8f044116e767-byteLenAt">
// The server contract (docs/specs/freeform/api.md) expresses provenance and
// output-edit ranges in BYTE offsets (UTF-8); JavaScript strings index UTF-16
// code units. These helpers convert between the two.

/** UTF-8 byte length of the code point starting at UTF-16 index `i`. */
function byteLenAt(text: string, i: number): { bytes: number; units: number } {
  const cp = text.codePointAt(i)!;
  if (cp < 0x80) return { bytes: 1, units: 1 };
  if (cp < 0x800) return { bytes: 2, units: 1 };
  if (cp < 0x10000) return { bytes: 3, units: 1 };
  return { bytes: 4, units: 2 };
}

</storycode:copy>
<a id="f-8f044116e767-asciiCache"></a>
### `asciiCache`

Whether a text is pure ASCII, remembered for the last few texts seen. In
ASCII, byte offsets and UTF-16 offsets are the same number, and the
provenance of one file asks this question hundreds of times per redraw
against the same two strings — so the answer is worth keeping. The regex is
native and linear; the walk it replaces is interpreted and quadratic over a
file's provenance list. Keyed by the string itself: V8 caches a string's
hash, so a repeated lookup of the same (large) string is a pointer check.

<storycode:copy id="f-8f044116e767-asciiCache">
// Whether a text is pure ASCII, remembered for the last few texts seen. In
// ASCII, byte offsets and UTF-16 offsets are the same number, and the
// provenance of one file asks this question hundreds of times per redraw
// against the same two strings — so the answer is worth keeping. The regex is
// native and linear; the walk it replaces is interpreted and quadratic over a
// file's provenance list. Keyed by the string itself: V8 caches a string's
// hash, so a repeated lookup of the same (large) string is a pointer check.
const asciiCache = new Map<string, boolean>();
</storycode:copy>
<a id="f-8f044116e767-ASCII_CACHE_SIZE"></a>
### `ASCII_CACHE_SIZE`

<storycode:copy id="f-8f044116e767-ASCII_CACHE_SIZE">
const ASCII_CACHE_SIZE = 16;
</storycode:copy>
<a id="f-8f044116e767-isAscii"></a>
### `isAscii`

<storycode:copy id="f-8f044116e767-isAscii">
function isAscii(text: string): boolean {
  let known = asciiCache.get(text);
  if (known === undefined) {
    // eslint-disable-next-line no-control-regex
    known = !/[^\x00-\x7f]/.test(text);
    if (asciiCache.size >= ASCII_CACHE_SIZE) {
      asciiCache.delete(asciiCache.keys().next().value as string);
    }
    asciiCache.set(text, known);
  }
  return known;
}

</storycode:copy>
<a id="f-8f044116e767-charToByte"></a>
### `charToByte`

UTF-8 byte offset of UTF-16 index `charIndex` in `text`.

<storycode:copy id="f-8f044116e767-charToByte">
/** UTF-8 byte offset of UTF-16 index `charIndex` in `text`. */
export function charToByte(text: string, charIndex: number): number {
  if (isAscii(text)) return Math.max(0, Math.min(charIndex, text.length));
  let bytes = 0;
  let i = 0;
  while (i < charIndex && i < text.length) {
    const { bytes: b, units } = byteLenAt(text, i);
    bytes += b;
    i += units;
  }
  return bytes;
}

</storycode:copy>
<a id="f-8f044116e767-byteToChar"></a>
### `byteToChar`

UTF-16 index for UTF-8 byte offset `byteIndex` in `text` (clamped).

<storycode:copy id="f-8f044116e767-byteToChar">
/** UTF-16 index for UTF-8 byte offset `byteIndex` in `text` (clamped). */
export function byteToChar(text: string, byteIndex: number): number {
  if (isAscii(text)) return Math.max(0, Math.min(byteIndex, text.length));
  let bytes = 0;
  let i = 0;
  while (i < text.length) {
    if (bytes >= byteIndex) return i;
    const { bytes: b, units } = byteLenAt(text, i);
    bytes += b;
    i += units;
  }
  return text.length;
}

</storycode:copy>
<a id="f-8f044116e767-bytesToChars"></a>
### `bytesToChars`

Convert a provenance list in one forward scan, retaining input order.
Repeated byteToChar calls rescan a Unicode session from its beginning for
every link, which can block token-bound UI replies as a session grows.

<storycode:copy id="f-8f044116e767-bytesToChars">
/** Convert a provenance list in one forward scan, retaining input order.
 * Repeated byteToChar calls rescan a Unicode session from its beginning for
 * every link, which can block token-bound UI replies as a session grows. */
export function bytesToChars(text: string, offsets: readonly number[]): number[] {
  if (isAscii(text)) return offsets.map(offset => Math.max(0, Math.min(offset, text.length)));
  const ordered = offsets.map((offset, index) => ({ offset: Number.isNaN(offset) ? Infinity : offset, index }))
    .sort((left, right) => left.offset - right.offset);
  const result = new Array<number>(offsets.length);
  let bytes = 0, units = 0;
  for (const target of ordered) {
    while (units < text.length && bytes < target.offset) {
      const next = byteLenAt(text, units);
      bytes += next.bytes; units += next.units;
    }
    result[target.index] = units;
  }
  return result;
}

</storycode:copy>
<a id="f-8f044116e767-byteLength"></a>
### `byteLength`

<storycode:copy id="f-8f044116e767-byteLength">
/** Total UTF-8 byte length of `text`. */
export function byteLength(text: string): number {
  return charToByte(text, text.length);
}
</storycode:copy>

### Assemble `apps/web/src/lib/offsets.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/offsets.ts"><storycode:paste select="#f-8f044116e767-byteLenAt"/><storycode:paste select="#f-8f044116e767-asciiCache"/><storycode:paste select="#f-8f044116e767-ASCII_CACHE_SIZE"/><storycode:paste select="#f-8f044116e767-isAscii"/><storycode:paste select="#f-8f044116e767-charToByte"/><storycode:paste select="#f-8f044116e767-byteToChar"/><storycode:paste select="#f-8f044116e767-bytesToChars"/><storycode:paste select="#f-8f044116e767-byteLength"/></storycode:file>

<a id="output-ff5acfecaec5"></a>
## `apps/web/src/shell/StatusBar.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-ff5acfecaec5-focused"></a>
### `focused`

<storycode:copy id="f-ff5acfecaec5-focused">
import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";

import { StatusBar } from "./StatusBar";

// The caret is read off the focused editor by the bar itself (it polls, so
// the workspace above it never re-renders for a keystroke); the tests hand it
// a stand-in editor, or none.
const focused = vi.hoisted(() => ({ view: null as unknown }));
</storycode:copy>
<a id="f-ff5acfecaec5-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ff5acfecaec5-expression_statement">
vi.mock("../editor/activeEditor", () => ({ focusedEditor: () => focused.view }));
</storycode:copy>
<a id="f-ff5acfecaec5-fakeEditorAt"></a>
### `fakeEditorAt`

<storycode:copy id="f-ff5acfecaec5-fakeEditorAt">
const fakeEditorAt = (line: number, column: number) => ({
  dom: { isConnected: true },
  state: {
    selection: { main: { head: 100 } },
    doc: { lineAt: () => ({ number: line, from: 100 - (column - 1) }) },
  },
});

</storycode:copy>
<a id="f-ff5acfecaec5-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ff5acfecaec5-expression_statement-2">
afterEach(() => {
  cleanup();
  focused.view = null;
});

</storycode:copy>
<a id="f-ff5acfecaec5-bar"></a>
### `bar`

<storycode:copy id="f-ff5acfecaec5-bar">
const bar = (over: Partial<React.ComponentProps<typeof StatusBar>> = {}) =>
  render(
    <StatusBar
      problems={{ errors: 0, warnings: 0 }}
      needsAttention={0}
      path={null}
      zoom={1}
      {...over}
    />,
  );

</storycode:copy>
<a id="f-ff5acfecaec5-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ff5acfecaec5-expression_statement-3">
describe("what the status bar always says", () => {
  it("gives the counts a name, because the digits alone are ambiguous", () => {
    bar({ problems: { errors: 2, warnings: 1 } });
    expect(screen.getByRole("button", { name: "2 errors, 1 warning" })).toBeTruthy();
  });

  it("says so plainly when nothing is wrong", () => {
    bar();
    expect(screen.getByRole("button", { name: "No problems" })).toBeTruthy();
  });

  it("goes to the next problem when the counts are pressed", () => {
    const onProblems = vi.fn();
    bar({ problems: { errors: 1, warnings: 0 }, onProblems });
    fireEvent.click(screen.getByRole("button", { name: "1 error" }));
    expect(onProblems).toHaveBeenCalled();
  });
});

</storycode:copy>
<a id="f-ff5acfecaec5-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-ff5acfecaec5-expression_statement-4">
describe("what it leaves out", () => {
  it("spends no slot on a constant 100% zoom", () => {
    // A status bar that always says "100%" has given a permanent row to
    // something nobody reads.
    bar({ zoom: 1 });
    expect(screen.queryByText("100%")).toBeNull();
    cleanup();
    bar({ zoom: 1.5 });
    expect(screen.getByText("150%")).toBeTruthy();
  });

  it("says nothing about terminals when none are waiting", () => {
    bar({ needsAttention: 0 });
    expect(screen.queryByText(/waiting/)).toBeNull();
  });

  it("shows the waiting count when there is one, and walks the queue", () => {
    const onAttention = vi.fn();
    bar({ needsAttention: 2, onAttention });
    const button = screen.getByRole("button", { name: "2 terminals needing you" });
    fireEvent.click(button);
    expect(onAttention).toHaveBeenCalled();
  });

  it("shows no caret position when nothing is focused", async () => {
    bar();
    await new Promise((r) => requestAnimationFrame(() => r(null)));
    expect(screen.queryByText(/^Ln /)).toBeNull();
  });
});

</storycode:copy>
<a id="f-ff5acfecaec5-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-ff5acfecaec5-expression_statement-5">
describe("what it says about the view", () => {
  it("shows the caret and the path of the focused file", async () => {
    focused.view = fakeEditorAt(42, 7);
    bar({ path: "src/main.rs" });
    expect(await screen.findByText("Ln 42, Col 7")).toBeTruthy();
    expect(screen.getByText("src/main.rs")).toBeTruthy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/shell/StatusBar.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/StatusBar.test.tsx"><storycode:paste select="#f-ff5acfecaec5-focused"/><storycode:paste select="#f-ff5acfecaec5-expression_statement"/><storycode:paste select="#f-ff5acfecaec5-fakeEditorAt"/><storycode:paste select="#f-ff5acfecaec5-expression_statement-2"/><storycode:paste select="#f-ff5acfecaec5-bar"/><storycode:paste select="#f-ff5acfecaec5-expression_statement-3"/><storycode:paste select="#f-ff5acfecaec5-expression_statement-4"/><storycode:paste select="#f-ff5acfecaec5-expression_statement-5"/></storycode:file>

<a id="output-3459ed3c3032"></a>
## `apps/web/src/shell/StatusBar.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-3459ed3c3032-StatusBarProps"></a>
### `StatusBarProps`

<storycode:copy id="f-3459ed3c3032-StatusBarProps">
import { useId } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type React from "react";
// The status bar: the one line that is always true.
//
// A status bar earns its permanent row by answering questions you would
// otherwise have to go and look for, and it loses that row the moment it
// starts showing things nobody reads. So the rule here is that everything on
// it is either a COUNT that changes, or the identity of what you are looking
// at — and nothing is on it merely because there was space.
//
// Left is the state of the work: how much is wrong, and what is running.
// Right is the state of the view: where the caret is, what the file is, how
// far it is zoomed. That split is the one every editor uses, and it is worth
// keeping because it means a glance to one side answers one kind of question.

import { useEffect, useState } from "react";
import { focusedEditor } from "../editor/activeEditor";
import { problemsLabel, type ProblemCounts } from "../lib/problems";
import { zoomLabel } from "../lib/zoom";

export interface StatusBarProps {
  problems: ProblemCounts;
  /** Terminals that want something from a person, right now. */
  needsAttention: number;
  /** The branch, and how much is uncommitted. Absent for a folder that is not
   * a repository, which is entirely normal for a folder of notes. */
  git?: { branch: string; dirty: number } | null;
  /** Show the commit graph. */
  onGit?: () => void;
  /** The focused file's path, or null when nothing is focused. */
  path: string | null;
  /** Controls the workspace wants at hand on the right — the provenance
   * layer toggles. */
  extra?: React.ReactNode;
  /** Whole-window zoom. Shown only when it is not 100%: a status bar that
   * always says "100%" has spent a slot on a constant. */
  zoom: number;
  /** Go to the next problem. */
  onProblems?: () => void;
  /** Walk the attention queue — the same verb ⌘J has. */
  onAttention?: () => void;
}

</storycode:copy>
<a id="f-3459ed3c3032-StatusBar"></a>
### `StatusBar`

<storycode:copy id="f-3459ed3c3032-StatusBar">
export function StatusBar({
  problems,
  needsAttention,
  path,
  zoom,
  git,
  onProblems,
  onAttention,
  onGit,
  extra,
}: StatusBarProps) {
  const parent = `workspace-status:${useId()}`;
  const revision = useUiRevision(() => [problems, needsAttention, path, zoom, git]);
  useUiReading("workspace status", { problems, needsAttention, path, zoom, git: git ?? null });
  const actions = [
    { id: "git", label: "Show status branch history", callback: git ? onGit : undefined, visible: !!git },
    { id: "problems", label: "Show status problems", callback: onProblems, visible: true },
    { id: "attention", label: "Go to next waiting terminal", callback: onAttention, visible: needsAttention > 0 },
  ];
  const invoke = useUiCommands(actions.filter(action => action.visible).map(action => ({ key: `${parent}:${action.id}`, command: {
    parent, subject: parent, label: action.label, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !!action.callback, reason: "No handler is attached to this status control" }),
    execute: async () => { if (!action.callback) return { status: "unavailable", reason: "This status control has no attached handler" }; await action.callback(); return { status: "completed", value: { view_requested: true, scope: "Callback returned; inspect the resulting view" } }; },
  } })));
  const [error, setError] = useState("");
  const act = (id: string) => { void invoke(`${parent}:${id}`).catch(reason => setError(String(reason))); };
  const clean = problems.errors === 0 && problems.warnings === 0;
  return (
    <footer className="status-bar" role="contentinfo">
      {error && <span role="alert">{error}</span>}
      <div className="status-bar__left">
        {/* Leftmost, the way every editor puts it — and only when there IS a
            branch. A permanent "no repository" would spend a slot saying
            nothing. */}
        {git && (
          <button
            type="button"
            className="status-bar__item"
            onClick={() => act("git")} disabled={!onGit}
            data-tip={
              git.dirty > 0
                ? `On ${git.branch} — ${git.dirty} file${git.dirty === 1 ? "" : "s"} changed. Show history.`
                : `On ${git.branch}, nothing uncommitted. Show history.`
            }
            aria-label={`Branch ${git.branch}${git.dirty > 0 ? `, ${git.dirty} changed` : ""}`}
          >
            <span className="status-bar__glyph" aria-hidden>
              ⑂
            </span>
            {git.branch}
            {git.dirty > 0 && (
              <span className="status-bar__dirty">{git.dirty}</span>
            )}
          </button>
        )}
        <button
          type="button"
          className={`status-bar__item${clean ? "" : " status-bar__item--loud"}`}
          onClick={() => act("problems")} disabled={!onProblems}
          data-tip={
            clean
              ? "Nothing is wrong right now"
              : "Show every problem. F8 goes to the next one."
          }
          aria-label={problemsLabel(problems)}
        >
          <span className="status-bar__glyph" aria-hidden>
            ✕
          </span>
          {problems.errors}
          <span className="status-bar__glyph" aria-hidden>
            ⚠
          </span>
          {problems.warnings}
        </button>

        {/* Only when there is something to say. A permanent "0 waiting" is a
            slot spent on a constant. */}
        {needsAttention > 0 && (
          <button
            type="button"
            className="status-bar__item status-bar__item--attention"
            onClick={() => act("attention")} disabled={!onAttention}
            data-tip="Go to the next terminal that needs you"
            aria-label={`${needsAttention} terminal${needsAttention === 1 ? "" : "s"} needing you`}
          >
            <span className="status-bar__glyph" aria-hidden>
              ▮
            </span>
            {needsAttention} waiting
          </button>
        )}
      </div>

      <div className="status-bar__right">
        {extra}
        <CaretPosition />
        {path && (
          <span
            className="status-bar__item status-bar__path mono"
            data-tip={path}
          >
            {path}
          </span>
        )}
        {zoom !== 1 && (
          <span
            className="status-bar__item"
            data-tip="Window zoom — ⌘0 for actual size"
          >
            {zoomLabel(zoom)}
          </span>
        )}
      </div>
    </footer>
  );
}

</storycode:copy>
<a id="f-3459ed3c3032-CaretPosition"></a>
### `CaretPosition`

Where the caret is, polled from the focused editor.
Its own component, on purpose: this is the only thing in the window that
changes on every keystroke, and it is a few characters of text. Held in the
workspace's state it re-rendered every pane, rail and ruler in the window
eight times a second while you typed; held here it re-renders this span.
Polled on a frame rather than hooked into every editor: there is no one
editor to hook, panes come and go, and a status bar that is one frame
behind the caret is indistinguishable from one that is not.

<storycode:copy id="f-3459ed3c3032-CaretPosition">
/**
 * Where the caret is, polled from the focused editor.
 *
 * Its own component, on purpose: this is the only thing in the window that
 * changes on every keystroke, and it is a few characters of text. Held in the
 * workspace's state it re-rendered every pane, rail and ruler in the window
 * eight times a second while you typed; held here it re-renders this span.
 * Polled on a frame rather than hooked into every editor: there is no one
 * editor to hook, panes come and go, and a status bar that is one frame
 * behind the caret is indistinguishable from one that is not.
 */
function CaretPosition() {
  const [caret, setCaret] = useState<{ line: number; column: number } | null>(
    null,
  );
  useEffect(() => {
    let frame: number | null = null;
    const read = () => {
      frame = null;
      const view = focusedEditor();
      if (!view || !view.dom.isConnected) {
        setCaret((current) => (current === null ? current : null));
        return;
      }
      const head = view.state.selection.main.head;
      const line = view.state.doc.lineAt(head);
      const next = { line: line.number, column: head - line.from + 1 };
      // Same place, same object, no render.
      setCaret((current) =>
        current && current.line === next.line && current.column === next.column
          ? current
          : next,
      );
    };
    const tick = () => {
      if (frame === null) frame = requestAnimationFrame(read);
    };
    const timer = window.setInterval(tick, 120);
    tick();
    return () => {
      window.clearInterval(timer);
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, []);
  useUiReading("focused caret position", caret);
  if (!caret) return null;
  return (
    <span className="status-bar__item" data-tip="Line and column of the caret">
      Ln {caret.line}, Col {caret.column}
    </span>
  );
}
</storycode:copy>

### Assemble `apps/web/src/shell/StatusBar.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/StatusBar.tsx"><storycode:paste select="#f-3459ed3c3032-StatusBarProps"/><storycode:paste select="#f-3459ed3c3032-StatusBar"/><storycode:paste select="#f-3459ed3c3032-CaretPosition"/></storycode:file>

<a id="output-22de632562cd"></a>
## `apps/web/src/views/sessionRegistry.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Keystroke Redraws Only The Editor](contract.md).

<a id="f-22de632562cd-session"></a>
### `session`

— a session hook re-runs on every keystroke; the registry must not tell the
workspace about a publish whose every field is the one it already holds.

<storycode:copy id="f-22de632562cd-session">
// Guarantee: docs/guarantees/authoring/a-keystroke-redraws-only-the-editor.md
// — a session hook re-runs on every keystroke; the registry must not tell the
// workspace about a publish whose every field is the one it already holds.
import { describe, expect, it, vi } from "vitest";
import { SessionRegistry, type DocSession } from "./documentSession";

const session = (over: Partial<DocSession> = {}): DocSession =>
  ({ docId: "d1", doc: null, syncState: "idle", ...over }) as unknown as DocSession;

</storycode:copy>
<a id="f-22de632562cd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-22de632562cd-expression_statement">
describe("the session registry", () => {
  it("emits on the first publish and on a real change", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    expect(heard).toHaveBeenCalledTimes(1);
    registry.publish(session({ syncState: "editing" }));
    expect(heard).toHaveBeenCalledTimes(2);
    expect(registry.get("d1")?.syncState).toBe("editing");
  });

  it("stays silent when a fresh object carries the same fields", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    const first = session();
    registry.publish(first);
    const version = registry.version;
    registry.publish(session()); // a new object, the same session
    expect(heard).toHaveBeenCalledTimes(1);
    expect(registry.version).toBe(version);
    // And the held object is the first one: a subscriber reading the snapshot
    // sees a stable identity, which is what lets it skip its own render.
    expect(registry.get("d1")).toBe(first);
  });

  it("notices a field added or removed, not only one replaced", () => {
    const registry = new SessionRegistry();
    const heard = vi.fn();
    registry.subscribe(heard);
    registry.publish(session());
    registry.publish(session({ banner: { kind: "pass", text: "ok" } }));
    expect(heard).toHaveBeenCalledTimes(2);
  });
});
</storycode:copy>

### Assemble `apps/web/src/views/sessionRegistry.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/sessionRegistry.test.ts"><storycode:paste select="#f-22de632562cd-session"/><storycode:paste select="#f-22de632562cd-expression_statement"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/debug/useDebugger.ts](../../../mechanisms/apps/web/src/debug/useDebugger.ts.md#output-597e991b795b)
- [apps/web/src/editor/DocumentEditor.tsx](../../../mechanisms/apps/web/src/editor/DocumentEditor.tsx.md#output-aab82870e517)
- [apps/web/src/editor/EditorRuler.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-75b1988d3607)
- [apps/web/src/editor/embedded.ts](../../../mechanisms/apps/web/src/editor/embedded.ts.md#output-df9508b237a8)
- [apps/web/src/lib/offsets.ts](contract.md#output-8f044116e767)
- [apps/web/src/shell/Ribbons.tsx](../../../mechanisms/apps/web/src/shell/Ribbons.tsx.md#output-b916e85d3b0b)
- [apps/web/src/shell/StatusBar.tsx](contract.md#output-3459ed3c3032)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/documentSession.tsx](../../../mechanisms/apps/web/src/views/documentSession.tsx.md#output-bfb59577f48b)

### Tests and fixtures

- [apps/web/src/editor/embedded.test.ts](../../../shared/files/apps/web/src/editor/embedded.test.ts.md)
- [apps/web/src/lib/offsets.test.ts](../../../mechanisms/apps/web/src/lib/offsets.test.ts.md#output-dea322d4979a)
- [apps/web/src/shell/StatusBar.test.tsx](contract.md#output-ff5acfecaec5)
- [apps/web/src/views/sessionRegistry.test.ts](contract.md#output-22de632562cd)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
