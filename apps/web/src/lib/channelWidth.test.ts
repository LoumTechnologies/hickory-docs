import { describe, expect, it } from "vitest";

import {
  CHANNEL_WIDTH_DEFAULT,
  CHANNEL_WIDTH_KEY,
  CHANNEL_WIDTH_MAX,
  CHANNEL_WIDTH_MIN,
  CHANNEL_WIDTH_PRESETS,
  clampChannelWidth,
  loadChannelWidth,
  saveChannelWidth,
} from "./channelWidth";

describe("clampChannelWidth", () => {
  it("passes a sane width through, rounded to whole pixels", () => {
    expect(clampChannelWidth(48)).toBe(48);
    expect(clampChannelWidth(47.6)).toBe(48);
    expect(clampChannelWidth("72")).toBe(72);
  });

  it("clamps into the sane range", () => {
    expect(clampChannelWidth(0)).toBe(CHANNEL_WIDTH_MIN);
    expect(clampChannelWidth(-40)).toBe(CHANNEL_WIDTH_MIN);
    expect(clampChannelWidth(4000)).toBe(CHANNEL_WIDTH_MAX);
  });

  it("falls back to the default on anything that is not a number", () => {
    expect(clampChannelWidth("sparkles")).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(NaN)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(Infinity)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(undefined)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(clampChannelWidth(null)).toBe(CHANNEL_WIDTH_DEFAULT);
  });

  it("accepts every preset unchanged", () => {
    for (const preset of CHANNEL_WIDTH_PRESETS) {
      expect(clampChannelWidth(preset.px)).toBe(preset.px);
    }
  });
});

describe("channel width persistence", () => {
  it("defaults to 48px with nothing stored", () => {
    localStorage.removeItem(CHANNEL_WIDTH_KEY);
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(CHANNEL_WIDTH_DEFAULT).toBe(48);
  });

  it("round-trips a saved width through localStorage", () => {
    saveChannelWidth(72);
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe("72");
    expect(loadChannelWidth()).toBe(72);
    saveChannelWidth(24);
    expect(loadChannelWidth()).toBe(24);
  });

  it("clamps on the way in AND on the way out", () => {
    saveChannelWidth(4000);
    expect(localStorage.getItem(CHANNEL_WIDTH_KEY)).toBe(String(CHANNEL_WIDTH_MAX));
    localStorage.setItem(CHANNEL_WIDTH_KEY, "3");
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_MIN);
  });

  it("recovers from a value nothing ever wrote", () => {
    localStorage.setItem(CHANNEL_WIDTH_KEY, "wide");
    expect(loadChannelWidth()).toBe(CHANNEL_WIDTH_DEFAULT);
  });

  it("works without any storage at all", () => {
    expect(loadChannelWidth(null)).toBe(CHANNEL_WIDTH_DEFAULT);
    expect(() => saveChannelWidth(48, null)).not.toThrow();
  });
});
