<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/embed/uiHostSession.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [An embed has no implicit host](../../../../../../guarantees/embedding/an-embed-has-no-implicit-host/contract.md)
- [An iframe validates its host](../../../../../../guarantees/embedding/an-iframe-validates-its-host/contract.md)

Captured SHA-256: `9011a12d27287eec284f5765dd0a909dfde72d6dd68a676d8426dae8ec64c026`.

<book:entry path="apps/web/src/embed/uiHostSession.test.tsx">
// Guarantees: docs/guarantees/embedding/an-iframe-validates-its-host.md
// docs/guarantees/embedding/an-embed-has-no-implicit-host.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { createEmbedUiSession } from "./uiHostSession";
import { UiActionContext } from "../lib/uiActionBridge";
import { EditingSurface } from "./EditingSurface";
import type { UiReceipt } from "../lib/uiActions";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
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
it("preserves the actual read-only editor policy for parent requests", async () => {
  const session = createEmbedUiSession(() => {}), changed = vi.fn();
  render(<UiActionContext.Provider value={session.registry}><EditingSurface value="historical" ariaLabel="captured.md" onChange={changed} readOnly /></UiActionContext.Provider>);
  const actions = session.registry.groups().groups.flatMap(parent => session.registry.observe(parent, 64).actions);
  const replace = actions.find(action => action.label === "Replace embedded document range")!;
  await act(async () => { expect(await session.invoke({ window: session.scope.window, method: "ui_act", arguments: { handle: replace.handle, arguments: { from: 0, to: 1, text: "x" }, request_id: "readonly" } })).toMatchObject({ status: "disabled" }); });
  expect(changed).not.toHaveBeenCalled(); session.dispose();
});
</book:entry>

</book:doc>
