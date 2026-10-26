<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/MoreActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-590ab520e87f"></a>
## `apps/web/src/components/MoreActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-590ab520e87f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-590ab520e87f-expression_statement">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";
import { MoreActions } from "./MoreActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-590ab520e87f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-590ab520e87f-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps the clicked controller available across delayed intent persistence before closing its disclosure", async () => {
  let release!: () => void;
  const effect = vi.fn();
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async receipt => {
    if (receipt.stage === "requested" && receipt.action.label === "Run owned action") await new Promise<void>(resolve => { release = resolve; });
  });
  function Owned() {
    const invoke = useUiCommands([{ key: "owned", command: { parent: "owned", subject: "owned", label: "Run owned action", parameters: {}, prerequisites: [], read: () => ({ revision: "current", enabled: true }), execute: async () => { effect(); return { status: "completed", value: { invoked: true } }; } } }]);
    return <button onClick={() => void invoke("owned")}>Run owned action</button>;
  }
  render(<UiActionContext.Provider value={registry}><MoreActions label="Options"><Owned /></MoreActions></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  expect(actions().some(a => a.label === "Run owned action")).toBe(false);
  await act(async () => { fireEvent.click(screen.getByLabelText("Options")); });
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Run owned action" })); });
  expect(release).toBeTypeOf("function");
  expect(effect).not.toHaveBeenCalled();
  expect(document.querySelector<HTMLDetailsElement>(".more-actions")!.open).toBe(true);
  await act(async () => { release(); });
  await waitFor(() => expect(effect).toHaveBeenCalledOnce());
  expect(document.querySelector<HTMLDetailsElement>(".more-actions")!.open).toBe(false);
  expect(actions().some(a => a.label === "Run owned action")).toBe(false);
});

</storycode:copy>
<a id="f-590ab520e87f-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-590ab520e87f-expression_statement-3">
it("does not let an old action completion close a newly reopened menu", async () => {
  let finish!: () => void;
  const started = vi.fn();
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  function Owned() {
    const invoke = useUiCommands([{ key: "job", command: { parent: "job", subject: "job", label: "Start owned job", parameters: {}, prerequisites: [], read: () => ({ revision: "current", enabled: true }), execute: async () => { started(); await new Promise<void>(resolve => { finish = resolve; }); return { status: "completed", value: { finished: true } }; } } }]);
    return <button onClick={() => void invoke("job")}>Start owned job</button>;
  }
  render(<UiActionContext.Provider value={registry}><MoreActions label="Options"><Owned /></MoreActions></UiActionContext.Provider>);
  await act(async () => { fireEvent.click(screen.getByLabelText("Options")); });
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Start owned job" })); });
  expect(started).toHaveBeenCalledOnce();
  await act(async () => { fireEvent.click(screen.getByLabelText("Options")); });
  expect(document.querySelector<HTMLDetailsElement>(".more-actions")!.open).toBe(false);
  await act(async () => { fireEvent.click(screen.getByLabelText("Options")); });
  await act(async () => { finish(); });
  expect(document.querySelector<HTMLDetailsElement>(".more-actions")!.open).toBe(true);
});
</storycode:copy>

### Assemble `apps/web/src/components/MoreActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/MoreActions.test.tsx"><storycode:paste select="#f-590ab520e87f-expression_statement"/><storycode:paste select="#f-590ab520e87f-expression_statement-2"/><storycode:paste select="#f-590ab520e87f-expression_statement-3"/></storycode:file>

</storycode:doc>
