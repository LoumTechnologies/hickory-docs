<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/workspaceRibbonSources.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-6c40ecb65b67"></a>
## `apps/web/src/views/workspaceRibbonSources.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-6c40ecb65b67-workspaceRibbonSources"></a>
### `workspaceRibbonSources`

<storycode:copy id="f-6c40ecb65b67-workspaceRibbonSources">
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
</storycode:copy>

### Assemble `apps/web/src/views/workspaceRibbonSources.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/workspaceRibbonSources.ts"><storycode:paste select="#f-6c40ecb65b67-workspaceRibbonSources"/></storycode:file>

</storycode:doc>
