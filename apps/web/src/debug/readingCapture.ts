import type { ObservationReading } from "./observations";
const readings = new Map<symbol, ObservationReading>();
const listeners = new Set<() => void>();
/** Trusted debugger adapter capture; never populated by source text or DOM events. */
export function retainDebugReading(owner: symbol, reading: ObservationReading | null) {
  if (reading) readings.set(owner, reading); else readings.delete(owner);
  for (const listener of listeners) listener();
}
export function capturedDebugReadings() { return [...readings.values()]; }
export function onDebugReading(listener: () => void) { listeners.add(listener); return () => { listeners.delete(listener); }; }
