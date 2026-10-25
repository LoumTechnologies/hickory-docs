<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/workspaceRibbonSources.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `44d9811478ecd0e39f089c48c5d55c2272537e7552ab70d687cdc7c685cacf2a`.

<book:entry path="apps/web/src/views/workspaceRibbonSources.ts">
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
</book:entry>

</book:doc>
