<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/flushSaves.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-059a72059161"></a>
## `apps/web/src/lib/flushSaves.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-059a72059161-FLUSH_SAVES_EVENT"></a>
### `FLUSH_SAVES_EVENT`

<storycode:copy id="f-059a72059161-FLUSH_SAVES_EVENT">
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

</storycode:copy>
<a id="f-059a72059161-requestFlushSaves"></a>
### `requestFlushSaves`

Ask every self-saving pane to write whatever it is holding, now.

<storycode:copy id="f-059a72059161-requestFlushSaves">
/** Ask every self-saving pane to write whatever it is holding, now. */
export function requestFlushSaves(): void {
  window.dispatchEvent(new CustomEvent(FLUSH_SAVES_EVENT));
  viewBuffers.all().filter(buffer => buffer.dirty).forEach(buffer => { void buffer.save(); });
}

</storycode:copy>
<a id="f-059a72059161-onFlushSaves"></a>
### `onFlushSaves`

Answer the request. Returns the unsubscribe.

<storycode:copy id="f-059a72059161-onFlushSaves">
/** Answer the request. Returns the unsubscribe. */
export function onFlushSaves(handler: () => void): () => void {
  window.addEventListener(FLUSH_SAVES_EVENT, handler);
  return () => window.removeEventListener(FLUSH_SAVES_EVENT, handler);
}
</storycode:copy>

### Assemble `apps/web/src/lib/flushSaves.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/flushSaves.ts"><storycode:paste select="#f-059a72059161-FLUSH_SAVES_EVENT"/><storycode:paste select="#f-059a72059161-requestFlushSaves"/><storycode:paste select="#f-059a72059161-onFlushSaves"/></storycode:file>

</storycode:doc>
