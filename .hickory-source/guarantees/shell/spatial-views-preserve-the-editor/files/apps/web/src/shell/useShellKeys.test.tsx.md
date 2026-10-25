<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/useShellKeys.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Spatial Views Preserve The Editor](../../../../../contract.md)

Captured SHA-256: `fbde6820770ff6cdaabfc2d2410e93b698c79f0f1549103f50f4151dec2cecf6`.

<book:entry path="apps/web/src/shell/useShellKeys.test.tsx">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useShellKeys } from "./ShellView";
import { freeform, open, tab, type Layout } from "./layout";
afterEach(cleanup);
it("shares actual keyboard split/search/close handlers and preserves the owning unsaved-close outcome", async () => {
  const layout = open(freeform(), tab("document", "owned.md"));
  const onLayout = vi.fn(), search = vi.fn(), close = vi.fn(async () => ({ status: "requires_input" as const, request: "unsaved-owned", reason: "Choose how to retain the unsaved buffer" }));
  const registry = new UiActions({ workspace: "p", window: "keys" }, () => true, async () => {});
  function Harness({ current = layout, hidden = false }: { current?: Layout; hidden?: boolean }) { useShellKeys(current, onLayout, search, close); return <div data-workspace hidden={hidden} />; }
  const mounted = render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:keys", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  await act(async () => fireEvent.keyDown(window, { key: "\\", ctrlKey: true }));
  await waitFor(() => expect(onLayout).toHaveBeenCalledTimes(1));
  expect(onLayout.mock.calls[0][0].root).toMatchObject({ type: "split", direction: "row" });
  await run("Split focused workspace pane", { axis: "column" });
  expect(onLayout.mock.calls[1][0].root).toMatchObject({ type: "split", direction: "column" });
  await run("Open folder search from workspace"); expect(search).toHaveBeenCalledTimes(1);
  await act(async () => fireEvent.keyDown(window, { key: "f", ctrlKey: true, shiftKey: true }));
  await waitFor(() => expect(search).toHaveBeenCalledTimes(2));
  expect(await run("Close focused workspace tab")).toMatchObject({ outcome: { status: "requires_input", request: "unsaved-owned" } });
  await act(async () => fireEvent.keyDown(window, { key: "w", ctrlKey: true }));
  await waitFor(() => expect(close).toHaveBeenCalledTimes(2));
  expect(onLayout).toHaveBeenCalledTimes(2);
  expect(layout.root.type).toBe("pane");
  const old = actions().find(action => action.label === "Split focused workspace pane")!;
  mounted.rerender(<UiActionContext.Provider value={registry}><Harness hidden /></UiActionContext.Provider>);
  await act(async () => fireEvent.keyDown(window, { key: "\\", ctrlKey: true }));
  expect(onLayout).toHaveBeenCalledTimes(2);
  expect(await registry.act(registry.scope, { id: "acp:keys", kind: "agent" }, { handle: old.handle, arguments: { axis: "row" }, request_id: "hidden" })).not.toMatchObject({ outcome: { status: "completed" } });
});
</book:entry>

</book:doc>
