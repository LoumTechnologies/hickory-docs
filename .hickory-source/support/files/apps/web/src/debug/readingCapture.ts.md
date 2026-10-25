<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/readingCapture.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `cd16e1181d7e86884706a656cc731d781214d5973db1a01d71646d7677ea8fba`.

<book:entry path="apps/web/src/debug/readingCapture.ts">
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
</book:entry>

</book:doc>
