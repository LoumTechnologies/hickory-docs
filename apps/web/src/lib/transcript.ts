import type { TranscriptEvent } from "../api/types";

// Reading a transcript's event stream for facts about the run.
//
// Turning the same events into a *screen* is `watchStream.ts`, and it is a
// different job: this file answers questions about what happened, that one
// produces the bytes a terminal was sent. The playback-timing helpers that
// used to live here (`segmentsAt`, `finalSegments`, `transcriptDuration`)
// went with the `<pre>`-based transcript card they existed to drive — an
// emulator applies the escape sequences those functions had to sidestep.

/** Concatenated stdout of the transcript (used e.g. to detect SVG figures). */
export function stdoutOf(events: TranscriptEvent[]): string {
  let out = "";
  for (const e of events) if (e.kind === "out") out += e.data;
  return out;
}

/** Exit code of the last exit event, or null if the run never exited. */
export function exitCodeOf(events: TranscriptEvent[]): number | null {
  for (let i = events.length - 1; i >= 0; i--) {
    const e = events[i];
    if (e.kind === "exit") return e.code;
  }
  return null;
}
