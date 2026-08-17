import { describe, expect, it } from "vitest";

import { clusterRowTops } from "./wrapRows";

describe("clusterRowTops", () => {
  it("returns one top per visual row for tidy row-per-rect input", () => {
    expect(
      clusterRowTops([
        { top: 0, bottom: 20, width: 100 },
        { top: 20, bottom: 40, width: 60 },
      ]),
    ).toEqual([0, 20]);
  });

  it("merges same-row fragments whose tops differ (styled spans), out of order", () => {
    // Real measured shape (a markdown line with inline-code spans): rects
    // arrive unsorted, and spans on one row report tops a few pixels apart
    // but overlapping. Tops 891 / 916 / 920 / 916 are TWO rows, not four.
    expect(
      clusterRowTops([
        { top: 891, bottom: 912, width: 300 },
        { top: 916, bottom: 937, width: 40 },
        { top: 920, bottom: 933, width: 55 },
        { top: 916, bottom: 937, width: 80 },
      ]),
    ).toEqual([891, 916]);
  });

  it("drops zero-width and zero-height rects (collapsed wrap-point fragments)", () => {
    expect(
      clusterRowTops([
        { top: 0, bottom: 20, width: 100 },
        { top: 20, bottom: 20, width: 100 },
        { top: 20, bottom: 40, width: 0 },
      ]),
    ).toEqual([0]);
  });

  it("is empty for no rects (an empty line)", () => {
    expect(clusterRowTops([])).toEqual([]);
  });

  it("keeps rows that merely graze each other distinct (a wrapped heading)", () => {
    // Real measured shape: a wrapped h1's tall glyph boxes overlap the next
    // row's by ~1.5px (bottom 170.6, next top 169.1). A couple of pixels is
    // not half a row — these are two rows, not one.
    expect(
      clusterRowTops([
        { top: 133.6, bottom: 170.6, width: 210 },
        { top: 133.6, bottom: 170.6, width: 25 },
        { top: 169.1, bottom: 206.6, width: 77 },
      ]),
    ).toEqual([133.6, 169.1]);
  });

  it("keeps three rows of small-print text distinct", () => {
    // Row height below the editor default (21.3px against 25.1px) — the
    // cluster boundary is overlap, not any assumed row height.
    expect(
      clusterRowTops([
        { top: 1157, bottom: 1174, width: 200 },
        { top: 1178, bottom: 1195, width: 210 },
        { top: 1199, bottom: 1216, width: 90 },
        { top: 1178, bottom: 1195, width: 30 },
        { top: 1199, bottom: 1216, width: 30 },
      ]),
    ).toEqual([1157, 1178, 1199]);
  });
});
