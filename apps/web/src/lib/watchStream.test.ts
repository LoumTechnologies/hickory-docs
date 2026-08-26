import { describe, expect, it } from "vitest";
import { showsCommands, watchBytes } from "./watchStream";
import type { TranscriptEvent } from "../api/types";

const ok = (data: string): TranscriptEvent[] => [
  { t: 0, kind: "cmd", data: "build" },
  { t: 1, kind: "out", data },
  { t: 2, kind: "exit", code: 0 },
];

describe("watchBytes", () => {
  it("passes output through byte-for-byte, escape sequences included", () => {
    // The whole reason the watching binding is an emulator: these bytes must
    // arrive unmangled, because only the parser downstream can tell that
    // `\x1b[32m` is a colour and `\r` is a cursor move.
    const raw = "\x1b[32mPASS\x1b[0m 12 tests\r\x1b[K\x1b[32mPASS\x1b[0m 40 tests\n";
    expect(watchBytes(ok(raw), { showCommands: false })).toBe(raw);
  });

  it("does not tell stdout and stderr apart", () => {
    // Two streams, one screen. The executor already interleaved them by
    // time; recolouring one of them here would say something the run did not.
    const events: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "a" },
      { t: 1, kind: "err", data: "b" },
      { t: 2, kind: "out", data: "c" },
    ];
    expect(watchBytes(events, { showCommands: false })).toBe("abc");
  });

  it("suppresses the prompt for a single command and keeps it for several", () => {
    const one: TranscriptEvent[] = [{ t: 0, kind: "cmd", data: "cat f" }];
    const two: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "printf a > f" },
      { t: 1, kind: "cmd", data: "cat f" },
    ];
    expect(showsCommands(one)).toBe(false);
    expect(showsCommands(two)).toBe(true);
    expect(watchBytes(one, { showCommands: showsCommands(one) })).toBe("");
    const bytes = watchBytes(two, { showCommands: showsCommands(two) });
    expect(bytes).toContain("$ \x1b[22mcat f");
    expect(bytes).toContain("$ \x1b[22mprintf a > f");
  });

  it("reports a non-zero exit and stays silent about a zero one", () => {
    expect(watchBytes(ok("x"), { showCommands: false })).toBe("x");
    const failed: TranscriptEvent[] = [{ t: 0, kind: "exit", code: 2 }];
    expect(watchBytes(failed, { showCommands: false })).toContain("[exit 2]");
  });

  it("resets before every decoration it writes", () => {
    // A command whose output left the terminal bold-red must not bleed into
    // the prompt of the next command, or into our exit line.
    const events: TranscriptEvent[] = [
      { t: 0, kind: "cmd", data: "one" },
      { t: 1, kind: "out", data: "\x1b[1;31mangry" },
      { t: 2, kind: "cmd", data: "two" },
      { t: 3, kind: "exit", code: 1 },
    ];
    const bytes = watchBytes(events, { showCommands: true });
    expect(bytes).toContain("\x1b[0m\x1b[2m$ \x1b[22mtwo");
    expect(bytes).toContain("\x1b[0m\x1b[31m[exit 1]");
  });

  it("emits only the tail from `from`, so a live run appends", () => {
    const events: TranscriptEvent[] = [
      { t: 0, kind: "out", data: "first" },
      { t: 1, kind: "out", data: "second" },
    ];
    expect(watchBytes(events, { showCommands: false }, 1)).toBe("second");
    expect(watchBytes(events, { showCommands: false }, 2)).toBe("");
  });
});
