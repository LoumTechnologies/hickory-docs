<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/environments/WorkspaceProblems.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-73aefd6a0e47"></a>
## `apps/web/src/environments/WorkspaceProblems.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-73aefd6a0e47-WorkspaceProblems"></a>
### `WorkspaceProblems`

<storycode:copy id="f-73aefd6a0e47-WorkspaceProblems">
import { ProblemsPanel, problemRows, type ProblemRow } from "../components/ProblemsPanel";
import { EnvironmentPanel } from "./EnvironmentPanel";
import type { Environments } from "./useEnvironments";
import type { LspDiagnostic } from "../lsp/client";
import type { SessionRegistry } from "../views/documentSession";
import { allFileProblems } from "../lib/fileProblems";

export function WorkspaceProblems({ registry, environments, openHit, openDocument, navigate, onClose }: {
  registry: SessionRegistry;
  environments: Environments;
  openHit: (path: string, line: number) => void;
  openDocument: (id: string) => unknown;
  navigate: (path: string) => void;
  onClose: () => void;
}) {
  const documents: { docId: string; path: string; diagnostics: readonly LspDiagnostic[] }[] = [
    ...registry.all().map((open) => ({ docId: open.docId, path: open.doc?.path ?? open.docId, diagnostics: open.lspDiagnostics ?? [] })),
    ...allFileProblems().map((file) => ({ docId: `file:${file.path}`, path: file.path, diagnostics: file.diagnostics })),
  ];
  const onPick = (row: ProblemRow) => {
    onClose();
    if (row.docId.startsWith("file:")) { openHit(row.path, row.diagnostic.range.start.line + 1); return; }
    openDocument(row.docId);
    navigate(`/docs/${row.docId}`);
    // The destination editor mounts after its tab opens.
    window.setTimeout(() => registry.get(row.docId)?.revealDocLine(row.diagnostic.range.start.line), 0);
  };
  return <ProblemsPanel rows={problemRows(documents)} onPick={onPick} onClose={onClose}
    environments={<EnvironmentPanel environments={environments} />} />;
}
</storycode:copy>

### Assemble `apps/web/src/environments/WorkspaceProblems.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/environments/WorkspaceProblems.tsx"><storycode:paste select="#f-73aefd6a0e47-WorkspaceProblems"/></storycode:file>

</storycode:doc>
