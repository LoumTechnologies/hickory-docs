// Protects docs/guarantees/lineage/a-ribbon-is-drawn-where-it-is-asked-for.md
import { describe, expect, it } from "vitest";

import {
  caretTouches,
  loadRibbonVisibility,
  RIBBON_VISIBILITY_KEY,
  saveRibbonVisibility,
} from "./ribbonVisibility";

describe("ribbon visibility persistence", () => {
  it("defaults to the caret with nothing stored", () => {
    localStorage.removeItem(RIBBON_VISIBILITY_KEY);
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveRibbonVisibility("always");
    expect(localStorage.getItem(RIBBON_VISIBILITY_KEY)).toBe("always");
    expect(loadRibbonVisibility()).toBe("always");
    saveRibbonVisibility("caret");
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("falls back to the caret on a value nothing ever wrote", () => {
    localStorage.setItem(RIBBON_VISIBILITY_KEY, "sometimes");
    expect(loadRibbonVisibility()).toBe("caret");
  });

  it("works without any storage at all", () => {
    expect(loadRibbonVisibility(null)).toBe("caret");
    expect(() => saveRibbonVisibility("always", null)).not.toThrow();
  });
});

describe("whether the caret is in an involved block", () => {
  const block = { from: 10, to: 20 };

  it("is in when the caret sits inside", () => {
    expect(caretTouches(block, { from: 15, to: 15 })).toBe(true);
  });

  it("counts both edges — a caret at the block's end is still in it", () => {
    expect(caretTouches(block, { from: 10, to: 10 })).toBe(true);
    expect(caretTouches(block, { from: 20, to: 20 })).toBe(true);
  });

  it("is out on either side", () => {
    expect(caretTouches(block, { from: 9, to: 9 })).toBe(false);
    expect(caretTouches(block, { from: 21, to: 21 })).toBe(false);
  });

  it("counts a selection that merely overlaps, in either direction", () => {
    expect(caretTouches(block, { from: 0, to: 12 })).toBe(true);
    expect(caretTouches(block, { from: 18, to: 40 })).toBe(true);
    // A backwards selection is the same selection.
    expect(caretTouches(block, { from: 40, to: 18 })).toBe(true);
    expect(caretTouches(block, { from: 30, to: 40 })).toBe(false);
  });
});
