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
export const SEVERITY_WARNING = 2;
export const SEVERITY_INFO = 3;
export const SEVERITY_HINT = 4;

export interface ProblemCounts {
  errors: number;
  warnings: number;
}

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

/** What the status bar's problems section says out loud, for screen readers
 * and for the hover — the digits alone are ambiguous without the words. */
export function problemsLabel({ errors, warnings }: ProblemCounts): string {
  if (errors === 0 && warnings === 0) return "No problems";
  const parts: string[] = [];
  if (errors > 0) parts.push(`${errors} error${errors === 1 ? "" : "s"}`);
  if (warnings > 0) parts.push(`${warnings} warning${warnings === 1 ? "" : "s"}`);
  return parts.join(", ");
}
