<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/nextWorkspaceProblem.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b7e03171fcb6"></a>
## `apps/web/src/shell/nextWorkspaceProblem.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b7e03171fcb6-nextWorkspaceProblem"></a>
### `nextWorkspaceProblem`

F8 reads errors and warnings in the selected editor. A diagnostic must
name the exact source and a valid coordinate before it can move the caret.

<storycode:copy id="f-b7e03171fcb6-nextWorkspaceProblem">
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
</storycode:copy>

### Assemble `apps/web/src/shell/nextWorkspaceProblem.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/nextWorkspaceProblem.ts"><storycode:paste select="#f-b7e03171fcb6-nextWorkspaceProblem"/></storycode:file>

</storycode:doc>
