import { describe, expect, it } from "vitest";

// Protects docs/guarantees/authoring/the-gutters-never-skip-a-number.md
import { ICON_GAP, ICON_SIZE, iconVisible, popoverTop, groupRailIcons, gutterSlots } from "./cardRail";

const PITCH = ICON_SIZE + ICON_GAP;

// Contract: .hickory-source/guarantees/authoring/the-editor-keeps-its-space/contract.md
describe("grouping the rail’s icons", () => {
  it("uses new columns and free rows before reserving an overflow slot", () => {
    // Contract: .hickory-source/frontends/contract.md
    expect(gutterSlots(6, 40, 20, 3)).toEqual({ visible: 6, overflow: 0 });
    expect(gutterSlots(7, 40, 20, 3)).toEqual({ visible: 5, overflow: 2 });
    expect(gutterSlots(3, 20, 20, 3)).toEqual({ visible: 3, overflow: 0 });
    expect(gutterSlots(3, 40, 20, 1)).toEqual({ visible: 1, overflow: 2 });
  });
  it("leaves separated icons at their source rows", () => {
    expect(groupRailIcons([0, 100, 300], 600).tops).toEqual([0, 100, 300]);
  });
  it("reserves overflow without displacing the following row", () => {
    expect(groupRailIcons([100, 100, 100, 144], 600)).toEqual({
      tops: [100, null, null, 144], overflow: [{ top: 122, indices: [1, 2] }],
    });
  });
  it("keeps adjacent row centers and puts each excess group in its own grid", () => {
    expect(groupRailIcons([100, 100, 120, 120], 142)).toEqual({
      tops: [null, null, null, null], overflow: [{ top: 100, indices: [0, 1] }, { top: 120, indices: [2, 3] }],
    });
  });
  it("uses spare room below the last populated row and handles empty input", () => {
    expect(groupRailIcons([0, 0, 0], PITCH * 3)).toEqual({ tops: [0, PITCH, PITCH * 2], overflow: [] });
    expect(groupRailIcons([], 600)).toEqual({ tops: [], overflow: [] });
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
