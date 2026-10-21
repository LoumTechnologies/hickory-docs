import { useSyncExternalStore } from "react";

export interface ReadingView {
  title: string;
  path: string;
  before: string;
  after: string;
  status: string;
  editable?: boolean;
  codeUri?: string;
  debugOwner?: string;
  location?: {line:number;character:number};
  dispose?: () => void;
  project?: import("../editor/hickLang").ProjectEntry[];
  decide?: (accepted: boolean, newText?: string) => Promise<void | import("./uiActions").UiOutcome>;
}
export const OPEN_READING = "hickory.open-reading";
const readings = new Map<string, ReadingView>();
const listeners = new Set<() => void>();
let version = 0;
function changed() { version++; listeners.forEach(listener => listener()); }
export const readingStatus = (id: string) => readings.get(id)?.status;
export function useReadingsVersion() {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => listeners.delete(listener); }, () => version);
}
export function publishReading(id: string, view: ReadingView) { const previous = readings.get(id); readings.set(id, view); previous?.dispose?.(); changed(); }
export function forgetReading(id: string) { readings.get(id)?.dispose?.(); readings.delete(id); changed(); }
export function openReading(id: string, title: string, background = false) {
  window.dispatchEvent(new CustomEvent(OPEN_READING, { detail: { id, title, ...(background ? { background: true } : {}) } }));
}
export function openCommit(sha: string) { openReading(`commit:${sha}`, "Commit"); }
export function useReading(id: string) {
  return useSyncExternalStore(listener => { listeners.add(listener); return () => listeners.delete(listener); }, () => readings.get(id));
}
