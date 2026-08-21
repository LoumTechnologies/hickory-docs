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

import { useEffect, useMemo, useState } from "react";

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
}

export function FormulaDebugger({
  language,
  rows,
  revision,
  onStep,
  onClose,
}: FormulaDebuggerProps) {
  const [steps, setSteps] = useState<FormulaStep[] | null>(null);
  const [trouble, setTrouble] = useState<string | null>(null);
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
        setSteps(trace.steps);
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
        setSteps([]);
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

  return (
    <div className="formula-debug" data-testid="formula-debug">
      <div className="formula-debug__bar" role="toolbar" aria-label="Step through the formulas">
        <button
          type="button"
          className="btn btn-small"
          disabled={first || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => setAt(0)}
          data-tip="Back to the first cell that ran"
        >
          ⏮
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={first || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => setAt((n) => Math.max(0, n - 1))}
          data-tip="The cell that ran before this one"
        >
          ◀ Back
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={last || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => setAt((n) => Math.min(total - 1, n + 1))}
          data-tip="The next cell to run"
        >
          Step ▶
        </button>
        <button
          type="button"
          className="btn btn-small"
          disabled={last || total === 0}
          onMouseDown={(event) => event.preventDefault()}
          onClick={() => setAt(Math.max(0, total - 1))}
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
          onClick={onClose}
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
