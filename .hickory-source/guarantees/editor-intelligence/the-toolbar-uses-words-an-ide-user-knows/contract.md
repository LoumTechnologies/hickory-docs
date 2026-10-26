<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Run and test from the pane menu without adding a toolbar row

For: maintainers and authors selecting the current editor or debugger behavior.

Open the document menu while a run is active. Run and Test must refer to that document and stay disabled until the run permits another action.

## What must happen

Run executes the document; Test matches the hick test operation. These document-wide actions use the pane-header menu, while per-cell actions remain on the editor rail. Run and Test are disabled while a document run is in flight. A baseline action is named for the output baseline it establishes. The problem count opens the cross-document problem list; F8 retains next-problem navigation.

## Limits of the evidence

The focused pane test uses injected session callbacks. It does not establish native menu placement, the runtime test outcome or every problem-list interaction.



## Implementation

<a id="output-073ca08e3d2c"></a>
## `apps/web/src/components/ProblemsPanel.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Run and test from the pane menu without adding a toolbar row](contract.md).

<a id="f-073ca08e3d2c-expression_statement"></a>
### `expression_statement`

Protects docs/guarantees/editor-intelligence/a-problem-count-opens-a-list.md

<storycode:copy id="f-073ca08e3d2c-expression_statement">
// Protects docs/guarantees/editor-intelligence/a-problem-count-opens-a-list.md

import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ProblemsPanel, problemRows } from "./ProblemsPanel";

afterEach(cleanup);

</storycode:copy>
<a id="f-073ca08e3d2c-at"></a>
### `at`

<storycode:copy id="f-073ca08e3d2c-at">
const at = (line: number, severity: number, message: string) => ({
  range: { start: { line, character: 0 }, end: { line, character: 4 } },
  severity,
  message,
});

</storycode:copy>
<a id="f-073ca08e3d2c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-073ca08e3d2c-expression_statement-2">
describe("the rows behind the count", () => {
  it("puts the worst first, then orders by where they are", () => {
    const rows = problemRows([
      { docId: "b", path: "b.md", diagnostics: [at(9, 2, "a warning"), at(2, 1, "later error")] },
      { docId: "a", path: "a.md", diagnostics: [at(4, 1, "earlier error")] },
    ]);
    expect(rows.map((r) => r.diagnostic.message)).toEqual([
      "earlier error", // error, a.md
      "later error", // error, b.md
      "a warning", // warning, last whatever its line
    ]);
  });

  it("leaves out information and hints, the way the count does", () => {
    // A list that reaches four hundred rows is a list nobody opens, and the
    // number in the status bar would stop matching what opening it shows.
    const rows = problemRows([
      {
        docId: "a",
        path: "a.md",
        diagnostics: [at(1, 3, "consider this"), at(2, 4, "a hint"), at(3, 2, "a warning")],
      },
    ]);
    expect(rows.map((r) => r.diagnostic.message)).toEqual(["a warning"]);
  });
});

</storycode:copy>
<a id="f-073ca08e3d2c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-073ca08e3d2c-expression_statement-3">
describe("the panel", () => {
  it("lists each problem with where it is, and reports the pick", () => {
    const onPick = vi.fn();
    const rows = problemRows([
      { docId: "d1", path: "notes/a.md", diagnostics: [at(11, 1, "undefined name `foo`")] },
    ]);
    render(<ProblemsPanel rows={rows} onPick={onPick} onClose={() => {}} />);

    expect(screen.getByText("1 problem")).toBeTruthy();
    expect(screen.getByText("undefined name `foo`")).toBeTruthy();
    // One-based, like every editor's gutter and every compiler's output.
    expect(screen.getByText("notes/a.md:12")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: /undefined name/ }));
    expect(onPick).toHaveBeenCalledWith(rows[0]);
  });

  it("says nothing is wrong rather than showing an empty box", () => {
    render(<ProblemsPanel rows={[]} onPick={() => {}} onClose={() => {}} />);
    expect(screen.getByText("Nothing is wrong right now")).toBeTruthy();
  });
});
</storycode:copy>

### Assemble `apps/web/src/components/ProblemsPanel.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ProblemsPanel.test.tsx"><storycode:paste select="#f-073ca08e3d2c-expression_statement"/><storycode:paste select="#f-073ca08e3d2c-at"/><storycode:paste select="#f-073ca08e3d2c-expression_statement-2"/><storycode:paste select="#f-073ca08e3d2c-expression_statement-3"/></storycode:file>

<a id="output-90498f11be4a"></a>
## `apps/web/src/components/ProblemsPanel.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Run and test from the pane menu without adding a toolbar row](contract.md).

<a id="f-90498f11be4a-ProblemRow"></a>
### `ProblemRow`

Every problem, in a list you can click.
The status bar has always carried the count. Clicking it jumped to the next
diagnostic in the focused document — which is a real verb and the wrong
first one: a person who reads "3 errors" wants to know WHAT the three are
before deciding which to look at, and the jump could only ever reach the
document that happened to have focus. A count you cannot expand is a number
with no way in.
So the chip opens this, every editor's Problems list, and F8 keeps the
jump. Sorted by severity and then by position, because "which is worst" is
the question a list of problems is opened to answer.

<storycode:copy id="f-90498f11be4a-ProblemRow">
// Every problem, in a list you can click.
//
// The status bar has always carried the count. Clicking it jumped to the next
// diagnostic in the focused document — which is a real verb and the wrong
// first one: a person who reads "3 errors" wants to know WHAT the three are
// before deciding which to look at, and the jump could only ever reach the
// document that happened to have focus. A count you cannot expand is a number
// with no way in.
//
// So the chip opens this, every editor's Problems list, and F8 keeps the
// jump. Sorted by severity and then by position, because "which is worst" is
// the question a list of problems is opened to answer.

import { useUiListActions } from "../lib/uiListActions";
import type { LspDiagnostic } from "../lsp/client";
import type { ReactNode } from "react";
import { SEVERITY_ERROR, SEVERITY_WARNING, severityOf } from "../lib/problems";

/** One row: a diagnostic, and which document it belongs to. */
export interface ProblemRow {
  docId: string;
  path: string;
  diagnostic: LspDiagnostic;
}

</storycode:copy>
<a id="f-90498f11be4a-problemRows"></a>
### `problemRows`

The rows, worst first, then by where they are.
Information and hints are left out for the same reason they are left out of
the count: a list that reaches four hundred rows is a list nobody opens.

<storycode:copy id="f-90498f11be4a-problemRows">
/**
 * The rows, worst first, then by where they are.
 *
 * Information and hints are left out for the same reason they are left out of
 * the count: a list that reaches four hundred rows is a list nobody opens.
 */
export function problemRows(
  documents: Iterable<{ docId: string; path: string; diagnostics: readonly LspDiagnostic[] }>,
): ProblemRow[] {
  const rows: ProblemRow[] = [];
  for (const { docId, path, diagnostics } of documents) {
    for (const diagnostic of diagnostics) {
      if (severityOf(diagnostic) > SEVERITY_WARNING) continue;
      rows.push({ docId, path, diagnostic });
    }
  }
  return rows.sort((a, b) => {
    const bySeverity = severityOf(a.diagnostic) - severityOf(b.diagnostic);
    if (bySeverity !== 0) return bySeverity;
    const byPath = a.path.localeCompare(b.path);
    if (byPath !== 0) return byPath;
    return (
      a.diagnostic.range.start.line - b.diagnostic.range.start.line ||
      a.diagnostic.range.start.character - b.diagnostic.range.start.character
    );
  });
}

</storycode:copy>
<a id="f-90498f11be4a-ProblemsPanel"></a>
### `ProblemsPanel`

<storycode:copy id="f-90498f11be4a-ProblemsPanel">
export function ProblemsPanel({
  rows,
  onPick,
  onClose,
  environments,
}: {
  rows: readonly ProblemRow[];
  onPick: (row: ProblemRow) => void;
  onClose: () => void;
  environments?: ReactNode;
}) {
  const actions = useUiListActions("Problems", rows, { includes: "Errors and warnings; information and hints excluded" }, onPick, onClose);
  return (
    <aside className="refs-panel problems-panel" role="dialog" aria-label="Problems">
      <header className="refs-head">
        <span>
          {rows.length === 0
            ? environments ? "Problems and project environments" : "Nothing is wrong right now"
            : `${rows.length} problem${rows.length === 1 ? "" : "s"}`}
        </span>
        <button className="btn-link" aria-label="Close problems" onClick={() => void actions.close()}>
          ×
        </button>
      </header>
      {environments}
      {rows.length > 0 && (
        <ul className="refs-list">
          {rows.map((row, index) => {
            const error = severityOf(row.diagnostic) === SEVERITY_ERROR;
            const line = row.diagnostic.range.start.line + 1;
            return (
              <li key={`${row.docId}:${line}:${index}`}>
                <button
                  type="button"
                  className="refs-item"
                  onClick={() => void actions.pick(index)}
                  data-tip={row.diagnostic.message}
                >
                  <span
                    className={`problems-panel__glyph problems-panel__glyph--${error ? "error" : "warn"}`}
                    aria-hidden
                  >
                    {error ? "✕" : "⚠"}
                  </span>
                  <span className="problems-panel__message">{row.diagnostic.message}</span>
                  {/* Where, last and dim: the message is what a person scans,
                      and a column of paths in front of it buries them. */}
                  <span className="refs-where mono">
                    {row.path}:{line}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </aside>
  );
}
</storycode:copy>

### Assemble `apps/web/src/components/ProblemsPanel.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ProblemsPanel.tsx"><storycode:paste select="#f-90498f11be4a-ProblemRow"/><storycode:paste select="#f-90498f11be4a-problemRows"/><storycode:paste select="#f-90498f11be4a-ProblemsPanel"/></storycode:file>

## Implementation review

### Implementation

- [apps/web/src/components/ProblemsPanel.tsx](contract.md#output-90498f11be4a)
- [apps/web/src/components/RefactorBadge.tsx](../../../mechanisms/apps/web/src/components/RefactorBadge.tsx.md#output-c40aef0d3c52)
- [apps/web/src/views/WorkspaceView.tsx](../../../mechanisms/apps/web/src/views/WorkspaceView.tsx.md#output-47afe3575948)
- [apps/web/src/views/workspaceTabs.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-b5a48fd337ae)

### Tests and fixtures

- [apps/web/src/components/ProblemsPanel.test.tsx](contract.md#output-073ca08e3d2c)
- [apps/web/src/views/workspaceTabs.test.tsx](../../../mechanisms/apps/web/src/views/workspaceTabs.tsx.md#output-5370130a675d)

### Focused witnesses

- `disables both buttons while the document's run is in flight` in `apps/web/src/views/workspaceTabs.test.tsx`.

### Evidence status

- Implementation review: not verified. Retention does not renew an implementation review.
- Executed checks: [the run record](../../../evidence/focused-checks.md) lists actual executions.
- Human judgment: stakeholder priorities confirmed; individual requirement acceptance remains unrecorded.
- Dependencies: exact cited-file hashes are in the capture and source entries. Transitive source and environment changes can also stale evidence.

</storycode:doc>
