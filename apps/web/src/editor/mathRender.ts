// LaTeX, drawn where it was written.
//
// KaTeX is imported LAZILY, and for the same reason mermaid is: the marketing
// site builds from this source tree, and a page that shows no equations must
// not carry a typesetting engine and its fonts. The stylesheet comes along on
// the same dynamic import, so the fonts are code-split with the code that
// needs them.
//
// The rendered form REPLACES its source, inline, exactly the way a task
// checkbox does — and it steps aside for the caret on the same reasoning:
// an equation you cannot see the source of is an equation you cannot fix.
// That is also, deliberately, the whole "switch between source and rendered"
// gesture for inline maths: put the caret in it. A `<hick:math>` BLOCK gets
// the heavier treatment, a rail icon beside it, because a block is a
// paragraph the reader may want in source form without moving the cursor
// into it (see editor/rendered.ts and editor/cards.ts).
//
// Display maths whose `$$` markers own their lines is a block replacement —
// a fold, taking the rows it occupies rather than adding one — so the gutters
// keep counting truthfully. Display maths written mid-line renders inline
// instead; a block replacement that does not cover whole lines is not
// something CodeMirror can take out of the height map.

import { StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension, Range } from "@codemirror/state";
import { Decoration, EditorView, ViewPlugin, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";

import { mathSpans } from "../lib/math";
import type { MathSpan } from "../lib/math";

type Katex = { renderToString(tex: string, options?: object): string };

let katexPromise: Promise<Katex> | null = null;

/** The engine, loaded once per page. */
function katex(): Promise<Katex> {
  if (!katexPromise) {
    katexPromise = (async () => {
      // The stylesheet is part of the engine: KaTeX's output is a pile of
      // spans that means nothing without it.
      await import("katex/dist/katex.min.css");
      return (await import("katex")).default as unknown as Katex;
    })();
  }
  return katexPromise;
}

/**
 * Typeset `source` into `el`, replacing whatever is there.
 *
 * Never throws and never leaves the element empty: half-typed LaTeX is the
 * common case while someone is writing, not an exception, so a source that
 * does not parse yet keeps showing its own text with an error tone rather
 * than vanishing. `throwOnError: false` lets KaTeX draw the parts it did
 * understand, which is far more useful mid-edit than an empty box.
 */
export async function renderMathInto(
  el: HTMLElement,
  source: string,
  display: boolean,
): Promise<void> {
  const tex = source.trim();
  if (!tex) return;
  try {
    const engine = await katex();
    el.innerHTML = engine.renderToString(tex, {
      displayMode: display,
      throwOnError: false,
      output: "html",
      // The document is the app's theme; an equation that ignored it would be
      // black type in a dark document.
      trust: false,
    });
    el.classList.remove("cm-math--unparsed");
  } catch (error) {
    // Reaching here means the engine itself failed to load, not that the
    // LaTeX was wrong — `throwOnError: false` handles that above.
    el.textContent = tex;
    el.classList.add("cm-math--unparsed");
    // `data-tip`, never `title`: every hover hint in this app is drawn by the
    // themed tooltip layer. See
    // docs/guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme.md.
    el.dataset.tip = error instanceof Error ? error.message : String(error);
  }
}

class MathWidget extends WidgetType {
  constructor(
    private readonly source: string,
    private readonly display: boolean,
    private readonly block: boolean,
  ) {
    super();
  }

  // Identity is the LaTeX and how it is set — never the position. Typing a
  // paragraph above an equation moves it, and re-typesetting every equation
  // below the caret on each keystroke is work with no visible result.
  eq(other: MathWidget) {
    return (
      other.source === this.source &&
      other.display === this.display &&
      other.block === this.block
    );
  }

  toDOM() {
    const el = document.createElement(this.block ? "div" : "span");
    el.className = `cm-math${this.display ? " cm-math--display" : ""}`;
    el.style.touchAction = "manipulation";
    // The source stands in until the engine arrives, so nothing flashes
    // empty on a cold load and a page with no KaTeX still says something
    // true.
    el.textContent = this.source.trim();
    void renderMathInto(el, this.source, this.display);
    return el;
  }

  get estimatedHeight() {
    return this.block ? 48 : -1;
  }

  ignoreEvent() {
    return true;
  }
}

/** Where the maths of this buffer is. The `.md` editor passes its own
 * verbatim ranges so `$PATH` inside a cell stays a shell variable. */
export type MathSource = (state: EditorState) => readonly MathSpan[];

/** The default source: scan the whole buffer as prose. */
export const proseMath: MathSource = (state) => mathSpans(state.doc.toString());

/** The lines any cursor or selection currently touches. */
function activeLines(state: EditorState): Set<number> {
  const lines = new Set<number>();
  for (const range of state.selection.ranges) {
    const first = state.doc.lineAt(range.from).number;
    const last = state.doc.lineAt(range.to).number;
    for (let n = first; n <= last; n++) lines.add(n);
  }
  return lines;
}

/** Whether a span begins at the start of a line and ends at the end of one —
 * the condition for taking its rows out of the height map. */
export function ownsItsLines(state: EditorState, span: MathSpan): boolean {
  const first = state.doc.lineAt(span.from);
  const last = state.doc.lineAt(span.to);
  return first.from === span.from && last.to === span.to && last.to > first.from;
}

function buildMath(state: EditorState, source: MathSource): DecorationSet {
  const spans = source(state);
  if (spans.length === 0) return Decoration.none;
  const live = activeLines(state);
  const ranges: Range<Decoration>[] = [];
  for (const span of spans) {
    const first = state.doc.lineAt(span.from).number;
    const last = state.doc.lineAt(span.to).number;
    let touched = false;
    for (let n = first; n <= last && !touched; n++) touched = live.has(n);
    const block = span.display && ownsItsLines(state, span);
    if (state.field(mathSourceModes, false)?.get(span.from) ?? touched) {
      if (block) {
        for (let n = first; n <= last; n++) {
          ranges.push(Decoration.line({ attributes: { style: "text-align:center" } }).range(state.doc.line(n).from));
        }
      }
      continue;
    }
    ranges.push(
      Decoration.replace({
        widget: new MathWidget(span.source, span.display, block),
        block,
      }).range(span.from, span.to),
    );
  }
  return Decoration.set(ranges, true);
}

/**
 * The rendered-maths extension.
 *
 * A StateField, not a ViewPlugin: display maths changes vertical layout, and
 * only field-provided decorations are folded into the same update CodeMirror
 * measures its height map in.
 */
export const setMathSource = StateEffect.define<{ at: number; source: boolean }>();
export const mathSourceModes = StateField.define<ReadonlyMap<number, boolean>>({
  create: () => new Map(),
  update(value, tr) {
    const next = new Map([...value].filter(([, source]) => source || !tr.docChanged).map(([at, source]) => [tr.changes.mapPos(at, 1), source]));
    for (const effect of tr.effects) if (effect.is(setMathSource)) next.set(effect.value.at, effect.value.source);
    return next;
  },
});
export function renderedMath(source: MathSource = proseMath): Extension {
  // The capture listener also sees events inside replacement widgets, whose
  // ordinary events are intentionally ignored by CodeMirror.
  const activation = ViewPlugin.define(view => {
    function toggle(event: MouseEvent | PointerEvent): boolean {
      const target = event.target;
      if (!(target instanceof Element)) return false;
      const widget = target.closest(".cm-math");
      const pos = widget ? view.posAtDOM(widget) : view.posAtCoords({ x: event.clientX, y: event.clientY }, false);
      if (pos == null) return false;
      const state = view.state;
      const spans = source(state);
      const span = spans.find(span => span.from <= pos && pos < span.to)
        ?? (widget ? spans.find(span => span.to === pos) : undefined);
      if (!span) return false;
      const mode = state.field(mathSourceModes).get(span.from);
      if (!widget && mode !== true && !activeLines(state).has(state.doc.lineAt(pos).number)) return false;
      event.preventDefault(); event.stopPropagation();
      const bounds = widget?.querySelector(".katex")?.getBoundingClientRect() ?? widget?.getBoundingClientRect();
      const sourceAt = span.source && state.sliceDoc(span.from, span.to).indexOf(span.source);
      const body = span.source.trim();
      const leading = span.source.length - span.source.trimStart().length;
      const ratio = bounds?.width ? Math.max(0, Math.min(1, (event.clientX - bounds.left) / bounds.width)) : 0.5;
      const characters = typeof Intl.Segmenter === "function"
        ? [...new Intl.Segmenter(undefined, { granularity: "grapheme" }).segment(body)].map(item => item.segment)
        : [...body];
      const caret = widget && typeof sourceAt === "number" && sourceAt >= 0
        ? span.from + sourceAt + leading + characters.slice(0, Math.round(ratio * characters.length)).join("").length
        : undefined;
      view.dispatch({ effects: setMathSource.of({ at: span.from, source: !!widget }), ...(caret === undefined ? {} : { selection: { anchor: caret } }) });
      if (widget) view.focus();
      return true;
    }
    let touch: { x: number; y: number; time: number } | undefined;
    let start: { x: number; y: number; id: number } | undefined;
    let lastTouchAt = -1000;
    const doubleClick = (event: MouseEvent) => { if (performance.now() - lastTouchAt >= 400) toggle(event); };
    const down = (event: PointerEvent) => {
      if (event.pointerType === "touch") start = { x: event.clientX, y: event.clientY, id: event.pointerId };
    };
    const up = (event: PointerEvent) => {
      if (start?.id !== event.pointerId) return;
      const moved = Math.hypot(event.clientX - start.x, event.clientY - start.y);
      start = undefined;
      if (moved > 12) { touch = undefined; return; }
      const now = performance.now(), previous = touch;
      lastTouchAt = now;
      touch = { x: event.clientX, y: event.clientY, time: now };
      if (previous && now - previous.time < 350 &&
          Math.hypot(event.clientX - previous.x, event.clientY - previous.y) < 24 && toggle(event)) touch = undefined;
    };
    const cancel = () => { start = undefined; touch = undefined; };
    view.dom.addEventListener("dblclick", doubleClick, true);
    view.dom.addEventListener("pointerdown", down, true);
    view.dom.addEventListener("pointerup", up, true);
    view.dom.addEventListener("pointercancel", cancel, true);
    return { destroy() {
      view.dom.removeEventListener("dblclick", doubleClick, true);
      view.dom.removeEventListener("pointerdown", down, true);
      view.dom.removeEventListener("pointerup", up, true);
      view.dom.removeEventListener("pointercancel", cancel, true);
    } };
  });
  return [mathSourceModes, activation, StateField.define<DecorationSet>({
    create: (state) => buildMath(state, source),
    update: (value, tr) =>
      tr.docChanged || tr.selection || tr.effects.some(effect => effect.is(setMathSource)) ? buildMath(tr.state, source) : value,
    provide: (f) => EditorView.decorations.from(f),
  })];
}
