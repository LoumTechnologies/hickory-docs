// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { cleanup, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api, ApiError } from "../api/client";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { useTerminalActions } from "./uiTerminalActions";
import type { Terminals } from "./useTerminals";

afterEach(() => { cleanup(); vi.restoreAllMocks(); });
function fixture(prompt: unknown = null) {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const controller = { sessions: [{ id: "terminal-1", title: "setup", prompt }], attention: [], anchors: {}, turbo: false, error: null,
    open: vi.fn().mockResolvedValue({ id: "real-job" }), close: vi.fn(), answer: vi.fn(), interrupt: vi.fn(), setTurbo: vi.fn(), anchor: vi.fn(), unanchor: vi.fn(), resumeAnchor: vi.fn(), refresh: vi.fn(),
  } as unknown as Terminals;
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  renderHook(() => useTerminalActions(controller), { wrapper });
  let n = 0;
  const invoke = (label: string, arguments_ = {}, human = false) => {
    const action = registry.observe(registry.groups().groups[0]).actions.find(item => item.label === label)!;
    return registry.act(registry.scope, { id: human ? "human" : "python:turn", kind: human ? "human" : "agent" }, { handle: action.handle, arguments: arguments_, request_id: String(++n) });
  };
  return { controller, invoke };
}
it("opens through the shared terminal controller and reports its real job", async () => {
  const f = fixture();
  expect(await f.invoke("Open host terminal", { cwd: ".", argv: ["node", "--version"], title: "version", monitor: false, worktree_branch: "" })).toMatchObject({ status: "result", outcome: { status: "queued", job: "real-job" } });
  expect(f.controller.open).toHaveBeenCalledWith({ cwd: ".", argv: ["node", "--version"], title: "version", monitor: false, worktree_branch: undefined });
});
it("preserves a current host prompt refusal even before the next UI poll", async () => {
  const input = vi.spyOn(api, "terminalInput").mockRejectedValue(new ApiError(409, "Current host prompt requires a human"));
  const f = fixture(null);
  expect(await f.invoke("Input terminal bytes: setup", { data: "yes\n" })).toMatchObject({ status: "result", outcome: { status: "requires_input", reason: "Current host prompt requires a human" } });
  expect(input).toHaveBeenCalledWith("terminal-1", "yes\n", true);
  expect(f.controller.answer).not.toHaveBeenCalled(); expect(f.controller.refresh).toHaveBeenCalled();
});
it("requires actual human input for opaque prompts and enabling automatic answers", async () => {
  const input = vi.spyOn(api, "terminalInput").mockResolvedValue(undefined);
  const f = fixture({ text: "Approve?", choices: [] });
  for (const [label, args] of [["Answer terminal prompt: setup", { send: "yes" }], ["Input terminal bytes: setup", { data: "yes\n" }], ["Set terminal automatic-answer mode", { enabled: true }]] as const) {
    expect(await f.invoke(label, args)).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  }
  expect(f.controller.answer).not.toHaveBeenCalled(); expect(input).not.toHaveBeenCalled(); expect(f.controller.setTurbo).not.toHaveBeenCalled();
  expect(await f.invoke("Answer terminal prompt: setup", { send: "yes" }, true)).toMatchObject({ status: "result", outcome: { status: "completed" } });
  expect(f.controller.answer).toHaveBeenCalledWith("terminal-1", "yes");
});
