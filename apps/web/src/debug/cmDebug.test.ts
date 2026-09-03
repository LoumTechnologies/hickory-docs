import { EditorState, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { beforeAll, describe, expect, it } from "vitest";
import {
  breakpointLine,
  debugEditor,
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

