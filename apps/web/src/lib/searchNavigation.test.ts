import { describe, expect, it } from "vitest";
import { lineRangeToChars, resolveSearchHit } from "./searchNavigation";
import type { SearchHit } from "../api/types";

function hit(path: string, startLine = 1, endLine = 1): SearchHit {
  return { path, start_line: startLine, end_line: endLine, score: 1, snippet: "" };
}

describe("where a search hit navigates", () => {
  const context = {
    currentDocPath: "docs/stats.hick",
    docs: [
      { id: "doc-1", path: "docs/stats.hick" },
      { id: "doc-2", path: "docs/other.hick" },
    ],
    outputs: [{ path: "out/stats.py", content: "import sys\n\nprint(1)\n" }],
  };

  it("scrolls the current document rather than reloading it", () => {
    // The current doc is also in the folder listing; the line wins.
    expect(resolveSearchHit(hit("docs/stats.hick", 7, 9), context)).toEqual({
      kind: "current-doc",
      line: 6,
    });
  });

  it("routes to another document by its id", () => {
    expect(resolveSearchHit(hit("docs/other.hick", 3, 3), context)).toEqual({
      kind: "doc",
      id: "doc-2",
    });
  });

  it("matches a server-relative path against the listed one", () => {
    // The search index answers relative to the served folder; the app may
    // know the same file under an absolute path. samePath bridges the two.
    const absolute = { ...context, currentDocPath: "/home/me/project/docs/stats.hick" };
    expect(resolveSearchHit(hit("docs/stats.hick", 2, 2), absolute)).toEqual({
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
