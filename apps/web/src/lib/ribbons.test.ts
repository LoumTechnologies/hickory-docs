import { describe, expect, it } from "vitest";
import { deriveRibbons, RIBBON_PALETTE_SIZE } from "./ribbons";
import { weaveOutputs } from "./weave";
import { WEAVE_SOURCE } from "../mock/mockData";
import type { OutputFile } from "../api/types";

const DOC = "docs/weave-demo.hick";

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
        { start: 0, end: 3, origin: { kind: "paste", doc_path: "d.hick", span: [10, 13] } },
        { start: 3, end: 6, origin: { kind: "paste", doc_path: "d.hick", span: [10, 13] } },
      ],
    };
    const ribbons = deriveRibbons(file, "d.hick", "0123456789abc.hick source");
    expect(ribbons).toHaveLength(2);
    expect(ribbons[0].color).toBe(ribbons[1].color);
  });

  it("provenance from other documents is skipped (no local anchor)", () => {
    const file: OutputFile = {
      path: "out.py",
      language: "python",
      content: "xy",
      provenance: [
        { start: 0, end: 1, origin: { kind: "literal", doc_path: "other.hick", span: [0, 1] } },
        { start: 1, end: 2, origin: { kind: "literal", doc_path: "mine.hick", span: [0, 1] } },
      ],
    };
    const ribbons = deriveRibbons(file, "mine.hick", "s");
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
