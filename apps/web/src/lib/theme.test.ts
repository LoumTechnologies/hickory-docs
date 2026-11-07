import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";

import { applyStoredTheme, applyTheme, loadTheme, saveTheme, THEME_KEY } from "./theme";

afterEach(() => {
  vi.unstubAllGlobals();
  localStorage.removeItem(THEME_KEY);
  delete document.documentElement.dataset.theme;
  document.querySelector('meta[name="color-scheme"]')?.remove();
});

describe("theme persistence", () => {
  it.each([true, false])("uses system light preference %s without saving it", (matches) => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({ matches })));
    expect(loadTheme()).toBe(matches ? "light" : "dark");
    expect(localStorage.getItem(THEME_KEY)).toBeNull();
  });

  it("defaults to dark when system preference is unavailable", () => {
    localStorage.removeItem(THEME_KEY);
    expect(loadTheme()).toBe("dark");
  });

  it("round-trips each saved choice over the opposite system preference", () => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({ matches: true })));
    for (const theme of ["light", "warm-dark", "dark", "fern", "forest"] as const) {
      saveTheme(theme);
      expect(localStorage.getItem(THEME_KEY)).toBe(theme);
      expect(loadTheme()).toBe(theme);
    }
  });

  it("uses the system on a value nothing ever wrote", () => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({ matches: true })));
    localStorage.setItem(THEME_KEY, "solarized");
    expect(loadTheme()).toBe("light");
  });

  it("uses the system when storage throws", () => {
    vi.stubGlobal("matchMedia", vi.fn(() => ({ matches: true })));
    expect(loadTheme({ getItem() { throw new Error("disabled"); } })).toBe("light");
  });

  it("works without any storage at all", () => {
    expect(loadTheme(null)).toBe("dark");
    expect(() => saveTheme("light", null)).not.toThrow();
  });
});

describe("applying a theme to the document", () => {
  it("tracks system changes until an explicit choice is saved", () => {
    let matches = true;
    let change = () => {};
    const addEventListener = vi.fn((_event, listener) => { change = listener; });
    vi.stubGlobal("matchMedia", vi.fn(() => ({ matches, addEventListener })));
    const doc = document.implementation.createHTMLDocument();
    expect(applyStoredTheme(doc)).toBe("light");
    applyStoredTheme(doc);
    expect(addEventListener).toHaveBeenCalledTimes(1);
    matches = false;
    change();
    expect(doc.documentElement.dataset.theme).toBe("dark");
    saveTheme("warm-dark");
    matches = true;
    change();
    expect(doc.documentElement.dataset.theme).toBe("warm-dark");
  });

  it.each(["index.html", "site.html"])("resolves %s before first paint", (file) => {
    const html = new TextDecoder().decode(readFileSync(file));
    const script = html.match(/<script>([\s\S]*?)<\/script>/)![1];
    for (const matches of [true, false]) {
      vi.stubGlobal("matchMedia", vi.fn(() => ({ matches })));
      for (const stored of [null, "invalid", "dark", "warm-dark", "light", "fern", "forest"]) {
        if (stored === null) localStorage.removeItem(THEME_KEY);
        else localStorage.setItem(THEME_KEY, stored);
        const doc = document.implementation.createHTMLDocument();
        doc.head.innerHTML = '<meta name="color-scheme" content="dark">';
        new Function("document", script)(doc);
        expect(doc.documentElement.dataset.theme).toBe(loadTheme());
        expect(doc.querySelector("meta")!.content).toBe(["light", "fern"].includes(loadTheme()) ? "light" : "dark");
      }
    }
  });

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
