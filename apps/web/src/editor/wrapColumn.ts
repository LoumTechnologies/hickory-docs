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
/** Wider than this and the measure has stopped meaning anything. */
export const WRAP_MAX = 200;
/** The measure most prose is set at, and what a document opens with. */
export const WRAP_DEFAULT = 80;

/** A column count that is safe to lay out with, whatever it came from. */
export function clampWrapColumn(value: unknown): number {
  const n = typeof value === "number" ? Math.round(value) : Number.NaN;
  if (!Number.isFinite(n)) return WRAP_DEFAULT;
  return Math.min(WRAP_MAX, Math.max(WRAP_MIN, n));
}

/** Move the prose measure to this column. */
export const setWrapColumn = StateEffect.define<number>();

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

/** The measure of a state, for anything drawing beside the editor. */
export function wrapColumnOf(state: EditorState): number {
  return state.field(wrapColumnField, false) ?? WRAP_DEFAULT;
}

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

/** Which byte ranges of this buffer are code — the lines that must not wrap. */
export type CodeRanges = (state: EditorState) => readonly (readonly [number, number])[];

const codeLine = Decoration.line({ class: "cm-code-line" });

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
