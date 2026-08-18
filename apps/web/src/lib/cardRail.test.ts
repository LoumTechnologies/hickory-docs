import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/the-gutters-never-skip-a-number.md
import { ICON_GAP, ICON_SIZE, iconVisible, popoverTop, stackIcons } from "./cardRail";

const PITCH = ICON_SIZE + ICON_GAP;

describe("stacking the rail's icons", () => {
  it("leaves well-separated icons exactly level with their lines", () => {
    expect(stackIcons([0, 100, 300])).toEqual([0, 100, 300]);
  });

  it("pushes a crowded icon DOWN, never up", () => {
    // Up would draw an icon above the line it belongs to, so a reader
    // scanning from a line to the rail would have to look backwards.
    const tops = stackIcons([100, 102, 104]);
    expect(tops[0]).toBe(100);
    expect(tops[1]).toBe(100 + PITCH);
    expect(tops[2]).toBe(100 + 2 * PITCH);
  });

  it("stays monotonic, so rail order is document order", () => {
    const tops = stackIcons([0, 1, 2, 500, 501, 40]);
    for (let i = 1; i < tops.length; i++) expect(tops[i]).toBeGreaterThan(tops[i - 1]);
  });

  it("recovers as soon as there is room again", () => {
    const tops = stackIcons([0, 1, 900]);
    expect(tops[2]).toBe(900);
  });

  it("handles an empty document", () => {
    expect(stackIcons([])).toEqual([]);
  });
});

describe("which icons are worth drawing", () => {
  const band = { top: 100, bottom: 400 };

  it("draws what is in the band", () => {
    expect(iconVisible(200, band)).toBe(true);
  });

  it("draws one icon-height either side, so nothing appears mid-scroll", () => {
    expect(iconVisible(band.top - ICON_SIZE, band)).toBe(true);
    expect(iconVisible(band.bottom + ICON_SIZE, band)).toBe(true);
  });

  it("skips what is far away", () => {
    expect(iconVisible(-500, band)).toBe(false);
    expect(iconVisible(5000, band)).toBe(false);
  });
});

describe("where the popover opens", () => {
  const box = { top: 0, height: 600 };

  it("opens level with its icon when there is room", () => {
    expect(popoverTop(200, 320, box)).toBe(200);
  });

  it("slides up only as far as it must to fit", () => {
    // 600 - 320 - 8 = 272 is the lowest top that still shows the buttons.
    expect(popoverTop(500, 320, box)).toBe(272);
  });

  it("never rises above the top of the editor", () => {
    expect(popoverTop(-100, 320, box)).toBe(8);
  });

  it("pins to the top when it is taller than the editor, and scrolls inside", () => {
    expect(popoverTop(300, 900, box)).toBe(8);
  });
});
