import { describe, expect, it } from "vitest";
import { parseHickDoc } from "./hickDoc";
import {
  computeFoldRanges,
  foldRangeForLine,
  FOLDABLE_BLOCKS,
  ingestedFileFolds,
  workFoldRanges,
} from "./folding";
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

  it("folds the agent's work but keeps its closing tag line visible", () => {
    // docs/guarantees/agent/a-session-is-the-conversation.md — a folded tool
    // call still reads `<hick:tool …>` … `</hick:tool>`.
    const text = [
      "<hick:session>",
      "<hick:assistant>",
      "Looking.",
      '<hick:tool name="read_doc">',
      "<hick:input>a.md</hick:input>",
      "</hick:tool>",
      "</hick:assistant>",
      '<hick:observation source="action-0" exit="0">',
      "3",
      "</hick:observation>",
      "</hick:session>",
    ].join("\n");
    const all = rangesOf(text);
    const tool = all.find((r) => r.name === "tool")!;
    expect(tool.from).toBe(text.indexOf("\n", text.indexOf("<hick:tool")));
    expect(tool.to).toBe(text.indexOf("\n</hick:tool>"));
    const obs = all.find((r) => r.name === "observation")!;
    expect(obs.to).toBe(text.indexOf("\n</hick:observation>"));
    // Assistant turns still fold through their closing tag: they are said,
    // not done, and a reader folding one wants it gone.
    const assistant = all.find((r) => r.name === "assistant")!;
    expect(assistant.to).toBe(text.indexOf("</hick:assistant>") + "</hick:assistant>".length);
    // The initial folds are exactly the work.
    expect(workFoldRanges(all).map((r) => r.from)).toEqual([tool.from, obs.from]);
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

// Protects docs/guarantees/authoring/an-ingested-scaffold-opens-as-a-tree.md
describe("an ingested scaffold folds as a tree", () => {
  // A dotnet-shaped ingest: path-sorted, one directory holding two files and
  // one holding a single file, plus files at the block's own root.
  const INGEST = [
    '<hick:exec container="sdk" mount="project:out">',
    '<hick:copy id="scaffold">dotnet new webapi -o out</hick:copy>',
    '<hick:ingested from="#scaffold" sha256="9f2c" at="2026-09-01" files="5" skipped="6">',
    '<hick:file path="service/Controllers/HomeController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class HomeController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Controllers/WeatherController.cs">using Microsoft.AspNetCore.Mvc;',
    "",
    "public class WeatherController : ControllerBase { }",
    "</hick:file>",
    '<hick:file path="service/Program.cs">var builder = WebApplication.CreateBuilder(args);',
    "builder.Build().Run();",
    "</hick:file>",
    '<hick:file path="service/Properties/launchSettings.json">{',
    '  "profiles": {}',
    "}",
    "</hick:file>",
    '<hick:file path="service/appsettings.json">{',
    '  "AllowedHosts": "*"',
    "}",
    "</hick:file>",
    "</hick:ingested>",
    "</hick:exec>",
    "",
  ].join("\n");

  const ingestRanges = () => rangesOf(INGEST);
  const fileFolds = () =>
    ingestRanges().filter((r) => r.kind === "block" && r.name === "file");

  it("labels each ingested file with how many lines it holds", () => {
    // Content starts on the tag's own line, so the label counts the file, not
    // the rows the fold hides.
    expect(fileFolds().map((r) => r.label)).toEqual([
      "3 lines",
      "3 lines",
      "2 lines",
      "3 lines",
      "3 lines",
    ]);
  });

  it("labels the block itself with the number of files it carries", () => {
    const whole = ingestRanges().find(
      (r) => r.kind === "block" && r.name === "ingested",
    );
    // Counted from the file blocks actually present, not from `files=`: the
    // attribute records what the run wrote, and the document can be edited.
    expect(whole?.label).toBe("5 files");
  });

  it("folds a directory of two or more files, naming it", () => {
    const dirs = ingestRanges().filter((r) => r.kind === "ingested-dir");
    expect(dirs.map((r) => r.label)).toEqual(["Controllers/ · 2 files, 6 lines"]);
    // It starts where the first file's LINE starts — a directory has no line
    // of its own to keep visible — and ends with the last file's closing tag.
    const first = INGEST.indexOf('<hick:file path="service/Controllers/Home');
    expect(dirs[0].from).toBe(first);
    expect(dirs[0].to).toBe(
      INGEST.indexOf("</hick:file>", INGEST.indexOf("WeatherController : ")) +
        "</hick:file>".length,
    );
  });

  it("does not fold a run at the scaffold's own root", () => {
    // Two files sit directly in service/, split by Properties/ between them —
    // and even adjacent they would not fold: a root run is the block's claim
    // told again, in pieces.
    const adjacent = [
      '<hick:ingested from="#s" sha256="a" at="2026-09-01" files="3" skipped="0">',
      '<hick:file path="service/a.txt">a',
      "</hick:file>",
      '<hick:file path="service/b.txt">b',
      "</hick:file>",
      '<hick:file path="service/deep/c.txt">c',
      "</hick:file>",
      "</hick:ingested>",
      "",
    ].join("\n");
    expect(rangesOf(adjacent).some((r) => r.kind === "ingested-dir")).toBe(false);
  });

  it("does not fold a directory holding one file, nor one holding them all", () => {
    const dirs = ingestRanges().filter((r) => r.kind === "ingested-dir");
    // Properties/ has a single file: the fold would stand for the line under it.
    expect(dirs.some((r) => r.label?.startsWith("Properties/"))).toBe(false);
    // And a run covering every file is the block, which folds one line higher.
    const all = [
      '<hick:ingested from="#s" sha256="a" at="2026-09-01" files="2" skipped="0">',
      '<hick:file path="app/a.txt">a',
      "</hick:file>",
      '<hick:file path="app/b.txt">b',
      "</hick:file>",
      "</hick:ingested>",
      "",
    ].join("\n");
    expect(rangesOf(all).some((r) => r.kind === "ingested-dir")).toBe(false);
  });

  it("leaves a hand-written hick:file alone — no label, no fold on open", () => {
    const own = [
      '<hick:file path="notes.txt">mine',
      "still mine",
      "</hick:file>",
      "",
    ].join("\n");
    const [r] = rangesOf(own).filter((x) => x.name === "file");
    expect(r.label).toBeUndefined();
    expect(ingestedFileFolds(rangesOf(own))).toEqual([]);
  });

  it("opens with every ingested file folded, and nothing else", () => {
    const folds = ingestedFileFolds(ingestRanges());
    expect(folds).toHaveLength(5);
    // Each fold starts at the end of its own tag line, so the path stays on
    // screen — that visible line per file IS the tree.
    for (const f of folds) expect(INGEST[f.from]).toBe("\n");
    expect(folds.map((f) => f.to)).toEqual(
      fileFolds().map((r) => r.to),
    );
  });

  it("hands the directory fold to a click on the run's first line", () => {
    const dir = ingestRanges().find((r) => r.kind === "ingested-dir")!;
    const lineEnd = INGEST.indexOf("\n", dir.from);
    // Two ranges start on that line — the directory and the first file's own.
    // The outermost wins, which is the rule every nested block already uses.
    expect(foldRangeForLine(ingestRanges(), dir.from, lineEnd)).toEqual({
      from: dir.from,
      to: dir.to,
    });
  });
});
