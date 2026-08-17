import { afterEach, describe, expect, it } from "vitest";

import { applyStoredTheme, applyTheme, loadTheme, saveTheme, THEME_KEY } from "./theme";

afterEach(() => {
  localStorage.removeItem(THEME_KEY);
  delete document.documentElement.dataset.theme;
  document.querySelector('meta[name="color-scheme"]')?.remove();
});

describe("theme persistence", () => {
  it("defaults to dark with nothing stored", () => {
    localStorage.removeItem(THEME_KEY);
    expect(loadTheme()).toBe("dark");
  });

  it("round-trips each saved choice through localStorage", () => {
    for (const theme of ["light", "warm-dark", "dark"] as const) {
      saveTheme(theme);
      expect(localStorage.getItem(THEME_KEY)).toBe(theme);
      expect(loadTheme()).toBe(theme);
    }
  });

  it("falls back to dark on a value nothing ever wrote", () => {
    localStorage.setItem(THEME_KEY, "solarized");
    expect(loadTheme()).toBe("dark");
  });

  it("works without any storage at all", () => {
    expect(loadTheme(null)).toBe("dark");
    expect(() => saveTheme("light", null)).not.toThrow();
  });
});

describe("applying a theme to the document", () => {
  it("sets data-theme on the root element", () => {
    applyTheme("warm-dark");
    expect(document.documentElement.dataset.theme).toBe("warm-dark");
    applyTheme("light");
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("keeps the color-scheme meta on the theme's side of light/dark", () => {
    const meta = document.createElement("meta");
    meta.setAttribute("name", "color-scheme");
    meta.setAttribute("content", "dark");
    document.head.appendChild(meta);

    applyTheme("light");
    expect(meta.getAttribute("content")).toBe("light");
    applyTheme("warm-dark");
    expect(meta.getAttribute("content")).toBe("dark");
    applyTheme("dark");
    expect(meta.getAttribute("content")).toBe("dark");
  });

  it("survives a document with no color-scheme meta", () => {
    expect(() => applyTheme("light")).not.toThrow();
  });

  it("applyStoredTheme applies exactly what loadTheme reads", () => {
    localStorage.setItem(THEME_KEY, "light");
    expect(applyStoredTheme()).toBe("light");
    expect(document.documentElement.dataset.theme).toBe("light");

    localStorage.setItem(THEME_KEY, "garbage");
    expect(applyStoredTheme()).toBe("dark");
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
