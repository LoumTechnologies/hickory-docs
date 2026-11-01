<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/workspaceSearch.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a5b848b0817a"></a>
## `apps/web/src/lib/workspaceSearch.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-a5b848b0817a-SearchFile"></a>
### `SearchFile`

<storycode:copy id="f-a5b848b0817a-SearchFile">
import { rememberReadingState, restoreReadingState } from "./contextProjection";
// Live open buffers supplement the folder index; their bytes replace stale disk hits.
import { Decoration, EditorView, ViewPlugin } from "@codemirror/view";
import { getSearchQuery, search, SearchQuery, setSearchQuery } from "@codemirror/search";
import type { SearchResponse } from "../api/types";
import { SearchIndex } from "../lineage/bm25";
import { samePath } from "./paths";

export interface SearchFile { path: string; content: string }
</storycode:copy>
<a id="f-a5b848b0817a-plainEditors"></a>
### `plainEditors`

Project search uses a literal, case-insensitive query for editor marks, independently
of the ranked result list. Keep that query after choosing a result so newly mounted
documents show the same matches. Clearing the input clears the marks. Local Find
uses the same CodeMirror query, including its regex and case options. Decorations
cover visible text only; the minimap reads every match from the current buffer.

<storycode:copy id="f-a5b848b0817a-plainEditors">
const plainEditors = new Map<EditorView, string>();
let projectQuery = "";
const highlightedEditors = new Set<EditorView>();

export function highlightProjectSearch(query: string): void {
  projectQuery = query;
  for (const view of highlightedEditors) {
    view.dispatch({ effects: setSearchQuery.of(new SearchQuery({ search: query, literal: true })) });
  }
}

export function searchMatchPositions(view: EditorView): { from: number; to: number }[] {
  const query = getSearchQuery(view.state);
  const positions: { from: number; to: number }[] = [];
  if (query.valid) {
    const cursor = query.getCursor(view.state);
    for (let next = cursor.next(); !next.done; next = cursor.next()) {
      positions.push({ from: next.value.from, to: next.value.to });
    }
  }
  return positions;
}

export const persistentSearchHighlights = [search(), ViewPlugin.fromClass(class {
  decorations = Decoration.none;
  constructor(private view: EditorView) {
    highlightedEditors.add(view);
    // Dispatch after construction: CodeMirror forbids dispatch during an update.
    queueMicrotask(() => {
      if (highlightedEditors.has(view) && projectQuery) {
        view.dispatch({ effects: setSearchQuery.of(new SearchQuery({ search: projectQuery, literal: true })) });
      }
    });
    this.draw(view);
  }
  update(update: import("@codemirror/view").ViewUpdate) {
    if (update.docChanged || update.viewportChanged || !getSearchQuery(update.startState).eq(getSearchQuery(update.state))) this.draw(update.view);
  }
  draw(view: EditorView) {
    const query = getSearchQuery(view.state);
    const marks = [];
    if (query.valid) for (const range of view.visibleRanges) {
      const cursor = query.getCursor(view.state, range.from, range.to);
      for (let next = cursor.next(); !next.done; next = cursor.next()) {
        const match = next.value;
        if (match.to > match.from) marks.push(Decoration.mark({ class: "cm-searchMatch" }).range(match.from, match.to));
      }
    }
    this.decorations = Decoration.set(marks, true);
  }
  destroy() { /* Removed views must never receive later query dispatches. */
    highlightedEditors.delete(this.view);
  }
}, { decorations: plugin => plugin.decorations })];

</storycode:copy>
<a id="f-a5b848b0817a-registerSearchEditor"></a>
### `registerSearchEditor`

<storycode:copy id="f-a5b848b0817a-registerSearchEditor">
export function registerSearchEditor(path: string, view: EditorView): () => void {
  restoreReadingState(`file:${path}`, view);
  plainEditors.set(view, path);
  return () => { rememberReadingState(`file:${path}`, view); plainEditors.delete(view); };
}

</storycode:copy>
<a id="f-a5b848b0817a-openPlainSearchFiles"></a>
### `openPlainSearchFiles`

<storycode:copy id="f-a5b848b0817a-openPlainSearchFiles">
export function openPlainSearchFiles(): SearchFile[] {
  return [...plainEditors].map(([view, path]) => ({ path, content: view.state.doc.toString() }));
}

</storycode:copy>
<a id="f-a5b848b0817a-plainSearchEditor"></a>
### `plainSearchEditor`

<storycode:copy id="f-a5b848b0817a-plainSearchEditor">
export function plainSearchEditor(path: string): EditorView | undefined {
  return [...plainEditors].find(([view, name]) => name === path && view.dom.isConnected)?.[0];
}

</storycode:copy>
<a id="f-a5b848b0817a-searchWorkspace"></a>
### `searchWorkspace`

<storycode:copy id="f-a5b848b0817a-searchWorkspace">
export async function searchWorkspace(
  query: string,
  limit: number,
  files: readonly SearchFile[],
  folderSearch?: (query: string, limit: number) => Promise<SearchResponse>,
  folderRoot?: string,
): Promise<SearchResponse> {
  const canonical = (path: string) => {
    const normalized = path.replace(/\\/g, "/").replace(/^\.\//, "");
    return /^(\/|[A-Za-z]:\/)/.test(normalized) ? normalized : `${folderRoot?.replace(/\/$/, "")}/${normalized}`;
  };
  const matches = (a: string, b: string) => folderRoot ? canonical(a) === canonical(b) : samePath(a, b);
  const unique = files.filter((file, index) =>
    files.findIndex((other) => matches(other.path, file.path)) === index,
  );
  const byPath = new Map(unique.map((file) => [file.path, file.content.split("\n")]));
  const index = new SearchIndex(unique.map((file) => ({
    path: file.path, lines: byPath.get(file.path)!, kind: "document" as const,
  })));
  const liveHits = index.search(query, limit).map((hit) => ({
    path: hit.file, start_line: hit.line + 1, end_line: hit.line + 1,
    score: hit.score, snippet: byPath.get(hit.file)![hit.line],
  }));
  const folder = folderSearch ? await folderSearch(query, 50) : { semantic: false, hits: [] };
  // The rankings use different corpora. Alternate their ranked results rather
  // than comparing scores that have different meanings.
  const diskHits = folder.hits.filter((hit) => !unique.some((file) => matches(file.path, hit.path)));
  const hits = [];
  for (let i = 0; i < Math.max(liveHits.length, diskHits.length); i++) {
    if (liveHits[i]) hits.push(liveHits[i]);
    if (diskHits[i]) hits.push(diskHits[i]);
  }
  return { semantic: folder.semantic, hits: hits.slice(0, limit) };
}
</storycode:copy>

### Assemble `apps/web/src/lib/workspaceSearch.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/workspaceSearch.ts"><storycode:paste select="#f-a5b848b0817a-SearchFile"/><storycode:paste select="#f-a5b848b0817a-plainEditors"/><storycode:paste select="#f-a5b848b0817a-registerSearchEditor"/><storycode:paste select="#f-a5b848b0817a-openPlainSearchFiles"/><storycode:paste select="#f-a5b848b0817a-plainSearchEditor"/><storycode:paste select="#f-a5b848b0817a-searchWorkspace"/></storycode:file>

</storycode:doc>
