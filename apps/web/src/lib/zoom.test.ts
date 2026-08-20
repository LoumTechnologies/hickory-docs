import { describe, expect, it } from "vitest";
import {
  ZOOM_DEFAULT,
  ZOOM_STEPS,
  applyZoom,
  clampZoom,
  zoomCommandFor,
  zoomIn,
  zoomLabel,
  zoomOut,
} from "./zoom";

describe("the zoom ladder", () => {
  it("has actual size on it", () => {
    // Otherwise "reset" lands between two steps and every later step is off
    // by that amount.
    expect(ZOOM_STEPS).toContain(ZOOM_DEFAULT);
  });

  it("steps up and down without drifting", () => {
    // The reason this is a ladder rather than repeated ×1.1: multiplication
    // lands on 1.0000000002 and the actual-size check quietly stops working.
    let level: number = ZOOM_DEFAULT;
    for (let i = 0; i < 4; i++) level = zoomIn(level);
    for (let i = 0; i < 4; i++) level = zoomOut(level);
    expect(level).toBe(ZOOM_DEFAULT);
  });

  it("stops at the ends rather than wrapping round", () => {
    expect(zoomOut(ZOOM_STEPS[0])).toBe(ZOOM_STEPS[0]);
    expect(zoomIn(ZOOM_STEPS[ZOOM_STEPS.length - 1])).toBe(ZOOM_STEPS[ZOOM_STEPS.length - 1]);
  });

  it("snaps a stored or hand-edited value onto the ladder", () => {
    expect(clampZoom(1.12)).toBe(1.1);
    expect(clampZoom(99)).toBe(3);
    expect(clampZoom(0)).toBe(0.5);
    expect(clampZoom("big")).toBe(ZOOM_DEFAULT);
    expect(clampZoom(undefined)).toBe(ZOOM_DEFAULT);
  });

  it("says a level the way a person reads one", () => {
    expect(zoomLabel(1)).toBe("100%");
    expect(zoomLabel(1.25)).toBe("125%");
    expect(zoomLabel(0.67)).toBe("67%");
  });
});

describe("the zoom keys", () => {
  const press = (key: string, mods: Partial<{ metaKey: boolean; ctrlKey: boolean; altKey: boolean }> = {}) =>
    zoomCommandFor({ key, metaKey: false, ctrlKey: false, altKey: false, ...mods });

  it("needs a command modifier", () => {
    expect(press("=")).toBeNull();
    expect(press("=", { metaKey: true })).toBe("in");
    expect(press("=", { ctrlKey: true })).toBe("in");
  });

  it("accepts every spelling of plus and minus a keyboard produces", () => {
    // `+` is `=` with Shift on a US layout and its own key elsewhere, and
    // engines disagree about the numpad. This is not defensive coding — it is
    // the only way ⌘+ works on a German keyboard.
    for (const key of ["=", "+"]) expect(press(key, { metaKey: true })).toBe("in");
    for (const key of ["-", "_"]) expect(press(key, { metaKey: true })).toBe("out");
  });

  it("reads zero as actual size", () => {
    expect(press("0", { metaKey: true })).toBe("reset");
  });

  it("still recognises the command with Alt held, which only picks the scope", () => {
    expect(press("=", { metaKey: true, altKey: true })).toBe("in");
  });

  it("ignores everything else", () => {
    expect(press("k", { metaKey: true })).toBeNull();
  });
});

describe("applying a command", () => {
  it("resets to actual size from anywhere", () => {
    expect(applyZoom(2.5, "reset")).toBe(ZOOM_DEFAULT);
    expect(applyZoom(0.5, "reset")).toBe(ZOOM_DEFAULT);
  });

  it("moves one rung", () => {
    expect(applyZoom(1, "in")).toBe(1.1);
    expect(applyZoom(1, "out")).toBe(0.9);
  });
});
