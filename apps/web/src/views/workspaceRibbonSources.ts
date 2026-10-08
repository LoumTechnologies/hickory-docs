import type { RibbonSource } from "../shell/Ribbons";
import type { DocSession, SessionRegistry } from "./documentSession";
import { samePath } from "../lib/paths";
import { lensSources } from "../lib/lensSources";
import { openPlainSearchFiles, plainSearchEditor } from "../lib/workspaceSearch";

/** Receipts can connect to any open passage, independently of the focused note. */
export function workspaceRibbonSources(focused: DocSession | null | undefined, registry: SessionRegistry,
  docIdByPath: (path: string) => { path: string }): RibbonSource[] {
  const sources: RibbonSource[] = [];
  const add = (source: RibbonSource) => {
    if (!sources.some(s => samePath(s.docPath, source.docPath))) sources.push(source);
  };
  for (const session of [focused, ...registry.all()]) {
    if (!session?.doc) continue;
    add({ view: session.docEditor ?? undefined, docPath: session.doc.path,
      docSource: session.docEditor?.state.doc.toString() ?? session.doc.source });
  }
  for (const file of focused?.outputs.values() ?? []) {
    for (const provenance of file.provenance) {
      if (provenance.origin.kind === "synthetic") continue;
      const { path } = docIdByPath(provenance.origin.doc_path);
      add({ docPath: path, docSource: "" });
    }
  }
  for (const file of openPlainSearchFiles()) {
    add({ docPath: file.path, docSource: file.content, view: plainSearchEditor(file.path) });
  }
  for (const lens of lensSources()) add({ docPath: lens.path, docSource: lens.view.state.doc.toString(), view: lens.view });
  return sources;
}
