import { describe, expect, it } from "vitest";

import type { SessionState, TerminalSession } from "../api/types";
import { groupSessions, worstState } from "./sessionGroups";

function session(over: Partial<TerminalSession> & { id: string }): TerminalSession {
  return {
    title: over.id,
    cwd: "/w/app",
    monitor: false,
    state: "idle" as SessionState,
    since_ms: 0,
    branch: null,
    dirty: false,
    preview: "",
    prompt: null,
    exit_code: null,
    ...over,
  };
}

describe("worstState — what a folded row still says", () => {
  it("surfaces a blocked agent past everything else", () => {
    expect(
      worstState([
        session({ id: "a", state: "working" }),
        session({ id: "b", state: "needs-you" }),
        session({ id: "c", state: "failed" }),
      ]),
    ).toBe("needs-you");
  });

  it("prefers a failure to a finish, and a finish to work in progress", () => {
    expect(
      worstState([session({ id: "a", state: "finished" }), session({ id: "b", state: "failed" })]),
    ).toBe("failed");
    expect(
      worstState([session({ id: "a", state: "working" }), session({ id: "b", state: "finished" })]),
    ).toBe("finished");
  });

  it("is idle only when nothing is happening at all", () => {
    expect(worstState([session({ id: "a", state: "idle" })])).toBe("idle");
    expect(worstState([])).toBe("idle");
    expect(worstState([session({ id: "a", state: "working" })])).toBe("working");
  });
});

describe("groupSessions", () => {
  it("groups by directory in first-appearance order and names the project", () => {
    const groups = groupSessions([
      session({ id: "a", cwd: "/w/app" }),
      session({ id: "b", cwd: "/w/lib" }),
      session({ id: "c", cwd: "/w/app" }),
    ]);
    expect(groups.map((g) => g.name)).toEqual(["app", "lib"]);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(["a", "c"]);
  });

  it("gives each group the strongest claim under it", () => {
    const groups = groupSessions([
      session({ id: "a", cwd: "/w/app", state: "working" }),
      session({ id: "b", cwd: "/w/app", state: "needs-you" }),
    ]);
    expect(groups[0].state).toBe("needs-you");
  });

  it("leaves monitors out — they belong to the dock", () => {
    const groups = groupSessions([
      session({ id: "server", cwd: "/w/app", monitor: true, state: "working" }),
      session({ id: "task", cwd: "/w/app", state: "idle" }),
    ]);
    expect(groups).toHaveLength(1);
    expect(groups[0].sessions.map((s) => s.id)).toEqual(["task"]);
    expect(groups[0].state).toBe("idle");
  });
});
