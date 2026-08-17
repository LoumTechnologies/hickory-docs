import { describe, expect, it } from "vitest";

import { railLines, railWidthCh, rowBoxes, type RailBlock, textExtent } from "./rightRail";

// A ten-char-per-line "document": position 0..9 is line 1, 10..19 line 2, …
const lineAt = (pos: number) => Math.floor(pos / 10) + 1;

describe("railLines", () => {
  it("maps each unwrapped viewport block to a number entry", () => {
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      { from: 10, top: 20, height: 20 },
      { from: 20, top: 40, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "number" },
      { line: 3, top: 40, height: 20, kind: "number" },
    ]);
  });

  it("splits a wrapped line into a number row plus wrap rows at its measured tops", () => {
    // Line 2 soft-wraps into three visual rows: ONE block (what
    // viewportLineBlocks actually yields for wrapping), with the rows'
    // measured tops. First row carries the number; the continuation rows are
    // wrap marks, and together they tile the block exactly.
    const blocks: RailBlock[] = [
      { from: 0, top: 0, height: 20 },
      { from: 10, top: 20, height: 60, rowTops: [20, 40, 60] },
      { from: 20, top: 80, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "number" },
      { line: 2, top: 40, height: 20, kind: "wrap" },
      { line: 2, top: 60, height: 20, kind: "wrap" },
      { line: 3, top: 80, height: 20, kind: "number" },
    ]);
  });

  it("splits uneven rows (a styled line) where they were measured, not evenly", () => {
    // A wrapped heading: two rows of 35.5px against a 25px default line
    // height — the boundaries come from measurement, never from division.
    const blocks: RailBlock[] = [{ from: 0, top: 0, height: 71, rowTops: [0, 35.5] }];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 0, height: 35.5, kind: "number" },
      { line: 1, top: 35.5, height: 35.5, kind: "wrap" },
    ]);
  });

  it("treats consecutive blocks of one line as continuations (defensive)", () => {
    // Should the height map ever split one line across blocks, the later
    // blocks are wrap rows — the number still renders exactly once.
    const blocks: RailBlock[] = [
      { from: 10, top: 0, height: 20 },
      { from: 15, top: 20, height: 20 },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 2, top: 0, height: 20, kind: "number" },
      { line: 2, top: 20, height: 20, kind: "wrap" },
    ]);
  });

  it("is empty for an empty viewport", () => {
    expect(railLines([], lineAt)).toEqual([]);
  });

  it("keeps a single unmeasured tall block whole: one number entry, no wrap rows", () => {
    // Without measured row tops there is nothing to split on — never guess.
    expect(railLines([{ from: 0, top: 5, height: 64 }], lineAt)).toEqual([
      { line: 1, top: 5, height: 64, kind: "number" },
    ]);
  });

  it("gives widget rows no entry while wrap rows of the same line get marks", () => {
    // The disambiguation the two kinds of "extra height" need: a widget row
    // was already cut away by textExtent (the block passed in IS the text
    // extent), so only genuine wrap rows inside that extent produce entries.
    const widgetAndText = textExtent({ top: 100, height: 90 }, [
      { text: false, top: 100, height: 40 }, // block widget above the line
      { text: true, top: 140, height: 50 }, // the line's text, wrapped twice
    ]);
    const blocks: RailBlock[] = [
      { from: 0, top: widgetAndText.top, height: widgetAndText.height, rowTops: [140, 165] },
    ];
    expect(railLines(blocks, lineAt)).toEqual([
      { line: 1, top: 140, height: 25, kind: "number" },
      { line: 1, top: 165, height: 25, kind: "wrap" },
    ]);
  });
});

describe("rowBoxes", () => {
  it("tiles the extent exactly at the given row tops", () => {
    expect(rowBoxes({ top: 10, height: 50 }, [10, 35])).toEqual([
      { top: 10, height: 25 },
      { top: 35, height: 25 },
    ]);
  });

  it("returns the whole extent for zero or one row top", () => {
    expect(rowBoxes({ top: 10, height: 50 }, [])).toEqual([{ top: 10, height: 50 }]);
    expect(rowBoxes({ top: 10, height: 50 }, [10])).toEqual([{ top: 10, height: 50 }]);
  });

  it("ignores tops outside the extent or out of order", () => {
    expect(rowBoxes({ top: 10, height: 40 }, [10, 5, 30, 30, 60])).toEqual([
      { top: 10, height: 20 },
      { top: 30, height: 20 },
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
