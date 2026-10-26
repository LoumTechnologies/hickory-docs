<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/useShellKeys.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-0a350bbbd179"></a>
## `apps/web/src/shell/useShellKeys.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Spatial Views Preserve The Editor](../../../../../contract.md).

<a id="f-0a350bbbd179-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-0a350bbbd179-expression_statement">
// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useShellKeys } from "./ShellView";
import { freeform, open, tab, type Layout } from "./layout";
afterEach(cleanup);
</storycode:copy>
<a id="f-0a350bbbd179-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-0a350bbbd179-expression_statement-2">
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
</storycode:copy>

### Assemble `apps/web/src/shell/useShellKeys.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/useShellKeys.test.tsx"><storycode:paste select="#f-0a350bbbd179-expression_statement"/><storycode:paste select="#f-0a350bbbd179-expression_statement-2"/></storycode:file>

</storycode:doc>
