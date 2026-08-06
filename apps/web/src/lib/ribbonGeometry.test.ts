import { describe, expect, it } from "vitest";
import { bandAround, clampBand, ribbonPath, thicknessFor } from "./ribbonGeometry";

describe("clampBand", () => {
  it("passes a fully visible band through untouched", () => {
    expect(clampBand(120, 140, 100, 500)).toEqual({ yTop: 120, yBot: 140, clamped: false });
  });

  it("clamps a band that pokes above the pane", () => {
    expect(clampBand(80, 140, 100, 500)).toEqual({ yTop: 100, yBot: 140, clamped: true });
  });

  it("clamps a band that pokes below the pane", () => {
    expect(clampBand(480, 560, 100, 500)).toEqual({ yTop: 480, yBot: 500, clamped: true });
  });

  it("collapses a band fully above to a sliver at the top edge", () => {
    expect(clampBand(0, 50, 100, 500)).toEqual({ yTop: 100, yBot: 102, clamped: true });
  });

  it("collapses a band fully below to a sliver at the bottom edge", () => {
    expect(clampBand(600, 700, 100, 500)).toEqual({ yTop: 498, yBot: 500, clamped: true });
  });
});

describe("thicknessFor", () => {
  it("is proportional to the byte share", () => {
    expect(thicknessFor(50, 100, 26)).toBeCloseTo(13);
  });

  it("caps at maxPx and floors at minPx", () => {
    expect(thicknessFor(100, 100, 26)).toBe(26);
    expect(thicknessFor(1, 1_000_000, 26, 3)).toBe(3);
  });

  it("degrades safely on zero totals", () => {
    expect(thicknessFor(10, 0)).toBe(3);
    expect(thicknessFor(0, 10)).toBe(3);
  });
});

describe("bandAround", () => {
  it("centers the thickness on the band midpoint", () => {
    expect(bandAround(100, 120, 10)).toEqual({ yTop: 105, yBot: 115 });
  });
});

describe("ribbonPath", () => {
  it("builds a closed two-bezier ribbon between the two bands", () => {
    const d = ribbonPath(10, 100, 110, 200, 300, 340);
    expect(d.startsWith("M 10 100")).toBe(true);
    expect(d.endsWith("Z")).toBe(true);
    // Horizontal tangents: control points at the midpoint x.
    expect(d).toContain("C 105 100 105 300 200 300");
    expect(d).toContain("L 200 340");
    expect(d).toContain("C 105 340 105 110 10 110");
  });

  it("rounds coordinates to a tenth to keep paths compact", () => {
    const d = ribbonPath(0.123, 1.056, 2.849, 3.301, 4.05, 5.999);
    expect(d).toContain("M 0.1 1.1");
    expect(d).toContain("6");
    expect(d).not.toMatch(/\d\.\d\d/);
  });
});
