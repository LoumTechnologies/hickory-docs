import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useWorkspaceMenuActions } from "./uiWorkspaceMenuActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("checks live focused source before dispatch even without a React render and shares human routing", async () => {
  const effects = vi.fn(async () => ({ status: "completed" as const, value: { handler_returned: true } }));
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let focus = "first", source = "PRIVATE INITIAL SOURCE";
  let human!: ReturnType<typeof useWorkspaceMenuActions>;
  function Harness() { human = useWorkspaceMenuActions(() => ({ visible: true, guard: [focus, source], status: { focus } }), effects); return null; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  const old = registry.observe(parent).actions.find(a => a.label === "Save focused buffer")!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain(source);
  focus = "second"; source = "DIFFERENT PRIVATE SOURCE";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(effects).not.toHaveBeenCalled();
  const current = registry.observe(parent).actions.find(a => a.label === "Save focused buffer")!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: current.handle, arguments: {}, request_id: "save" }); });
  expect(effects).toHaveBeenLastCalledWith("save", { id: "python:turn", kind: "agent" });
  await act(async () => { await human("save"); });
  expect(effects).toHaveBeenLastCalledWith("save", { id: "local-user", kind: "human" });
});
