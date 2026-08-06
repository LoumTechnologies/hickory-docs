import { describe, expect, it } from "vitest";
import { parseHickDoc } from "./hickDoc";
import { computeFoldRanges, foldRangeForLine, FOLDABLE_BLOCKS } from "./folding";
import { WEAVE_SOURCE, CLI_SOURCE } from "../mock/mockData";

function rangesOf(text: string) {
  return computeFoldRanges(parseHickDoc(text), text);
}

describe("computeFoldRanges — hick blocks", () => {
  it("folds an exec block body, keeping the opening tag line visible", () => {
    const text = 'intro\n<hick:exec container="shell">\nls -la\n</hick:exec>\ntail\n';
    const [r] = rangesOf(text).filter((r) => r.kind === "block");
    expect(r.name).toBe("exec");
    // Fold starts at the end of the opening-tag line…
    expect(r.from).toBe(text.indexOf("\n", text.indexOf("<hick:exec")));
    // …and swallows the body plus the closing tag.
    expect(r.to).toBe(text.indexOf("</hick:exec>") + "</hick:exec>".length);
  });

  it("folds copy/cut/file/when/container/session-turn blocks", () => {
    const text = [
      '<hick:copy id="a">',
      "x = 1",
      "</hick:copy>",
      '<hick:cut id="b">',
      "y = 2",
      "</hick:cut>",
      '<hick:file path="f.py" language="python">',
      "z = 3",
      "</hick:file>",
      '<hick:when test="ci">',
      "gated",
      "</hick:when>",
      '<hick:container name="env">',
      '<hick:allow network="github.com:443" />',
      "</hick:container>",
      "<hick:user>",
      "hello",
      "</hick:user>",
    ].join("\n");
    const blocks = rangesOf(text).filter((r) => r.kind === "block");
    const names = blocks.map((r) => r.name).sort();
    expect(names).toEqual(["container", "copy", "cut", "file", "user", "when"]);
    for (const r of blocks) expect(r.to).toBeGreaterThan(r.from);
  });

  it("skips self-closing and single-line blocks", () => {
    const text =
      '<hick:container name="shell" image="debian:12" />\n' +
      "<hick:exec>echo hi</hick:exec>\n";
    expect(rangesOf(text).filter((r) => r.kind === "block")).toEqual([]);
  });

  it("folds an unclosed block to the end of the document", () => {
    const text = '<hick:exec container="shell">\nnever closed\n';
    const [r] = rangesOf(text).filter((r) => r.kind === "block");
    expect(r.to).toBe(text.length);
  });

  it("covers every foldable multi-line block of the weave demo (d3)", () => {
    const blocks = rangesOf(WEAVE_SOURCE).filter((r) => r.kind === "block");
    // Two copy slots, one file, one exec.
    expect(blocks.map((r) => r.name).sort()).toEqual(["copy", "copy", "exec", "file"]);
    for (const r of blocks) expect(FOLDABLE_BLOCKS.has(r.name!)).toBe(true);
  });
});

describe("computeFoldRanges — markdown headings", () => {
  it("folds a section up to the next same-or-higher heading", () => {
    const text = "# A\naaa\n## B\nbbb\n# C\nccc";
    const rs = rangesOf(text).filter((r) => r.kind === "heading");
    expect(rs).toHaveLength(3);
    const a = rs.find((r) => r.from === text.indexOf("\naaa"))!;
    // A folds aaa + the B subsection, stopping before "# C".
    expect(text.slice(a.from, a.to)).toBe("\naaa\n## B\nbbb");
    const b = rs.find((r) => r.from === text.indexOf("\nbbb"))!;
    expect(text.slice(b.from, b.to)).toBe("\nbbb");
    const c = rs.find((r) => text.slice(r.from, r.to) === "\nccc");
    expect(c).toBeTruthy();
  });

  it("trims trailing blank lines and skips empty sections", () => {
    const text = "# A\nbody\n\n\n# B\n# C\ntail\n";
    const rs = rangesOf(text).filter((r) => r.kind === "heading");
    // A ends after "body" (blank lines trimmed); B has no content → no fold.
    const a = rs[0];
    expect(text.slice(a.from, a.to)).toBe("\nbody");
    expect(rs.some((r) => r.from === "# A\nbody\n\n\n# B".length)).toBe(false);
  });

  it("headings inside verbatim block bodies are not headings (CLI doc)", () => {
    const rs = rangesOf(CLI_SOURCE);
    // Exactly one heading in the quickstart doc.
    expect(rs.filter((r) => r.kind === "heading")).toHaveLength(1);
  });
});

describe("foldRangeForLine (the foldService predicate)", () => {
  const text = "# A\naaa\n<hick:exec>\nls\n</hick:exec>\n";
  const ranges = rangesOf(text);

  it("returns the range starting on the queried line", () => {
    const headLine: [number, number] = [0, 3]; // "# A"
    const r = foldRangeForLine(ranges, ...headLine);
    expect(r).toBeTruthy();
    expect(r!.from).toBe(3);
  });

  it("returns null for lines that start nothing", () => {
    const aaaFrom = text.indexOf("aaa");
    expect(foldRangeForLine(ranges, aaaFrom, aaaFrom + 3)).toBeNull();
  });

  it("prefers the outermost range when several start on one line", () => {
    const rs = [
      { from: 10, to: 20, kind: "block" as const },
      { from: 10, to: 40, kind: "block" as const },
    ];
    expect(foldRangeForLine(rs, 5, 10)!.to).toBe(40);
  });
});
