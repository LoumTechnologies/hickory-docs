<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/spatialTerminals.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f32af4e498bd"></a>
## `apps/web/src/shell/spatialTerminals.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md).

<a id="f-f32af4e498bd-treeEndpoint"></a>
### `treeEndpoint`

<storycode:copy id="f-f32af4e498bd-treeEndpoint">
import { EditorView } from "@codemirror/view";
import { foldedRanges } from "@codemirror/language";
import { samePath } from "../lib/paths";
import { visibleElementRect } from "../lib/visibleElementRect";

/** A collapsed directory is a visible address for its descendants, not a new provenance claim. */
export function treeEndpoint(container: HTMLElement, target: string): { rect: DOMRect; folder: boolean; element: HTMLElement } | null {
  if (!container.hasAttribute("data-spatial-files")) {
    const tile = [...container.querySelectorAll<HTMLElement>("[data-shell-target]")].find(el => samePath(el.dataset.shellTarget ?? "", target) && el.dataset.spatialCenter !== "true");
    const rail = tile && container.querySelector<HTMLElement>(`[data-spatial-files=${tile.dataset.spatialSide}]`);
    if (rail) return treeEndpoint(rail, target);
  }
  const rows = [...container.querySelectorAll<HTMLElement>("[data-tree-path]")];
  for (const row of rows) if (samePath(row.dataset.treePath ?? "", target)) { const rect = visibleElementRect(row); if (rect) return { rect, folder: false, element: row }; }
  const path = target.replace(/\\/g, "/");
  const folders = rows.filter(row => row.dataset.treeKind === "directory" && (path.startsWith(`${row.dataset.treePath}/`) || path.includes(`/${row.dataset.treePath}/`))).sort((a, b) => (b.dataset.treePath?.length ?? 0) - (a.dataset.treePath?.length ?? 0));
  for (const row of folders) {
    const dom = row.closest<HTMLElement>(".cm-editor"), view = dom && EditorView.findFromDOM(dom), rect = visibleElementRect(row);
    if (!view || !rect) continue;
    let folded = false;
    try { const line = view.state.doc.lineAt(view.posAtDOM(row)); foldedRanges(view.state).between(line.from, line.to, () => { folded = true; }); } catch { continue; }
    if (folded) return { rect, folder: true, element: row };
  }
  return null;
}

</storycode:copy>
<a id="f-f32af4e498bd-spatialEditorVisible"></a>
### `spatialEditorVisible`

<storycode:copy id="f-f32af4e498bd-spatialEditorVisible">
export function spatialEditorVisible(view: EditorView | undefined, container: HTMLElement): EditorView | undefined {
  if (!view) return;
  const tile = view.dom.closest<HTMLElement>("[data-spatial-key]");
  if (!tile) return view;
  if (tile.hidden || tile.dataset.expanded !== "true" || getComputedStyle(tile).visibility === "hidden") return;
  if (tile.dataset.spatialCenter !== "true" && treeEndpoint(container, tile.dataset.shellTarget ?? "")?.folder) return;
  return view;
}
</storycode:copy>

### Assemble `apps/web/src/shell/spatialTerminals.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/spatialTerminals.ts"><storycode:paste select="#f-f32af4e498bd-treeEndpoint"/><storycode:paste select="#f-f32af4e498bd-spatialEditorVisible"/></storycode:file>

</storycode:doc>
