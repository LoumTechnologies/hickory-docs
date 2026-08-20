// The native menu's side of the bargain.
//
// The desktop shell forwards menu picks into the page as a DOM CustomEvent
// named `hickory-menu` (see apps/desktop/src-tauri/src/lib.rs) — a plain
// window event rather than Tauri IPC, so this file has no Tauri dependency
// and the same build runs unchanged in a browser, where the event simply
// never fires.

export type MenuAction =
  | "new"
  | "save"
  | "save-as"
  | "save-all"
  | "print"
  // Zoom, at two scopes: the whole window, and the focused tab alone.
  | "zoom-in"
  | "zoom-out"
  | "zoom-reset"
  | "zoom-tab-in"
  | "zoom-tab-out"
  | "zoom-tab-reset"
  | "settings"
  | "files"
  // Terminals: open one, and walk the attention queue.
  | "terminal"
  | "attention"
  /** Open the Insert panel; the suffixed form opens it on one element, which
   * is how every item of the native Insert submenu arrives. */
  | "insert"
  | `insert:${string}`;

export const MENU_EVENT = "hickory-menu";

/** Two arrivals of the same action inside this window are one keypress —
 * the accelerator and a re-dispatched event, not two intents. */
export const DEDUPE_MS = 100;

const ACTIONS: ReadonlySet<string> = new Set([
  "new",
  "save",
  "save-as",
  "save-all",
  "print",
  "zoom-in",
  "zoom-out",
  "zoom-reset",
  "zoom-tab-in",
  "zoom-tab-out",
  "zoom-tab-reset",
  "settings",
  "files",
  "insert",
  "terminal",
  "attention",
]);

/** The element a menu action names, or null when it names none. The id is
 * checked against the catalogue by whoever opens the panel — a newer shell
 * naming an element this build does not have opens the panel, which is a
 * better answer than nothing happening. */
export function insertTarget(action: MenuAction): string | null {
  return action.startsWith("insert:") ? action.slice("insert:".length) : null;
}

/**
 * Subscribe to native menu actions. Returns the unsubscribe.
 *
 * Unknown details are ignored (a newer shell talking to an older page should
 * do nothing, not crash), and the same action arriving twice within
 * {@link DEDUPE_MS} is handled once — the guard against an accelerator that
 * reaches the page by two routes.
 */
export function onMenuAction(
  handler: (action: MenuAction) => void,
  now: () => number = Date.now,
): () => void {
  let lastAction: string | null = null;
  let lastAt = -Infinity;

  const listener = (event: Event) => {
    const detail = (event as CustomEvent).detail;
    if (typeof detail !== "string") return;
    // `insert:<element>` is one action per element of the native Insert
    // submenu; the id is not validated here so a shell that grows an element
    // before the page does still opens the panel.
    if (!ACTIONS.has(detail) && !/^insert:[\w.-]+$/.test(detail)) return;
    const at = now();
    if (detail === lastAction && at - lastAt < DEDUPE_MS) return;
    lastAction = detail;
    lastAt = at;
    handler(detail as MenuAction);
  };

  window.addEventListener(MENU_EVENT, listener);
  return () => window.removeEventListener(MENU_EVENT, listener);
}
