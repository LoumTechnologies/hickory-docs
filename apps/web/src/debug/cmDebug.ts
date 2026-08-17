// The debugger, drawn in the editor.
//
// Four things, in the order they matter:
//
//  1. **The gutter**, with four states rather than one — a breakpoint, one the
//     adapter could not bind, a conditional one, and the line execution is
//     paused on. "Where I stopped" and "where I asked to stop" are different
//     facts, and after a step the second is usually somewhere else.
//  2. **Inline values** at the end of each line while paused, which is the
//     single biggest thing a JetBrains debugger does and mostly removes the
//     reason to look at a variables pane at all.
//  3. **The paused line** itself, tinted.
//  4. Hover, which is handled in `cmLsp` because a tooltip has to merge three
//     sources — a diagnostic, a runtime value, and a type — and merging them
//     in one place is the only way they end up in a sensible order.
//
// LAYOUT RULE, inherited from the rest of the editor: nothing here changes a
// line's HEIGHT. Split view measures line geometry to draw its ribbons, and a
// taller line is a ribbon pointing at the wrong place. Inline values are
// inline widgets; the paused line is a background.

import { RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { EditorState, Extension } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  GutterMarker,
  WidgetType,
  gutter,
  lineNumbers,
} from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import type { Frame, Variable } from "./client";
import { isDebuggable } from "./languages";
import { languageForBlock } from "../editor/hickDoc";
import { structureOf } from "../editor/wysiwyg";

/** One gutter dot. */
export interface BreakpointMark {
  line: number;
  verified: boolean;
  conditional: boolean;
  /** Why it could not bind, shown on hover. */
  message?: string;
}

/** Whether a mark is broken: unbindable, and able to say why. */
function isBroken(mark: BreakpointMark): boolean {
  return !mark.verified && !!mark.message;
}

/** One line of the call stack below the one execution is stopped at. */
export interface StackMark {
  line: number;
  /** The frame's id, so clicking the mark can select it. */
  id: number;
  /** The function whose frame this is, for the tooltip. */
  name: string;
  /** 1 for the caller, 2 for its caller, and so on. */
  depth: number;
}

/**
 * The callers to mark, from the stack the debugger reported.
 *
 * The top frame is where execution IS — that one is the paused arrow, and
 * repeating it as a caller would say the program is in two places. Frames
 * outside this document have no line here. And when two frames share a line —
 * a generator expression and the function that drives it — the nearer one
 * wins, because the tooltip should name the frame you would step out into
 * first.
 */
export function stackMarksOf(frames: readonly Frame[], pausedLine: number | null): StackMark[] {
  const out: StackMark[] = [];
  const seen = new Set<number>();
  frames.slice(1).forEach((frame, index) => {
    const line = frame.line;
    if (line === null || line === undefined) return;
    if (!frame.in_document) return;
    if (line === pausedLine || seen.has(line)) return;
    seen.add(line);
    out.push({ line, id: frame.id, name: frame.name, depth: index + 1 });
  });
  return out;
}

/**
 * The line a breakpoint on `line` should actually go on, or null when the
 * line cannot hold one.
 *
 * Two rules, both of which a person can see for themselves and would be
 * annoyed to have to discover from a hollow dot:
 *
 *  * **Prose cannot hold a breakpoint.** A document is mostly text about the
 *    program; only the lines inside a `hick:file` block in a language with a
 *    debug adapter become code that runs.
 *  * **A blank line cannot hold one either** — nor a line that is only a
 *    comment marker, though that is the adapter's business. The breakpoint
 *    slides down to the next line with something on it, inside the same
 *    block, which is what every editor does and what the adapter would do
 *    anyway, only sooner and visibly.
 */
export function breakpointLine(state: EditorState, line: number): number | null {
  const structure = structureOf(state);
  const doc = state.doc;
  if (line < 0 || line >= doc.lines) return null;
  const at = doc.line(line + 1);

  const block = structure.blocks.find(
    (candidate) =>
      candidate.name === "file" &&
      at.from >= candidate.contentFrom &&
      at.to <= candidate.contentTo &&
      isDebuggable(languageForBlock(structure, candidate)),
  );
  if (!block) return null;

  const lastLine = doc.lineAt(block.contentTo).number;
  for (let number = line + 1; number <= lastLine; number += 1) {
    if (doc.line(number).text.trim().length > 0) return number - 1;
  }
  return null;
}

export const setBreakpointMarks = StateEffect.define<BreakpointMark[]>();
export const setPausedLine = StateEffect.define<number | null>();
export const setInlineValues = StateEffect.define<Variable[]>();
export const setStackMarks = StateEffect.define<StackMark[]>();
export const setWatchValues = StateEffect.define<WatchValue[]>();
/** Flash a line, to say "here" after moving somewhere. */
export const flashLine = StateEffect.define<number | null>();

// --- the gutter -------------------------------------------------------------

/** The red dot on its own. */
function dotElement(mark: BreakpointMark): HTMLElement {
  const dot = document.createElement("span");
  dot.className =
    "cm-bp" +
    (mark.verified ? "" : " cm-bp-unverified") +
    (mark.conditional ? " cm-bp-conditional" : "") +
    // Broken is its own look: a breakpoint the debugger refused is not the
    // same as one it has not bound yet, and the difference decides whether
    // you are waiting or fixing something.
    (isBroken(mark) ? " cm-bp-broken" : "");
  // The reason, on the thing it is about. Without this, an unbindable
  // breakpoint is a mystery rather than a message — and with it, the message
  // needs to be nowhere else.
  dot.title = mark.message
    ? mark.message
    : mark.conditional
      ? "Conditional breakpoint"
      : "Breakpoint";
  return dot;
}

function arrowElement(): HTMLElement {
  const arrow = document.createElement("span");
  arrow.className = "cm-paused-arrow";
  arrow.title = "Execution is paused here";
  return arrow;
}

class DotMarker extends GutterMarker {
  constructor(private readonly mark: BreakpointMark) {
    super();
  }
  eq(other: DotMarker) {
    return (
      other.mark.verified === this.mark.verified &&
      other.mark.conditional === this.mark.conditional
    );
  }
  toDOM() {
    return dotElement(this.mark);
  }
}

class PausedMarker extends GutterMarker {
  toDOM() {
    return arrowElement();
  }
}

/**
 * A caller's line: where execution will return to.
 *
 * The stack is the way out of where you are, and reading it only as a list in
 * a panel means holding two pictures at once — the list, and where those lines
 * are in the document. A hollow arrow beside each caller puts the stack in the
 * one place the code already is.
 */
class FrameMarker extends GutterMarker {
  constructor(private readonly mark: StackMark) {
    super();
  }
  eq(other: FrameMarker) {
    return other.mark.depth === this.mark.depth && other.mark.name === this.mark.name;
  }
  toDOM() {
    const arrow = document.createElement("span");
    arrow.className = "cm-frame-arrow";
    arrow.title =
      (this.mark.depth === 1
        ? `Called from ${this.mark.name}`
        : `${this.mark.depth} frames up: ${this.mark.name}`) + " — click to view this frame";
    return arrow;
  }
}

/**
 * Paused *on* a breakpoint: both, stacked.
 *
 * Showing only the arrow loses the fact that there is a breakpoint here —
 * so continuing appears to stop for no reason, and the person cannot see the
 * dot they would click to remove. JetBrains draws the arrow over the dot, and
 * the two read as one thing: "stopped, at a breakpoint you set".
 */
class PausedAtBreakpointMarker extends GutterMarker {
  constructor(private readonly mark: BreakpointMark) {
    super();
  }
  eq(other: PausedAtBreakpointMarker) {
    return (
      other.mark.verified === this.mark.verified &&
      other.mark.conditional === this.mark.conditional
    );
  }
  toDOM() {
    const stack = document.createElement("span");
    stack.className = "cm-bp-stack";
    stack.title = "Paused at a breakpoint";
    stack.appendChild(dotElement(this.mark));
    stack.appendChild(arrowElement());
    return stack;
  }
}

const breakpointField = StateField.define<BreakpointMark[]>({
  create: () => [],
  update(marks, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setBreakpointMarks)) return effect.value;
    }
    return marks;
  },
});

const stackField = StateField.define<StackMark[]>({
  create: () => [],
  update(marks, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setStackMarks)) return effect.value;
    }
    return marks;
  },
});

const pausedField = StateField.define<number | null>({
  create: () => null,
  update(line, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setPausedLine)) return effect.value;
    }
    return line;
  },
});

/** The tint on the paused line. A background only — never a height. */
const pausedHighlight = EditorView.decorations.compute([pausedField], (state) => {
  const line = state.field(pausedField);
  if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
  const from = state.doc.line(line + 1).from;
  return Decoration.set([Decoration.line({ class: "cm-paused-line" }).range(from)]);
});

const flashField = StateField.define<number | null>({
  create: () => null,
  update(line, tr) {
    for (const effect of tr.effects) {
      if (effect.is(flashLine)) return effect.value;
    }
    return line;
  },
});

/** The brief tint that says "here" after moving to another frame's line. */
const flashHighlight = EditorView.decorations.compute([flashField], (state) => {
  const line = state.field(flashField);
  if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
  const from = state.doc.line(line + 1).from;
  return Decoration.set([Decoration.line({ class: "cm-flash-line" }).range(from)]);
});

/**
 * Go to a line and say so.
 *
 * Scrolling somewhere silently leaves the person to find what changed; a
 * flash that fades tells them where they landed without leaving a mark that
 * competes with the paused line a second later.
 */
export function revealLine(view: EditorView, line: number, holdMs = 900): void {
  if (line < 0 || line >= view.state.doc.lines) return;
  const at = view.state.doc.line(line + 1).from;
  view.dispatch({
    selection: { anchor: at },
    scrollIntoView: true,
    effects: flashLine.of(line),
  });
  window.setTimeout(() => {
    // Only clear our own flash: another line may have been revealed since.
    if (view.state.field(flashField, false) === line) {
      view.dispatch({ effects: flashLine.of(null) });
    }
  }, holdMs);
}

// --- inline values ----------------------------------------------------------

/** The greyed `x = 1` at the end of a line while paused. */
class InlineValue extends WidgetType {
  constructor(
    private readonly text: string,
    private readonly className = "cm-debug-value",
  ) {
    super();
  }
  // Without this, every value is redrawn on every update and the caret
  // flickers as the DOM under it is replaced.
  eq(other: InlineValue) {
    return other.text === this.text && other.className === this.className;
  }
  toDOM() {
    const span = document.createElement("span");
    span.className = this.className;
    span.textContent = this.text;
    // Not text: it must never be selected, copied, or counted as part of
    // the document somebody is editing.
    span.setAttribute("aria-hidden", "true");
    return span;
  }
  ignoreEvent() {
    return false;
  }
}

const inlineField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const effect of tr.effects) {
      if (!effect.is(setInlineValues)) continue;
      deco = buildInline(tr.state.doc, effect.value, tr.state.field(pausedField, false) ?? null);
    }
    return deco;
  },
  provide: (field) => EditorView.decorations.from(field),
});

/**
 * Place each variable at the end of the line that mentions it.
 *
 * Matched by identifier rather than by anything the adapter said, because DAP
 * does not report where a variable is written — only what it holds. Whole-word
 * matching keeps `sum` out of `subtotal`, and only lines at or above the
 * paused one are annotated: a value shown beside a line that has not run yet
 * is a lie about the program's state.
 */
export function inlinePlacements(
  doc: { lines: number; line(n: number): { from: number; to: number; text: string } },
  variables: Variable[],
  pausedLine: number | null,
): { at: number; text: string }[] {
  if (pausedLine === null) return [];
  const named = variables.filter(
    (variable) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(variable.name) && variable.value !== "",
  );
  const out: { at: number; text: string }[] = [];
  const upto = Math.min(pausedLine, doc.lines - 1);
  for (let index = 0; index <= upto; index++) {
    const line = doc.line(index + 1);
    const shown: string[] = [];
    for (const variable of named) {
      const pattern = new RegExp(`\\b${variable.name}\\b`);
      if (pattern.test(line.text)) shown.push(`${variable.name} = ${abbreviate(variable.value)}`);
    }
    if (shown.length > 0) out.push({ at: line.to, text: shown.join("  ") });
  }
  return out;
}

/** A long value would push the line off the screen; the pane has the rest. */
function abbreviate(value: string): string {
  const flat = value.replace(/\s+/g, " ").trim();
  return flat.length > 48 ? `${flat.slice(0, 47)}…` : flat;
}

function buildInline(
  doc: { lines: number; line(n: number): { from: number; to: number; text: string } },
  variables: Variable[],
  pausedLine: number | null,
): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  for (const placement of inlinePlacements(doc, variables, pausedLine)) {
    builder.add(
      placement.at,
      placement.at,
      Decoration.widget({ widget: new InlineValue(placement.text), side: 1 }),
    );
  }
  return builder.finish();
}

// --- watches ----------------------------------------------------------------

/** One watched expression, with what it currently holds (null: no answer). */
export interface WatchValue {
  expression: string;
  value: string | null;
}

const watchesField = StateField.define<WatchValue[]>({
  create: () => [],
  update(watches, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setWatchValues)) return effect.value;
    }
    return watches;
  },
});

/**
 * The line each watch belongs beside: the first one that mentions the
 * expression, or the paused line when none does.
 *
 * The value is appended after the line's end as a widget — the same shape as
 * inline values — so it can never change where the text itself wraps. Nothing
 * is shown while nothing is paused: a watch's value belongs to a stopped
 * frame, and showing it afterwards would be showing the past as the present.
 */
export function watchPlacements(
  doc: { lines: number; line(n: number): { from: number; to: number; text: string } },
  watches: WatchValue[],
  pausedLine: number | null,
): { at: number; text: string }[] {
  if (pausedLine === null || watches.length === 0) return [];
  const byLine = new Map<number, string[]>();
  for (const watch of watches) {
    const text = `${watch.expression} = ${watch.value ?? "…"}`;
    let placed: number | null = null;
    // A bare identifier matches whole words only (`sum` must not land inside
    // `subtotal`); anything longer is matched literally.
    const bare = /^[A-Za-z_][A-Za-z0-9_]*$/.test(watch.expression)
      ? new RegExp(`\\b${watch.expression}\\b`)
      : null;
    for (let index = 0; index < doc.lines; index += 1) {
      const line = doc.line(index + 1);
      if (bare ? bare.test(line.text) : line.text.includes(watch.expression)) {
        placed = index;
        break;
      }
    }
    const line = placed ?? Math.min(pausedLine, doc.lines - 1);
    byLine.set(line, [...(byLine.get(line) ?? []), text]);
  }
  return [...byLine.entries()]
    .sort((a, b) => a[0] - b[0])
    .map(([line, texts]) => ({ at: doc.line(line + 1).to, text: texts.join("  ") }));
}

const watchDecorations = EditorView.decorations.compute(
  ["doc", watchesField, pausedField],
  (state) => {
    const builder = new RangeSetBuilder<Decoration>();
    const placements = watchPlacements(
      state.doc,
      state.field(watchesField),
      state.field(pausedField),
    );
    for (const placement of placements) {
      builder.add(
        placement.at,
        placement.at,
        // side 2: after the inline values on the same line, and still an
        // inline widget — appended after the line's end, never inside it.
        Decoration.widget({
          widget: new InlineValue(placement.text, "cm-debug-value cm-debug-watch"),
          side: 2,
        }),
      );
    }
    return builder.finish();
  },
);

// --- inline eval at the paused line -----------------------------------------

/**
 * The affordance at the end of the paused line: a quiet `eval…` that expands
 * into an input. Enter answers in the selected frame; Shift-Enter promotes
 * the expression to a watch; Escape closes.
 *
 * An inline widget appended after the line end, exactly like inline values:
 * it adds nothing to the text flow mid-line, so wrapping is untouched.
 */
class EvalWidget extends WidgetType {
  constructor(
    private readonly line: number,
    private readonly options: DebugEditorOptions,
  ) {
    super();
  }
  // Same line, same widget: the DOM (and whatever is typed in it) survives
  // unrelated dispatches while the program stays paused here.
  eq(other: EvalWidget) {
    return other.line === this.line;
  }
  toDOM() {
    const root = document.createElement("span");
    root.className = "cm-eval";
    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "cm-eval-toggle";
    toggle.textContent = "eval…";
    toggle.title = "Evaluate an expression in the paused frame";
    const options = this.options;

    const close = () => {
      root.textContent = "";
      root.appendChild(toggle);
    };
    const open = () => {
      root.textContent = "";
      const input = document.createElement("input");
      input.className = "cm-eval-input";
      input.placeholder = "expression — Enter evaluates, Shift-Enter watches";
      input.setAttribute("aria-label", "Evaluate an expression in the paused frame");
      const result = document.createElement("span");
      result.className = "cm-eval-result";
      input.addEventListener("keydown", (event) => {
        // The editor must not treat typing here as typing in the document.
        event.stopPropagation();
        if (event.key === "Escape") {
          close();
          return;
        }
        if (event.key !== "Enter") return;
        const expression = input.value.trim();
        if (!expression) return;
        if (event.shiftKey) {
          // The one-keystroke promotion: what you just asked becomes a watch.
          options.onAddWatch?.(expression);
          result.textContent = "→ watching";
          return;
        }
        result.textContent = "…";
        void options.onEvaluate?.(expression).then((value) => {
          result.textContent = value === null ? "(no value)" : `= ${value}`;
        });
      });
      root.appendChild(input);
      root.appendChild(result);
      input.focus();
    };
    toggle.addEventListener("mousedown", (event) => {
      // mousedown, not click: the editor takes the selection on mousedown
      // and the button would lose the event to it.
      event.preventDefault();
      event.stopPropagation();
      open();
    });
    root.appendChild(toggle);
    return root;
  }
  ignoreEvent() {
    // Everything inside is the widget's own business, not the editor's.
    return true;
  }
}

// --- the extension ----------------------------------------------------------

export interface DebugEditorOptions {
  /** Toggle a breakpoint on this 0-based document line. */
  onToggleBreakpoint: (line: number) => void;
  /** Show this frame's line and values (a gutter stack mark was clicked). */
  onSelectFrame?: (id: number) => void;
  /** Answer an expression in the selected frame, for the inline eval. */
  onEvaluate?: (expression: string) => Promise<string | null>;
  /** Add an expression to the watch list. */
  onAddWatch?: (expression: string) => void;
}

/**
 * What a click on the gutter beside `line` should do.
 *
 * The rule mirrors what is drawn: a line showing a stack mark (a caller, with
 * nothing louder on it) selects that frame; everywhere else the click is
 * about breakpoints, snapped to the line that can hold one.
 */
export function gutterAction(
  state: EditorState,
  line: number,
): { kind: "frame"; id: number } | { kind: "breakpoint"; line: number } | null {
  const paused = state.field(pausedField, false) === line;
  const hasBreakpoint =
    state.field(breakpointField, false)?.some((mark) => mark.line === line) ?? false;
  if (!paused && !hasBreakpoint) {
    const frame = state.field(stackField, false)?.find((mark) => mark.line === line);
    if (frame) return { kind: "frame", id: frame.id };
  }
  const target = breakpointLine(state, line);
  return target === null ? null : { kind: "breakpoint", line: target };
}

/** The paused arrow, which carries no state either. */
const PAUSED_MARKER = new PausedMarker();

/** An empty cell, so widget blocks take up a row here as they do beside. */
const SPACER = new (class extends GutterMarker {
  toDOM() {
    return document.createElement("span");
  }
})();

/**
 * Which line a gutter click meant.
 *
 * The pointer's own position, not the block's start: this editor wraps long
 * lines and replaces markup with widgets, so one gutter element can cover
 * several rows of screen, and taking the block's first line puts the
 * breakpoint above where the person clicked. The block is the fallback for a
 * synthetic event with no coordinates.
 */
function lineAtEvent(view: EditorView, block: { from: number }, event: Event): number {
  const mouse = event as MouseEvent;
  if (typeof mouse.clientY === "number") {
    const pos = view.posAtCoords({ x: view.contentDOM.getBoundingClientRect().left + 1, y: mouse.clientY }, false);
    if (pos !== null) return view.state.doc.lineAt(pos).number - 1;
  }
  return view.state.doc.lineAt(block.from).number - 1;
}

/**
 * The empty-line marker: invisible until the pointer is over the gutter.
 *
 * Shared rather than made per line, because it carries no state.
 */
const HOVER_TARGET = new (class extends GutterMarker {
  toDOM() {
    const dot = document.createElement("span");
    dot.className = "cm-bp-ghost";
    dot.title = "Click to set a breakpoint";
    return dot;
  }
})();

export function debugEditor(options: DebugEditorOptions): Extension[] {
  // The eval affordance follows the paused line. Built here because its
  // widget needs the callbacks; drawn only when a handler exists to answer.
  const evalAffordance = EditorView.decorations.compute(["doc", pausedField], (state) => {
    if (!options.onEvaluate) return Decoration.none;
    const line = state.field(pausedField);
    if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
    const at = state.doc.line(line + 1).to;
    return Decoration.set([
      // side 3: after this line's inline values and watches.
      Decoration.widget({ widget: new EvalWidget(line, options), side: 3 }).range(at),
    ]);
  });
  return [
    // Numbers first, then the dots. Without them there is no way to say which
    // line anything is on — and in a document whose editor hides and folds
    // markup, "the fourth line I can see" is not the fourth line of the file,
    // which is the only line number the debugger and the document agree on.
    lineNumbers(),
    breakpointField,
    pausedField,
    stackField,
    flashField,
    pausedHighlight,
    flashHighlight,
    inlineField,
    watchesField,
    watchDecorations,
    evalAffordance,
    gutter({
      class: "cm-breakpoint-gutter",
      // Everything is drawn per LINE, not per position.
      //
      // A `markers` RangeSet places a marker at a document position, and
      // CodeMirror puts it in whichever block contains that position. This
      // editor renders markup as block widgets, so a line's start can belong
      // to a widget's block — and the dot appears a row or two below the line
      // it belongs to, which was measured in the running app: click line 29,
      // dot on 31. `lineMarker` is handed each line's own cell, the same cell
      // its number is in, so the two cannot drift apart.
      lineMarker: (view, block) => {
        const line = view.state.doc.lineAt(block.from).number - 1;
        const paused = view.state.field(pausedField) === line;
        const mark = view.state.field(breakpointField).find((m) => m.line === line);
        if (paused && mark) return new PausedAtBreakpointMarker(mark);
        if (paused) return PAUSED_MARKER;
        if (mark) return new DotMarker(mark);
        // A caller's line, when nothing louder is on it: the way out of here.
        const frame = view.state.field(stackField).find((f) => f.line === line);
        if (frame) return new FrameMarker(frame);
        // Nothing here yet. The ghost appears only where a breakpoint could
        // actually go: offering one beside a paragraph is an invitation to
        // find out later that it was never possible.
        return breakpointLine(view.state, line) === null ? null : HOVER_TARGET;
      },
      // A cell for every widget block, too.
      //
      // This editor renders markup as block widgets, and a gutter that skips
      // them ends up with FEWER cells than the line-number gutter beside it:
      // 63 against 64, measured in the running app. Every marker below the
      // first widget then sits one cell low — a red dot beside the next line
      // down, and a paused arrow that points at the wrong line. An empty
      // marker keeps the two gutters cell for cell.
      widgetMarker: () => SPACER,
      // Without this, line markers are recomputed only when the document or
      // the viewport changes — so a breakpoint appeared on the next scroll or
      // keystroke rather than on the click that set it.
      lineMarkerChange: (update) =>
        update.startState.field(breakpointField) !== update.state.field(breakpointField) ||
        update.startState.field(pausedField) !== update.state.field(pausedField) ||
        update.startState.field(stackField) !== update.state.field(stackField),
      domEventHandlers: {
        mousedown(view, block, event) {
          const clicked = lineAtEvent(view, block, event);
          // A stack mark selects its frame; anywhere else is a breakpoint,
          // snapped to the line that can hold one — or refused quietly: a dot
          // that appears and then reports it could not bind is worse than no
          // dot.
          const action = gutterAction(view.state, clicked);
          if (action?.kind === "frame") options.onSelectFrame?.(action.id);
          else if (action?.kind === "breakpoint") options.onToggleBreakpoint(action.line);
          return true;
        },
      },
    }),
    EditorView.theme({
      ".cm-breakpoint-gutter": { width: "1.1rem", cursor: "pointer" },
      // One marker per cell, centred, and never wrapping: the alignment
      // between a dot and its line is the whole contract of a gutter.
      ".cm-breakpoint-gutter .cm-gutterElement": {
        display: "flex",
        justifyContent: "center",
        alignItems: "flex-start",
        overflow: "hidden",
      },
    }),
  ];
}
