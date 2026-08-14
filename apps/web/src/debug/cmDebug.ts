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

import { RangeSet, RangeSetBuilder, StateEffect, StateField } from "@codemirror/state";
import type { Extension } from "@codemirror/state";
import { Decoration, EditorView, GutterMarker, WidgetType, gutter } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import type { Variable } from "./client";

/** One gutter dot. */
export interface BreakpointMark {
  line: number;
  verified: boolean;
  conditional: boolean;
  /** Why it could not bind, shown on hover. */
  message?: string;
}

export const setBreakpointMarks = StateEffect.define<BreakpointMark[]>();
export const setPausedLine = StateEffect.define<number | null>();
export const setInlineValues = StateEffect.define<Variable[]>();

// --- the gutter -------------------------------------------------------------

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
    const dot = document.createElement("span");
    dot.className =
      "cm-bp" +
      (this.mark.verified ? "" : " cm-bp-unverified") +
      (this.mark.conditional ? " cm-bp-conditional" : "");
    // The reason a hollow dot is hollow. Without this, an unbindable
    // breakpoint is a mystery rather than a message.
    dot.title = this.mark.message
      ? this.mark.message
      : this.mark.conditional
        ? "Conditional breakpoint"
        : "Breakpoint";
    return dot;
  }
}

class PausedMarker extends GutterMarker {
  toDOM() {
    const arrow = document.createElement("span");
    arrow.className = "cm-paused-arrow";
    arrow.title = "Execution is paused here";
    return arrow;
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

// --- inline values ----------------------------------------------------------

/** The greyed `x = 1` at the end of a line while paused. */
class InlineValue extends WidgetType {
  constructor(private readonly text: string) {
    super();
  }
  // Without this, every value is redrawn on every update and the caret
  // flickers as the DOM under it is replaced.
  eq(other: InlineValue) {
    return other.text === this.text;
  }
  toDOM() {
    const span = document.createElement("span");
    span.className = "cm-debug-value";
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

// --- the extension ----------------------------------------------------------

export interface DebugEditorOptions {
  /** Toggle a breakpoint on this 0-based document line. */
  onToggleBreakpoint: (line: number) => void;
}

export function debugEditor(options: DebugEditorOptions): Extension[] {
  return [
    breakpointField,
    pausedField,
    pausedHighlight,
    inlineField,
    gutter({
      class: "cm-breakpoint-gutter",
      markers: (view) => {
        const marks = view.state.field(breakpointField);
        const paused = view.state.field(pausedField);
        const ranges: { from: number; value: GutterMarker }[] = [];
        for (const mark of marks) {
          if (mark.line < 0 || mark.line >= view.state.doc.lines) continue;
          ranges.push({
            from: view.state.doc.line(mark.line + 1).from,
            value: new DotMarker(mark),
          });
        }
        if (paused !== null && paused >= 0 && paused < view.state.doc.lines) {
          ranges.push({
            from: view.state.doc.line(paused + 1).from,
            value: new PausedMarker(),
          });
        }
        // RangeSet needs ascending order, and the paused line is appended
        // last rather than in place.
        ranges.sort((a, b) => a.from - b.from);
        return RangeSet.of(
          ranges.map((range) => range.value.range(range.from)),
          true,
        );
      },
      domEventHandlers: {
        mousedown(view, block) {
          const line = view.state.doc.lineAt(block.from).number - 1;
          options.onToggleBreakpoint(line);
          return true;
        },
      },
    }),
    EditorView.theme({
      ".cm-breakpoint-gutter": { width: "1.1rem", cursor: "pointer" },
    }),
  ];
}
