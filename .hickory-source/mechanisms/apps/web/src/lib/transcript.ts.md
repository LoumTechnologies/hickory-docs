<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/transcript.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-3b4cb24d1182"></a>
## `apps/web/src/lib/transcript.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-3b4cb24d1182-stdoutOf"></a>
### `stdoutOf`

<storycode:copy id="f-3b4cb24d1182-stdoutOf">
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

</storycode:copy>
<a id="f-3b4cb24d1182-exitCodeOf"></a>
### `exitCodeOf`

Exit code of the last exit event, or null if the run never exited.

<storycode:copy id="f-3b4cb24d1182-exitCodeOf">
/** Exit code of the last exit event, or null if the run never exited. */
export function exitCodeOf(events: TranscriptEvent[]): number | null {
  for (let i = events.length - 1; i >= 0; i--) {
    const e = events[i];
    if (e.kind === "exit") return e.code;
  }
  return null;
}
</storycode:copy>

### Assemble `apps/web/src/lib/transcript.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/transcript.ts"><storycode:paste select="#f-3b4cb24d1182-stdoutOf"/><storycode:paste select="#f-3b4cb24d1182-exitCodeOf"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-846652af15cf"></a>
## `apps/web/src/lib/transcript.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-846652af15cf-events"></a>
### `events`

<storycode:copy id="f-846652af15cf-events">
import { describe, expect, it } from "vitest";
import type { TranscriptEvent } from "../api/types";
import { exitCodeOf, stdoutOf } from "./transcript";

const events: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "hick --version" },
  { t: 200, kind: "out", data: "hick " },
  { t: 300, kind: "out", data: "0.4.2\n" },
  { t: 350, kind: "err", data: "warning: cache cold\n" },
  { t: 400, kind: "exit", code: 0 },
];

</storycode:copy>
<a id="f-846652af15cf-expression_statement"></a>
### `expression_statement`

The playback-timing tests that used to live here went with `segmentsAt`
and friends: an emulator applies escape sequences itself, so nothing needs
to reconstruct a screen from timed segments any more. What a transcript
looks like as terminal bytes is `watchStream.test.ts`.

<storycode:copy id="f-846652af15cf-expression_statement">
// The playback-timing tests that used to live here went with `segmentsAt`
// and friends: an emulator applies escape sequences itself, so nothing needs
// to reconstruct a screen from timed segments any more. What a transcript
// looks like as terminal bytes is `watchStream.test.ts`.
describe("reading a transcript", () => {
  it("stdoutOf concatenates only stdout", () => {
    expect(stdoutOf(events)).toBe("hick 0.4.2\n");
  });

  it("exitCodeOf reports the last exit, or null when still running", () => {
    expect(exitCodeOf(events)).toBe(0);
    expect(exitCodeOf(events.slice(0, 3))).toBeNull();
  });
});
</storycode:copy>

### Assemble `apps/web/src/lib/transcript.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/transcript.test.ts"><storycode:paste select="#f-846652af15cf-events"/><storycode:paste select="#f-846652af15cf-expression_statement"/></storycode:file>


</storycode:doc>
