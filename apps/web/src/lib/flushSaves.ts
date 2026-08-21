// "Save All", for panes that save themselves.
//
// A document has a session the workspace can reach and tell to save. A plain
// file does not: it owns its own debounced saver, and only it knows whether
// anything is pending. Rather than lifting that state up — which would put a
// re-render on the typing path to power a menu item — the workspace announces
// the intent and each pane answers for its own buffer.
//
// A window event rather than a context, for the same reason the native menu
// bridge is one: nothing that listens here needs to be under any particular
// provider, and a pane that is not mounted simply does not answer.

export const FLUSH_SAVES_EVENT = "hickory:flush-saves";

/** Ask every self-saving pane to write whatever it is holding, now. */
export function requestFlushSaves(): void {
  window.dispatchEvent(new CustomEvent(FLUSH_SAVES_EVENT));
}

/** Answer the request. Returns the unsubscribe. */
export function onFlushSaves(handler: () => void): () => void {
  window.addEventListener(FLUSH_SAVES_EVENT, handler);
  return () => window.removeEventListener(FLUSH_SAVES_EVENT, handler);
}
