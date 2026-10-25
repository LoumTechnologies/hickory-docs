<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/spatialRelations.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../../guarantees/shell/spatial-views-preserve-the-editor/contract.md)

Captured SHA-256: `c0e480c38778f87db034c8e6970724291856023d25c48009a49673e7ff35dec6`.

<book:entry path="apps/web/src/shell/spatialRelations.ts">
import type { EditorView } from "@codemirror/view";
import { deriveRibbons } from "../lib/ribbons";
import { samePath } from "../lib/paths";
import { plainSearchEditor } from "../lib/workspaceSearch";
import { bandBetween, type RibbonFile, type RibbonLink, type RibbonSource } from "./Ribbons";

export interface SpatialRelation {
  key: string;
  fromPath: string;
  toPath: string;
  from?: EditorView;
  to?: EditorView;
  fromRange?: [number, number];
  toRange?: [number, number];
  anchor?: HTMLElement;
}

export function spatialRelations(sources: readonly RibbonSource[], files: readonly RibbonFile[], links: readonly RibbonLink[]): SpatialRelation[] {
  const editor = (path: string) => sources.find(source => samePath(source.docPath, path))?.view
    ?? files.find(file => samePath(file.file.path, path))?.view ?? plainSearchEditor(path);
  const range = (view: EditorView | undefined, lines: [number, number] | undefined): [number, number] | undefined => {
    if (!view || !lines) return;
    const count = view.state.doc.lines;
    return [view.state.doc.line(Math.max(1, Math.min(count, lines[0]))).from, view.state.doc.line(Math.max(1, Math.min(count, lines[1]))).to];
  };
  const out: SpatialRelation[] = links.map(link => ({ key: link.key, fromPath: link.from.path, toPath: link.to.path,
    from: editor(link.from.path), to: editor(link.to.path), fromRange: range(editor(link.from.path), link.from.lines),
    toRange: range(editor(link.to.path), link.to.lines), anchor: link.anchor }));
  for (const source of sources) for (const { file, view } of files) {
    for (const ribbon of deriveRibbons(file, source.docPath, source.docSource)) out.push({ key: ribbon.key, fromPath: source.docPath,
      toPath: file.path, from: source.view, to: view, fromRange: ribbon.sourceSpan, toRange: ribbon.outputRange });
  }
  return out;
}

export function relationBand(relation: SpatialRelation, reverse = false): [number, number] | null {
  if (!reverse && relation.anchor?.isConnected) { const r = relation.anchor.getBoundingClientRect(); return [r.top, r.bottom]; }
  const editor = reverse ? relation.to : relation.from, range = reverse ? relation.toRange : relation.fromRange;
  if (!editor?.dom.isConnected || !range) return null;
  try { return bandBetween(editor, Math.min(range[0], editor.state.doc.length), Math.min(range[1], editor.state.doc.length)); }
  catch { return null; }
}
</book:entry>

</book:doc>
