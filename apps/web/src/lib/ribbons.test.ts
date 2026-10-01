import { describe, expect, it } from "vitest";
import { deriveRibbons, drawnRange, fragmentKey, groupBySourceBlock, RIBBON_PALETTE_SIZE } from "./ribbons";
import { weaveOutputs } from "./weave";
import { WEAVE_SOURCE } from "../mock/mockData";
import type { OutputFile, Provenance } from "../api/types";

const DOC = "docs/weave-demo.md";

describe("deriveRibbons — from the weave demo's real provenance", () => {
  const [file] = weaveOutputs(WEAVE_SOURCE, DOC);

  it("yields one ribbon per non-synthetic provenance entry", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    const editable = file.provenance.filter((p) => p.origin.kind !== "synthetic");
    expect(ribbons).toHaveLength(editable.length);
    expect(ribbons.length).toBeGreaterThanOrEqual(4); // 2 pastes + literals
  });

  it("synthetic ranges get no ribbon", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    expect(file.provenance.some((p) => p.origin.kind === "synthetic")).toBe(true);
    expect(ribbons.every((r) => (r.kind as string) !== "synthetic")).toBe(true);
  });

  it("output text of each ribbon equals the source text of its span", () => {
    for (const r of deriveRibbons(file, DOC, WEAVE_SOURCE)) {
      expect(file.content.slice(...r.outputRange)).toBe(
        WEAVE_SOURCE.slice(...r.sourceSpan),
      );
    }
  });

  it("thickness input is the output byte length", () => {
    for (const r of deriveRibbons(file, DOC, WEAVE_SOURCE)) {
      expect(r.bytes).toBe(r.outputRange[1] - r.outputRange[0]); // ASCII demo
      expect(r.bytes).toBeGreaterThan(0);
    }
  });

  it("distinct source fragments get distinct stable colors, in order", () => {
    const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);
    const seen = new Map<string, number>();
    for (const r of ribbons) {
      const k = `${r.sourceByteSpan[0]}:${r.sourceByteSpan[1]}`;
      if (seen.has(k)) expect(r.color).toBe(seen.get(k));
      else seen.set(k, r.color);
      expect(r.color).toBeGreaterThanOrEqual(0);
      expect(r.color).toBeLessThan(RIBBON_PALETTE_SIZE);
    }
    // First few distinct fragments walk the palette in order.
    expect([...seen.values()].slice(0, RIBBON_PALETTE_SIZE)).toEqual(
      [...seen.values()].slice(0, RIBBON_PALETTE_SIZE).map((_, i) => i),
    );
  });
});

describe("deriveRibbons — filtering and repeats", () => {
  it("the same fragment pasted twice keeps one color across two ribbons", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "abcabc",
      provenance: [
        { start: 0, end: 3, origin: { kind: "paste", doc_path: "d.md", span: [10, 13] } },
        { start: 3, end: 6, origin: { kind: "paste", doc_path: "d.md", span: [10, 13] } },
      ],
    };
    const ribbons = deriveRibbons(file, "d.md", "0123456789abc.md source");
    expect(ribbons).toHaveLength(2);
    expect(ribbons[0].color).toBe(ribbons[1].color);
  });

  it("provenance from other documents is skipped (no local anchor)", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "xy",
      provenance: [
        { start: 0, end: 1, origin: { kind: "literal", doc_path: "other.md", span: [0, 1] } },
        { start: 1, end: 2, origin: { kind: "literal", doc_path: "mine.md", span: [0, 1] } },
      ],
    };
    const ribbons = deriveRibbons(file, "mine.md", "s");
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].key).toBe("out.py:1");
  });

  it("empty output ranges are skipped", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "x",
      provenance: [
        { start: 1, end: 1, origin: { kind: "literal", doc_path: "d", span: [0, 0] } },
      ],
    };
    expect(deriveRibbons(file, "d", "src")).toHaveLength(0);
  });

  it("converts byte offsets to char offsets for non-ASCII content", () => {
    // "é" is 2 UTF-8 bytes but 1 UTF-16 unit.
    const source = "é<x>";
    const file: OutputFile = {
      path: "o",
      language: "text",
      content: "é!",
      provenance: [{ start: 2, end: 3, origin: { kind: "literal", doc_path: "d", span: [2, 3] } }],
    };
    const [r] = deriveRibbons(file, "d", source);
    expect(r.outputRange).toEqual([1, 2]); // after the 2-byte é
    expect(r.sourceSpan).toEqual([1, 2]);
  });
});

describe("groupBySourceBlock — one terminal band per source block", () => {
  const [file] = weaveOutputs(WEAVE_SOURCE, DOC);
  const ribbons = deriveRibbons(file, DOC, WEAVE_SOURCE);

  it("collapses fragments of one block into one group keyed by its byte span", () => {
    const groups = groupBySourceBlock(ribbons);
    const spans = new Set(ribbons.map((r) => fragmentKey(r)));
    expect(groups.size).toBe(spans.size);
    // A file a block feeds twice is one statement, not two.
    expect(groups.size).toBeLessThanOrEqual(ribbons.length);
  });

  it("sums the bytes of every fragment in the group", () => {
    const groups = groupBySourceBlock(ribbons);
    let total = 0;
    for (const group of groups.values()) total += group.bytes;
    expect(total).toBe(ribbons.reduce((sum, r) => sum + r.bytes, 0));
  });

  it("keeps the FIRST ribbon of the group, so a click lands where the block's text starts", () => {
    const groups = groupBySourceBlock(ribbons);
    for (const [key, group] of groups) {
      const first = ribbons.find((r) => fragmentKey(r) === key);
      expect(group.ribbon).toBe(first);
    }
  });
});

describe("whitespaceOnly", () => {
  // A whitespace-only attribution is true but says nothing until someone is
  // working exactly there — the overlay draws these on hover only.
  const file = (content: string, provenance: Provenance[]): OutputFile => ({
    path: "out.txt",
    content,
    provenance,
    language: "text",
  });
  const literal = (start: number, end: number, span: [number, number]): Provenance => ({
    start,
    end,
    origin: { kind: "literal", doc_path: "doc.md", span },
  });

  it("flags a ribbon whose attributed bytes are blank lines and indentation", () => {
    const content = "code\n\n    \nmore";
    const ribbons = deriveRibbons(
      file(content, [literal(4, 11, [10, 17])]),
      "doc.md",
      "0123456789\n\n    \nabc",
    );
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].whitespaceOnly).toBe(true);
  });

  it("keeps a ribbon with any visible character always-on", () => {
    const content = "code\n\nx\nmore";
    const ribbons = deriveRibbons(
      file(content, [literal(4, 8, [10, 14])]),
      "doc.md",
      "0123456789\n\nx\nzzzzzz",
    );
    expect(ribbons).toHaveLength(1);
    expect(ribbons[0].whitespaceOnly).toBe(false);
  });
});

describe("drawnRange", () => {
  // A span's DRAWN extent covers only lines where it owns visible
  // characters — trailing newlines/indentation on someone else's line must
  // not inflate a line-granular brace into a false claim.
  it("drops a leading newline belonging to a foreign line", () => {
    //            0         1
    //            0123456789012345678
    const text = "</hick:copy>\ncode\n";
    // Span starts at the newline after the foreign tag.
    expect(drawnRange(text, 12, 18)).toEqual([13, 18]);
  });

  it("drops trailing indentation on a line whose text is foreign", () => {
    const text = "own line\n    foreign\n";
    // Span covers "own line\n" plus the next line's indentation only.
    expect(drawnRange(text, 0, 13)).toEqual([0, 9]);
  });

  it("keeps blank lines the span covers", () => {
    const text = "a\n\n\nb\n";
    expect(drawnRange(text, 0, 6)).toEqual([0, 6]);
  });

  it("returns null for a whitespace-only span on foreign lines", () => {
    const text = "foreign\nalso foreign\n";
    expect(drawnRange(text, 7, 8)).toBeNull();
  });

  it("keeps a span fully inside one owned line", () => {
    const text = "abc def\n";
    expect(drawnRange(text, 4, 7)).toEqual([4, 7]);
  });

  it("never widens beyond the original span", () => {
    const text = "abcdef\n";
    const trimmed = drawnRange(text, 2, 4);
    expect(trimmed).toEqual([2, 4]);
  });
});
