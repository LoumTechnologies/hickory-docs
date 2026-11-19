import { viewBuffers } from "./viewBuffers";
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

/** Flush ordinary saves and include detached drafts for explicit Save All. */
export function requestFlushSaves(includeDetached = false): void {
  window.dispatchEvent(new CustomEvent(FLUSH_SAVES_EVENT, {detail: includeDetached}));
  viewBuffers.all().filter(buffer => buffer.dirty).forEach(buffer => { void buffer.save(); });
}

/** Answer the request. Returns the unsubscribe. */
export function onFlushSaves(handler: (includeDetached: boolean) => void): () => void {
  const flush = (event: Event) => handler((event as CustomEvent<unknown>).detail === true);
  window.addEventListener(FLUSH_SAVES_EVENT, flush);
  return () => window.removeEventListener(FLUSH_SAVES_EVENT, flush);
}
