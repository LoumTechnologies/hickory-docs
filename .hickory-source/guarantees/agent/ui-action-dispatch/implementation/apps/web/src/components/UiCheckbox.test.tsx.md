<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/UiCheckbox.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4557771256d8"></a>
## `apps/web/src/components/UiCheckbox.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-4557771256d8-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4557771256d8-expression_statement">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiCheckbox } from "./UiCheckbox";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
</storycode:copy>
<a id="f-4557771256d8-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4557771256d8-expression_statement-2">
it("retires a same-valued setting handle when its owner callback is replaced", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const first = vi.fn(), second = vi.fn();
  const draw = (callback: typeof first, owner = "first") => <UiActionContext.Provider value={registry}><UiCheckbox label="setting" owner={owner} checked={false} onCheckedChange={callback} /></UiActionContext.Provider>;
  const view = render(draw(first));
  const handle = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions).find(action => action.label === "Set setting")!.handle;
  const run = async (target: string) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: target, arguments: { checked: true }, request_id: crypto.randomUUID() }); });
    return result;
  };
  const stale = handle(); view.rerender(draw(second));
  expect(await run(stale)).not.toMatchObject({ status: "result" });
  expect(first).not.toHaveBeenCalled(); expect(second).not.toHaveBeenCalled();
  expect(await run(handle())).toMatchObject({ status: "result", outcome: { value: { checked_requested: true, persisted: false } } });
  expect(second).toHaveBeenCalledExactlyOnceWith(true);
  await act(async () => { fireEvent.click(screen.getByRole("checkbox")); });
  expect(second).toHaveBeenCalledTimes(2);
  const priorOwner = handle(); view.rerender(draw(second, "other"));
  expect(await run(priorOwner)).not.toMatchObject({ status: "result" });
});
</storycode:copy>
<a id="f-4557771256d8-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4557771256d8-expression_statement-3">
it("excludes pending changes, surfaces callback failures and keeps late errors on their owner", async () => {
  let reject!: (error: Error) => void;
  const callback = vi.fn(() => new Promise((_resolve, fail) => { reject = fail; }));
  const replacement = vi.fn();
  const view = render(<UiCheckbox label="setting" checked={false} onCheckedChange={callback} />);
  fireEvent.click(screen.getByRole("checkbox")); fireEvent.click(screen.getByRole("checkbox"));
  expect(callback).toHaveBeenCalledTimes(1);
  expect((screen.getByRole("checkbox") as HTMLInputElement).disabled).toBe(true);
  await act(async () => { reject(Error("actual setting refusal")); });
  expect(screen.getByRole("alert").textContent).toBe("actual setting refusal");
  fireEvent.click(screen.getByRole("checkbox"));
  view.rerender(<UiCheckbox label="setting" checked={false} onCheckedChange={replacement} />);
  expect((screen.getByRole("checkbox") as HTMLInputElement).disabled).toBe(true);
  await act(async () => { reject(Error("retired setting refusal")); });
  expect(screen.queryByRole("alert")).toBeNull();
  expect((screen.getByRole("checkbox") as HTMLInputElement).disabled).toBe(false);
  fireEvent.click(screen.getByRole("checkbox"));
  expect(replacement).toHaveBeenCalledExactlyOnceWith(true);
});
</storycode:copy>

### Assemble `apps/web/src/components/UiCheckbox.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/UiCheckbox.test.tsx"><storycode:paste select="#f-4557771256d8-expression_statement"/><storycode:paste select="#f-4557771256d8-expression_statement-2"/><storycode:paste select="#f-4557771256d8-expression_statement-3"/></storycode:file>

</storycode:doc>
