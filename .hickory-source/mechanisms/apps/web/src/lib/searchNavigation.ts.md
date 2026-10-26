<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/searchNavigation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c72b68d0862a"></a>
## `apps/web/src/lib/searchNavigation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-c72b68d0862a-SearchNavigation"></a>
### `SearchNavigation`

Where a project-search hit can take you.
The panel knows nothing about documents or panes: it hands each hit to this
resolver, which answers with a navigation the rest of the app already knows
how to perform — the editor's line reveal for the document on screen, the
router for another document, the ribbons' open-and-reveal for a generated
file. A hit that resolves to none stays visible but disabled: the path is
still an answer, even when there is nowhere to click through to.

<storycode:copy id="f-c72b68d0862a-SearchNavigation">
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

</storycode:copy>
<a id="f-c72b68d0862a-SearchContext"></a>
### `SearchContext`

<storycode:copy id="f-c72b68d0862a-SearchContext">
export interface SearchContext {
  currentDocPath: string | null;
  /** Every document in the folder, by the path it was listed under. */
  docs: readonly { id: string; path: string }[];
  /** The current document's generated files, content included — the range of
   * a hit in one has to be computed before its pane exists. */
  outputs: readonly { path: string; content: string }[];
}

</storycode:copy>
<a id="f-c72b68d0862a-resolveSearchHit"></a>
### `resolveSearchHit`

<storycode:copy id="f-c72b68d0862a-resolveSearchHit">
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

</storycode:copy>
<a id="f-c72b68d0862a-lineRangeToChars"></a>
### `lineRangeToChars`

The UTF-16 char range covering 1-based lines `startLine`..`endLine` of
`content`, clamped to the lines that exist. The trailing newline stays
outside the range, so a reveal selects the text and not the break after it.

<storycode:copy id="f-c72b68d0862a-lineRangeToChars">
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
</storycode:copy>

### Assemble `apps/web/src/lib/searchNavigation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/searchNavigation.ts"><storycode:paste select="#f-c72b68d0862a-SearchNavigation"/><storycode:paste select="#f-c72b68d0862a-SearchContext"/><storycode:paste select="#f-c72b68d0862a-resolveSearchHit"/><storycode:paste select="#f-c72b68d0862a-lineRangeToChars"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-d048d0998364"></a>
## `apps/web/src/lib/searchNavigation.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-d048d0998364-hit"></a>
### `hit`

<storycode:copy id="f-d048d0998364-hit">
import { describe, expect, it } from "vitest";
import { lineRangeToChars, resolveSearchHit } from "./searchNavigation";
import type { SearchHit } from "../api/types";

function hit(path: string, startLine = 1, endLine = 1): SearchHit {
  return { path, start_line: startLine, end_line: endLine, score: 1, snippet: "" };
}

</storycode:copy>
<a id="f-d048d0998364-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-d048d0998364-expression_statement">
describe("where a search hit navigates", () => {
  const context = {
    currentDocPath: "docs/stats.md",
    docs: [
      { id: "doc-1", path: "docs/stats.md" },
      { id: "doc-2", path: "docs/other.md" },
    ],
    outputs: [{ path: "out/stats.py", content: "import sys\n\nprint(1)\n" }],
  };

  it("scrolls the current document rather than reloading it", () => {
    // The current doc is also in the folder listing; the line wins.
    expect(resolveSearchHit(hit("docs/stats.md", 7, 9), context)).toEqual({
      kind: "current-doc",
      line: 6,
    });
  });

  it("routes to another document by its id", () => {
    expect(resolveSearchHit(hit("docs/other.md", 3, 3), context)).toEqual({
      kind: "doc",
      id: "doc-2",
    });
  });

  it("matches a server-relative path against the listed one", () => {
    // The search index answers relative to the served folder; the app may
    // know the same file under an absolute path. samePath bridges the two.
    const absolute = { ...context, currentDocPath: "/home/me/project/docs/stats.md" };
    expect(resolveSearchHit(hit("docs/stats.md", 2, 2), absolute)).toEqual({
      kind: "current-doc",
      line: 1,
    });
  });

  it("opens a generated file at the hit's char range", () => {
    expect(resolveSearchHit(hit("out/stats.py", 3, 3), context)).toEqual({
      kind: "generated",
      path: "out/stats.py",
      range: [12, 20],
    });
  });

  it("disables a hit that belongs to nothing open", () => {
    expect(resolveSearchHit(hit("vendor/lib.rs"), context)).toEqual({ kind: "none" });
  });
});

</storycode:copy>
<a id="f-d048d0998364-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-d048d0998364-expression_statement-2">
describe("line ranges become char ranges", () => {
  const content = "one\ntwo\nthree\n";

  it("covers a single line without its newline", () => {
    expect(lineRangeToChars(content, 2, 2)).toEqual([4, 7]);
  });

  it("covers a multi-line range from first line start to last line end", () => {
    expect(lineRangeToChars(content, 1, 2)).toEqual([0, 7]);
  });

  it("clamps lines past the end of the file", () => {
    // The index may be a run behind the file on disk; a stale line number
    // must land somewhere visible rather than throw.
    expect(lineRangeToChars(content, 99, 120)).toEqual([14, 14]);
    expect(lineRangeToChars("one\ntwo", 2, 9)).toEqual([4, 7]);
  });

  it("treats an inverted or zero range as the start line", () => {
    expect(lineRangeToChars(content, 2, 1)).toEqual([4, 7]);
    expect(lineRangeToChars(content, 0, 0)).toEqual([0, 3]);
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/searchNavigation.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/searchNavigation.test.ts"><storycode:paste select="#f-d048d0998364-hit"/><storycode:paste select="#f-d048d0998364-expression_statement"/><storycode:paste select="#f-d048d0998364-expression_statement-2"/></storycode:file>


</storycode:doc>
