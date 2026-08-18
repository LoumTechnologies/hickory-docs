import { describe, expect, it } from "vitest";

import type { TerminalSession } from "../api/types";
import { dockTone, monitors } from "./monitorDock";

function session(over: Partial<TerminalSession>): TerminalSession {
  return {
    id: "t1",
    title: "task",
    cwd: "/w",
    monitor: false,
    state: "working",
    since_ms: 0,
    branch: null,
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
    ...over,
  };
}

describe("the monitor dock", () => {
  it("holds only the monitors", () => {
    const all = [
      session({ id: "task" }),
      session({ id: "server", monitor: true }),
    ];
    expect(monitors(all).map((s) => s.id)).toEqual(["server"]);
  });

  it("stays quiet while its processes are running", () => {
    expect(dockTone([session({ id: "server", monitor: true, state: "working" })])).toBe(
      "quiet",
    );
  });

  it("goes amber when a monitor dies, however it died", () => {
    expect(dockTone([session({ monitor: true, state: "failed" })])).toBe("amber");
    expect(dockTone([session({ monitor: true, state: "finished" })])).toBe("amber");
  });

  it("ignores ordinary sessions — a failed task belongs in the queue, not the dock", () => {
    expect(dockTone([session({ monitor: false, state: "failed" })])).toBe("quiet");
  });
});
