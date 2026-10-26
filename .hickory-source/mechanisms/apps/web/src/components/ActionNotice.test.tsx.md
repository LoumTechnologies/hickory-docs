<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/ActionNotice.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-b3427fb03beb"></a>
## `apps/web/src/components/ActionNotice.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-b3427fb03beb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-b3427fb03beb-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { ActionNotice } from "./ActionNotice";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);

</storycode:copy>
<a id="f-b3427fb03beb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-b3427fb03beb-expression_statement-2">
it("shares recovery callbacks and refuses a stale callback with the same visible label", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const retry = vi.fn(), replacement = vi.fn(), dismiss = vi.fn();
  const mount = (run: () => void) => <UiActionContext.Provider value={registry}><ActionNotice kind="fail"
    retry={{ label: "Retry Save", run }} onDismiss={dismiss}>Save not confirmed for <strong>notes.md</strong>.</ActionNotice></UiActionContext.Provider>;
  const view = render(mount(retry));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const old = actions().find(action => action.label === "Retry Save")!.handle;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old, arguments: {}, request_id: "retry" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { callback_requested: true } } });
  expect(retry).toHaveBeenCalledOnce();
  fireEvent.click(screen.getByRole("button", { name: "Retry Save" }));
  await act(async () => {});
  expect(retry).toHaveBeenCalledTimes(2);
  view.rerender(mount(replacement));
  expect(await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(replacement).not.toHaveBeenCalled();
  await act(async () => { await registry.act(registry.scope, { id: "acp:turn", kind: "agent" }, {
    handle: actions().find(action => action.label === "Dismiss action notice")!.handle, arguments: {}, request_id: "dismiss",
  }); });
  expect(dismiss).toHaveBeenCalledOnce();
});
</storycode:copy>

### Assemble `apps/web/src/components/ActionNotice.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/ActionNotice.test.tsx"><storycode:paste select="#f-b3427fb03beb-expression_statement"/><storycode:paste select="#f-b3427fb03beb-expression_statement-2"/></storycode:file>

</storycode:doc>
