<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/terminal/uiTerminalActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-91310c95ec4c"></a>
## `apps/web/src/terminal/uiTerminalActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-91310c95ec4c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-91310c95ec4c-expression_statement">
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
</storycode:copy>
<a id="f-91310c95ec4c-fixture"></a>
### `fixture`

<storycode:copy id="f-91310c95ec4c-fixture">
function fixture(prompt: unknown = null) {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const controller = { sessions: [{ id: "terminal-1", title: "setup", prompt }], attention: [], anchors: {}, turbo: false, error: null,
    open: vi.fn().mockResolvedValue({ id: "real-job" }), close: vi.fn().mockResolvedValue(true), answer: vi.fn().mockResolvedValue(true), interrupt: vi.fn().mockResolvedValue(true), setTurbo: vi.fn().mockResolvedValue(true), anchor: vi.fn().mockResolvedValue(true), unanchor: vi.fn().mockResolvedValue(true), resumeAnchor: vi.fn().mockResolvedValue(true), refresh: vi.fn(),
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
</storycode:copy>
<a id="f-91310c95ec4c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-91310c95ec4c-expression_statement-2">
it("opens through the shared terminal controller and reports its real job", async () => {
  const f = fixture();
  expect(await f.invoke("Open host terminal", { cwd: ".", argv: ["node", "--version"], title: "version", monitor: false, worktree_branch: "" })).toMatchObject({ status: "result", outcome: { status: "queued", job: "real-job" } });
  expect(f.controller.open).toHaveBeenCalledWith({ cwd: ".", argv: ["node", "--version"], title: "version", monitor: false, worktree_branch: undefined });
});
</storycode:copy>
<a id="f-91310c95ec4c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-91310c95ec4c-expression_statement-3">
it("preserves a current host prompt refusal even before the next UI poll", async () => {
  const input = vi.spyOn(api, "terminalInput").mockRejectedValue(new ApiError(409, "Current host prompt requires a human"));
  const f = fixture(null);
  expect(await f.invoke("Input terminal bytes: setup", { data: "yes\n" })).toMatchObject({ status: "result", outcome: { status: "requires_input", reason: "Current host prompt requires a human" } });
  expect(input).toHaveBeenCalledWith("terminal-1", "yes\n", true);
  expect(f.controller.answer).not.toHaveBeenCalled(); expect(f.controller.refresh).toHaveBeenCalled();
});
</storycode:copy>
<a id="f-91310c95ec4c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-91310c95ec4c-expression_statement-4">
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
</storycode:copy>
<a id="f-91310c95ec4c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-91310c95ec4c-expression_statement-5">
it("does not turn a refused host operation into a completed agent action", async () => {
  const f = fixture();
  vi.mocked(f.controller.interrupt).mockResolvedValue(false);
  expect(await f.invoke("Interrupt terminal: setup")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("failed") } });
  expect(f.controller.interrupt).toHaveBeenCalledWith("terminal-1");
});
</storycode:copy>

### Assemble `apps/web/src/terminal/uiTerminalActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/terminal/uiTerminalActions.test.tsx"><storycode:paste select="#f-91310c95ec4c-expression_statement"/><storycode:paste select="#f-91310c95ec4c-fixture"/><storycode:paste select="#f-91310c95ec4c-expression_statement-2"/><storycode:paste select="#f-91310c95ec4c-expression_statement-3"/><storycode:paste select="#f-91310c95ec4c-expression_statement-4"/><storycode:paste select="#f-91310c95ec4c-expression_statement-5"/></storycode:file>

</storycode:doc>
