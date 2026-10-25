<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/flushSaves.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `f983e73079d2c2ab9f8cbdb1bb9639ee7192438b887292f918aa4beaf7c0ab14`.

<book:entry path="apps/web/src/lib/flushSaves.ts">
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

/** Ask every self-saving pane to write whatever it is holding, now. */
export function requestFlushSaves(): void {
  window.dispatchEvent(new CustomEvent(FLUSH_SAVES_EVENT));
  viewBuffers.all().filter(buffer => buffer.dirty).forEach(buffer => { void buffer.save(); });
}

/** Answer the request. Returns the unsubscribe. */
export function onFlushSaves(handler: () => void): () => void {
  window.addEventListener(FLUSH_SAVES_EVENT, handler);
  return () => window.removeEventListener(FLUSH_SAVES_EVENT, handler);
}
</book:entry>

</book:doc>
