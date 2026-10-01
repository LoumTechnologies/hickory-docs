import { describe, expect, it } from "vitest";

import { SearchIndex, tokenize } from "./bm25";
import type { FileModel } from "./model";

const file = (path: string, text: string): FileModel => ({
  path,
  kind: path.endsWith(".md") ? "document" : "generated",
  lines: text.split("\n"),
});

const CORPUS = [
  file(
    "notes.md",
    [
      "def load_runs(path):",
      "    return json.load(open(path))",
      "def summarise(runs):",
      "    return statistics.median(runs)",
      "The median is taken over fifty runs.",
      "def unrelated():",
      "    pass",
    ].join("\n"),
  ),
  file(
    "app.py",
    ["def loadRuns(path):", "    pass", "def main():", "    print(loadRuns('x'))"].join("\n"),
  ),
];

describe("tokenizing for search", () => {
  it("splits identifiers so either naming convention finds the other", () => {
    expect(tokenize("loadRuns")).toContain("load");
    expect(tokenize("loadRuns")).toContain("runs");
    expect(tokenize("load_runs")).toContain("load");
    expect(tokenize("load_runs")).toContain("runs");
    // The whole identifier is kept too, so an exact name still scores.
    expect(tokenize("load_runs")).toContain("load_runs");
  });

  it("drops punctuation and single characters", () => {
    expect(tokenize("return json.load(open(path))")).toEqual(
      expect.arrayContaining(["return", "json", "load", "open", "path"]),
    );
    expect(tokenize("a + b")).toEqual([]);
  });
});

describe("ranked search", () => {
  const index = new SearchIndex(CORPUS);

  it("finds an identifier written in the other convention", () => {
    // The thing substring search cannot do: `load_runs` finding `loadRuns`.
    const hits = index.search("load_runs");
    const files = new Set(hits.map((h) => h.file));
    expect(files.has("notes.md")).toBe(true);
    expect(files.has("app.py")).toBe(true);
  });

  it("ranks a rare term above a common one", () => {
    const hits = index.search("median runs");
    // Lines with `median` (rare) beat lines with only `runs` (common).
    expect(hits[0].file).toBe("notes.md");
    const top = CORPUS[0].lines[hits[0].line];
    expect(top.toLowerCase()).toContain("median");
  });

  it("puts a literal match above a merely statistical one", () => {
    const hits = index.search("statistics.median");
    expect(hits[0].exact).toBe(true);
    expect(CORPUS[0].lines[hits[0].line]).toContain("statistics.median(runs)");
  });

  it("answers a multi-word query with the line that has most of it", () => {
    const hits = index.search("load open path");
    expect(CORPUS[0].lines[hits[0].line]).toContain("json.load(open(path))");
  });

  it("returns nothing for an empty query rather than everything", () => {
    expect(index.search("   ")).toEqual([]);
  });

  it("groups hits by file for folding a column to its matches", () => {
    const byFile = index.searchByFile("loadRuns");
    expect(byFile.get("app.py")?.size).toBeGreaterThan(0);
    // Line numbers are 0-based indices into that file.
    for (const [path, lines] of byFile) {
      const model = CORPUS.find((f) => f.path === path)!;
      for (const line of lines) expect(model.lines[line]).toBeDefined();
    }
  });

  it("never loses a line that literally contains the query", () => {
    // Statistics must not overrule what somebody typed.
    const hits = index.search("fifty");
    expect(hits.some((h) => CORPUS[0].lines[h.line].includes("fifty"))).toBe(true);
  });
});
