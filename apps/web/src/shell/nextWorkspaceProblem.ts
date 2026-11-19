import { EditorView } from "@codemirror/view";
import { severityOf } from "../lib/problems";
import type { LspDiagnostic } from "../lsp/client";
import { diagnosticBoundSource } from "../lsp/diagnosticRevision";
import type { LspPosition } from "../lsp/positions";
import { authoredSource } from "../simple/SourceText";

export function validProblemPosition(lines: readonly string[], {line, character}: LspPosition): boolean {
  if (!Number.isSafeInteger(line) || line < 0 || !Number.isSafeInteger(character) || character < 0) return false;
  const row = lines[line]?.replace(/\r$/, "");
  return row !== undefined && character <= row.length &&
    !/[\uD800-\uDBFF][\uDC00-\uDFFF]/.test(row.slice(character - 1, character + 1));
}

export function nextWorkspaceProblem(view: EditorView | null | undefined, diagnostics: readonly LspDiagnostic[]) {
  if (!view || diagnostics.length === 0) return;
  const source = authoredSource(view.state);
  const lines = source.split("\n");
  const ranked = diagnostics
    .filter(d => severityOf(d) <= 2 && diagnosticBoundSource(d) === source && validProblemPosition(lines, d.range.start))
    .map(({range: {start}}) => view.state.doc.line(start.line + 1).from + start.character)
    .sort((a, b) => a - b);
  if (ranked.length === 0) return;
  const head = view.state.selection.main.head;
  const next = ranked.find(at => at > head) ?? ranked[0];
  view.dispatch({selection: {anchor: next}, effects: EditorView.scrollIntoView(next, {y: "center"})});
  view.focus();
}
