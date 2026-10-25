<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/viewBuffers.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Saved means the displayed bytes reached the file](../../../../../../guarantees/authoring/saved-means-the-displayed-bytes-reached-the-file/contract.md)
- [Unsaved Work Survives Closing The App](../../../../../../guarantees/authoring/unsaved-work-survives-closing-the-app/contract.md)

Captured SHA-256: `5c9aa309e8d994f51ee0ba6b84fa7fa92eaed310a3f96b532457473fdfc75c39`.

<book:entry path="apps/web/src/lib/viewBuffers.ts">
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
const buffers = new Map<string, ViewBuffer>();
const listeners = new Set<() => void>();
let version = 0;
const changed = () => { version++; listeners.forEach(listener => listener()); };
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
export function useViewBuffersVersion() {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => { listeners.delete(listener); }; }, () => version);
}
</book:entry>

</book:doc>
