<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/theme.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-952a43adabdb"></a>
## `apps/web/src/lib/theme.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-952a43adabdb-Theme"></a>
### `Theme`

The UI theme: which of the three token sets in styles.css dresses the app.
A presentation preference, so it persists per browser (localStorage), with
the operating system’s light/dark preference used when unset or unrecognisable.
Applied as `data-theme` on <html>. An inline script in index.html applies
the stored value BEFORE the bundle loads, so the first paint is already the
right theme; this module is the same logic for everything after that
(initial React mount, and re-application when the settings picker saves).

<storycode:copy id="f-952a43adabdb-Theme">
// The UI theme: which of the three token sets in styles.css dresses the app.
// A presentation preference, so it persists per browser (localStorage), with
// the operating system’s light/dark preference used when unset or unrecognisable.
//
// Applied as `data-theme` on <html>. An inline script in index.html applies
// the stored value BEFORE the bundle loads, so the first paint is already the
// right theme; this module is the same logic for everything after that
// (initial React mount, and re-application when the settings picker saves).

export type Theme = "dark" | "warm-dark" | "light";

</storycode:copy>
<a id="f-952a43adabdb-THEME_KEY"></a>
### `THEME_KEY`

<storycode:copy id="f-952a43adabdb-THEME_KEY">
export const THEME_KEY = "hickory.theme";

</storycode:copy>
<a id="f-952a43adabdb-THEMES"></a>
### `THEMES`

<storycode:copy id="f-952a43adabdb-THEMES">
export const THEMES: readonly Theme[] = ["dark", "warm-dark", "light"];

</storycode:copy>
<a id="f-952a43adabdb-isTheme"></a>
### `isTheme`

<storycode:copy id="f-952a43adabdb-isTheme">
const isTheme = (value: unknown): value is Theme =>
  value === "dark" || value === "warm-dark" || value === "light";

</storycode:copy>
<a id="f-952a43adabdb-loadTheme"></a>
### `loadTheme`

A saved choice overrides the system. Missing, invalid or inaccessible storage
uses the system preference; hosts without matchMedia retain the dark fallback.

<storycode:copy id="f-952a43adabdb-loadTheme">
/** Prefer a saved choice; otherwise follow the operating system. */
export function loadTheme(
  storage?: Pick<Storage, "getItem"> | null,
): Theme {
  try {
    const source = storage === undefined
      ? (typeof localStorage === "undefined" ? null : localStorage)
      : storage;
    const stored = source?.getItem(THEME_KEY);
    if (isTheme(stored)) return stored;
  } catch {
    // Storage can be disabled by browser policy; system appearance still works.
  }
  return typeof window !== "undefined" && window.matchMedia?.("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

</storycode:copy>
<a id="f-952a43adabdb-saveTheme"></a>
### `saveTheme`

<storycode:copy id="f-952a43adabdb-saveTheme">
export function saveTheme(
  theme: Theme,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(THEME_KEY, theme);
}

</storycode:copy>
<a id="f-952a43adabdb-applyTheme"></a>
### `applyTheme`

Dress the document in a theme: `data-theme` on the root element selects the
token block, and the color-scheme meta keeps whatever the browser paints
itself (form controls, scrollbars, the pre-CSS page) on the right side.

<storycode:copy id="f-952a43adabdb-applyTheme">
/**
 * Dress the document in a theme: `data-theme` on the root element selects the
 * token block, and the color-scheme meta keeps whatever the browser paints
 * itself (form controls, scrollbars, the pre-CSS page) on the right side.
 */
export function applyTheme(theme: Theme, doc: Document = document): void {
  doc.documentElement.dataset.theme = theme;
  doc
    .querySelector('meta[name="color-scheme"]')
    ?.setAttribute("content", theme === "light" ? "light" : "dark");
}

</storycode:copy>
<a id="f-952a43adabdb-applyStoredTheme"></a>
### `applyStoredTheme`

Boot applies the resolved theme and listens for system changes. Resolve storage
again on each change so a later explicit settings choice always wins. Track each
document once to avoid duplicate listeners if initialization repeats.

<storycode:copy id="f-952a43adabdb-applyStoredTheme">
const watchedDocuments = new WeakSet<Document>();

/** Apply the preference and keep an unsaved default in sync with the OS. */
export function applyStoredTheme(doc: Document = document): Theme {
  const theme = loadTheme();
  applyTheme(theme, doc);
  if (!watchedDocuments.has(doc) && typeof window !== "undefined" && window.matchMedia) {
    const preference = window.matchMedia("(prefers-color-scheme: light)");
    preference.addEventListener("change", () => applyTheme(loadTheme(), doc));
    watchedDocuments.add(doc);
  }
  return theme;
}
</storycode:copy>

### Assemble `apps/web/src/lib/theme.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/theme.ts"><storycode:paste select="#f-952a43adabdb-Theme"/><storycode:paste select="#f-952a43adabdb-THEME_KEY"/><storycode:paste select="#f-952a43adabdb-THEMES"/><storycode:paste select="#f-952a43adabdb-isTheme"/><storycode:paste select="#f-952a43adabdb-loadTheme"/><storycode:paste select="#f-952a43adabdb-saveTheme"/><storycode:paste select="#f-952a43adabdb-applyTheme"/><storycode:paste select="#f-952a43adabdb-applyStoredTheme"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-23d56ddbc20a"></a>
## `apps/web/src/lib/theme.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-23d56ddbc20a-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-23d56ddbc20a-expression_statement">
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";

import { applyStoredTheme, applyTheme, loadTheme, saveTheme, THEME_KEY } from "./theme";

afterEach(() => {
  vi.unstubAllGlobals();
  localStorage.removeItem(THEME_KEY);
  delete document.documentElement.dataset.theme;
  document.querySelector('meta[name="color-scheme"]')?.remove();
});

</storycode:copy>
<a id="f-23d56ddbc20a-expression_statement-2"></a>
### `expression_statement`

These cases distinguish a system default from an explicit saved override, including
invalid values and unavailable storage. The default must never persist itself.

<storycode:copy id="f-23d56ddbc20a-expression_statement-2">
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
    for (const theme of ["light", "warm-dark", "dark"] as const) {
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

</storycode:copy>
<a id="f-23d56ddbc20a-expression_statement-3"></a>
### `expression_statement`

Drive a system change after boot and after a saved override. Execute both real
HTML startup scripts against the same preference matrix to detect a first-paint
default that disagrees with the bundled resolver. Read the HTML as bytes and
decode UTF-8 explicitly: the web project’s minimal Node declarations expose only
the byte-returning filesystem API, while script extraction needs a string.

<storycode:copy id="f-23d56ddbc20a-expression_statement-3">
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
      for (const stored of [null, "invalid", "dark", "warm-dark", "light"]) {
        if (stored === null) localStorage.removeItem(THEME_KEY);
        else localStorage.setItem(THEME_KEY, stored);
        const doc = document.implementation.createHTMLDocument();
        doc.head.innerHTML = '<meta name="color-scheme" content="dark">';
        new Function("document", script)(doc);
        expect(doc.documentElement.dataset.theme).toBe(loadTheme());
        expect(doc.querySelector("meta")!.content).toBe(loadTheme() === "light" ? "light" : "dark");
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
</storycode:copy>

### Assemble `apps/web/src/lib/theme.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/theme.test.ts"><storycode:paste select="#f-23d56ddbc20a-expression_statement"/><storycode:paste select="#f-23d56ddbc20a-expression_statement-2"/><storycode:paste select="#f-23d56ddbc20a-expression_statement-3"/></storycode:file>


</storycode:doc>
