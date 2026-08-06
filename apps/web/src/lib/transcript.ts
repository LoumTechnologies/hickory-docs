import type { TranscriptEvent } from "../api/types";

// Pure transcript-playback timing: given TranscriptEvent[] (t in milliseconds
// from run start) and a playhead time, compute what the terminal shows.

export interface TranscriptSegment {
  kind: "cmd" | "out" | "err" | "exit";
  text: string;
  exitCode?: number;
}

/** Total duration of a transcript in ms (time of the last event). */
export function transcriptDuration(events: TranscriptEvent[]): number {
  let max = 0;
  for (const e of events) if (e.t > max) max = e.t;
  return max;
}

/**
 * Segments visible at playhead time `t` (inclusive). Consecutive out/err
 * chunks of the same kind are merged so styling spans whole runs of output.
 */
export function segmentsAt(events: TranscriptEvent[], t: number): TranscriptSegment[] {
  const segments: TranscriptSegment[] = [];
  for (const e of events) {
    if (e.t > t) break;
    if (e.kind === "cmd") {
      segments.push({ kind: "cmd", text: e.data });
    } else if (e.kind === "exit") {
      segments.push({ kind: "exit", text: `exit ${e.code}`, exitCode: e.code });
    } else {
      const last = segments[segments.length - 1];
      if (last && last.kind === e.kind) last.text += e.data;
      else segments.push({ kind: e.kind, text: e.data });
    }
  }
  return segments;
}

/** Full transcript text as it would appear when playback completes. */
export function finalSegments(events: TranscriptEvent[]): TranscriptSegment[] {
  return segmentsAt(events, Infinity);
}

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
