<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/monitorDock.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Monitor Never Steals Focus](../../../../../contract.md)

Captured SHA-256: `dd0557d0dd6eda5f1c07f1639b75a01fee6172dec23a3f6c4103dcbbe4525356`.

<book:entry path="apps/web/src/lib/monitorDock.test.ts">
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
</book:entry>

</book:doc>
