// Where a project-search hit can take you.
//
// The panel knows nothing about documents or panes: it hands each hit to this
// resolver, which answers with a navigation the rest of the app already knows
// how to perform — the editor's line reveal for the document on screen, the
// router for another document, the ribbons' open-and-reveal for a generated
// file. A hit that resolves to none stays visible but disabled: the path is
// still an answer, even when there is nowhere to click through to.

import type { SearchHit } from "../api/types";
import { samePath } from "./paths";

export type SearchNavigation =
  /** The document already open: scroll its editor. `line` is 0-based, the
   * unit `revealLine` speaks. */
  | { kind: "current-doc"; line: number }
  /** Another document in this folder: a route change. */
  | { kind: "doc"; id: string }
  /** A file the current document generates: open a pane and reveal the
   * range, in UTF-16 char offsets — the editor's unit. */
  | { kind: "generated"; path: string; range: [number, number] }
  /** No open document owns this file and it is not itself a document. */
  | { kind: "none" };

export interface SearchContext {
  currentDocPath: string | null;
  /** Every document in the folder, by the path it was listed under. */
  docs: readonly { id: string; path: string }[];
  /** The current document's generated files, content included — the range of
   * a hit in one has to be computed before its pane exists. */
  outputs: readonly { path: string; content: string }[];
}

export function resolveSearchHit(hit: SearchHit, context: SearchContext): SearchNavigation {
  // The current document wins over its entry in the folder listing: "go to
  // the document you are already reading" means the line, not a reload.
  if (context.currentDocPath && samePath(hit.path, context.currentDocPath)) {
    return { kind: "current-doc", line: Math.max(0, hit.start_line - 1) };
  }
  const doc = context.docs.find((entry) => samePath(hit.path, entry.path));
  if (doc) return { kind: "doc", id: doc.id };
  const output = context.outputs.find((entry) => samePath(hit.path, entry.path));
  if (output) {
    return {
      kind: "generated",
      path: output.path,
      range: lineRangeToChars(output.content, hit.start_line, hit.end_line),
    };
  }
  return { kind: "none" };
}

/**
 * The UTF-16 char range covering 1-based lines `startLine`..`endLine` of
 * `content`, clamped to the lines that exist. The trailing newline stays
 * outside the range, so a reveal selects the text and not the break after it.
 */
export function lineRangeToChars(
  content: string,
  startLine: number,
  endLine: number,
): [number, number] {
  const lines = content.split("\n");
  const first = Math.min(Math.max(1, startLine), lines.length);
  const last = Math.min(Math.max(first, endLine), lines.length);
  let offset = 0;
  let from = 0;
  for (let i = 1; i <= last; i++) {
    const end = offset + lines[i - 1].length;
    if (i === first) from = offset;
    if (i === last) return [from, end];
    offset = end + 1;
  }
  return [0, 0];
}
