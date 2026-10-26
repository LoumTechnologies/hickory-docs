<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/changeFlash.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f289f35509d4"></a>
## `apps/web/src/editor/changeFlash.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-f289f35509d4-FlashRange"></a>
### `FlashRange`

A fading highlight on text that changed because of an edit somewhere else.
Both directions of the two-way edit flow end with text appearing in a
buffer the user was not typing in: an output edit resolves into the
document, and a document edit re-weaves into every generated pane. The
update itself is silent — the flash is the sign that says WHAT moved, and
then gets out of the way (~1.2s, CSS-animated fade, self-removing).
Mechanics: `flashSpans` installs the field on first use (the same
appendConfig trick rangeHighlight.ts uses for the document editor), adds a
batch of mark decorations, and schedules that batch's removal after the
fade completes. Ranges map through document changes in between, so a flash
stays on its text while the user keeps typing; a batch is dropped whole
when its timer fires, never someone else's still-fading batch.

<storycode:copy id="f-f289f35509d4-FlashRange">
// A fading highlight on text that changed because of an edit somewhere else.
//
// Both directions of the two-way edit flow end with text appearing in a
// buffer the user was not typing in: an output edit resolves into the
// document, and a document edit re-weaves into every generated pane. The
// update itself is silent — the flash is the sign that says WHAT moved, and
// then gets out of the way (~1.2s, CSS-animated fade, self-removing).
//
// Mechanics: `flashSpans` installs the field on first use (the same
// appendConfig trick rangeHighlight.ts uses for the document editor), adds a
// batch of mark decorations, and schedules that batch's removal after the
// fade completes. Ranges map through document changes in between, so a flash
// stays on its text while the user keeps typing; a batch is dropped whole
// when its timer fires, never someone else's still-fading batch.

import { StateEffect, StateField } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { computeEdits } from "../lib/diff";

export interface FlashRange {
  from: number;
  to: number;
}

</storycode:copy>
<a id="f-f289f35509d4-FLASH_MS"></a>
### `FLASH_MS`

How long a flash lives. The CSS animation is 1.2s; the decorations come
off a beat later so the fade is never cut short.

<storycode:copy id="f-f289f35509d4-FLASH_MS">
/** How long a flash lives. The CSS animation is 1.2s; the decorations come
 * off a beat later so the fade is never cut short. */
export const FLASH_MS = 1400;

</storycode:copy>
<a id="f-f289f35509d4-nextBatch"></a>
### `nextBatch`

<storycode:copy id="f-f289f35509d4-nextBatch">
let nextBatch = 1;

</storycode:copy>
<a id="f-f289f35509d4-addFlash"></a>
### `addFlash`

Add a batch of flash marks. Positions map through concurrent changes.

<storycode:copy id="f-f289f35509d4-addFlash">
/** Add a batch of flash marks. Positions map through concurrent changes. */
export const addFlash = StateEffect.define<{ batch: number; ranges: FlashRange[] }>({
  map: (value, mapping) => ({
    batch: value.batch,
    ranges: value.ranges.map((r) => ({
      from: mapping.mapPos(r.from, 1),
      to: mapping.mapPos(r.to, -1),
    })),
  }),
});

</storycode:copy>
<a id="f-f289f35509d4-expireFlash"></a>
### `expireFlash`

Remove one batch's marks (its fade has finished).

<storycode:copy id="f-f289f35509d4-expireFlash">
/** Remove one batch's marks (its fade has finished). */
export const expireFlash = StateEffect.define<number>();

</storycode:copy>
<a id="f-f289f35509d4-changeFlashField"></a>
### `changeFlashField`

<storycode:copy id="f-f289f35509d4-changeFlashField">
export const changeFlashField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) {
      if (e.is(addFlash)) {
        const max = tr.state.doc.length;
        const marks = [];
        for (const r of e.value.ranges) {
          const from = Math.max(0, Math.min(r.from, max));
          const to = Math.max(0, Math.min(r.to, max));
          if (to > from) {
            marks.push(
              Decoration.mark({
                class: "cm-change-flash",
                batch: e.value.batch,
              }).range(from, to),
            );
          }
        }
        if (marks.length > 0) deco = deco.update({ add: marks, sort: true });
      } else if (e.is(expireFlash)) {
        const batch = e.value;
        deco = deco.update({ filter: (_f, _t, value) => value.spec.batch !== batch });
      }
    }
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

</storycode:copy>
<a id="f-f289f35509d4-flashSpans"></a>
### `flashSpans`

Flash `ranges` in `view`: install the field if this editor has never
flashed, add the marks, and remove them once the fade is over.
Empty and collapsed ranges are dropped; a call with nothing visible
dispatches nothing.

<storycode:copy id="f-f289f35509d4-flashSpans">
/**
 * Flash `ranges` in `view`: install the field if this editor has never
 * flashed, add the marks, and remove them once the fade is over.
 *
 * Empty and collapsed ranges are dropped; a call with nothing visible
 * dispatches nothing.
 */
export function flashSpans(view: EditorView, ranges: FlashRange[]): void {
  const visible = ranges.filter((r) => r.to > r.from);
  if (visible.length === 0) return;
  const batch = nextBatch++;
  const effects: StateEffect<unknown>[] = [addFlash.of({ batch, ranges: visible })];
  if (!view.state.field(changeFlashField, false)) {
    effects.unshift(StateEffect.appendConfig.of(changeFlashField));
  }
  view.dispatch({ effects });
  setTimeout(() => {
    // The view may be gone by now (tab closed mid-fade); a dispatch into a
    // destroyed view throws, and a flash is never worth an error.
    try {
      view.dispatch({ effects: expireFlash.of(batch) });
    } catch {
      /* view destroyed */
    }
  }, FLASH_MS);
}

</storycode:copy>
<a id="f-f289f35509d4-syncAndFlash"></a>
### `syncAndFlash`

Bring `view`'s buffer to `content` with minimal edits, flashing what
arrived. Returns whether anything changed.
This is the receiving end of a change made SOMEWHERE ELSE (the up-loop
re-wove this pane's file). The own-edit guard is the diff itself: content
identical to the buffer — the round-trip of what the user just typed here —
produces no edits, so it neither disturbs the cursor nor flashes.
The dispatch carries no user event, so an `onLocalEdit`-style listener
keyed on input/delete/undo will not echo it back to the server.

<storycode:copy id="f-f289f35509d4-syncAndFlash">
/**
 * Bring `view`'s buffer to `content` with minimal edits, flashing what
 * arrived. Returns whether anything changed.
 *
 * This is the receiving end of a change made SOMEWHERE ELSE (the up-loop
 * re-wove this pane's file). The own-edit guard is the diff itself: content
 * identical to the buffer — the round-trip of what the user just typed here —
 * produces no edits, so it neither disturbs the cursor nor flashes.
 *
 * The dispatch carries no user event, so an `onLocalEdit`-style listener
 * keyed on input/delete/undo will not echo it back to the server.
 */
export function syncAndFlash(view: EditorView, content: string): boolean {
  const edits = computeEdits(view.state.doc.toString(), content);
  if (edits.length === 0) return false;
  view.dispatch({
    changes: edits.map((e) => ({ from: e.start, to: e.end, insert: e.text })),
  });
  // Where each edit's new text sits after ALL edits applied: earlier edits
  // shift later ones by their length delta (edits are ascending, disjoint).
  const ranges: FlashRange[] = [];
  let delta = 0;
  for (const e of edits) {
    const from = e.start + delta;
    ranges.push({ from, to: from + e.text.length });
    delta += e.text.length - (e.end - e.start);
  }
  flashSpans(view, ranges);
  return true;
}
</storycode:copy>

### Assemble `apps/web/src/editor/changeFlash.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/changeFlash.ts"><storycode:paste select="#f-f289f35509d4-FlashRange"/><storycode:paste select="#f-f289f35509d4-FLASH_MS"/><storycode:paste select="#f-f289f35509d4-nextBatch"/><storycode:paste select="#f-f289f35509d4-addFlash"/><storycode:paste select="#f-f289f35509d4-expireFlash"/><storycode:paste select="#f-f289f35509d4-changeFlashField"/><storycode:paste select="#f-f289f35509d4-flashSpans"/><storycode:paste select="#f-f289f35509d4-syncAndFlash"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d1a2cb215410"></a>
## `apps/web/src/editor/changeFlash.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d1a2cb215410-editor"></a>
### `editor`

@vitest-environment jsdom
The fading change-flash: marks on text that changed because of an edit
somewhere else. What matters: the marks appear where asked, follow the
text through further edits, come off on their own once the fade is over —
and NEVER appear for content identical to the buffer, which is how the
user's own just-typed edit round-tripping through the server stays silent.

<storycode:copy id="f-d1a2cb215410-editor">
// @vitest-environment jsdom
//
// The fading change-flash: marks on text that changed because of an edit
// somewhere else. What matters: the marks appear where asked, follow the
// text through further edits, come off on their own once the fade is over —
// and NEVER appear for content identical to the buffer, which is how the
// user's own just-typed edit round-tripping through the server stays silent.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { EditorState, type Extension } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { FLASH_MS, changeFlashField, flashSpans, syncAndFlash } from "./changeFlash";

function editor(doc: string, extensions: Extension = []) {
  return new EditorView({
    parent: document.body,
    state: EditorState.create({ doc, extensions }),
  });
}

</storycode:copy>
<a id="f-d1a2cb215410-flashed"></a>
### `flashed`

<storycode:copy id="f-d1a2cb215410-flashed">
/** The flashed slices of the buffer, in order. */
function flashed(view: EditorView): string[] {
  const field = view.state.field(changeFlashField, false);
  if (!field) return [];
  const out: string[] = [];
  const cursor = field.iter();
  while (cursor.value) {
    out.push(view.state.doc.sliceString(cursor.from, cursor.to));
    cursor.next();
  }
  return out;
}

</storycode:copy>
<a id="f-d1a2cb215410-views"></a>
### `views`

<storycode:copy id="f-d1a2cb215410-views">
let views: EditorView[] = [];
</storycode:copy>
<a id="f-d1a2cb215410-track"></a>
### `track`

<storycode:copy id="f-d1a2cb215410-track">
const track = (v: EditorView) => (views.push(v), v);

</storycode:copy>
<a id="f-d1a2cb215410-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d1a2cb215410-expression_statement">
beforeEach(() => vi.useFakeTimers());
</storycode:copy>
<a id="f-d1a2cb215410-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d1a2cb215410-expression_statement-2">
afterEach(() => {
  for (const v of views.splice(0)) v.destroy();
  vi.useRealTimers();
});

</storycode:copy>
<a id="f-d1a2cb215410-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-d1a2cb215410-expression_statement-3">
describe("flashSpans", () => {
  it("marks the given ranges, installing the field on first use", () => {
    const view = track(editor("hello woven world"));
    expect(view.state.field(changeFlashField, false)).toBeUndefined();
    flashSpans(view, [{ from: 6, to: 11 }]);
    expect(flashed(view)).toEqual(["woven"]);
    expect(view.dom.querySelector(".cm-change-flash")?.textContent).toBe("woven");
  });

  it("drops collapsed and empty ranges without dispatching", () => {
    const view = track(editor("abc"));
    flashSpans(view, [{ from: 1, to: 1 }]);
    flashSpans(view, []);
    expect(view.state.field(changeFlashField, false)).toBeUndefined();
  });

  it("maps the marks through edits made while they fade", () => {
    const view = track(editor("one two three"));
    flashSpans(view, [{ from: 4, to: 7 }]); // "two"
    view.dispatch({ changes: { from: 0, insert: "zero " } });
    expect(flashed(view)).toEqual(["two"]);
  });

  it("removes its marks once the fade is over", () => {
    const view = track(editor("fading text"));
    flashSpans(view, [{ from: 0, to: 6 }]);
    expect(flashed(view)).toEqual(["fading"]);
    vi.advanceTimersByTime(FLASH_MS + 1);
    expect(flashed(view)).toEqual([]);
  });

  it("expires each batch alone: an older flash going out never takes a newer one with it", () => {
    const view = track(editor("first second"));
    flashSpans(view, [{ from: 0, to: 5 }]);
    vi.advanceTimersByTime(FLASH_MS / 2);
    flashSpans(view, [{ from: 6, to: 12 }]);
    vi.advanceTimersByTime(FLASH_MS / 2 + 1); // first batch expires
    expect(flashed(view)).toEqual(["second"]);
    vi.advanceTimersByTime(FLASH_MS); // second batch expires
    expect(flashed(view)).toEqual([]);
  });

  it("survives the view being destroyed mid-fade", () => {
    const view = editor("short lived");
    flashSpans(view, [{ from: 0, to: 5 }]);
    view.destroy();
    expect(() => vi.advanceTimersByTime(FLASH_MS + 1)).not.toThrow();
  });
});

</storycode:copy>
<a id="f-d1a2cb215410-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-d1a2cb215410-expression_statement-4">
describe("syncAndFlash", () => {
  it("updates the buffer to the incoming content and flashes what arrived", () => {
    const view = track(editor("line one\nline two\n"));
    const changed = syncAndFlash(view, "line one\nline 2 now\nline three\n");
    expect(changed).toBe(true);
    expect(view.state.doc.toString()).toBe("line one\nline 2 now\nline three\n");
    // Every flashed slice is text the sync itself introduced.
    expect(flashed(view).join("")).toContain("2 now");
  });

  it("flashes nothing for content identical to the buffer — the own-edit round-trip", () => {
    const view = track(editor("typed right here\n"));
    const changed = syncAndFlash(view, "typed right here\n");
    expect(changed).toBe(false);
    expect(flashed(view)).toEqual([]);
  });

  it("does not read as a user edit, so a save loop keyed on input events stays quiet", () => {
    let userEdits = 0;
    const view = track(
      editor("before\n", [
        EditorView.updateListener.of((u) => {
          // The same guard OutputEditorPane uses before calling onLocalEdit.
          if (
            u.docChanged &&
            u.transactions.some((t) => t.isUserEvent("input") || t.isUserEvent("delete"))
          ) {
            userEdits += 1;
          }
        }),
      ]),
    );
    syncAndFlash(view, "after\n");
    expect(view.state.doc.toString()).toBe("after\n");
    expect(userEdits).toBe(0);
  });

  it("keeps the cursor where the user left it when the change is elsewhere", () => {
    const view = track(editor("alpha\nbeta\ngamma\n"));
    view.dispatch({ selection: { anchor: 3 } }); // inside "alpha"
    syncAndFlash(view, "alpha\nbeta\ngamma\ndelta\n");
    expect(view.state.selection.main.head).toBe(3);
  });
});
</storycode:copy>

### Assemble `apps/web/src/editor/changeFlash.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/changeFlash.test.ts"><storycode:paste select="#f-d1a2cb215410-editor"/><storycode:paste select="#f-d1a2cb215410-flashed"/><storycode:paste select="#f-d1a2cb215410-views"/><storycode:paste select="#f-d1a2cb215410-track"/><storycode:paste select="#f-d1a2cb215410-expression_statement"/><storycode:paste select="#f-d1a2cb215410-expression_statement-2"/><storycode:paste select="#f-d1a2cb215410-expression_statement-3"/><storycode:paste select="#f-d1a2cb215410-expression_statement-4"/></storycode:file>


</storycode:doc>
