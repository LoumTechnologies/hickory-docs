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
