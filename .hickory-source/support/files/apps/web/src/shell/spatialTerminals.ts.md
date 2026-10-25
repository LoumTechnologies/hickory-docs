<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/spatialTerminals.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `b6faf0c9f0026a5541b17b0de5dfc6ea74511a76169918237ae2175329286cee`.

<book:entry path="apps/web/src/shell/spatialTerminals.ts">
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

export function spatialEditorVisible(view: EditorView | undefined, container: HTMLElement): EditorView | undefined {
  if (!view) return;
  const tile = view.dom.closest<HTMLElement>("[data-spatial-key]");
  if (!tile) return view;
  if (tile.hidden || tile.dataset.expanded !== "true" || getComputedStyle(tile).visibility === "hidden") return;
  if (tile.dataset.spatialCenter !== "true" && treeEndpoint(container, tile.dataset.shellTarget ?? "")?.folder) return;
  return view;
}
</book:entry>

</book:doc>
