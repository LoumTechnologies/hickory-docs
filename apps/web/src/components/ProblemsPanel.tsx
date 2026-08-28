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

import type { LspDiagnostic } from "../lsp/client";
import { SEVERITY_ERROR, SEVERITY_WARNING, severityOf } from "../lib/problems";

/** One row: a diagnostic, and which document it belongs to. */
export interface ProblemRow {
  docId: string;
  path: string;
  diagnostic: LspDiagnostic;
}

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

export function ProblemsPanel({
  rows,
  onPick,
  onClose,
}: {
  rows: readonly ProblemRow[];
  onPick: (row: ProblemRow) => void;
  onClose: () => void;
}) {
  return (
    <aside className="refs-panel problems-panel" role="dialog" aria-label="Problems">
      <header className="refs-head">
        <span>
          {rows.length === 0
            ? "Nothing is wrong right now"
            : `${rows.length} problem${rows.length === 1 ? "" : "s"}`}
        </span>
        <button className="btn-link" aria-label="Close problems" onClick={onClose}>
          ×
        </button>
      </header>
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
                  onClick={() => onPick(row)}
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
