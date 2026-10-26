<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/problems.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-bb7edd8c4f6c"></a>
## `apps/web/src/lib/problems.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-bb7edd8c4f6c-SEVERITY_ERROR"></a>
### `SEVERITY_ERROR`

How many problems there are, across everything open.
The count in a status bar is a promise: "this is how much is wrong right
now". Two things make that promise easy to break, and both are handled
here rather than at the point of display.
**Severity is a number on the wire, and 0 is not "no severity".** LSP
numbers severity 1–4 (error, warning, information, hint). A diagnostic with
no severity at all is, by the specification, up to the client — and treating
it as an error is the safer reading, because a language server that omits
severity is far more often reporting a compile failure than a style hint.
**Hints are not problems.** An inlay-ish "this could be simplified" in the
warning count is how a status bar reaches 400 and stops being read. Only
errors and warnings are counted; information and hints are shown in the
editor and left out of the tally.

<storycode:copy id="f-bb7edd8c4f6c-SEVERITY_ERROR">
// How many problems there are, across everything open.
//
// The count in a status bar is a promise: "this is how much is wrong right
// now". Two things make that promise easy to break, and both are handled
// here rather than at the point of display.
//
// **Severity is a number on the wire, and 0 is not "no severity".** LSP
// numbers severity 1–4 (error, warning, information, hint). A diagnostic with
// no severity at all is, by the specification, up to the client — and treating
// it as an error is the safer reading, because a language server that omits
// severity is far more often reporting a compile failure than a style hint.
//
// **Hints are not problems.** An inlay-ish "this could be simplified" in the
// warning count is how a status bar reaches 400 and stops being read. Only
// errors and warnings are counted; information and hints are shown in the
// editor and left out of the tally.

import type { LspDiagnostic } from "../lsp/client";

export const SEVERITY_ERROR = 1;
</storycode:copy>
<a id="f-bb7edd8c4f6c-SEVERITY_WARNING"></a>
### `SEVERITY_WARNING`

<storycode:copy id="f-bb7edd8c4f6c-SEVERITY_WARNING">
export const SEVERITY_WARNING = 2;
</storycode:copy>
<a id="f-bb7edd8c4f6c-SEVERITY_INFO"></a>
### `SEVERITY_INFO`

<storycode:copy id="f-bb7edd8c4f6c-SEVERITY_INFO">
export const SEVERITY_INFO = 3;
</storycode:copy>
<a id="f-bb7edd8c4f6c-SEVERITY_HINT"></a>
### `SEVERITY_HINT`

<storycode:copy id="f-bb7edd8c4f6c-SEVERITY_HINT">
export const SEVERITY_HINT = 4;

</storycode:copy>
<a id="f-bb7edd8c4f6c-ProblemCounts"></a>
### `ProblemCounts`

<storycode:copy id="f-bb7edd8c4f6c-ProblemCounts">
export interface ProblemCounts {
  errors: number;
  warnings: number;
}

</storycode:copy>
<a id="f-bb7edd8c4f6c-severityOf"></a>
### `severityOf`

<storycode:copy id="f-bb7edd8c4f6c-severityOf">
/** Whether a diagnostic counts, and as what. */
export function severityOf(diagnostic: Pick<LspDiagnostic, "severity">): number {
  // Absent severity reads as an error: a server that omits it is far more
  // often reporting a compile failure than a style hint, and undercounting a
  // real error is the worse mistake of the two.
  const value = diagnostic.severity;
  if (typeof value !== "number" || !Number.isFinite(value)) return SEVERITY_ERROR;
  if (value < SEVERITY_ERROR || value > SEVERITY_HINT) return SEVERITY_ERROR;
  return value;
}

</storycode:copy>
<a id="f-bb7edd8c4f6c-countProblems"></a>
### `countProblems`

Errors and warnings across every diagnostic given.

<storycode:copy id="f-bb7edd8c4f6c-countProblems">
/** Errors and warnings across every diagnostic given. */
export function countProblems(
  diagnostics: Iterable<Pick<LspDiagnostic, "severity">>,
): ProblemCounts {
  let errors = 0;
  let warnings = 0;
  for (const diagnostic of diagnostics) {
    const severity = severityOf(diagnostic);
    if (severity === SEVERITY_ERROR) errors++;
    else if (severity === SEVERITY_WARNING) warnings++;
  }
  return { errors, warnings };
}

</storycode:copy>
<a id="f-bb7edd8c4f6c-totalProblems"></a>
### `totalProblems`

<storycode:copy id="f-bb7edd8c4f6c-totalProblems">
/** The counts of several documents, added up. */
export function totalProblems(
  perDocument: Iterable<Iterable<Pick<LspDiagnostic, "severity">>>,
): ProblemCounts {
  let errors = 0;
  let warnings = 0;
  for (const document of perDocument) {
    const counts = countProblems(document);
    errors += counts.errors;
    warnings += counts.warnings;
  }
  return { errors, warnings };
}

</storycode:copy>
<a id="f-bb7edd8c4f6c-problemsLabel"></a>
### `problemsLabel`

What the status bar's problems section says out loud, for screen readers
and for the hover — the digits alone are ambiguous without the words.

<storycode:copy id="f-bb7edd8c4f6c-problemsLabel">
/** What the status bar's problems section says out loud, for screen readers
 * and for the hover — the digits alone are ambiguous without the words. */
export function problemsLabel({ errors, warnings }: ProblemCounts): string {
  if (errors === 0 && warnings === 0) return "No problems";
  const parts: string[] = [];
  if (errors > 0) parts.push(`${errors} error${errors === 1 ? "" : "s"}`);
  if (warnings > 0) parts.push(`${warnings} warning${warnings === 1 ? "" : "s"}`);
  return parts.join(", ");
}
</storycode:copy>

### Assemble `apps/web/src/lib/problems.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/problems.ts"><storycode:paste select="#f-bb7edd8c4f6c-SEVERITY_ERROR"/><storycode:paste select="#f-bb7edd8c4f6c-SEVERITY_WARNING"/><storycode:paste select="#f-bb7edd8c4f6c-SEVERITY_INFO"/><storycode:paste select="#f-bb7edd8c4f6c-SEVERITY_HINT"/><storycode:paste select="#f-bb7edd8c4f6c-ProblemCounts"/><storycode:paste select="#f-bb7edd8c4f6c-severityOf"/><storycode:paste select="#f-bb7edd8c4f6c-countProblems"/><storycode:paste select="#f-bb7edd8c4f6c-totalProblems"/><storycode:paste select="#f-bb7edd8c4f6c-problemsLabel"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-18a193fc73ac"></a>
## `apps/web/src/lib/problems.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-18a193fc73ac-d"></a>
### `d`

<storycode:copy id="f-18a193fc73ac-d">
import { describe, expect, it } from "vitest";
import {
  SEVERITY_HINT,
  SEVERITY_INFO,
  SEVERITY_WARNING,
  countProblems,
  problemsLabel,
  severityOf,
  totalProblems,
} from "./problems";

const d = (severity?: number) => ({ severity });

</storycode:copy>
<a id="f-18a193fc73ac-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-18a193fc73ac-expression_statement">
describe("what counts as a problem", () => {
  it("counts errors and warnings", () => {
    expect(countProblems([d(1), d(1), d(SEVERITY_WARNING)])).toEqual({
      errors: 2,
      warnings: 1,
    });
  });

  it("leaves information and hints out of the tally", () => {
    // A "this could be simplified" in the warning count is how a status bar
    // reaches 400 and stops being read. They still show in the editor.
    expect(countProblems([d(SEVERITY_INFO), d(SEVERITY_HINT)])).toEqual({
      errors: 0,
      warnings: 0,
    });
  });

  it("reads a missing severity as an error", () => {
    // The specification leaves it to the client. A server that omits it is
    // far more often reporting a compile failure than a style hint, and
    // undercounting a real error is the worse mistake.
    expect(severityOf(d(undefined))).toBe(1);
    expect(countProblems([d(undefined)])).toEqual({ errors: 1, warnings: 0 });
  });

  it("reads a nonsense severity as an error too", () => {
    expect(severityOf(d(0))).toBe(1);
    expect(severityOf(d(99))).toBe(1);
    expect(severityOf(d(Number.NaN))).toBe(1);
  });

  it("counts nothing for nothing", () => {
    expect(countProblems([])).toEqual({ errors: 0, warnings: 0 });
  });
});

</storycode:copy>
<a id="f-18a193fc73ac-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-18a193fc73ac-expression_statement-2">
describe("adding up several documents", () => {
  it("sums them", () => {
    expect(
      totalProblems([[d(1), d(2)], [d(2)], []]),
    ).toEqual({ errors: 1, warnings: 2 });
  });
});

</storycode:copy>
<a id="f-18a193fc73ac-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-18a193fc73ac-expression_statement-3">
describe("what the status bar says out loud", () => {
  it("says the quiet case plainly", () => {
    expect(problemsLabel({ errors: 0, warnings: 0 })).toBe("No problems");
  });

  it("gets the plurals right", () => {
    expect(problemsLabel({ errors: 1, warnings: 0 })).toBe("1 error");
    expect(problemsLabel({ errors: 2, warnings: 1 })).toBe("2 errors, 1 warning");
  });

  it("leaves out the half that is zero", () => {
    expect(problemsLabel({ errors: 0, warnings: 3 })).toBe("3 warnings");
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/problems.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/problems.test.ts"><storycode:paste select="#f-18a193fc73ac-d"/><storycode:paste select="#f-18a193fc73ac-expression_statement"/><storycode:paste select="#f-18a193fc73ac-expression_statement-2"/><storycode:paste select="#f-18a193fc73ac-expression_statement-3"/></storycode:file>


</storycode:doc>
