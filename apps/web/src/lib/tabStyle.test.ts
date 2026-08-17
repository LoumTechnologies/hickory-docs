import { describe, expect, it } from "vitest";

import { loadTabStyle, saveTabStyle, TAB_STYLE_KEY } from "./tabStyle";

describe("tab style persistence", () => {
  it("defaults to top with nothing stored", () => {
    localStorage.removeItem(TAB_STYLE_KEY);
    expect(loadTabStyle()).toBe("top");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveTabStyle("side");
    expect(localStorage.getItem(TAB_STYLE_KEY)).toBe("side");
    expect(loadTabStyle()).toBe("side");
    saveTabStyle("top");
    expect(loadTabStyle()).toBe("top");
  });

  it("falls back to top on a value nothing ever wrote", () => {
    localStorage.setItem(TAB_STYLE_KEY, "diagonal");
    expect(loadTabStyle()).toBe("top");
  });

  it("works without any storage at all", () => {
    expect(loadTabStyle(null)).toBe("top");
    expect(() => saveTabStyle("side", null)).not.toThrow();
  });
});
