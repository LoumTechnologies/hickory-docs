import { describe, expect, it } from "vitest";

import { railLines, railWidthCh, type RailBlock, textExtent } from "./rightRail";

// A ten-char-per-line "document": position 0..9 is line 1, 10..19 line 2, …
const lineAt = (pos: number) => Math.floor(pos / 10) + 1;

describe("railLines", () => {
  it("maps each viewport block to its line number and box", () => {
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      { from: 10, top: 20, height: 20 },
      { from: 20, top: 40, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20 },
      { line: 2, top: 20, height: 20 },
      { line: 3, top: 40, height: 20 },
    ]);
  });

  it("merges consecutive blocks of one wrapped line into a single entry", () => {
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      // Line 2 split across two height-map entries (wrapping): one rail
      // entry spanning both, so the rail stays cell-for-cell with the left
      // gutter.
      { from: 10, top: 20, height: 20 },
      { from: 15, top: 40, height: 20 },
      { from: 20, top: 60, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20 },
      { line: 2, top: 20, height: 40 },
      { line: 3, top: 60, height: 20 },
    ]);
  });

  it("is empty for an empty viewport", () => {
    expect(railLines([], lineAt)).toEqual([]);
  });

  it("keeps a taller block's full height (a line with a widget under it)", () => {
    expect(railLines([{ from: 0, top: 5, height: 64 }], lineAt)).toEqual([
      { line: 1, top: 5, height: 64 },
    ]);
  });
});

describe("railWidthCh", () => {
  it("grows with the digit count", () => {
    expect(railWidthCh(9)).toBe(3.5);
    expect(railWidthCh(99)).toBe(3.5);
    expect(railWidthCh(100)).toBe(4.5);
    expect(railWidthCh(12345)).toBe(6.5);
  });

  it("never collapses below a readable floor", () => {
    expect(railWidthCh(1)).toBe(3.5);
    expect(railWidthCh(0)).toBe(3.5);
  });
});

describe("textExtent", () => {
  // Protects the gutter-parity behavior: the number sits beside the TEXT,
  // and a block widget's rows stay blank on BOTH gutters.
  it("uses the text sub-block of a composite line block", () => {
    const extent = textExtent({ top: 100, height: 60 }, [
      { text: false, top: 100, height: 40 }, // widget above the line
      { text: true, top: 140, height: 20 },
    ]);
    expect(extent).toEqual({ top: 140, height: 20 });
  });

  it("keeps the whole block when it is plain text", () => {
    expect(textExtent({ top: 10, height: 20 }, null)).toEqual({ top: 10, height: 20 });
  });

  it("falls back to the block when no text child exists", () => {
    const extent = textExtent({ top: 0, height: 30 }, [{ text: false, top: 0, height: 30 }]);
    expect(extent).toEqual({ top: 0, height: 30 });
  });
});
