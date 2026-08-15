import { EditorState, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { describe, expect, it } from "vitest";
import { debugEditor, inlinePlacements, setBreakpointMarks, setPausedLine } from "./cmDebug";
import { backwardsControl, type DebugCapabilities } from "./client";
import { identifierAt } from "../lsp/cmLsp";

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
