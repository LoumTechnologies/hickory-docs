import { describe, expect, it } from "vitest";
import type { TranscriptEvent } from "../api/types";
import {
  exitCodeOf,
  finalSegments,
  segmentsAt,
  stdoutOf,
  transcriptDuration,
} from "./transcript";

const events: TranscriptEvent[] = [
  { t: 0, kind: "cmd", data: "hickory --version" },
  { t: 200, kind: "out", data: "hickory " },
  { t: 300, kind: "out", data: "0.4.2\n" },
  { t: 350, kind: "err", data: "warning: cache cold\n" },
  { t: 400, kind: "exit", code: 0 },
];

describe("transcript timing", () => {
  it("computes duration as the last event time", () => {
    expect(transcriptDuration(events)).toBe(400);
    expect(transcriptDuration([])).toBe(0);
  });

  it("shows nothing before the first event, cmd at t=0", () => {
    expect(segmentsAt(events, -1)).toEqual([]);
    expect(segmentsAt(events, 0)).toEqual([{ kind: "cmd", text: "hickory --version" }]);
  });

  it("reveals output chunks as the playhead advances, merging same-kind runs", () => {
    const at250 = segmentsAt(events, 250);
    expect(at250).toEqual([
      { kind: "cmd", text: "hickory --version" },
      { kind: "out", text: "hickory " },
    ]);
    const at300 = segmentsAt(events, 300);
    expect(at300[1]).toEqual({ kind: "out", text: "hickory 0.4.2\n" });
  });

  it("keeps err separate from out and includes exit at the end", () => {
    const all = finalSegments(events);
    expect(all.map((s) => s.kind)).toEqual(["cmd", "out", "err", "exit"]);
    expect(all[3].exitCode).toBe(0);
  });

  it("scrubbing is deterministic: same t yields same segments", () => {
    expect(segmentsAt(events, 350)).toEqual(segmentsAt(events, 350));
  });

  it("stdoutOf concatenates only stdout", () => {
    expect(stdoutOf(events)).toBe("hickory 0.4.2\n");
  });

  it("exitCodeOf reports the last exit, or null when still running", () => {
    expect(exitCodeOf(events)).toBe(0);
    expect(exitCodeOf(events.slice(0, 3))).toBeNull();
  });
});
