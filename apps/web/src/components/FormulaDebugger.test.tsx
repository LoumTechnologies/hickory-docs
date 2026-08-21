// Protects docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";

import { api } from "../api/client";
import type { FormulaStep } from "../api/types";

import { FormulaDebugger } from "./FormulaDebugger";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

const STEPS: FormulaStep[] = [
  {
    cell: "B1",
    level: 0,
    expression: "A1*2",
    bindings: [{ cell: "A1", text: "10", kind: "number" }],
    value: "20",
    error: null,
  },
  {
    cell: "A2",
    level: 1,
    expression: "B1+5",
    bindings: [{ cell: "B1", text: "20", kind: "number" }],
    value: "25",
    error: null,
  },
];

const panel = (over: Partial<React.ComponentProps<typeof FormulaDebugger>> = {}) =>
  render(
    <FormulaDebugger
      language="python"
      rows={[["10", "=A1*2"], ["=B1+5", ""]]}
      revision="r1"
      onStep={() => {}}
      onClose={() => {}}
      {...over}
    />,
  );

const where = () => screen.getByTestId("formula-debug-where").textContent;

describe("stepping through the formulas", () => {
  it("starts on the first cell that ran, not on the first cell in the grid", () => {
    // The order is the host's, and it is the thing being debugged. A1 is a
    // literal: it never ran, so it has no step.
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel();
    return waitFor(() => {
      expect(where()).toContain("Step 1 of 2");
      expect(where()).toContain("B1");
    });
  });

  it("says which batch a cell went out in, because that is the round trip", () => {
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel();
    return waitFor(() => expect(where()).toContain("batch 1 of 2"));
  });

  it("shows what the cell READ, which the grid cannot", async () => {
    // A value in a cell is what it says now; a formula saw what its
    // references were worth when its own turn came.
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel();
    await waitFor(() => expect(screen.getByTestId("formula-debug-expression")).toBeTruthy());
    expect(screen.getByTestId("formula-debug-expression").textContent).toContain("A1*2");
    expect(screen.getByRole("row", { name: /A1/ }).textContent).toContain("10");
    expect(screen.getByTestId("formula-debug-result").textContent).toBe("→ 20");
  });

  it("steps forward and back, and runs to the end", async () => {
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel();
    await waitFor(() => expect(where()).toContain("Step 1 of 2"));
    fireEvent.click(screen.getByRole("button", { name: "Step ▶" }));
    expect(where()).toContain("Step 2 of 2");
    expect(where()).toContain("A2");
    fireEvent.click(screen.getByRole("button", { name: "◀ Back" }));
    expect(where()).toContain("Step 1 of 2");
    fireEvent.click(screen.getByRole("button", { name: "⏭" }));
    expect(where()).toContain("Step 2 of 2");
  });

  it("cannot step past either end", async () => {
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel();
    await waitFor(() => expect(where()).toContain("Step 1 of 2"));
    expect(screen.getByRole("button", { name: "◀ Back" })).toHaveProperty("disabled", true);
    fireEvent.click(screen.getByRole("button", { name: "Step ▶" }));
    expect(screen.getByRole("button", { name: "Step ▶" })).toHaveProperty("disabled", true);
  });

  it("names a blank cell `empty` rather than drawing nothing", async () => {
    // A blank cell is not the empty string — `sum` skips one and not the
    // other — and a debugger that showed both as nothing would hide it.
    vi.spyOn(api, "traceFormulas").mockResolvedValue({
      steps: [
        {
          cell: "A1",
          level: 0,
          expression: "Z9",
          bindings: [{ cell: "Z9", text: "", kind: "empty" }],
          value: "",
          error: null,
        },
      ],
      values: {},
      errors: {},
    });
    panel();
    await waitFor(() => expect(screen.getByText("empty")).toBeTruthy());
  });

  it("shows the language's own message when the stepped cell broke", async () => {
    vi.spyOn(api, "traceFormulas").mockResolvedValue({
      steps: [
        {
          cell: "A1",
          level: 0,
          expression: "nope",
          bindings: [],
          value: null,
          error: "NameError: name 'nope' is not defined",
        },
      ],
      values: {},
      errors: {},
    });
    panel();
    await waitFor(() =>
      expect(screen.getByTestId("formula-debug-result").textContent).toContain("NameError"),
    );
  });

  it("says a cell that reads nothing ran in the first batch", async () => {
    vi.spyOn(api, "traceFormulas").mockResolvedValue({
      steps: [
        { cell: "A1", level: 0, expression: "1+1", bindings: [], value: "2", error: null },
      ],
      values: {},
      errors: {},
    });
    panel();
    await waitFor(() => expect(screen.getByText(/Reads no other cell/)).toBeTruthy());
  });

  it("has nothing to step through in a circle, and says what the circle is", async () => {
    // A circle has no order; inventing one to step through would be the
    // debugger telling its first lie.
    vi.spyOn(api, "traceFormulas").mockResolvedValue({
      steps: [],
      values: {},
      errors: { A1: "these cells depend on each other in a circle: A1 → B1" },
    });
    panel();
    await waitFor(() => expect(where()).toBe("Nothing to step through"));
    expect(screen.getByRole("status").textContent).toContain("circle");
  });

  it("says so, rather than sitting empty, when nothing can evaluate", async () => {
    vi.spyOn(api, "traceFormulas").mockRejectedValue(
      new Error("formulas in `python` need one of: python3, python"),
    );
    panel();
    await waitFor(() => expect(screen.getByRole("status").textContent).toMatch(/python3/));
  });

  it("reports the step it is on, so the grid can mark the cell", async () => {
    const onStep = vi.fn();
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    panel({ onStep });
    await waitFor(() => expect(onStep).toHaveBeenCalledWith(STEPS[0]));
    fireEvent.click(screen.getByRole("button", { name: "Step ▶" }));
    expect(onStep).toHaveBeenCalledWith(STEPS[1]);
  });

  it("clears the mark when it closes, so no cell stays highlighted", async () => {
    const onStep = vi.fn();
    vi.spyOn(api, "traceFormulas").mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    const view = panel({ onStep });
    await waitFor(() => expect(onStep).toHaveBeenCalledWith(STEPS[0]));
    view.unmount();
    expect(onStep).toHaveBeenLastCalledWith(null);
  });

  it("traces again when the table changes, rather than describing a table that is gone", async () => {
    const trace = vi
      .spyOn(api, "traceFormulas")
      .mockResolvedValue({ steps: STEPS, values: {}, errors: {} });
    const view = panel();
    await waitFor(() => expect(trace).toHaveBeenCalledTimes(1));
    view.rerender(
      <FormulaDebugger
        language="python"
        rows={[["11", "=A1*2"]]}
        revision="r2"
        onStep={() => {}}
        onClose={() => {}}
      />,
    );
    await waitFor(() => expect(trace).toHaveBeenCalledTimes(2));
  });
});
