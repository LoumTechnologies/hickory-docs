<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# The Measure Says What It Measures

For: Authors editing executable documents and the files those documents produce.

Given the ruler above a document editor, when it is measuring prose then it
names its unit (`chars/line`), shades the range that is comfortable to read
(45–75), and says in its tooltip that the number is an average of a
proportional face rather than a count of columns. Given a caret inside a
table, the same strip stops measuring prose entirely and names the table's
columns instead.

The existing requirement is retained. Its original detail is below; implementation evidence has not been renewed merely by reorganizing it.

## Read the detailed requirements

- [Requirements 1](requirements/01.md)


Earlier wording and all later notes remain in [the preserved original](../../../support/files/docs/guarantees/authoring/the-measure-says-what-it-measures.md.md). The paragraph ledger records their exact ranges and dispositions.

Requirement decision: Retain the existing requirement without inferring new intent. Separate evidence history and retain follow-up paragraphs for review; no new implementation verification is claimed.

Implementation review: not verified. Executed checks and human judgment are recorded separately. Source and test changes stale earlier evidence.


## Read the retained requirement detail

- [Requirement detail](requirements/01.md)
<storycode:include file="requirements/01.md" />

## Implementation

<a id="output-5e96c7a88c1c"></a>
## `apps/web/src/editor/wrapColumn.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The Measure Says What It Measures](contract.md).

<a id="f-5e96c7a88c1c-WRAP_MIN"></a>
### `WRAP_MIN`

Where prose wraps, and where code refuses to.
These are two different jobs that a single `lineWrapping` cannot do:
- **Prose wants a measure.** A paragraph set across a 2000px window is
unreadable; typography has known that for five centuries, which is why
every word processor has a right margin you can drag.
- **Code wants its lines.** A shell command broken across two rows is a
command you can no longer read as one thing, and an aligned table in a
fenced block is destroyed by wrapping. Code scrolls sideways instead —
and it gets the WHOLE pane to do it in, because the prose measure is
about reading sentences, and a code block is not a sentence.
So the editor wraps by default and code lines opt out, line by line. The
mechanism is deliberately CSS rather than a second editor: `.cm-line` is a
block box, so a `max-width` on it wraps the text inside it at that width,
and a code line that sets `white-space: pre` and drops the cap simply
overflows into the scroller CodeMirror already gives it. Heights are
measured from the DOM either way, so the height map, the gutters, and the
right rail all keep agreeing about where a line is.
The measure is a PIXEL width computed from the view's own character width,
not a `ch` value in CSS. The ruler above the editor has to draw its ticks
and its marker at exactly the same places, and the only way two components
can agree about that is to share one measurement taken from the one element
that has the real font on it.

<storycode:copy id="f-5e96c7a88c1c-WRAP_MIN">
// Where prose wraps, and where code refuses to.
//
// These are two different jobs that a single `lineWrapping` cannot do:
//
//  - **Prose wants a measure.** A paragraph set across a 2000px window is
//    unreadable; typography has known that for five centuries, which is why
//    every word processor has a right margin you can drag.
//  - **Code wants its lines.** A shell command broken across two rows is a
//    command you can no longer read as one thing, and an aligned table in a
//    fenced block is destroyed by wrapping. Code scrolls sideways instead —
//    and it gets the WHOLE pane to do it in, because the prose measure is
//    about reading sentences, and a code block is not a sentence.
//
// So the editor wraps by default and code lines opt out, line by line. The
// mechanism is deliberately CSS rather than a second editor: `.cm-line` is a
// block box, so a `max-width` on it wraps the text inside it at that width,
// and a code line that sets `white-space: pre` and drops the cap simply
// overflows into the scroller CodeMirror already gives it. Heights are
// measured from the DOM either way, so the height map, the gutters, and the
// right rail all keep agreeing about where a line is.
//
// The measure is a PIXEL width computed from the view's own character width,
// not a `ch` value in CSS. The ruler above the editor has to draw its ticks
// and its marker at exactly the same places, and the only way two components
// can agree about that is to share one measurement taken from the one element
// that has the real font on it.

import { StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin } from "@codemirror/view";
import type { DecorationSet, ViewUpdate } from "@codemirror/view";

/** Narrower than this and a sentence is a column of words. */
export const WRAP_MIN = 30;
</storycode:copy>
<a id="f-5e96c7a88c1c-WRAP_MAX"></a>
### `WRAP_MAX`

Wider than this and the measure has stopped meaning anything.

<storycode:copy id="f-5e96c7a88c1c-WRAP_MAX">
/** Wider than this and the measure has stopped meaning anything. */
export const WRAP_MAX = 200;
</storycode:copy>
<a id="f-5e96c7a88c1c-WRAP_DEFAULT"></a>
### `WRAP_DEFAULT`

The measure most prose is set at, and what a document opens with.

<storycode:copy id="f-5e96c7a88c1c-WRAP_DEFAULT">
/** The measure most prose is set at, and what a document opens with. */
export const WRAP_DEFAULT = 80;

</storycode:copy>
<a id="f-5e96c7a88c1c-clampWrapColumn"></a>
### `clampWrapColumn`

A column count that is safe to lay out with, whatever it came from.

<storycode:copy id="f-5e96c7a88c1c-clampWrapColumn">
/** A column count that is safe to lay out with, whatever it came from. */
export function clampWrapColumn(value: unknown): number {
  const n = typeof value === "number" ? Math.round(value) : Number.NaN;
  if (!Number.isFinite(n)) return WRAP_DEFAULT;
  return Math.min(WRAP_MAX, Math.max(WRAP_MIN, n));
}

</storycode:copy>
<a id="f-5e96c7a88c1c-setWrapColumn"></a>
### `setWrapColumn`

<storycode:copy id="f-5e96c7a88c1c-setWrapColumn">
/** Move the prose measure to this column. */
export const setWrapColumn = StateEffect.define<number>();

</storycode:copy>
<a id="f-5e96c7a88c1c-wrapColumnField"></a>
### `wrapColumnField`

<storycode:copy id="f-5e96c7a88c1c-wrapColumnField">
/** The current measure, in columns. */
export const wrapColumnField = StateField.define<number>({
  create: () => WRAP_DEFAULT,
  update(value, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setWrapColumn)) return clampWrapColumn(effect.value);
    }
    return value;
  },
});

</storycode:copy>
<a id="f-5e96c7a88c1c-wrapColumnOf"></a>
### `wrapColumnOf`

The measure of a state, for anything drawing beside the editor.

<storycode:copy id="f-5e96c7a88c1c-wrapColumnOf">
/** The measure of a state, for anything drawing beside the editor. */
export function wrapColumnOf(state: EditorState): number {
  return state.field(wrapColumnField, false) ?? WRAP_DEFAULT;
}

</storycode:copy>
<a id="f-5e96c7a88c1c-wrapPixels"></a>
### `wrapPixels`

Where the measure falls, in pixels from the left edge of the text.
`defaultCharacterWidth` is measured by CodeMirror against the content
element's real font, which is what makes the dotted line in the editor and
the marker on the ruler land on the same pixel.

<storycode:copy id="f-5e96c7a88c1c-wrapPixels">
/**
 * Where the measure falls, in pixels from the left edge of the text.
 *
 * `defaultCharacterWidth` is measured by CodeMirror against the content
 * element's real font, which is what makes the dotted line in the editor and
 * the marker on the ruler land on the same pixel.
 */
export function wrapPixels(view: EditorView, column = wrapColumnOf(view.state)): number {
  return view.defaultCharacterWidth * clampWrapColumn(column);
}

</storycode:copy>
<a id="f-5e96c7a88c1c-CodeRanges"></a>
### `CodeRanges`

Which byte ranges of this buffer are code — the lines that must not wrap.

<storycode:copy id="f-5e96c7a88c1c-CodeRanges">
/** Which byte ranges of this buffer are code — the lines that must not wrap. */
export type CodeRanges = (state: EditorState) => readonly (readonly [number, number])[];

</storycode:copy>
<a id="f-5e96c7a88c1c-codeLine"></a>
### `codeLine`

<storycode:copy id="f-5e96c7a88c1c-codeLine">
const codeLine = Decoration.line({ class: "cm-code-line" });

</storycode:copy>
<a id="f-5e96c7a88c1c-buildCodeLines"></a>
### `buildCodeLines`

<storycode:copy id="f-5e96c7a88c1c-buildCodeLines">
function buildCodeLines(state: EditorState, ranges: CodeRanges): DecorationSet {
  const spans = ranges(state);
  if (spans.length === 0) return Decoration.none;
  const marked: Range<Decoration>[] = [];
  const seen = new Set<number>();
  for (const [from, to] of spans) {
    if (to <= from) continue;
    const first = state.doc.lineAt(Math.max(0, Math.min(from, state.doc.length)));
    const last = state.doc.lineAt(Math.max(0, Math.min(to, state.doc.length)));
    for (let n = first.number; n <= last.number; n++) {
      if (seen.has(n)) continue;
      seen.add(n);
      marked.push(codeLine.range(state.doc.line(n).from));
    }
  }
  marked.sort((a, b) => a.from - b.from);
  return Decoration.set(marked, true);
}

</storycode:copy>
<a id="f-5e96c7a88c1c-proseWrap"></a>
### `proseWrap`

The prose-measure extension.
Two halves that have to stay in step: a StateField marking which lines are
code (vertical layout depends on it, so it cannot be a plugin), and a
ViewPlugin writing the measure onto the editor as a custom property.
The property goes on `view.dom` rather than into a stylesheet because it
is per-EDITOR: two documents open side by side may be set at different
measures, and a rule in styles.css could only say one thing for both.

<storycode:copy id="f-5e96c7a88c1c-proseWrap">
/**
 * The prose-measure extension.
 *
 * Two halves that have to stay in step: a StateField marking which lines are
 * code (vertical layout depends on it, so it cannot be a plugin), and a
 * ViewPlugin writing the measure onto the editor as a custom property.
 *
 * The property goes on `view.dom` rather than into a stylesheet because it
 * is per-EDITOR: two documents open side by side may be set at different
 * measures, and a rule in styles.css could only say one thing for both.
 */
export function proseWrap(codeRanges: CodeRanges): Extension {
  return [
    wrapColumnField,
    EditorView.lineWrapping,
    StateField.define<DecorationSet>({
      create: (state) => buildCodeLines(state, codeRanges),
      update: (value, tr) => (tr.docChanged ? buildCodeLines(tr.state, codeRanges) : value),
      provide: (f) => EditorView.decorations.from(f),
    }),
    ViewPlugin.define((view) => {
      const write = () => {
        view.dom.style.setProperty("--prose-wrap", `${wrapPixels(view)}px`);
        // The gutters' real width, for anything that needs to draw INTO that
        // lane. A rendered table is a fold, so the gutter has one number for
        // the whole block and cannot label its rows; the grid draws its own
        // row lane and pulls it left by this much. Written here rather than
        // measured separately because this is already the callback that fires
        // when the font size or the line count changes the gutter's width.
        const gutters = view.dom.querySelector(".cm-gutters");
        const width = gutters ? gutters.getBoundingClientRect().width : 0;
        view.dom.style.setProperty("--cm-gutter-w", `${width}px`);
      };
      write();
      return {
        update(update: ViewUpdate) {
          // The character width changes with the font size, which is what the
          // zoom control moves; the column changes when the ruler is dragged.
          if (
            update.geometryChanged ||
            update.transactions.some((tr) => tr.effects.some((e) => e.is(setWrapColumn)))
          ) {
            write();
          }
        },
      };
    }),
  ];
}
</storycode:copy>

### Assemble `apps/web/src/editor/wrapColumn.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/wrapColumn.ts"><storycode:paste select="#f-5e96c7a88c1c-WRAP_MIN"/><storycode:paste select="#f-5e96c7a88c1c-WRAP_MAX"/><storycode:paste select="#f-5e96c7a88c1c-WRAP_DEFAULT"/><storycode:paste select="#f-5e96c7a88c1c-clampWrapColumn"/><storycode:paste select="#f-5e96c7a88c1c-setWrapColumn"/><storycode:paste select="#f-5e96c7a88c1c-wrapColumnField"/><storycode:paste select="#f-5e96c7a88c1c-wrapColumnOf"/><storycode:paste select="#f-5e96c7a88c1c-wrapPixels"/><storycode:paste select="#f-5e96c7a88c1c-CodeRanges"/><storycode:paste select="#f-5e96c7a88c1c-codeLine"/><storycode:paste select="#f-5e96c7a88c1c-buildCodeLines"/><storycode:paste select="#f-5e96c7a88c1c-proseWrap"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/editor/EditorRuler.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-75b1988d3607)
- [apps/web/src/editor/wrapColumn.ts](contract.md#output-5e96c7a88c1c)
- [apps/web/src/styles.css](../../../mechanisms/apps/web/src/styles.css.md#output-8695905e2358)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/e2e/startup.spec.ts](../../../mechanisms/apps/web/e2e/startup.spec.ts.md#output-4e63365d81de)
- [apps/web/src/App.test.tsx](../../../mechanisms/apps/web/src/App.tsx.md#output-65ac24c24624)
- [apps/web/src/editor/EditorRuler.test.tsx](../../../mechanisms/apps/web/src/editor/EditorRuler.tsx.md#output-a0ab5d3c94e1)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
