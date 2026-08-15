import { EditorState, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { beforeAll, describe, expect, it } from "vitest";
import {
  debugEditor,
  inlinePlacements,
  revealLine,
  setBreakpointMarks,
  setPausedLine,
  setStackMarks,
  stackMarksOf,
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

const DOC = Text.of([
  "def line_total(quantity, unit_price):",
  "    subtotal = quantity * unit_price",
  "    return subtotal",
]);

const vars = (entries: [string, string][]) =>
  entries.map(([name, value]) => ({ name, value, variables_reference: 0 }));

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

describe("the editor state the debugger drives", () => {
  it("builds without a document open", () => {
    // The extensions are installed whether or not a session exists, so they
    // must be inert until one does.
    const state = EditorState.create({ doc: "x = 1" });
    expect(state.doc.length).toBe(5);
  });
});

describe("the gutter is findable before it holds anything", () => {
  it("gives a line with a breakpoint exactly one marker", () => {
    // The ghost and the dot share a cell one dot wide, so two markers wrap:
    // the breakpoint renders on a second row inside the cell and reads as a
    // dot beside the NEXT line. Measured in the running app before it was
    // fixed; asserted here so it cannot come back.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\nc = 3\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({ effects: setBreakpointMarks.of([{ line: 1, verified: true, conditional: false }]) });
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
        setBreakpointMarks.of([{ line: 0, verified: true, conditional: false }]),
        setPausedLine.of(0),
      ],
    });

    const stack = view.dom.querySelector(".cm-bp-stack");
    expect(stack).not.toBeNull();
    expect(stack?.querySelector(".cm-bp")).not.toBeNull();
    expect(stack?.querySelector(".cm-paused-arrow")).not.toBeNull();
    // And the dot keeps saying what kind of breakpoint it is.
    view.dispatch({
      effects: setBreakpointMarks.of([{ line: 0, verified: false, conditional: true }]),
    });
    const dot = view.dom.querySelector(".cm-bp-stack .cm-bp");
    expect(dot?.className).toContain("cm-bp-unverified");
    expect(dot?.className).toContain("cm-bp-conditional");
    view.destroy();
  });

  it("draws a refused breakpoint as broken, with the reason on hover", () => {
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    view.dispatch({
      effects: setBreakpointMarks.of([
        { line: 0, verified: false, conditional: false, message: "Server disconnected" },
        // Not yet bound, but nothing has gone wrong: a different state.
        { line: 1, verified: false, conditional: false },
      ]),
    });

    const dots = [...view.dom.querySelectorAll(".cm-bp")];
    expect(dots[0].className).toContain("cm-bp-broken");
    expect((dots[0] as HTMLElement).title).toBe("Server disconnected");
    expect(dots[1].className).not.toContain("cm-bp-broken");
    expect(dots[1].className).toContain("cm-bp-unverified");
    view.destroy();
  });

  it("puts a marker on every line, not only lines with breakpoints", () => {
    // The first breakpoint is the one nobody can set: with markers only where
    // breakpoints already are, the strip is invisible and "click the gutter"
    // is advice about nothing.
    const extensions = debugEditor({ onToggleBreakpoint: () => {} });
    const state = EditorState.create({ doc: "a = 1\nb = 2\n", extensions });
    const view = new EditorView({ state });
    const gutters = view.dom.querySelectorAll(".cm-breakpoint-gutter .cm-bp-ghost");
    expect(gutters.length).toBeGreaterThan(0);
    view.destroy();
  });
});


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
          { line: 1, name: "a", depth: 1 },
          { line: 6, name: "<module>", depth: 2 },
        ]),
      ],
    });

    const frames = [...view.dom.querySelectorAll(".cm-frame-arrow")];
    expect(frames.length).toBe(2);
    expect((frames[0] as HTMLElement).title).toBe("Called from a");
    expect((frames[1] as HTMLElement).title).toContain("2 frames up");
    // The paused line keeps the solid arrow; a caller never gets one.
    expect(view.dom.querySelectorAll(".cm-paused-arrow").length).toBe(1);
    view.destroy();
  });

  it("keeps a breakpoint visible where a caller's line also has one", () => {
    // The breakpoint is the thing you can act on; the frame mark is context.
    const view = open();
    view.dispatch({
      effects: [
        setBreakpointMarks.of([{ line: 1, verified: true, conditional: false }]),
        setStackMarks.of([{ line: 1, name: "a", depth: 1 }]),
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
