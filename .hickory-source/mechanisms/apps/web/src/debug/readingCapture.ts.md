<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/readingCapture.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-cfde8c203c89"></a>
## `apps/web/src/debug/readingCapture.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-cfde8c203c89-readings"></a>
### `readings`

<storycode:copy id="f-cfde8c203c89-readings">
import type { ObservationReading } from "./observations";
const readings = new Map<symbol, ObservationReading>();
</storycode:copy>
<a id="f-cfde8c203c89-listeners"></a>
### `listeners`

<storycode:copy id="f-cfde8c203c89-listeners">
const listeners = new Set<() => void>();
</storycode:copy>
<a id="f-cfde8c203c89-retainDebugReading"></a>
### `retainDebugReading`

Trusted debugger adapter capture; never populated by source text or DOM events.

<storycode:copy id="f-cfde8c203c89-retainDebugReading">
/** Trusted debugger adapter capture; never populated by source text or DOM events. */
export function retainDebugReading(owner: symbol, reading: ObservationReading | null) {
  if (reading) readings.set(owner, reading); else readings.delete(owner);
  for (const listener of listeners) listener();
}
</storycode:copy>
<a id="f-cfde8c203c89-capturedDebugReadings"></a>
### `capturedDebugReadings`

<storycode:copy id="f-cfde8c203c89-capturedDebugReadings">
export function capturedDebugReadings() { return [...readings.values()]; }
</storycode:copy>
<a id="f-cfde8c203c89-onDebugReading"></a>
### `onDebugReading`

<storycode:copy id="f-cfde8c203c89-onDebugReading">
export function onDebugReading(listener: () => void) { listeners.add(listener); return () => { listeners.delete(listener); }; }
</storycode:copy>

### Assemble `apps/web/src/debug/readingCapture.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/readingCapture.ts"><storycode:paste select="#f-cfde8c203c89-readings"/><storycode:paste select="#f-cfde8c203c89-listeners"/><storycode:paste select="#f-cfde8c203c89-retainDebugReading"/><storycode:paste select="#f-cfde8c203c89-capturedDebugReadings"/><storycode:paste select="#f-cfde8c203c89-onDebugReading"/></storycode:file>

</storycode:doc>
