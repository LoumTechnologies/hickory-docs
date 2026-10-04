import { useSyncExternalStore } from "react";

export interface ReadingView {
  title: string;
  path: string;
  before: string;
  after: string;
  status: string;
  decide?: (accepted: boolean) => Promise<void>;
}
export const OPEN_READING = "hickory.open-reading";
const readings = new Map<string, ReadingView>();
const listeners = new Set<() => void>();
function changed() { listeners.forEach(listener => listener()); }
export function publishReading(id: string, view: ReadingView) { readings.set(id, view); changed(); }
export function forgetReading(id: string) { readings.delete(id); changed(); }
export function openReading(id: string, title: string) {
  window.dispatchEvent(new CustomEvent(OPEN_READING, { detail: { id, title } }));
}
export function openCommit(sha: string) { openReading(`commit:${sha}`, "Commit"); }
export function useReading(id: string) {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => listeners.delete(listener); }, () => readings.get(id));
}
