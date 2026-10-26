<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/FormulaDebugger.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3a3b3fc26b23"></a>
## `apps/web/src/components/FormulaDebugger.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Formula Is An Expression In A Real Language](../../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md).

<a id="f-3a3b3fc26b23-FormulaDebuggerProps"></a>
### `FormulaDebuggerProps`

Stepping through a table's formulas.
What is being debugged here is the ORDER, not the expression. Which cell
went first, what it read when its turn came, and what the value it read was
worth at that moment — that is the host's whole contribution to a formula
(see crates/hick-formula/src/lib.rs), and it is the part a person cannot
see by looking at the grid. A cell showing `#NAME` tells you it broke; only
the order tells you it broke because the cell above it was still empty when
it ran.
Stepping INSIDE an expression is a different tool and belongs to the
language: that is `hick-dap`, over a real debug adapter, in an exec cell.
A formula is one expression in someone else's language, and pretending this
panel could step through its sub-expressions would mean writing a debugger
per backend — the exact thing the backend protocol exists to avoid.
The steps come from the same host call that computes the grid, so the
panel and the values beside it can never disagree about what happened.

<storycode:copy id="f-3a3b3fc26b23-FormulaDebuggerProps">
// Stepping through a table's formulas.
//
// What is being debugged here is the ORDER, not the expression. Which cell
// went first, what it read when its turn came, and what the value it read was
// worth at that moment — that is the host's whole contribution to a formula
// (see crates/hick-formula/src/lib.rs), and it is the part a person cannot
// see by looking at the grid. A cell showing `#NAME` tells you it broke; only
// the order tells you it broke because the cell above it was still empty when
// it ran.
//
// Stepping INSIDE an expression is a different tool and belongs to the
// language: that is `hick-dap`, over a real debug adapter, in an exec cell.
// A formula is one expression in someone else's language, and pretending this
// panel could step through its sub-expressions would mean writing a debugger
// per backend — the exact thing the backend protocol exists to avoid.
//
// The steps come from the same host call that computes the grid, so the
// panel and the values beside it can never disagree about what happened.

import { useEffect, useId, useMemo, useState } from "react";

import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand } from "../lib/uiActions";
import { api } from "../api/client";
import type { FormulaStep } from "../api/types";

export interface FormulaDebuggerProps {
  /** The language the formulas are written in. */
  language: string;
  /** The grid, exactly as the CSV parses. */
  rows: string[][];
  /** Changes whenever the table's text does, so a trace is never stale: an
   * edit re-traces and starts again rather than leaving a step describing a
   * table that no longer exists. */
  revision: string;
  /** The step now showing, so the grid can mark the cell and what it read.
   * Null while nothing is loaded, or when there is nothing to step through. */
  onStep: (step: FormulaStep | null) => void;
  onClose: () => void;
  owner?: string;
}

</storycode:copy>
<a id="f-3a3b3fc26b23-FormulaDebugger"></a>
### `FormulaDebugger`

<storycode:copy id="f-3a3b3fc26b23-FormulaDebugger">
export function FormulaDebugger({
  language,
  rows,
  revision,
  onStep,
  onClose,
  owner,
}: FormulaDebuggerProps) {
  const traceKey = JSON.stringify([language, revision]);
  const [loadedFor, setLoadedFor] = useState<string | null>(null);
  const [storedSteps, setSteps] = useState<FormulaStep[] | null>(null);
  const [storedTrouble, setTrouble] = useState<string | null>(null);
  const steps = loadedFor === traceKey ? storedSteps : null;
  const trouble = loadedFor === traceKey ? storedTrouble : null;
  /** Which step is showing. Zero is the first cell's turn — there is no
   * "before the start" state, because a table has no line to be paused on. */
  const [at, setAt] = useState(0);

  useEffect(() => {
    let live = true;
    setSteps(null);
    setTrouble(null);
    api.traceFormulas(language, rows).then(
      (trace) => {
        if (!live) return;
        setLoadedFor(traceKey); setSteps(trace.steps);
        setAt(0);
        // A cycle answers with no steps and the circle in `errors`, which
        // the grid is already reporting on the cells themselves.
        setTrouble(
          trace.steps.length === 0 && Object.keys(trace.errors).length > 0
            ? Object.values(trace.errors)[0]
            : null,
        );
      },
      (error: unknown) => {
        if (!live) return;
        setLoadedFor(traceKey); setSteps([]);
        setTrouble(error instanceof Error ? error.message : String(error));
      },
    );
      return () => {
      live = false;
    };
    // `revision` stands in for the rows, which are rebuilt on every render.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [language, revision]);

  const step = steps && steps.length > 0 ? steps[Math.min(at, steps.length - 1)] : null;

  useEffect(() => {
    onStep(step);
  }, [step, onStep]);

  // Unmounting clears the grid's marks; without this the cell a closed
  // debugger was paused on stays highlighted forever.
  useEffect(() => () => onStep(null), [onStep]);

  const batches = useMemo(
    () => (steps ? new Set(steps.map((s) => s.level)).size : 0),
    [steps],
  );

  const total = steps?.length ?? 0;
  const first = at <= 0;
  const last = at >= total - 1;

  const parent = `formula-trace:${useId()}`;
  const currentRevision = useUiRevision(() => [traceKey, steps, trouble, at]);
  const choose = (index: number) => setAt(index);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: ["Current captured formula evaluation order; navigation is replay, not reverse execution"], read: () => ({ revision: currentRevision(), enabled, reason: !enabled ? "Wait for a current trace and choose an available step" : undefined, value: { owner: owner ?? null, index: total ? at : null, total } }), execute } });
  for (const [key, label, index, enabled] of [
    ["first", "First formula trace step", 0, !first && total > 0],
    ["previous", "Previous formula trace step", at - 1, !first && total > 0],
    ["next", "Next formula trace step", at + 1, !last && total > 0],
    ["last", "Last formula trace step", total - 1, !last && total > 0],
  ] as const) add(key, label, enabled, async () => { choose(index); return { status: "completed", value: { selected_index: index, scope: "Replay of captured cell evaluation order only" } }; });
  add("select", "Select formula trace step", total > 0, async args => { const index = args.index as number; if (index >= total) return { status: "unavailable", reason: "Step outside current trace" }; choose(index); return { status: "completed", value: { selected_index: index, scope: "Captured order only" } }; }, { index: { type: "number", minimum: 0, maximum: 1000000, integer: true } });
  add("close", "Close formula trace", true, async () => { onClose(); return { status: "completed", value: { close_requested: true } }; });
  add("read", "Read formula trace step bytes", true, async args => {
    const bytes = new TextEncoder().encode(JSON.stringify({ owner: owner ?? null, language, pending: steps === null, total, index: total ? at : null, batches, step, error: trouble, scope: "Actual host trace for the captured table revision supplied to this controller; evaluation order replay does not step inside an expression, reverse a process or independently verify correctness" }));
    const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
    if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside current formula step" };
    return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
  }, { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } });
  const invoke = useUiCommands(commands);
  const click = (key: string) => { void invoke(`${parent}:${key}`).catch(reason => setTrouble(String(reason))); };
  return (
    <div className="formula-debug" data-testid="formula-debug">
      <div className="formula-debug__bar" role="toolbar" aria-label="Step through the formulas">
        <button
          type="button"
          className="btn btn-small"
          disabled={first || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => click("first")}
          data-tip="Back to the first cell that ran"
        >
          ⏮
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={first || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => click("previous")}
          data-tip="The cell that ran before this one"
        >
          ◀ Back
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={last || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => click("next")}
          data-tip="The next cell to run"
        >
          Step ▶
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={last || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => click("last")}
          data-tip="Straight to the last cell"
        >
          ⏭
        </button>
        <span className="formula-debug__where muted" data-testid="formula-debug-where">
          {steps === null
            ? "Working out the order…"
            : total === 0
              ? "Nothing to step through"
              : `Step ${at + 1} of ${total} · ${step?.cell} · batch ${
                  (step?.level ?? 0) + 1
                } of ${batches}`}
        </span>
        <button
          type="button"
          className="btn btn-small"
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => click("close")}
          data-tip="Stop stepping"
        >
          Close
        </button>
      </div>

      {step && (
        <div className="formula-debug__body">
          <p className="formula-debug__expression mono" data-testid="formula-debug-expression">
            <span className="formula-debug__cell">{step.cell}</span>
            <span className="formula-debug__eq">=</span>
            {step.expression}
          </p>

          {/* What the cell READ. The one thing the grid cannot show: a value
              in a cell is what it says NOW, and a formula saw what its
              references were worth when its own turn came. */}
          {step.bindings.length > 0 ? (
            <table className="formula-debug__reads">
              <thead>
                <tr>
                  <th scope="col">Read</th>
                  <th scope="col">Was worth</th>
                </tr>
              </thead>
              <tbody>
                {step.bindings.map((binding) => (
                  <tr key={binding.cell}>
                    <th scope="row" className="mono">
                      {binding.cell}
                    </th>
                    <td className="mono">
                      {binding.kind === "empty" ? (
                        // A blank cell is not the empty string — `sum` skips
                        // one and not the other — so it is named rather than
                        // drawn as nothing at all.
                        <span className="formula-debug__empty">empty</span>
                      ) : (
                        binding.text
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          ) : (
            <p className="formula-debug__reads-none muted">
              Reads no other cell — it depends on nothing, so it ran in the first batch.
            </p>
          )}

          <p
            className={`formula-debug__result mono${
              step.error ? " formula-debug__result--bad" : ""
            }`}
            data-testid="formula-debug-result"
          >
            {step.error ? step.error : `→ ${step.value ?? ""}`}
          </p>
        </div>
      )}

      {trouble && (
        <p className="formula-debug__note muted" role="status">
          {trouble}
        </p>
      )}
      {steps !== null && total === 0 && !trouble && (
        <p className="formula-debug__note muted" role="status">
          This table has no formulas to run.
        </p>
      )}
    </div>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/FormulaDebugger.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FormulaDebugger.tsx"><storycode:paste select="#f-3a3b3fc26b23-FormulaDebuggerProps"/><storycode:paste select="#f-3a3b3fc26b23-FormulaDebugger"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-1acac206a244"></a>
## `apps/web/src/components/FormulaDebugger.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../guarantees/agent/ui-action-dispatch/contract.md), [A Formula Is An Expression In A Real Language](../../../../../guarantees/execution/a-formula-is-an-expression-in-a-real-language/contract.md), [Stepping A Table Replays The Order The Host Chose](../../../../../guarantees/execution/stepping-a-table-replays-the-order-the-host-chose/contract.md).

<a id="f-1acac206a244-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/execution/stepping-a-table-replays-the-order-the-host-chose.md

<storycode:copy id="f-1acac206a244-expression_statement">
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

</storycode:copy>
<a id="f-1acac206a244-STEPS"></a>
### `STEPS`

<storycode:copy id="f-1acac206a244-STEPS">
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

</storycode:copy>
<a id="f-1acac206a244-panel"></a>
### `panel`

<storycode:copy id="f-1acac206a244-panel">
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

</storycode:copy>
<a id="f-1acac206a244-where"></a>
### `where`

<storycode:copy id="f-1acac206a244-where">
const where = () => screen.getByTestId("formula-debug-where").textContent;

</storycode:copy>
<a id="f-1acac206a244-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-1acac206a244-expression_statement-2">
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

</storycode:copy>
<a id="f-1acac206a244-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-1acac206a244-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("drives captured trace navigation/read and rejects a prior trace after source changes", async () => {
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const { act } = await import("@testing-library/react");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.spyOn(api, "traceFormulas").mockResolvedValueOnce({ steps: STEPS, values: {}, errors: {} }).mockImplementation(() => new Promise(() => {}));
  const onStep = vi.fn();
  const content = (revision: string) => <UiActionContext.Provider value={registry}><FormulaDebugger owner="table-owner" language="python" rows={[["10", "=A1*2"]]} revision={revision} onStep={onStep} onClose={() => {}} /></UiActionContext.Provider>;
  const view = render(content("first"));
  await waitFor(() => expect(where()).toContain("Step 1 of 2"));
  const parent = registry.groups().groups.find(g => g.startsWith("formula-trace:"))!;
  const next = registry.observe(parent).actions.find(a => a.label === "Next formula trace step")!;
  expect(next.value).toMatchObject({ owner: "table-owner" });
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: next.handle, arguments: {}, request_id: "next" }); });
  expect(where()).toContain("Step 2 of 2");
  const read = registry.observe(parent).actions.find(a => a.label === "Read formula trace step bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes)))).toMatchObject({ index: 1, step: { cell: "A2", value: "25" } });
  view.rerender(content("changed"));
  expect(where()).toContain("Working out the order");
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old" })).toMatchObject({ status: "stale" });
  expect(onStep).toHaveBeenLastCalledWith(null);
});
</storycode:copy>

### Assemble `apps/web/src/components/FormulaDebugger.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/FormulaDebugger.test.tsx"><storycode:paste select="#f-1acac206a244-expression_statement"/><storycode:paste select="#f-1acac206a244-STEPS"/><storycode:paste select="#f-1acac206a244-panel"/><storycode:paste select="#f-1acac206a244-where"/><storycode:paste select="#f-1acac206a244-expression_statement-2"/><storycode:paste select="#f-1acac206a244-expression_statement-3"/></storycode:file>


</storycode:doc>
