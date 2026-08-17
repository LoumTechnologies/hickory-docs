// The UI theme: which of the three token sets in styles.css dresses the app.
// A presentation preference, so it persists per browser (localStorage), with
// a hard default of "dark" — the cool editor-grade palette that `:root`
// carries — for anything unset or unrecognisable.
//
// Applied as `data-theme` on <html>. An inline script in index.html applies
// the stored value BEFORE the bundle loads, so the first paint is already the
// right theme; this module is the same logic for everything after that
// (initial React mount, and re-application when the settings picker saves).

export type Theme = "dark" | "warm-dark" | "light";

export const THEME_KEY = "hickory.theme";

export const THEMES: readonly Theme[] = ["dark", "warm-dark", "light"];

const isTheme = (value: unknown): value is Theme =>
  value === "dark" || value === "warm-dark" || value === "light";

/** The persisted theme, defaulting to "dark" on anything unset or invalid. */
export function loadTheme(
  storage: Pick<Storage, "getItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): Theme {
  const stored = storage?.getItem(THEME_KEY);
  return isTheme(stored) ? stored : "dark";
}

export function saveTheme(
  theme: Theme,
  storage: Pick<Storage, "setItem"> | null = typeof localStorage === "undefined" ? null : localStorage,
): void {
  storage?.setItem(THEME_KEY, theme);
}

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

/** Load, apply, done — the one call the app boot and the picker both make. */
export function applyStoredTheme(doc: Document = document): Theme {
  const theme = loadTheme();
  applyTheme(theme, doc);
  return theme;
}
