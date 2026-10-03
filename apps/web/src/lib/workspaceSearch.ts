// Live open buffers supplement the folder index; their bytes replace stale disk hits.
import type { EditorView } from "@codemirror/view";
import type { SearchResponse } from "../api/types";
import { SearchIndex } from "../lineage/bm25";
import { samePath } from "./paths";

export interface SearchFile { path: string; content: string }
const plainEditors = new Map<EditorView, string>();

export function registerSearchEditor(path: string, view: EditorView): () => void {
  plainEditors.set(view, path);
  return () => { plainEditors.delete(view); };
}

export function openPlainSearchFiles(): SearchFile[] {
  return [...plainEditors].map(([view, path]) => ({ path, content: view.state.doc.toString() }));
}

export function plainSearchEditor(path: string): EditorView | undefined {
  return [...plainEditors].find(([view, name]) => name === path && view.dom.isConnected)?.[0];
}

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
