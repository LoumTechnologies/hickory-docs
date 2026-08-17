import { describe, expect, it } from "vitest";

import { loadRibbonStyle, RIBBON_STYLE_KEY, saveRibbonStyle } from "./ribbonStyle";

describe("ribbon style persistence", () => {
  it("defaults to braces with nothing stored", () => {
    localStorage.removeItem(RIBBON_STYLE_KEY);
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveRibbonStyle("braces");
    expect(localStorage.getItem(RIBBON_STYLE_KEY)).toBe("braces");
    expect(loadRibbonStyle()).toBe("braces");
    // "bands" stays a valid persisted value — only the default changed.
    saveRibbonStyle("bands");
    expect(loadRibbonStyle()).toBe("bands");
  });

  it("falls back to braces on a value nothing ever wrote", () => {
    localStorage.setItem(RIBBON_STYLE_KEY, "sparkles");
    expect(loadRibbonStyle()).toBe("braces");
  });

  it("works without any storage at all", () => {
    expect(loadRibbonStyle(null)).toBe("braces");
    expect(() => saveRibbonStyle("braces", null)).not.toThrow();
  });
});
