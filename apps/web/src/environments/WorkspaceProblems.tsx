import { ProblemsPanel, problemRows, type ProblemRow } from "../components/ProblemsPanel";
import { EnvironmentPanel } from "./EnvironmentPanel";
import type { Environments } from "./useEnvironments";
import type { LspDiagnostic } from "../lsp/client";
import type { SessionRegistry } from "../views/documentSession";
import { allFileProblems } from "../lib/fileProblems";

export function WorkspaceProblems({ registry, environments, onPick, onClose }: {
  registry: SessionRegistry;
  environments: Environments;
  onPick: (row: ProblemRow) => Promise<boolean>;
  onClose: () => void;
}) {
  const documents: { docId: string; path: string; diagnostics: readonly LspDiagnostic[] }[] = [
    ...registry.all().map((open) => ({ docId: open.docId, path: open.doc?.path ?? open.docId, diagnostics: open.lspDiagnostics ?? [] })),
    ...allFileProblems().map((file) => ({ docId: `file:${file.path}`, path: file.path, diagnostics: file.diagnostics })),
  ];
  return <ProblemsPanel rows={problemRows(documents)} onPick={row => {
    void onPick(row).then(opened => { if (opened) onClose(); });
  }} onClose={onClose}
    environments={<EnvironmentPanel environments={environments} />} />;
}
