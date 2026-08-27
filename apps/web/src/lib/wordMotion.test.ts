import { describe, expect, it } from "vitest";

import { loadWordMotion, saveWordMotion, WORD_MOTION_KEY } from "./wordMotion";

describe("word motion persistence", () => {
  it("defaults to whole words with nothing stored", () => {
    localStorage.removeItem(WORD_MOTION_KEY);
    expect(loadWordMotion()).toBe("word");
  });

  it("round-trips a saved choice through localStorage", () => {
    saveWordMotion("subword");
    expect(localStorage.getItem(WORD_MOTION_KEY)).toBe("subword");
    expect(loadWordMotion()).toBe("subword");
    saveWordMotion("word");
    expect(loadWordMotion()).toBe("word");
  });

  it("falls back to whole words on a value nothing ever wrote", () => {
    localStorage.setItem(WORD_MOTION_KEY, "camelhumps");
    expect(loadWordMotion()).toBe("word");
  });

  it("works without any storage at all", () => {
    expect(loadWordMotion(null)).toBe("word");
    expect(() => saveWordMotion("subword", null)).not.toThrow();
  });
});
