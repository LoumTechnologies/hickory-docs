<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/viewBuffers.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7b9e9d92a497"></a>
## `apps/web/src/lib/viewBuffers.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Saved means the displayed bytes reached the file](../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md), [Close or recover a document without losing the wrong draft](../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md).

<a id="f-7b9e9d92a497-ViewBuffer"></a>
### `ViewBuffer`

<storycode:copy id="f-7b9e9d92a497-ViewBuffer">
import { useSyncExternalStore } from "react";

/** Explicitly saved views participate in the same close and Save All flow as
 * files. Entries survive an inactive tab's unmount; the workspace forgets
 * closed tabs. A recovery policy must be declared, never assumed. */
export interface ViewBuffer {
  dirty: boolean;
  revision?: () => unknown;
  isDirty?: () => boolean;
  save: () => Promise<boolean>;
  discard: () => void | Promise<void>;
}
</storycode:copy>
<a id="f-7b9e9d92a497-buffers"></a>
### `buffers`

<storycode:copy id="f-7b9e9d92a497-buffers">
const buffers = new Map<string, ViewBuffer>();
</storycode:copy>
<a id="f-7b9e9d92a497-listeners"></a>
### `listeners`

<storycode:copy id="f-7b9e9d92a497-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-7b9e9d92a497-version"></a>
### `version`

<storycode:copy id="f-7b9e9d92a497-version">
let version = 0;
</storycode:copy>
<a id="f-7b9e9d92a497-changed"></a>
### `changed`

<storycode:copy id="f-7b9e9d92a497-changed">
const changed = () => { version++; listeners.forEach(listener => listener()); };
</storycode:copy>
<a id="f-7b9e9d92a497-viewBuffers"></a>
### `viewBuffers`

<storycode:copy id="f-7b9e9d92a497-viewBuffers">
export const viewBuffers = {
  get: (id: string) => buffers.get(id),
  all: () => [...buffers.values()],
  ids: () => [...buffers.keys()],
  set(id: string, buffer: ViewBuffer) {
    const dirtyChanged = buffers.get(id)?.dirty !== buffer.dirty;
    buffers.set(id, buffer);
    if (dirtyChanged) changed();
  },
  forget(id: string) { if (buffers.delete(id)) changed(); },
};
</storycode:copy>
<a id="f-7b9e9d92a497-useViewBuffersVersion"></a>
### `useViewBuffersVersion`

<storycode:copy id="f-7b9e9d92a497-useViewBuffersVersion">
export function useViewBuffersVersion() {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => { listeners.delete(listener); }; }, () => version);
}
</storycode:copy>

### Assemble `apps/web/src/lib/viewBuffers.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/viewBuffers.ts"><storycode:paste select="#f-7b9e9d92a497-ViewBuffer"/><storycode:paste select="#f-7b9e9d92a497-buffers"/><storycode:paste select="#f-7b9e9d92a497-listeners"/><storycode:paste select="#f-7b9e9d92a497-version"/><storycode:paste select="#f-7b9e9d92a497-changed"/><storycode:paste select="#f-7b9e9d92a497-viewBuffers"/><storycode:paste select="#f-7b9e9d92a497-useViewBuffersVersion"/></storycode:file>

</storycode:doc>
