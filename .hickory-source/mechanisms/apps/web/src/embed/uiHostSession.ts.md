<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/embed/uiHostSession.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-585a885211aa"></a>
## `apps/web/src/embed/uiHostSession.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An iframe validates its host](../../../../../guarantees/embedding/an-iframe-validates-its-host/contract.md), [Browser-local prediction practice uses the shared gate](../../../../../guarantees/verification/browser-predictions-use-the-shared-gate/contract.md).

<a id="f-585a885211aa-createEmbedUiSession"></a>
### `createEmbedUiSession`

<storycode:copy id="f-585a885211aa-createEmbedUiSession">
import { UiActions, type UiReceipt } from "../lib/uiActions";
import { dispatchUiFrame } from "../lib/uiActionBridge";

/** One parent-owned embedded document, with no native engine or provider authority. */
export function createEmbedUiSession(receipt: (value: UiReceipt) => void) {
  const window = `embed-${crypto.randomUUID()}`;
  const registry = new UiActions({ workspace: window, window }, () => true, async value => { receipt(value); });
  let disposed = false;
  return {
    registry,
    scope: registry.scope,
    async invoke(value: unknown) {
      if (disposed) throw Error("Embedded document scope is disposed");
      if (!value || typeof value !== "object" || Array.isArray(value)) throw Error("Expected structured embedded UI request");
      const data = value as Record<string, unknown>;
      if (Object.keys(data).some(key => !["window", "method", "arguments"].includes(key)) || data.window !== window) throw Error("Embedded UI request belongs to another document scope; use the latest loaded scope");
      if (typeof data.method !== "string" || !["ui_observe", "ui_act", "ui_cancel"].includes(data.method) || !data.arguments || typeof data.arguments !== "object" || Array.isArray(data.arguments)) throw Error("Invalid embedded UI method or arguments");
      return dispatchUiFrame(registry, { id: "embed-parent", actor: { id: "embed-parent", kind: "agent" }, method: data.method as string, arguments: data.arguments as Record<string, import("../lib/uiActions").UiValue> });
    },
    dispose() { disposed = true; registry.dispose(); },
  };
}
</storycode:copy>

### Assemble `apps/web/src/embed/uiHostSession.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/uiHostSession.ts"><storycode:paste select="#f-585a885211aa-createEmbedUiSession"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-5f023c549320"></a>
## `apps/web/src/embed/uiHostSession.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [An embed has no implicit host](../../../../../guarantees/embedding/an-embed-has-no-implicit-host/contract.md), [An iframe validates its host](../../../../../guarantees/embedding/an-iframe-validates-its-host/contract.md).

<a id="f-5f023c549320-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5f023c549320-expression_statement">
// Guarantees: docs/guarantees/embedding/an-iframe-validates-its-host.md
// docs/guarantees/embedding/an-embed-has-no-implicit-host.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createEmbedUiSession } from "./uiHostSession";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditingSurface } from "./EditingSurface";
import type { UiReceipt } from "../lib/uiActions";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-5f023c549320-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5f023c549320-expression_statement-2">
it("uses structured parent requests for the actual editor without an implicit engine", async () => {
  const fetch = vi.spyOn(globalThis, "fetch").mockRejectedValue(Error("No implicit engine"));
  const receipts: UiReceipt[] = [];
  const session = createEmbedUiSession(value => receipts.push(value)), changed = vi.fn();
  render(<UiActionContext.Provider value={session.registry}><EditingSurface value="captured source" ariaLabel="note.md" onChange={changed} /></UiActionContext.Provider>);
  const observe = await session.invoke({ window: session.scope.window, method: "ui_observe", arguments: { parent: "root" } }) as { groups: string[] };
  const actions = observe.groups.flatMap(parent => session.registry.observe(parent, 64).actions);
  const replace = actions.find(action => action.label === "Replace embedded document range")!;
  const call = { window: session.scope.window, method: "ui_act", arguments: { handle: replace.handle, arguments: { from: 0, to: 8, text: "edited" }, request_id: "edit-1" } };
  await act(async () => { expect(await session.invoke(call)).toMatchObject({ status: "result", outcome: { value: { draft_changed: true, saved: false } } }); });
  expect(changed).toHaveBeenCalledExactlyOnceWith("edited source");
  expect(await session.invoke(call)).toMatchObject({ status: "result" });
  expect(changed).toHaveBeenCalledTimes(1);
  expect(receipts.map(receipt => [receipt.stage, receipt.actor.kind, receipt.actor.id])).toEqual([["requested", "agent", "embed-parent"], ["result", "agent", "embed-parent"]]);
  expect(fetch).not.toHaveBeenCalled();
  session.dispose();
});
</storycode:copy>
<a id="f-5f023c549320-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5f023c549320-expression_statement-3">
it("refuses replaced scopes, actor injection, text commands and disposed sessions", async () => {
  const old = createEmbedUiSession(() => {}), current = createEmbedUiSession(() => {});
  await expect(current.invoke({ window: old.scope.window, method: "ui_observe", arguments: {} })).rejects.toThrow("another document scope");
  await expect(current.invoke({ window: current.scope.window, method: "ui_observe", arguments: {}, actor: { kind: "human" } })).rejects.toThrow("another document scope");
  await expect(current.invoke("ui_act action:quoted")).rejects.toThrow("structured");
  await expect(current.invoke({ window: current.scope.window, method: "eval", arguments: {} })).rejects.toThrow("Invalid");
  old.dispose();
  await expect(old.invoke({ window: old.scope.window, method: "ui_observe", arguments: {} })).rejects.toThrow("disposed");
  current.dispose();
});
</storycode:copy>
<a id="f-5f023c549320-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5f023c549320-expression_statement-4">
it("preserves the actual read-only editor policy for parent requests", async () => {
  const session = createEmbedUiSession(() => {}), changed = vi.fn();
  render(<UiActionContext.Provider value={session.registry}><EditingSurface value="historical" ariaLabel="captured.md" onChange={changed} readOnly /></UiActionContext.Provider>);
  const actions = session.registry.groups().groups.flatMap(parent => session.registry.observe(parent, 64).actions);
  const replace = actions.find(action => action.label === "Replace embedded document range")!;
  await act(async () => { expect(await session.invoke({ window: session.scope.window, method: "ui_act", arguments: { handle: replace.handle, arguments: { from: 0, to: 1, text: "x" }, request_id: "readonly" } })).toMatchObject({ status: "disabled" }); });
  expect(changed).not.toHaveBeenCalled(); session.dispose();
});
</storycode:copy>

### Assemble `apps/web/src/embed/uiHostSession.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/embed/uiHostSession.test.tsx"><storycode:paste select="#f-5f023c549320-expression_statement"/><storycode:paste select="#f-5f023c549320-expression_statement-2"/><storycode:paste select="#f-5f023c549320-expression_statement-3"/><storycode:paste select="#f-5f023c549320-expression_statement-4"/></storycode:file>


</storycode:doc>
