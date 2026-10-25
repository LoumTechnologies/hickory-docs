<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/transcript.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `c9322e9eeee6afed658a3f1fcd5feeb074087d8bf57f8239225602d2ea85a864`.

<book:entry path="apps/web/src/lib/transcript.test.ts">
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
</book:entry>

</book:doc>
