<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/cmDebug.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9df55a31763b"></a>
## `apps/web/src/debug/cmDebug.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Hover Hint Is Drawn In The App Theme](../../../../../guarantees/authoring/every-hover-hint-is-drawn-in-the-app-theme/contract.md), [A Breakpoint Can Carry A Condition](../../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [A Breakpoint That Is Not Bound Yet Is Not Refused](../../../../../guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md).

<a id="f-9df55a31763b-BreakpointMark"></a>
### `BreakpointMark`

<storycode:copy id="f-9df55a31763b-BreakpointMark">
import { breakpointFormActions, showBreakpointForm } from "./breakpointForm";
import { inlineEval } from "./inlineEval";
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

import { breakpointTarget } from "./cmDebugGutter";
import { Facet, RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
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
import type { BindState, Frame, Variable } from "./client";
import { wrapGutterMarkers } from "../editor/wrapGutter";
import { isDebuggable } from "./languages";
import { languageForBlock } from "../editor/hickDoc";
import { structureOf } from "../editor/wysiwyg";

/** One gutter dot. */
export interface BreakpointMark {
  line: number;
  state: BindState;
  /** True when anything below is set: drawn as a diamond, not a dot. */
  conditional: boolean;
  /** Why it could not bind, or why it is not confirmed yet, shown on hover. */
  message?: string;
  /** Stop only when this is true, in the debuggee's own language. */
  condition?: string;
  /** Stop only on the Nth hit — the adapter's own syntax. */
  hitCondition?: string;
  /** Log this and continue instead of stopping. */
  logMessage?: string;
}

</storycode:copy>
<a id="f-9df55a31763b-breakpointTip"></a>
### `breakpointTip`

What a breakpoint's tooltip should say.
A conditional breakpoint that says only "Conditional breakpoint" makes a
person open the editor to remember what they wrote. Saying it here costs a
string and answers the question where it is asked.

<storycode:copy id="f-9df55a31763b-breakpointTip">
/**
 * What a breakpoint's tooltip should say.
 *
 * A conditional breakpoint that says only "Conditional breakpoint" makes a
 * person open the editor to remember what they wrote. Saying it here costs a
 * string and answers the question where it is asked.
 */
export function breakpointTip(mark: BreakpointMark): string {
  if (mark.message) return mark.message;
  const parts: string[] = [];
  if (mark.logMessage) parts.push(`Logs ${JSON.stringify(mark.logMessage)} and continues`);
  if (mark.condition) parts.push(`Stops when ${mark.condition}`);
  if (mark.hitCondition) parts.push(`Hit count ${mark.hitCondition}`);
  if (parts.length === 0) return "Breakpoint";
  return parts.join(" · ");
}

</storycode:copy>
<a id="f-9df55a31763b-isBroken"></a>
### `isBroken`

Whether a mark is broken: refused outright, with a reason to show.

<storycode:copy id="f-9df55a31763b-isBroken">
/** Whether a mark is broken: refused outright, with a reason to show. */
function isBroken(mark: BreakpointMark): boolean {
  return mark.state === "refused";
}

</storycode:copy>
<a id="f-9df55a31763b-StackMark"></a>
### `StackMark`

One line of the call stack below the one execution is stopped at.

<storycode:copy id="f-9df55a31763b-StackMark">
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

</storycode:copy>
<a id="f-9df55a31763b-stackMarksOf"></a>
### `stackMarksOf`

The callers to mark, from the stack the debugger reported.
The top frame is where execution IS — that one is the paused arrow, and
repeating it as a caller would say the program is in two places. Frames
outside this document have no line here. And when two frames share a line —
a generator expression and the function that drives it — the nearer one
wins, because the tooltip should name the frame you would step out into
first.

<storycode:copy id="f-9df55a31763b-stackMarksOf">
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

</storycode:copy>
<a id="f-9df55a31763b-debugStateEffects"></a>
### `debugStateEffects`

The line a breakpoint on `line` should actually go on, or null when the
line cannot hold one.
Two rules, both of which a person can see for themselves and would be
annoyed to have to discover from a hollow dot:
* **Prose cannot hold a breakpoint.** A document is mostly text about the
program; only the lines inside a `hick:file` block in a language with a
debug adapter become code that runs.
* **A blank line cannot hold one either** — nor a line that is only a
comment marker, though that is the adapter's business. The breakpoint
slides down to the next line with something on it, inside the same
block, which is what every editor does and what the adapter would do
anyway, only sooner and visibly.
What the debugger's state looks like in an editor: the gutter dots, the
paused line, the inline values, the rest of the stack in the gutter, and
the watches at the end of their lines. One helper, so a document's editor
and a plain file's pane cannot draw the same session two different ways.

<storycode:copy id="f-9df55a31763b-debugStateEffects">
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
/**
 * What the debugger's state looks like in an editor: the gutter dots, the
 * paused line, the inline values, the rest of the stack in the gutter, and
 * the watches at the end of their lines. One helper, so a document's editor
 * and a plain file's pane cannot draw the same session two different ways.
 */
export function debugStateEffects(session: {
  breakpoints: readonly {
    line: number;
    state: BindState;
    message?: string;
    condition?: string;
    hit_condition?: string;
    log_message?: string;
  }[];
  pausedLine: number | null;
  variables: Variable[];
  frames: readonly Frame[];
  watches: WatchValue[];
}): StateEffect<unknown>[] {
  return [
    setBreakpointMarks.of(
      session.breakpoints.map((breakpoint) => ({
        line: breakpoint.line,
        state: breakpoint.state,
        // Computed, never assumed. This read `false` from the day it was
        // written, which made `cm-bp-conditional` unreachable styling for a
        // feature the engine had always supported.
        conditional: Boolean(
          breakpoint.condition || breakpoint.hit_condition || breakpoint.log_message,
        ),
        message: breakpoint.message,
        condition: breakpoint.condition,
        hitCondition: breakpoint.hit_condition,
        logMessage: breakpoint.log_message,
      })),
    ),
    setPausedLine.of(session.pausedLine),
    setInlineValues.of(session.variables),
    setStackMarks.of(stackMarksOf(session.frames, session.pausedLine)),
    setWatchValues.of(session.watches),
  ];
}

</storycode:copy>
<a id="f-9df55a31763b-breakpointLine"></a>
### `breakpointLine`

<storycode:copy id="f-9df55a31763b-breakpointLine">
export function breakpointLine(state: EditorState, line: number): number | null {
  const doc = state.doc;
  if (line < 0 || line >= doc.lines) return null;

  // A plain file: every line is code, in one language. The only rule left
  // is the blank-line one, and the block is the whole file.
  const whole = state.facet(wholeFileLanguage);
  if (whole !== null) {
    if (!isDebuggable(whole)) return null;
    for (let number = line + 1; number <= doc.lines; number += 1) {
      if (doc.line(number).text.trim().length > 0) return number - 1;
    }
    return null;
  }

  const structure = structureOf(state);
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

</storycode:copy>
<a id="f-9df55a31763b-wholeFileLanguage"></a>
### `wholeFileLanguage`

The language of a file whose EVERY line is code — a plain `src/main.rs`
or `app.py` opened in its own pane, as opposed to a document, where only
the lines inside a `hick:file` block are. Unset (null) in a document.

<storycode:copy id="f-9df55a31763b-wholeFileLanguage">
/**
 * The language of a file whose EVERY line is code — a plain `src/main.rs`
 * or `app.py` opened in its own pane, as opposed to a document, where only
 * the lines inside a `hick:file` block are. Unset (null) in a document.
 */
export const wholeFileLanguage = Facet.define<string | null, string | null>({
  combine: (values) => values.find((value) => value !== null) ?? null,
});

</storycode:copy>
<a id="f-9df55a31763b-setBreakpointMarks"></a>
### `setBreakpointMarks`

<storycode:copy id="f-9df55a31763b-setBreakpointMarks">
export const setBreakpointMarks = StateEffect.define<BreakpointMark[]>();
</storycode:copy>
<a id="f-9df55a31763b-setPausedLine"></a>
### `setPausedLine`

<storycode:copy id="f-9df55a31763b-setPausedLine">
export const setPausedLine = StateEffect.define<number | null>();
</storycode:copy>
<a id="f-9df55a31763b-setInlineValues"></a>
### `setInlineValues`

<storycode:copy id="f-9df55a31763b-setInlineValues">
export const setInlineValues = StateEffect.define<Variable[]>();
</storycode:copy>
<a id="f-9df55a31763b-setStackMarks"></a>
### `setStackMarks`

<storycode:copy id="f-9df55a31763b-setStackMarks">
export const setStackMarks = StateEffect.define<StackMark[]>();
</storycode:copy>
<a id="f-9df55a31763b-setWatchValues"></a>
### `setWatchValues`

<storycode:copy id="f-9df55a31763b-setWatchValues">
export const setWatchValues = StateEffect.define<WatchValue[]>();
</storycode:copy>
<a id="f-9df55a31763b-flashLine"></a>
### `flashLine`

Flash a line, to say "here" after moving somewhere.

<storycode:copy id="f-9df55a31763b-flashLine">
/** Flash a line, to say "here" after moving somewhere. */
export const flashLine = StateEffect.define<number | null>();

</storycode:copy>
<a id="f-9df55a31763b-dotElement"></a>
### `dotElement`

<storycode:copy id="f-9df55a31763b-dotElement">
// --- the gutter -------------------------------------------------------------

/** The red dot on its own. */
function dotElement(mark: BreakpointMark): HTMLElement {
  const dot = document.createElement("span");
  dot.className =
    "cm-bp" +
    (mark.state === "pending" ? " cm-bp-pending" : "") +
    (mark.conditional ? " cm-bp-conditional" : "") +
    // Broken is its own look: a breakpoint the debugger refused is not the
    // same as one it has not bound yet, and the difference decides whether
    // you are waiting or fixing something.
    (isBroken(mark) ? " cm-bp-broken" : "");
  // The reason, on the thing it is about. Without this, an unbindable
  // breakpoint is a mystery rather than a message — and with it, the message
  // needs to be nowhere else.
  dot.dataset.tip = breakpointTip(mark);
  return dot;
}

</storycode:copy>
<a id="f-9df55a31763b-arrowElement"></a>
### `arrowElement`

<storycode:copy id="f-9df55a31763b-arrowElement">
function arrowElement(): HTMLElement {
  const arrow = document.createElement("span");
  arrow.className = "cm-paused-arrow";
  arrow.dataset.tip = "Execution is paused here";
  return arrow;
}

</storycode:copy>
<a id="f-9df55a31763b-DotMarker"></a>
### `DotMarker`

<storycode:copy id="f-9df55a31763b-DotMarker">
class DotMarker extends GutterMarker {
  constructor(private readonly mark: BreakpointMark) {
    super();
  }
  eq(other: DotMarker) {
    return (
      other.mark.state === this.mark.state &&
      other.mark.conditional === this.mark.conditional &&
      breakpointTip(other.mark) === breakpointTip(this.mark)
    );
  }
  toDOM() {
    return dotElement(this.mark);
  }
}

</storycode:copy>
<a id="f-9df55a31763b-PausedMarker"></a>
### `PausedMarker`

<storycode:copy id="f-9df55a31763b-PausedMarker">
class PausedMarker extends GutterMarker {
  toDOM() {
    return arrowElement();
  }
}

</storycode:copy>
<a id="f-9df55a31763b-FrameMarker"></a>
### `FrameMarker`

A caller's line: where execution will return to.
The stack is the way out of where you are, and reading it only as a list in
a panel means holding two pictures at once — the list, and where those lines
are in the document. A hollow arrow beside each caller puts the stack in the
one place the code already is.

<storycode:copy id="f-9df55a31763b-FrameMarker">
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
    arrow.dataset.tip =
      (this.mark.depth === 1
        ? `Called from ${this.mark.name}`
        : `${this.mark.depth} frames up: ${this.mark.name}`) + " — click to view this frame";
    return arrow;
  }
}

</storycode:copy>
<a id="f-9df55a31763b-PausedAtBreakpointMarker"></a>
### `PausedAtBreakpointMarker`

Paused *on* a breakpoint: both, stacked.
Showing only the arrow loses the fact that there is a breakpoint here —
so continuing appears to stop for no reason, and the person cannot see the
dot they would click to remove. JetBrains draws the arrow over the dot, and
the two read as one thing: "stopped, at a breakpoint you set".

<storycode:copy id="f-9df55a31763b-PausedAtBreakpointMarker">
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
      other.mark.state === this.mark.state &&
      other.mark.conditional === this.mark.conditional &&
      breakpointTip(other.mark) === breakpointTip(this.mark)
    );
  }
  toDOM() {
    const stack = document.createElement("span");
    stack.className = "cm-bp-stack";
    stack.dataset.tip = "Paused at a breakpoint";
    stack.appendChild(dotElement(this.mark));
    stack.appendChild(arrowElement());
    return stack;
  }
}

</storycode:copy>
<a id="f-9df55a31763b-breakpointField"></a>
### `breakpointField`

<storycode:copy id="f-9df55a31763b-breakpointField">
const breakpointField = StateField.define<BreakpointMark[]>({
  create: () => [],
  update(marks, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setBreakpointMarks)) return effect.value;
    }
    return marks;
  },
});

</storycode:copy>
<a id="f-9df55a31763b-stackField"></a>
### `stackField`

<storycode:copy id="f-9df55a31763b-stackField">
const stackField = StateField.define<StackMark[]>({
  create: () => [],
  update(marks, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setStackMarks)) return effect.value;
    }
    return marks;
  },
});

</storycode:copy>
<a id="f-9df55a31763b-pausedField"></a>
### `pausedField`

<storycode:copy id="f-9df55a31763b-pausedField">
const pausedField = StateField.define<number | null>({
  create: () => null,
  update(line, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setPausedLine)) return effect.value;
    }
    return line;
  },
});

</storycode:copy>
<a id="f-9df55a31763b-pausedHighlight"></a>
### `pausedHighlight`

The tint on the paused line. A background only — never a height.

<storycode:copy id="f-9df55a31763b-pausedHighlight">
/** The tint on the paused line. A background only — never a height. */
const pausedHighlight = EditorView.decorations.compute([pausedField], (state) => {
  const line = state.field(pausedField);
  if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
  const from = state.doc.line(line + 1).from;
  return Decoration.set([Decoration.line({ class: "cm-paused-line" }).range(from)]);
});

</storycode:copy>
<a id="f-9df55a31763b-flashField"></a>
### `flashField`

<storycode:copy id="f-9df55a31763b-flashField">
const flashField = StateField.define<number | null>({
  create: () => null,
  update(line, tr) {
    for (const effect of tr.effects) {
      if (effect.is(flashLine)) return effect.value;
    }
    return line;
  },
});

</storycode:copy>
<a id="f-9df55a31763b-flashHighlight"></a>
### `flashHighlight`

The brief tint that says "here" after moving to another frame's line.

<storycode:copy id="f-9df55a31763b-flashHighlight">
/** The brief tint that says "here" after moving to another frame's line. */
const flashHighlight = EditorView.decorations.compute([flashField], (state) => {
  const line = state.field(flashField);
  if (line === null || line < 0 || line >= state.doc.lines) return Decoration.none;
  const from = state.doc.line(line + 1).from;
  return Decoration.set([Decoration.line({ class: "cm-flash-line" }).range(from)]);
});

</storycode:copy>
<a id="f-9df55a31763b-revealLine"></a>
### `revealLine`

Go to a line and say so.
Scrolling somewhere silently leaves the person to find what changed; a
flash that fades tells them where they landed without leaving a mark that
competes with the paused line a second later.

<storycode:copy id="f-9df55a31763b-revealLine">
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

</storycode:copy>
<a id="f-9df55a31763b-InlineValue"></a>
### `InlineValue`

--- inline values ----------------------------------------------------------
The greyed `x = 1` at the end of a line while paused.

<storycode:copy id="f-9df55a31763b-InlineValue">
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

</storycode:copy>
<a id="f-9df55a31763b-inlineField"></a>
### `inlineField`

<storycode:copy id="f-9df55a31763b-inlineField">
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

</storycode:copy>
<a id="f-9df55a31763b-inlinePlacements"></a>
### `inlinePlacements`

Place each variable at the end of the line that mentions it.
Matched by identifier rather than by anything the adapter said, because DAP
does not report where a variable is written — only what it holds. Whole-word
matching keeps `sum` out of `subtotal`, and only lines at or above the
paused one are annotated: a value shown beside a line that has not run yet
is a lie about the program's state.

<storycode:copy id="f-9df55a31763b-inlinePlacements">
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
      const label = `${variable.name} = ${abbreviate(variable.value)}`;
      if (new RegExp(`\\b${variable.name}\\b`).test(line.text) && !shown.includes(label)) shown.push(label);
    }
    if (shown.length > 0) out.push({ at: line.to, text: shown.join("  ") });
  }
  return out;
}

</storycode:copy>
<a id="f-9df55a31763b-abbreviate"></a>
### `abbreviate`

A long value would push the line off the screen; the pane has the rest.

<storycode:copy id="f-9df55a31763b-abbreviate">
/** A long value would push the line off the screen; the pane has the rest. */
function abbreviate(value: string): string {
  const flat = value.replace(/\s+/g, " ").trim();
  return flat.length > 48 ? `${flat.slice(0, 47)}…` : flat;
}

</storycode:copy>
<a id="f-9df55a31763b-buildInline"></a>
### `buildInline`

<storycode:copy id="f-9df55a31763b-buildInline">
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

</storycode:copy>
<a id="f-9df55a31763b-WatchValue"></a>
### `WatchValue`

--- watches ----------------------------------------------------------------
One watched expression, with what it currently holds (null: no answer).

<storycode:copy id="f-9df55a31763b-WatchValue">
// --- watches ----------------------------------------------------------------

/** One watched expression, with what it currently holds (null: no answer). */
export interface WatchValue {
  expression: string;
  value: string | null;
}

</storycode:copy>
<a id="f-9df55a31763b-watchesField"></a>
### `watchesField`

<storycode:copy id="f-9df55a31763b-watchesField">
const watchesField = StateField.define<WatchValue[]>({
  create: () => [],
  update(watches, tr) {
    for (const effect of tr.effects) {
      if (effect.is(setWatchValues)) return effect.value;
    }
    return watches;
  },
});

</storycode:copy>
<a id="f-9df55a31763b-watchPlacements"></a>
### `watchPlacements`

The line each watch belongs beside: the first one that mentions the
expression, or the paused line when none does.
The value is appended after the line's end as a widget — the same shape as
inline values — so it can never change where the text itself wraps. Nothing
is shown while nothing is paused: a watch's value belongs to a stopped
frame, and showing it afterwards would be showing the past as the present.

<storycode:copy id="f-9df55a31763b-watchPlacements">
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

</storycode:copy>
<a id="f-9df55a31763b-watchDecorations"></a>
### `watchDecorations`

<storycode:copy id="f-9df55a31763b-watchDecorations">
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

</storycode:copy>
<a id="f-9df55a31763b-DebugEditorOptions"></a>
### `DebugEditorOptions`

--- inline eval at the paused line -----------------------------------------
The affordance at the end of the paused line: a quiet `eval…` that expands
into an input. Enter answers in the selected frame; Shift-Enter promotes
the expression to a watch; Escape closes.
An inline widget appended after the line end, exactly like inline values:
it adds nothing to the text flow mid-line, so wrapping is untouched.
--- the extension ----------------------------------------------------------

<storycode:copy id="f-9df55a31763b-DebugEditorOptions">
// --- inline eval at the paused line -----------------------------------------

/**
 * The affordance at the end of the paused line: a quiet `eval…` that expands
 * into an input. Enter answers in the selected frame; Shift-Enter promotes
 * the expression to a watch; Escape closes.
 *
 * An inline widget appended after the line end, exactly like inline values:
 * it adds nothing to the text flow mid-line, so wrapping is untouched.
 */
// --- the extension ----------------------------------------------------------

export interface DebugEditorOptions {
  /**
   * Set for a plain file, whose every line is code in this one language.
   * A document leaves it unset and the gutter reads the block structure.
   */
  language?: string | null;
  /**
   * Whether this bundle supplies the line-number gutter. A document's editor
   * has no other source of line numbers; a plain file's pane already
   * declares its own, in the order it wants its gutters, and a second copy
   * would draw every number twice.
   */
  lineNumbers?: boolean;
  /** Toggle a breakpoint on this 0-based document line. */
  onToggleBreakpoint: (line: number) => void;
  /** Show this frame's line and values (a gutter stack mark was clicked). */
  onSelectFrame?: (id: number) => void;
  /** Answer an expression in the selected frame, for the inline eval. */
  onEvaluate?: (expression: string) => Promise<string | null>;
  /** Live debugger owner; replacing session/stop/frame retires pending inline values. */
  evaluationOwner?: () => unknown;
  /** Add an expression to the watch list. */
  onAddWatch?: (expression: string) => void;
  /**
   * Attach a condition, hit count, or log message to a breakpoint.
   *
   * Reached by right-clicking or alt-clicking the gutter, which is where
   * every other editor puts it. Absent in a pane that has no debugger, and
   * the gesture then falls back to an ordinary toggle rather than opening a
   * panel that could not do anything.
   */
  onSetBreakpointCondition?: (
    line: number,
    patch: { condition?: string; hit_condition?: string; log_message?: string },
  ) => void;
}

</storycode:copy>
<a id="f-9df55a31763b-openBreakpointEditor"></a>
### `openBreakpointEditor`

The little panel behind a right-click on a breakpoint.
Built as DOM rather than React for the same reason the inline evaluator is:
it belongs to the editor, and both the document pane and the plain-file
pane get it from this one function — the rule that stops the two drawing
one session two different ways.

<storycode:copy id="f-9df55a31763b-openBreakpointEditor">
/**
 * The little panel behind a right-click on a breakpoint.
 *
 * Built as DOM rather than React for the same reason the inline evaluator is:
 * it belongs to the editor, and both the document pane and the plain-file
 * pane get it from this one function — the rule that stops the two drawing
 * one session two different ways.
 */
export function openBreakpointEditor(
  view: EditorView,
  mark: BreakpointMark,
  onSet: (
    line: number,
    patch: { condition?: string; hit_condition?: string; log_message?: string },
  ) => void,
): void {
  showBreakpointForm(view, mark, onSet, () => view.state.field(breakpointField, false)?.find(item => item.line === mark.line));
}

</storycode:copy>
<a id="f-9df55a31763b-gutterAction"></a>
### `gutterAction`

What a click on the gutter beside `line` should do.
The rule mirrors what is drawn: a line showing a stack mark (a caller, with
nothing louder on it) selects that frame; everywhere else the click is
about breakpoints, snapped to the line that can hold one.

<storycode:copy id="f-9df55a31763b-gutterAction">
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

</storycode:copy>
<a id="f-9df55a31763b-gutterIntent"></a>
### `gutterIntent`

What a gutter gesture on `line` means.
Pure, and exported, for exactly the reason `gutterAction` is: a gutter's
`domEventHandlers` never fire under jsdom, which has no layout to resolve a
pointer's height against. The handlers below are two lines each so that
everything worth testing is here.

<storycode:copy id="f-9df55a31763b-gutterIntent">
/**
 * What a gutter gesture on `line` means.
 *
 * Pure, and exported, for exactly the reason `gutterAction` is: a gutter's
 * `domEventHandlers` never fire under jsdom, which has no layout to resolve a
 * pointer's height against. The handlers below are two lines each so that
 * everything worth testing is here.
 */
export function gutterIntent(
  state: EditorState,
  line: number,
  gesture: { alt?: boolean; context?: boolean },
  canEdit: boolean,
):
  | { kind: "frame"; id: number }
  | { kind: "toggle"; line: number }
  | { kind: "edit"; line: number }
  | null {
  const action = gutterAction(state, line);
  if (action?.kind === "frame") {
    // A right-click on a caller's arrow is not a breakpoint gesture at all.
    return gesture.context ? null : action;
  }
  if (action?.kind !== "breakpoint") return null;
  // Editing needs somewhere to send the condition. A pane without one falls
  // back to an ordinary toggle rather than opening a panel that can do
  // nothing — and a right-click there is left to the browser.
  if (!canEdit) return gesture.context ? null : { kind: "toggle", line: action.line };
  if (gesture.alt || gesture.context) return { kind: "edit", line: action.line };
  return { kind: "toggle", line: action.line };
}

</storycode:copy>
<a id="f-9df55a31763b-editBreakpoint"></a>
### `editBreakpoint`

Open the editor for the breakpoint on `line`, setting one first if there
is none.
Editing a line with no breakpoint is not an error: asking for "stop here
when x > 10" on a bare line is the common case, and making a person click
twice to express it is ceremony.

<storycode:copy id="f-9df55a31763b-editBreakpoint">
/**
 * Open the editor for the breakpoint on `line`, setting one first if there
 * is none.
 *
 * Editing a line with no breakpoint is not an error: asking for "stop here
 * when x > 10" on a bare line is the common case, and making a person click
 * twice to express it is ceremony.
 */
export function editBreakpoint(
  view: EditorView,
  line: number,
  options: DebugEditorOptions,
): void {
  const onSet = options.onSetBreakpointCondition;
  if (!onSet) {
    options.onToggleBreakpoint(line);
    return;
  }
  const existing = view.state.field(breakpointField, false)?.find((mark) => mark.line === line);
  if (!existing) options.onToggleBreakpoint(line);
  openBreakpointEditor(
    view,
    existing ?? { line, state: "pending", conditional: false },
    onSet,
  );
}

</storycode:copy>
<a id="f-9df55a31763b-PAUSED_MARKER"></a>
### `PAUSED_MARKER`

The paused arrow, which carries no state either.

<storycode:copy id="f-9df55a31763b-PAUSED_MARKER">
/** The paused arrow, which carries no state either. */
const PAUSED_MARKER = new PausedMarker();

</storycode:copy>
<a id="f-9df55a31763b-SPACER"></a>
### `SPACER`

An empty cell, so widget blocks take up a row here as they do beside.

<storycode:copy id="f-9df55a31763b-SPACER">
/** An empty cell, so widget blocks take up a row here as they do beside. */
const SPACER = new (class extends GutterMarker {
  toDOM() {
    return document.createElement("span");
  }
})();

</storycode:copy>
<a id="f-9df55a31763b-lineAtEvent"></a>
### `lineAtEvent`

Which line a gutter click meant.
The pointer's own position, not the block's start: this editor wraps long
lines and replaces markup with widgets, so one gutter element can cover
several rows of screen, and taking the block's first line puts the
breakpoint above where the person clicked. The block is the fallback for a
synthetic event with no coordinates.

<storycode:copy id="f-9df55a31763b-lineAtEvent">
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
  const target = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-breakpoint-line]") : null;
  if (target) return Number(target.dataset.breakpointLine);
  if (typeof mouse.clientY === "number") {
    const pos = view.posAtCoords({ x: view.contentDOM.getBoundingClientRect().left + 1, y: mouse.clientY }, false);
    if (pos !== null) return view.state.doc.lineAt(pos).number - 1;
  }
  return view.state.doc.lineAt(block.from).number - 1;
}

</storycode:copy>
<a id="f-9df55a31763b-debugEditor"></a>
### `debugEditor`

<storycode:copy id="f-9df55a31763b-debugEditor">
export function debugEditor(options: DebugEditorOptions): Extension[] {
  // The eval affordance follows the paused line. Built here because its
  // widget needs the callbacks; drawn only when a handler exists to answer.
  const evalAffordance = inlineEval(options, pausedField, setPausedLine);
  return [
    ...(options.language !== undefined ? [wholeFileLanguage.of(options.language)] : []),
    // Numbers first, then the dots. Without them there is no way to say which
    // line anything is on — and in a document whose editor hides and folds
    // markup, "the fourth line I can see" is not the fourth line of the file,
    // which is the only line number the debugger and the document agree on.
    //
    // A wrapped line keeps its number on the first visual row; every
    // continuation row gets a muted wrap mark instead of blank gutter, so
    // the tall cell reads as one logical line continuing.
    ...(options.lineNumbers === false ? [] : [lineNumbers(), wrapGutterMarkers()]),
    breakpointField,
    breakpointFormActions(options, (view, line) => {
      if (!view.dom.isConnected || !options.onSetBreakpointCondition) return false;
      const target = breakpointLine(view.state, line);
      if (target === null) return false;
      editBreakpoint(view, target, options); return true;
    }),
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
        return breakpointLine(view.state, line) === null ? null : breakpointTarget(line);
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
        // A stack mark selects its frame; anywhere else is a breakpoint,
        // snapped to the line that can hold one — or refused quietly: a dot
        // that appears and then reports it could not bind is worse than no
        // dot. Alt-click and right-click edit it instead, which is the
        // gesture every other editor uses; alt exists because a trackpad
        // without a second button has no other way to ask.
        mousedown(view, block, event) {
          const line = lineAtEvent(view, block, event);
          const intent = gutterIntent(
            view.state,
            line,
            { alt: (event as MouseEvent).altKey },
            Boolean(options.onSetBreakpointCondition),
          );
          if (intent?.kind === "frame") options.onSelectFrame?.(intent.id);
          else if (intent?.kind === "toggle") options.onToggleBreakpoint(intent.line);
          else if (intent?.kind === "edit") editBreakpoint(view, intent.line, options);
          return true;
        },
        contextmenu(view, block, event) {
          const line = lineAtEvent(view, block, event);
          const intent = gutterIntent(
            view.state,
            line,
            { context: true },
            Boolean(options.onSetBreakpointCondition),
          );
          if (intent?.kind !== "edit") return false;
          event.preventDefault();
          editBreakpoint(view, intent.line, options);
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
</storycode:copy>

### Assemble `apps/web/src/debug/cmDebug.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmDebug.ts"><storycode:paste select="#f-9df55a31763b-BreakpointMark"/><storycode:paste select="#f-9df55a31763b-breakpointTip"/><storycode:paste select="#f-9df55a31763b-isBroken"/><storycode:paste select="#f-9df55a31763b-StackMark"/><storycode:paste select="#f-9df55a31763b-stackMarksOf"/><storycode:paste select="#f-9df55a31763b-debugStateEffects"/><storycode:paste select="#f-9df55a31763b-breakpointLine"/><storycode:paste select="#f-9df55a31763b-wholeFileLanguage"/><storycode:paste select="#f-9df55a31763b-setBreakpointMarks"/><storycode:paste select="#f-9df55a31763b-setPausedLine"/><storycode:paste select="#f-9df55a31763b-setInlineValues"/><storycode:paste select="#f-9df55a31763b-setStackMarks"/><storycode:paste select="#f-9df55a31763b-setWatchValues"/><storycode:paste select="#f-9df55a31763b-flashLine"/><storycode:paste select="#f-9df55a31763b-dotElement"/><storycode:paste select="#f-9df55a31763b-arrowElement"/><storycode:paste select="#f-9df55a31763b-DotMarker"/><storycode:paste select="#f-9df55a31763b-PausedMarker"/><storycode:paste select="#f-9df55a31763b-FrameMarker"/><storycode:paste select="#f-9df55a31763b-PausedAtBreakpointMarker"/><storycode:paste select="#f-9df55a31763b-breakpointField"/><storycode:paste select="#f-9df55a31763b-stackField"/><storycode:paste select="#f-9df55a31763b-pausedField"/><storycode:paste select="#f-9df55a31763b-pausedHighlight"/><storycode:paste select="#f-9df55a31763b-flashField"/><storycode:paste select="#f-9df55a31763b-flashHighlight"/><storycode:paste select="#f-9df55a31763b-revealLine"/><storycode:paste select="#f-9df55a31763b-InlineValue"/><storycode:paste select="#f-9df55a31763b-inlineField"/><storycode:paste select="#f-9df55a31763b-inlinePlacements"/><storycode:paste select="#f-9df55a31763b-abbreviate"/><storycode:paste select="#f-9df55a31763b-buildInline"/><storycode:paste select="#f-9df55a31763b-WatchValue"/><storycode:paste select="#f-9df55a31763b-watchesField"/><storycode:paste select="#f-9df55a31763b-watchPlacements"/><storycode:paste select="#f-9df55a31763b-watchDecorations"/><storycode:paste select="#f-9df55a31763b-DebugEditorOptions"/><storycode:paste select="#f-9df55a31763b-openBreakpointEditor"/><storycode:paste select="#f-9df55a31763b-gutterAction"/><storycode:paste select="#f-9df55a31763b-gutterIntent"/><storycode:paste select="#f-9df55a31763b-editBreakpoint"/><storycode:paste select="#f-9df55a31763b-PAUSED_MARKER"/><storycode:paste select="#f-9df55a31763b-SPACER"/><storycode:paste select="#f-9df55a31763b-lineAtEvent"/><storycode:paste select="#f-9df55a31763b-debugEditor"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-8d12c7278102"></a>
## `apps/web/src/debug/cmDebug.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Breakpoint Can Carry A Condition](../../../../../guarantees/debugging/a-breakpoint-can-carry-a-condition/contract.md), [A Breakpoint That Is Not Bound Yet Is Not Refused](../../../../../guarantees/debugging/a-breakpoint-that-is-not-bound-yet-is-not-refused/contract.md), [Launch the artifact built from the selected compiled project](../../../../../guarantees/debugging/a-compiled-language-launches-what-a-build-produced/contract.md), [A Plain File Has The Same Debugger A Document Has](../../../../../guarantees/debugging/a-plain-file-has-the-same-debugger/contract.md).

<a id="f-8d12c7278102-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement">
import { EditorState, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import {
  breakpointLine,
  breakpointTip,
  debugEditor,
  debugStateEffects,
  editBreakpoint,
  gutterIntent,
  openBreakpointEditor,
  gutterAction,
  inlinePlacements,
  revealLine,
  setBreakpointMarks,
  setPausedLine,
  setStackMarks,
  setWatchValues,
  stackMarksOf,
  watchPlacements,
} from "./cmDebug";
import { backwardsControl, type DebugCapabilities } from "./client";
import { identifierAt } from "../lsp/cmLsp";

// CodeMirror measures text when asked to scroll somewhere, and jsdom has no
// layout: `Range.getClientRects` is missing entirely, and the failure surfaces
// as an unhandled error from an animation frame AFTER the test that caused it.
beforeAll(() => {
  if (!Range.prototype.getClientRects) {
    Range.prototype.getClientRects = () =>
      Object.assign([], { item: () => null }) as unknown as DOMRectList;
    Range.prototype.getBoundingClientRect = () => new DOMRect();
  }
});

</storycode:copy>
<a id="f-8d12c7278102-DOC"></a>
### `DOC`

<storycode:copy id="f-8d12c7278102-DOC">
const DOC = Text.of([
  "def line_total(quantity, unit_price):",
  "    subtotal = quantity * unit_price",
  "    return subtotal",
]);

</storycode:copy>
<a id="f-8d12c7278102-vars"></a>
### `vars`

<storycode:copy id="f-8d12c7278102-vars">
const vars = (entries: [string, string][]) =>
  entries.map(([name, value]) => ({ name, value, variables_reference: 0 }));

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-2">
describe("inline values", () => {
  it("puts each value at the end of a line that mentions it", () => {
    const placements = inlinePlacements(DOC, vars([["quantity", "2"]]), 1);
    expect(placements).toHaveLength(2); // the signature and the statement
    expect(placements[0].text).toBe("quantity = 2");
    expect(placements[0].at).toBe(DOC.line(1).to);
  });

  it("shows nothing below the paused line", () => {
    // A value beside a line that has not run yet is a lie about the state of
    // the program — `subtotal` does not exist until line 1 completes.
    const placements = inlinePlacements(DOC, vars([["subtotal", "19.98"]]), 1);
    expect(placements.map((p) => p.at)).not.toContain(DOC.line(3).to);
  });

  it("shows one inline value when the adapter repeats it across scopes", () => {
    const placements = inlinePlacements(DOC, vars([["quantity", "2"], ["quantity", "2"]]), 1);
    expect(placements.map(p=>p.text)).toEqual(["quantity = 2", "quantity = 2"]);
  });

  it("shows nothing at all when nothing is paused", () => {
    expect(inlinePlacements(DOC, vars([["quantity", "2"]]), null)).toEqual([]);
  });

  it("matches whole identifiers only", () => {
    // `sum` must not light up inside `subtotal`, which is the failure that
    // makes inline values look like noise rather than information.
    const placements = inlinePlacements(DOC, vars([["sum", "0"]]), 2);
    expect(placements).toEqual([]);
  });

  it("abbreviates a value that would push the line off screen", () => {
    const long = "x".repeat(200);
    const placements = inlinePlacements(DOC, vars([["quantity", long]]), 1);
    expect(placements[0].text.length).toBeLessThan(70);
    expect(placements[0].text.endsWith("…")).toBe(true);
  });

  it("ignores a variable with no name a person would recognise", () => {
    // Adapters return pseudo-entries like "special variables"; putting those
    // at the end of a line would be showing our plumbing.
    const placements = inlinePlacements(
      DOC,
      [{ name: "special variables", value: "{}", variables_reference: 4 }],
      1,
    );
    expect(placements).toEqual([]);
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-3">
describe("which way back this adapter offers", () => {
  const caps = (over: Partial<DebugCapabilities>): DebugCapabilities => ({
    conditional_breakpoints: true,
    hit_conditional_breakpoints: true,
    log_points: true,
    set_variable: true,
    restart_frame: false,
    step_in_targets: false,
    step_back: false,
    goto_targets: false,
    exception_filters: [],
    ...over,
  });

  it("prefers real reverse execution where it exists", () => {
    expect(backwardsControl(caps({ step_back: true }))?.kind).toBe("step_back");
  });

  it("offers drop frame next", () => {
    // js-debug and the JVM adapters.
    expect(backwardsControl(caps({ restart_frame: true }))?.kind).toBe("drop_frame");
  });

  it("falls back to moving the instruction pointer", () => {
    // debugpy: no drop frame, but it can jump — which is why the UI asks
    // rather than showing one control and letting it fail.
    const control = backwardsControl(caps({ goto_targets: true }));
    expect(control?.kind).toBe("jump");
    // And says what it actually does, because it does not rewind.
    expect(control?.hint).toContain("run again");
  });

  it("offers nothing when the adapter can do neither", () => {
    // Better than a disabled button with no explanation.
    expect(backwardsControl(caps({}))).toBeNull();
    expect(backwardsControl(null)).toBeNull();
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-4">
describe("the identifier under the pointer", () => {
  const line = "    subtotal = quantity * unit_price";

  it("finds the whole name, not the character", () => {
    expect(identifierAt(line, 6)).toBe("subtotal");
    expect(identifierAt(line, 20)).toBe("quantity");
  });

  it("includes attribute access, which is what you want the value of", () => {
    expect(identifierAt("self.lines[2]", 6)).toBe("self.lines");
  });

  it("is nothing on whitespace or an operator", () => {
    expect(identifierAt(line, 3)).toBeNull();
    expect(identifierAt(line, 13)).toBeNull();
  });

  it("is nothing on a number", () => {
    // Asking a debugger to evaluate `42` spends a round trip to be told 42.
    expect(identifierAt("x = 42", 5)).toBeNull();
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-5">
describe("the editor state the debugger drives", () => {
  it("builds without a document open", () => {
    // The extensions are installed whether or not a session exists, so they
    // must be inert until one does.
    const state = EditorState.create({ doc: "x = 1" });
    expect(state.doc.length).toBe(5);
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-6">
describe("the gutter is findable before it holds anything", () => {
  it("gives a line with a breakpoint exactly one marker", () => {
    // The ghost and the dot share a cell one dot wide, so two markers wrap:
    // the breakpoint renders on a second row inside the cell and reads as a
    // dot beside the NEXT line. Measured in the running app before it was
    // fixed; asserted here so it cannot come back.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\nc = 3\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({ effects: setBreakpointMarks.of([{ line: 1, state: "bound" as const, conditional: false }]) });
    view.dispatch({ effects: setPausedLine.of(2) });

    const cells = [...view.dom.querySelectorAll(".cm-breakpoint-gutter .cm-gutterElement")];
    for (const cell of cells) {
      expect(cell.children.length).toBeLessThanOrEqual(1);
    }
    // And each state lands in the cell for its own line: the dot on the line
    // with the breakpoint, the arrow on the paused one, nothing shared.
    const marked = cells.filter((cell) => cell.querySelector(".cm-bp, .cm-paused-arrow"));
    expect(marked.length).toBe(2);
    // And the markers that matter are the ones drawn.
    expect(view.dom.querySelectorAll(".cm-bp").length).toBe(1);
    expect(view.dom.querySelectorAll(".cm-paused-arrow").length).toBe(1);
    view.destroy();
  });

  it("shows the breakpoint and the arrow when it is stopped on one", () => {
    // Showing only the arrow loses the breakpoint: continuing then appears to
    // stop for no reason, and the dot you would click to remove it is gone.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({
      effects: [
        setBreakpointMarks.of([{ line: 0, state: "bound" as const, conditional: false }]),
        setPausedLine.of(0),
      ],
    });

    const stack = view.dom.querySelector(".cm-bp-stack");
    expect(stack).not.toBeNull();
    expect(stack?.querySelector(".cm-bp")).not.toBeNull();
    expect(stack?.querySelector(".cm-paused-arrow")).not.toBeNull();
    // And the dot keeps saying what kind of breakpoint it is.
    view.dispatch({
      effects: setBreakpointMarks.of([{ line: 0, state: "pending" as const, conditional: true }]),
    });
    const dot = view.dom.querySelector(".cm-bp-stack .cm-bp");
    expect(dot?.className).toContain("cm-bp-pending");
    expect(dot?.className).toContain("cm-bp-conditional");
    view.destroy();
  });

  it("tells apart bound, not-yet-bound, and refused — three states, not two", () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({
      effects: setBreakpointMarks.of([
        { line: 0, state: "refused" as const, conditional: false, message: "Server disconnected" },
        // Not confirmed yet, and nothing has gone wrong. Every breakpoint in
        // a compiled language looks like this until the module loads, which
        // is why it must not wear the refused mark.
        { line: 1, state: "pending" as const, conditional: false },
      ]),
    });

    const dots = [...view.dom.querySelectorAll(".cm-bp")];
    expect(dots[0].className).toContain("cm-bp-broken");
    expect((dots[0] as HTMLElement).dataset.tip).toBe("Server disconnected");
    expect(dots[1].className).not.toContain("cm-bp-broken");
    expect(dots[1].className).toContain("cm-bp-pending");
    view.destroy();
  });

  it("puts a target on code lines, and none beside prose", () => {
    // The first breakpoint is the one nobody can set: with markers only where
    // breakpoints already are, the strip is invisible. But a target beside a
    // paragraph is an invitation to find out later that it was never possible.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const doc = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
Prose.

<hick:file path="app.py">
x = 1
</hick:file>
</hick:doc>
`;
    const view = new EditorView({ state: EditorState.create({ doc, extensions }) });
    const ghosts = view.dom.querySelectorAll(".cm-breakpoint-gutter .cm-bp-ghost");
    expect(ghosts.length).toBeGreaterThan(0);
    // One code line in this document, so one target.
    expect(ghosts.length).toBe(1);
    view.destroy();
  });

});


</storycode:copy>
<a id="f-8d12c7278102-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-7">
describe("the stack, in the gutter", () => {
  const open = () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({
      doc: "def a():\n    b()\n\ndef b():\n    x = 1\n\na()\n",
      extensions,
    });
    return new EditorView({ state });
  };

  it("marks each caller's line, and not the one it is stopped at", () => {
    const view = open();
    view.dispatch({
      effects: [
        setPausedLine.of(4),
        setStackMarks.of([
          { line: 1, id: 7, name: "a", depth: 1 },
          { line: 6, id: 8, name: "<module>", depth: 2 },
        ]),
      ],
    });

    const frames = [...view.dom.querySelectorAll(".cm-frame-arrow")];
    expect(frames.length).toBe(2);
    expect((frames[0] as HTMLElement).dataset.tip).toContain("Called from a");
    expect((frames[1] as HTMLElement).dataset.tip).toContain("2 frames up");
    // The paused line keeps the solid arrow; a caller never gets one.
    expect(view.dom.querySelectorAll(".cm-paused-arrow").length).toBe(1);
    view.destroy();
  });

  it("keeps a breakpoint visible where a caller's line also has one", () => {
    // The breakpoint is the thing you can act on; the frame mark is context.
    const view = open();
    view.dispatch({
      effects: [
        setBreakpointMarks.of([{ line: 1, state: "bound" as const, conditional: false }]),
        setStackMarks.of([{ line: 1, id: 7, name: "a", depth: 1 }]),
      ],
    });
    expect(view.dom.querySelectorAll(".cm-bp").length).toBe(1);
    expect(view.dom.querySelectorAll(".cm-frame-arrow").length).toBe(0);
    view.destroy();
  });

  it("flashes the line it moves to, then stops", async () => {
    const view = open();
    revealLine(view, 3, 20);
    expect(view.dom.querySelectorAll(".cm-flash-line").length).toBe(1);
    // The caret moves too, so the keyboard follows the eye.
    expect(view.state.doc.lineAt(view.state.selection.main.head).number).toBe(4);

    await new Promise((resolve) => setTimeout(resolve, 60));
    expect(view.dom.querySelectorAll(".cm-flash-line").length).toBe(0);
    view.destroy();
  });

  it("ignores a line that is not in the document", () => {
    // A frame in a library file has no line here, and asking for one must not
    // throw in the middle of a step.
    const view = open();
    expect(() => revealLine(view, 9_000)).not.toThrow();
    expect(view.dom.querySelectorAll(".cm-flash-line").length).toBe(0);
    view.destroy();
  });
});


</storycode:copy>
<a id="f-8d12c7278102-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-8">
describe("which frames become gutter marks", () => {
  // The real shape, as the engine sends it: paused inside `line_total`,
  // called from a generator expression, from `order_total`, from the module.
  const FRAMES = [
    { id: 2, name: "line_total", line: 28, source: "orders.py", in_document: true },
    { id: 3, name: "<genexpr>", line: 35, source: "orders.py", in_document: true },
    { id: 4, name: "order_total", line: 35, source: "orders.py", in_document: true },
    { id: 5, name: "<module>", line: 39, source: "orders.py", in_document: true },
  ];

  it("marks the callers, not the line execution is on", () => {
    const marks = stackMarksOf(FRAMES, 28);
    expect(marks.map((m) => m.line)).toEqual([35, 39]);
    expect(marks[0]).toMatchObject({ name: "<genexpr>", depth: 1 });
    expect(marks[1]).toMatchObject({ name: "<module>", depth: 3 });
  });

  it("keeps the nearer frame when two share a line", () => {
    // `<genexpr>` and `order_total` are both on line 35; one arrow, and it
    // names the frame you would step out into first.
    expect(stackMarksOf(FRAMES, 28).filter((m) => m.line === 35)).toHaveLength(1);
  });

  it("skips frames outside the document and frames with no line", () => {
    const marks = stackMarksOf(
      [
        { id: 1, name: "top", line: 1, source: "orders.py", in_document: true },
        { id: 2, name: "json.loads", line: null, source: "json/__init__.py", in_document: false },
        { id: 3, name: "runner", line: null, source: "orders.py", in_document: true },
        { id: 4, name: "main", line: 9, source: "orders.py", in_document: true },
      ],
      1,
    );
    expect(marks.map((m) => m.name)).toEqual(["main"]);
  });

  it("marks nothing when the stack is one frame deep", () => {
    expect(
      stackMarksOf([{ id: 1, name: "<module>", line: 3, source: "orders.py", in_document: true }], 3),
    ).toEqual([]);
  });

  it("still marks a caller when nothing is paused", () => {
    // Selecting a frame moves the paused marker; the rest stay callers.
    expect(stackMarksOf(FRAMES, null).map((m) => m.line)).toEqual([35, 39]);
  });
});


</storycode:copy>
<a id="f-8d12c7278102-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-9">
describe("watches, at the end of the line that mentions them", () => {
  it("puts each watch beside the first line mentioning its expression", () => {
    const placements = watchPlacements(DOC, [{ expression: "unit_price", value: "9.99" }], 2);
    expect(placements).toEqual([{ at: DOC.line(1).to, text: "unit_price = 9.99" }]);
  });

  it("matches a whole identifier, not a fragment of a longer one", () => {
    // Watching `sum` must not land inside `subtotal`.
    const placements = watchPlacements(DOC, [{ expression: "sum", value: "0" }], 2);
    expect(placements[0].at).toBe(DOC.line(3).to); // the paused line, not line 2
  });

  it("falls back to the paused line when the expression appears nowhere", () => {
    const placements = watchPlacements(DOC, [{ expression: "quantity * 2", value: "4" }], 1);
    expect(placements).toEqual([{ at: DOC.line(2).to, text: "quantity * 2 = 4" }]);
  });

  it("finds a compound expression by literal match", () => {
    const placements = watchPlacements(
      DOC,
      [{ expression: "quantity * unit_price", value: "19.98" }],
      2,
    );
    expect(placements[0].at).toBe(DOC.line(2).to);
  });

  it("shows nothing while nothing is paused", () => {
    // A watch's value belongs to a stopped frame; afterwards it is the past
    // dressed up as the present.
    expect(watchPlacements(DOC, [{ expression: "quantity", value: "2" }], null)).toEqual([]);
  });

  it("says the value is still coming rather than inventing one", () => {
    const placements = watchPlacements(DOC, [{ expression: "quantity", value: null }], 1);
    expect(placements[0].text).toBe("quantity = …");
  });

  it("joins watches that share a line into one widget", () => {
    // Two widgets at the same position would be two ranges to keep ordered;
    // one string is one fact per line.
    const placements = watchPlacements(
      DOC,
      [
        { expression: "quantity", value: "2" },
        { expression: "unit_price", value: "9.99" },
      ],
      2,
    );
    expect(placements).toHaveLength(1);
    expect(placements[0].text).toBe("quantity = 2  unit_price = 9.99");
  });

  it("draws them as end-of-line widgets, leaving the text untouched", () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const doc = "a = 1\nb = 2\n";
    const view = new EditorView({ state: EditorState.create({ doc, extensions }) });
    view.dispatch({
      effects: [setPausedLine.of(1), setWatchValues.of([{ expression: "a", value: "1" }])],
    });
    const widget = view.dom.querySelector(".cm-debug-watch");
    expect(widget?.textContent).toBe("a = 1");
    // The document itself is exactly what it was: the widget adds no text.
    expect(view.state.doc.toString()).toBe(doc);
    view.destroy();
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-10"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-10">
describe("what a gutter click means", () => {
  const HICK = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
Prose.

<hick:file path="app.py">
x = 1
y = 2
z = 3
</hick:file>
</hick:doc>
`;
  const codeLine = (needle: string) => {
    const lines = HICK.split("\n");
    return lines.findIndex((line) => line.includes(needle));
  };

  const open = () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    return new EditorView({ state: EditorState.create({ doc: HICK, extensions }) });
  };

  it("selects the frame on a line with a stack mark", () => {
    const view = open();
    const line = codeLine("y = 2");
    view.dispatch({ effects: setStackMarks.of([{ line, id: 4, name: "caller", depth: 1 }]) });
    expect(gutterAction(view.state, line)).toEqual({ kind: "frame", id: 4 });
    view.destroy();
  });

  it("prefers the breakpoint where a caller's line also has one", () => {
    // The dot is what is drawn there (the frame arrow yields), so the click
    // must act on what the person can see.
    const view = open();
    const line = codeLine("y = 2");
    view.dispatch({
      effects: [
        setBreakpointMarks.of([{ line, state: "bound" as const, conditional: false }]),
        setStackMarks.of([{ line, id: 4, name: "caller", depth: 1 }]),
      ],
    });
    expect(gutterAction(view.state, line)).toEqual({ kind: "breakpoint", line });
    view.destroy();
  });

  it("toggles a breakpoint on a plain code line", () => {
    const view = open();
    const line = codeLine("x = 1");
    expect(gutterAction(view.state, line)).toEqual({ kind: "breakpoint", line });
    view.destroy();
  });

  it("refuses prose entirely", () => {
    const view = open();
    expect(gutterAction(view.state, codeLine("Prose."))).toBeNull();
    view.destroy();
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-11"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-11">
describe("where a breakpoint can go", () => {
  const DOC_SOURCE = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
# A heading

Prose about the program.

<hick:file path="app.py">
x = 1

y = 2
</hick:file>

<hick:file path="notes.md">
Not code.
</hick:file>
</hick:doc>
`;

  const state = () => EditorState.create({ doc: DOC_SOURCE });
  const lineOf = (needle: string) => {
    const state0 = state();
    for (let i = 1; i <= state0.doc.lines; i += 1) {
      if (state0.doc.line(i).text.includes(needle)) return i - 1;
    }
    throw new Error(`no line with ${needle}`);
  };

  it("refuses prose: a document is mostly text about the program", () => {
    expect(breakpointLine(state(), lineOf("# A heading"))).toBeNull();
    expect(breakpointLine(state(), lineOf("Prose about"))).toBeNull();
  });

  it("refuses a file in a language nothing can debug", () => {
    expect(breakpointLine(state(), lineOf("Not code."))).toBeNull();
  });

  it("takes a line of code as it is", () => {
    const line = lineOf("x = 1");
    expect(breakpointLine(state(), line)).toBe(line);
  });

  it("slides off a blank line onto the next line with something on it", () => {
    // What every editor does, and what the adapter would do anyway — only
    // sooner, and visibly.
    const blank = lineOf("x = 1") + 1;
    expect(state().doc.line(blank + 1).text.trim()).toBe("");
    expect(breakpointLine(state(), blank)).toBe(lineOf("y = 2"));
  });

  it("does not slide out of its block", () => {
    // The blank line after `y = 2` is the last line of the block; there is
    // nothing below it that belongs to this program.
    const trailing = lineOf("y = 2") + 1;
    expect(breakpointLine(state(), trailing)).toBeNull();
  });

  it("says nothing about a line outside the document", () => {
    expect(breakpointLine(state(), 9_000)).toBeNull();
    expect(breakpointLine(state(), -1)).toBeNull();
  });

  // `DEBUGGABLE_LANGUAGES` is kept in step with `hick_dap::discovery` by hand,
  // and it had fallen behind: csharp had an adapter, a `hick dap install`
  // recipe and a Build step, and this list still did not name it — so a `.cs`
  // block offered no ghost dot, the breakpoint gutter rendered no cells at
  // all, and a click in the gutter did nothing at all rather than refusing in
  // words. A file's own extension is what decides, with no `language=`.
  it("takes a line of C#, which netcoredbg debugs", () => {
    const doc = `<?xml version="1.0" encoding="UTF-8"?>
<hick:doc xmlns:hick="http://www.hickorydocs.com/1.0" weave="out.md">
<hick:file path="app/Program.cs">
Console.WriteLine("Hello, World!");
</hick:file>
<hick:file path="app/app.csproj">
<Project Sdk="Microsoft.NET.Sdk"></Project>
</hick:file>
</hick:doc>
`;
    const cs = EditorState.create({ doc });
    const lineWith = (needle: string) => {
      for (let i = 1; i <= cs.doc.lines; i += 1) {
        if (cs.doc.line(i).text.includes(needle)) return i - 1;
      }
      throw new Error(`no line with ${needle}`);
    };
    const code = lineWith("Console.WriteLine");
    expect(breakpointLine(cs, code)).toBe(code);
    // The project file beside it is XML, which nothing debugs.
    expect(breakpointLine(cs, lineWith("<Project Sdk"))).toBeNull();
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-12"></a>
### `expression_statement`

Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md

<storycode:copy id="f-8d12c7278102-expression_statement-12">
// Protects docs/guarantees/debugging/a-plain-file-has-the-same-debugger.md
describe("a plain file's breakpoints", () => {
  const plain = (language: string) =>
    EditorState.create({
      doc: "import os\n\nprint(1)\n",
      extensions: debugEditor({ language, lineNumbers: false, onToggleBreakpoint: () => {} }),
    });

  it("lets any line with code hold one, sliding a blank line down", () => {
    // No block structure to read: the whole file is code, and the only rule
    // left is the one about blank lines.
    expect(breakpointLine(plain("python"), 0)).toBe(0);
    expect(breakpointLine(plain("python"), 1)).toBe(2);
    // The trailing newline's empty line has nothing below it.
    expect(breakpointLine(plain("python"), 3)).toBeNull();
  });

  it("refuses every line of a language hick cannot debug", () => {
    expect(breakpointLine(plain("justfile"), 0)).toBeNull();
  });
});


</storycode:copy>
<a id="f-8d12c7278102-expression_statement-13"></a>
### `expression_statement`

Protects docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md

<storycode:copy id="f-8d12c7278102-expression_statement-13">
// Protects docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
describe("a conditional breakpoint is visibly one", () => {
  it("computes `conditional` from what is set, rather than assuming false", () => {
    const effects = debugStateEffects({
      breakpoints: [
        { line: 0, state: "bound" },
        { line: 1, state: "bound", condition: "i > 3" },
        { line: 2, state: "bound", log_message: "here" },
        { line: 3, state: "bound", hit_condition: "> 5" },
      ],
      pausedLine: null,
      variables: [],
      frames: [],
      watches: [],
    });
    const marks = (effects[0] as unknown as { value: { line: number; conditional: boolean }[] })
      .value;
    expect(marks.map((m) => m.conditional)).toEqual([false, true, true, true]);
  });

  it("draws the wedge, not a plain dot", () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({
      effects: setBreakpointMarks.of([
        { line: 0, state: "bound" as const, conditional: true, condition: "i > 3" },
      ]),
    });
    expect(view.dom.querySelector(".cm-bp-conditional")).not.toBeNull();
  });

  it("says what the condition is, where it is asked", () => {
    expect(breakpointTip({ line: 0, state: "bound", conditional: false })).toBe("Breakpoint");
    expect(
      breakpointTip({ line: 0, state: "bound", conditional: true, condition: "i > 3" }),
    ).toBe("Stops when i > 3");
    expect(
      breakpointTip({
        line: 0,
        state: "bound",
        conditional: true,
        logMessage: "at {i}",
        hitCondition: "> 2",
      }),
    ).toBe('Logs "at {i}" and continues · Hit count > 2');
    // A reason it could not bind still wins: that is the more urgent fact.
    expect(
      breakpointTip({
        line: 0,
        state: "refused",
        conditional: true,
        condition: "i > 3",
        message: "no code on this line",
      }),
    ).toBe("no code on this line");
  });

  it("redraws when only the condition changed", () => {
    // `eq` used to compare state and the conditional flag alone, so editing
    // a condition left the old tooltip in place.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({
      effects: setBreakpointMarks.of([
        { line: 0, state: "bound" as const, conditional: true, condition: "i > 3" },
      ]),
    });
    view.dispatch({
      effects: setBreakpointMarks.of([
        { line: 0, state: "bound" as const, conditional: true, condition: "i > 99" },
      ]),
    });
    expect(view.dom.querySelector<HTMLElement>(".cm-bp")?.dataset.tip).toBe("Stops when i > 99");
  });
});

</storycode:copy>
<a id="f-8d12c7278102-expression_statement-14"></a>
### `expression_statement`

<storycode:copy id="f-8d12c7278102-expression_statement-14">
describe("editing a breakpoint from the gutter", () => {
  const connectedViews: EditorView[] = [];
  afterEach(() => { for (const view of connectedViews.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
  // The gutter's own `domEventHandlers` never fire under jsdom, which has no
  // layout to resolve a pointer's height against — the same reason
  // `gutterAction` is a pure exported function rather than something a test
  // clicks. So the gesture is tested as an intent, and the panel directly.
  const withMarks = (
    marks: { line: number; state: "bound"; conditional: boolean; condition?: string }[],
  ) => {
    const view = new EditorView({
      parent: document.body.appendChild(document.createElement("div")),
      state: EditorState.create({
        doc: "a = 1\nb = 2\nc = 3\n",
        // A plain file, so every line is code and can hold a breakpoint —
        // the document path needs a hick block around it, which is tested
        // by `gutterAction`'s own cases above.
        extensions: debugEditor({ onToggleBreakpoint: () => {}, language: "python" }),
      }),
    });
    connectedViews.push(view);
    view.dispatch({ effects: setBreakpointMarks.of(marks) });
    return view;
  };

  it("toggles on a plain click and edits on alt or right click", () => {
    const view = withMarks([{ line: 0, state: "bound", conditional: false }]);
    expect(gutterIntent(view.state, 0, {}, true)).toEqual({ kind: "toggle", line: 0 });
    expect(gutterIntent(view.state, 0, { alt: true }, true)).toEqual({ kind: "edit", line: 0 });
    expect(gutterIntent(view.state, 0, { context: true }, true)).toEqual({ kind: "edit", line: 0 });
  });

  it("falls back to toggling in a pane that cannot set a condition", () => {
    const view = withMarks([{ line: 0, state: "bound", conditional: false }]);
    expect(gutterIntent(view.state, 0, { alt: true }, false)).toEqual({ kind: "toggle", line: 0 });
    // And a right-click there is left to the browser rather than swallowed.
    expect(gutterIntent(view.state, 0, { context: true }, false)).toBeNull();
  });

  it("still selects a frame, and never edits one", () => {
    const view = withMarks([]);
    view.dispatch({ effects: setStackMarks.of([{ line: 1, id: 7, name: "main", depth: 1 }]) });
    expect(gutterIntent(view.state, 1, {}, true)).toEqual({ kind: "frame", id: 7 });
    expect(gutterIntent(view.state, 1, { context: true }, true)).toBeNull();
  });

  it("shows what is already set", () => {
    const view = withMarks([]);
    openBreakpointEditor(
      view,
      { line: 1, state: "bound", conditional: true, condition: "i > 3", hitCondition: "> 2" },
      () => {},
    );
    const inputs = [...view.dom.querySelectorAll<HTMLInputElement>(".cm-bp-editor input")];
    expect(inputs).toHaveLength(3);
    expect(inputs.map((input) => input.value)).toEqual(["i > 3", "> 2", ""]);
    expect(view.dom.querySelector(".cm-bp-editor")?.getAttribute("aria-label")).toBe(
      "Breakpoint on line 2",
    );
  });

  it("applies what was typed on Enter, trimmed, and closes", () => {
    const set: [number, Record<string, string | undefined>][] = [];
    const view = withMarks([]);
    openBreakpointEditor(view, { line: 0, state: "bound", conditional: false }, (line, patch) =>
      set.push([line, patch]),
    );
    const inputs = [...view.dom.querySelectorAll<HTMLInputElement>(".cm-bp-editor input")];
    inputs[0].value = "  n == 3  ";
    inputs[0].dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));

    expect(set).toEqual([[0, { condition: "n == 3", hit_condition: "", log_message: "" }]]);
    expect(view.dom.querySelector(".cm-bp-editor")).toBeNull();
  });

  it("discards on Escape without setting anything", () => {
    const set: unknown[] = [];
    const view = withMarks([]);
    openBreakpointEditor(view, { line: 0, state: "bound", conditional: false }, (line, patch) =>
      set.push([line, patch]),
    );
    const input = view.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
    input.value = "nope";
    input.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    expect(set).toEqual([]);
    expect(view.dom.querySelector(".cm-bp-editor")).toBeNull();
  });

  it("never leaves two panels open", () => {
    const view = withMarks([]);
    openBreakpointEditor(view, { line: 0, state: "bound", conditional: false }, () => {});
    openBreakpointEditor(view, { line: 1, state: "bound", conditional: false }, () => {});
    expect(view.dom.querySelectorAll(".cm-bp-editor")).toHaveLength(1);
  });

  it("sets a breakpoint first when the line has none", () => {
    const toggled: number[] = [];
    const view = withMarks([]);
    editBreakpoint(view, 2, {
      onToggleBreakpoint: (line) => toggled.push(line),
      onSetBreakpointCondition: () => {},
    });
    expect(toggled).toEqual([2]);
    expect(view.dom.querySelector(".cm-bp-editor")).not.toBeNull();
  });

  it("does not add a second breakpoint when one is already there", () => {
    const toggled: number[] = [];
    const view = withMarks([{ line: 0, state: "bound", conditional: false }]);
    editBreakpoint(view, 0, {
      onToggleBreakpoint: (line) => toggled.push(line),
      onSetBreakpointCondition: () => {},
    });
    expect(toggled).toEqual([]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/debug/cmDebug.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/cmDebug.test.ts"><storycode:paste select="#f-8d12c7278102-expression_statement"/><storycode:paste select="#f-8d12c7278102-DOC"/><storycode:paste select="#f-8d12c7278102-vars"/><storycode:paste select="#f-8d12c7278102-expression_statement-2"/><storycode:paste select="#f-8d12c7278102-expression_statement-3"/><storycode:paste select="#f-8d12c7278102-expression_statement-4"/><storycode:paste select="#f-8d12c7278102-expression_statement-5"/><storycode:paste select="#f-8d12c7278102-expression_statement-6"/><storycode:paste select="#f-8d12c7278102-expression_statement-7"/><storycode:paste select="#f-8d12c7278102-expression_statement-8"/><storycode:paste select="#f-8d12c7278102-expression_statement-9"/><storycode:paste select="#f-8d12c7278102-expression_statement-10"/><storycode:paste select="#f-8d12c7278102-expression_statement-11"/><storycode:paste select="#f-8d12c7278102-expression_statement-12"/><storycode:paste select="#f-8d12c7278102-expression_statement-13"/><storycode:paste select="#f-8d12c7278102-expression_statement-14"/></storycode:file>


</storycode:doc>
