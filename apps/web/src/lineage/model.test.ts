import { describe, expect, it } from "vitest";

import {
  foldRange,
  fullRange,
  holesOf,
  matchingLines,
  nodeAt,
  normalizeRanges,
  revealRange,
  searchRanges,
  stageOf,
  walk,
  type FileModel,
  type LineageModel,
  type Link,
} from "./model";

const file = (lines: number): FileModel => ({
  path: "f.md",
  kind: "document",
  lines: Array.from({ length: lines }, (_, i) => `line ${i + 1}`),
});

describe("visible ranges and the holes between them", () => {
  it("merges ranges that touch, so a hole never hides nothing", () => {
    // The whole reason a hole disappears when its last line is revealed.
    expect(normalizeRanges([[0, 4], [5, 9]], 19)).toEqual([[0, 9]]);
    expect(normalizeRanges([[0, 6], [3, 9]], 19)).toEqual([[0, 9]]);
    expect(normalizeRanges([[0, 4], [7, 9]], 19)).toEqual([[0, 4], [7, 9]]);
  });

  it("folds a range in two when the fold lands in the middle", () => {
    expect(foldRange([[0, 19]], 5, 9, 19)).toEqual([[0, 4], [10, 19]]);
    expect(foldRange([[0, 19]], 0, 3, 19)).toEqual([[4, 19]]);
    expect(foldRange([[0, 19]], 15, 19, 19)).toEqual([[0, 14]]);
    expect(foldRange([[0, 19]], 7, 7, 19)).toEqual([[0, 6], [8, 19]]);
    expect(foldRange([[0, 19]], 0, 19, 19)).toEqual([]);
  });

  it("grows an existing hole rather than adding a second one beside it", () => {
    expect(foldRange([[0, 4], [10, 19]], 10, 12, 19)).toEqual([[0, 4], [13, 19]]);
  });

  it("moves either edge of a hole in either direction", () => {
    const max = 39;
    const ranges: [number, number][] = [[0, 9], [30, 39]];
    expect(holesOf(ranges, max)).toEqual([[10, 29]]);
    // top edge down (reveal from the top) / top edge up (swallow above)
    expect(holesOf(revealRange(ranges, 10, 19, max), max)).toEqual([[20, 29]]);
    expect(holesOf(foldRange(ranges, 0, 9, max), max)).toEqual([[0, 29]]);
    // bottom edge up (reveal from the bottom) / bottom edge down (swallow below)
    expect(holesOf(revealRange(ranges, 20, 29, max), max)).toEqual([[10, 19]]);
    expect(holesOf(foldRange(ranges, 30, 39, max), max)).toEqual([[10, 39]]);
  });

  it("reveals a file back to one whole range", () => {
    const f = file(20);
    const folded = foldRange(fullRange(f), 5, 9, 19);
    expect(revealRange(folded, 5, 9, 19)).toEqual([[0, 19]]);
  });
});

describe("search", () => {
  it("folds a file down to its matches with context", () => {
    const f: FileModel = {
      path: "f.md",
      kind: "document",
      lines: ["alpha", "beta", "gamma", "delta", "epsilon", "beta again"],
    };
    expect(matchingLines(f, "beta")).toEqual(new Set([1, 5]));
    expect(searchRanges(f, "beta", 1)).toEqual([[0, 2], [4, 5]]);
    // An empty query is not a filter that matches nothing; it is no filter.
    expect(searchRanges(f, "  ")).toEqual([[0, 5]]);
  });
});

describe("walking the graph", () => {
  const links: Link[] = [
    { from: "decision", to: "requirement", kind: "asserted" },
    { from: "requirement", to: "fragment", kind: "asserted" },
    { from: "fragment", to: "output", kind: "paste" },
  ];
  const all = new Set<"paste" | "asserted">(["paste", "asserted"]);

  it("follows every enabled kind", () => {
    expect(walk(links, all, "decision", "down")).toEqual(
      new Set(["requirement", "fragment", "output"]),
    );
    expect(walk(links, all, "output", "up")).toEqual(
      new Set(["fragment", "requirement", "decision"]),
    );
  });

  it("stops where a disabled kind would have carried it", () => {
    // Not cosmetic: with only computed links, a decision reaches nothing,
    // and that shortness is the honest state of the chain.
    const computed = new Set<"paste" | "asserted">(["paste"]);
    expect(walk(links, computed, "decision", "down")).toEqual(new Set());
    expect(walk(links, computed, "fragment", "down")).toEqual(new Set(["output"]));
  });

  it("terminates on a cycle", () => {
    const cyclic: Link[] = [
      { from: "a", to: "b", kind: "paste" },
      { from: "b", to: "a", kind: "paste" },
    ];
    expect(walk(cyclic, all, "a", "down")).toEqual(new Set(["b", "a"]));
  });
});

describe("stages and nodes", () => {
  const model: LineageModel = {
    files: new Map(),
    nodes: new Map([
      ["outer", { id: "outer", file: "d.md", startLine: 0, endLine: 20, label: "outer", kind: "file" }],
      ["inner", { id: "inner", file: "d.md", startLine: 5, endLine: 6, label: "inner", kind: "paste" }],
      ["elsewhere", { id: "elsewhere", file: "e.md", startLine: 0, endLine: 3, label: "e", kind: "copy" }],
    ]),
    links: [],
    stages: [
      { name: "d", doc: "d.md", files: ["d.md", "out/a.py"] },
      { name: "e", doc: "e.md", files: ["e.md"] },
    ],
  };

  it("finds the stage that owns a generated file", () => {
    expect(stageOf(model, "out/a.py")?.name).toBe("d");
    expect(stageOf(model, "nowhere.txt")).toBeUndefined();
  });

  it("picks the tightest node covering a line", () => {
    // A paste inside a file block must select the paste: the enclosing block
    // is always a correct answer and almost never the useful one.
    expect(nodeAt(model, "d.md", 5)?.id).toBe("inner");
    expect(nodeAt(model, "d.md", 12)?.id).toBe("outer");
    expect(nodeAt(model, "d.md", 30)).toBeUndefined();
  });
});
