// The native menu's side of the bargain.
//
// The desktop shell forwards menu picks into the page as a DOM CustomEvent
// named `hickory-menu` (see apps/desktop/src-tauri/src/lib.rs) — a plain
// window event rather than Tauri IPC, so this file has no Tauri dependency
// and the same build runs unchanged in a browser, where the event simply
// never fires.

export type MenuAction = "new" | "save" | "save-as" | "settings" | "files";

export const MENU_EVENT = "hickory-menu";

/** Two arrivals of the same action inside this window are one keypress —
 * the accelerator and a re-dispatched event, not two intents. */
export const DEDUPE_MS = 100;

const ACTIONS: ReadonlySet<string> = new Set([
  "new",
  "save",
  "save-as",
  "settings",
  "files",
] satisfies MenuAction[]);

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
    if (typeof detail !== "string" || !ACTIONS.has(detail)) return;
    const at = now();
    if (detail === lastAction && at - lastAt < DEDUPE_MS) return;
    lastAction = detail;
    lastAt = at;
    handler(detail as MenuAction);
  };

  window.addEventListener(MENU_EVENT, listener);
  return () => window.removeEventListener(MENU_EVENT, listener);
}
