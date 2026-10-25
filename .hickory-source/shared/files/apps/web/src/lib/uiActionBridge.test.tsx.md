<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/uiActionBridge.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../../guarantees/agent/ui-action-dispatch/contract.md)
- [Keep a prediction separate from the answer it is tested against](../../../../../../guarantees/verification/predictions-precede-managed-observation/contract.md)

Captured SHA-256: `c6e12f05c5f3d96b8f9c5ca20dab241f2ad52bf1e5eddfe72c597a46fdc37971`.

<book:entry path="apps/web/src/lib/uiActionBridge.test.tsx">
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { UiActions, type UiReceipt } from "./uiActions";
import { dispatchUiFrame, UiActionContext, UiActionsVisibility, useUiCommands } from "./uiActionBridge";
import { MoreActions } from "../components/MoreActions";
import { CardRail } from "../editor/CardRail";
import type { DocCard } from "../editor/cards";

const scope = { workspace: "practice", window: "actual-origin" };
const agent = { id: "python:turn", kind: "agent" as const };
function registry() {
  const receipts: UiReceipt[] = [];
  const actions = new UiActions(scope, () => true, async receipt => { receipts.push(receipt); });
  return { actions, receipts };
}

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
describe("registered human UI and structured agent transport", () => {
  it("retires controls on a hidden mounted surface and requires fresh handles after reopening", async () => {
    const { actions } = registry();
    const content = (visible: boolean) => <UiActionContext.Provider value={actions}><UiActionsVisibility visible={visible}><MoreActions label="Hidden panel controls"><button>Child</button></MoreActions></UiActionsVisibility></UiActionContext.Provider>;
    const { rerender } = render(content(true));
    const handle = actions.observe("menus").actions[0].handle;
    rerender(content(false)); expect(actions.observe("menus").actions).toEqual([]);
    expect(await actions.act(scope, agent, { handle, arguments: {}, request_id: "hidden" })).toMatchObject({ status: "unavailable" });
    rerender(content(true));
    expect(actions.observe("menus").actions[0].handle).not.toBe(handle);
  });
  it("lets the agent discover and operate the real menu, then the human close it", async () => {
    const { actions, receipts } = registry();
    render(<UiActionContext.Provider value={actions}><MoreActions label="Execution actions"><button>Existing child</button></MoreActions></UiActionContext.Provider>);
    const root = await dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_observe", arguments: {} }) as { groups: string[]; observation_bytes: number };
    expect(root.groups).toEqual(["menus"]);
    expect(root.observation_bytes).toBeGreaterThan(0);
    const descriptor = actions.observe("menus").actions[0];
    expect(descriptor.value).toBe(false);
    await act(async () => {
      const result = await dispatchUiFrame(actions, { id: "2", actor: agent, method: "ui_act", arguments: { handle: descriptor.handle, arguments: {}, request_id: "agent-open" } });
      expect(result).toMatchObject({ status: "result", outcome: { status: "completed", value: { open: true } } });
    });
    const menu = screen.getByLabelText("Execution actions").closest("details")!;
    expect(menu.open).toBe(true);
    fireEvent.click(screen.getByLabelText("Execution actions"));
    await waitFor(() => expect(menu.open).toBe(false));
    expect(receipts.filter(r => r.stage === "result").map(r => r.actor.kind)).toEqual(["agent", "human"]);
    expect(screen.queryByText(descriptor.handle)).toBeNull();
  });

  it("runs the existing cell callback once per request and never claims runtime verification", async () => {
    const { actions, receipts } = registry();
    const callback = vi.fn();
    const card: DocCard = { key: "exec-1", kind: "exec", index: 0, at: 1, from: 1, to: 4, label: "λ😀" };
    const props = { documentId: "deadline", view: null, cards: [card], stateOf: () => "idle" as const, openKey: null, renderedAt: [], actionsOf: () => ["run" as const], replayingAt: [], onAction: callback };
    const { rerender } = render(<UiActionContext.Provider value={actions}><CardRail {...props} /></UiActionContext.Provider>);
    const parent = actions.groups().groups[0];
    const descriptor = actions.observe(parent).actions[0];
    const frame = { id: "1", actor: agent, method: "ui_act", arguments: { handle: descriptor.handle, arguments: {}, request_id: "run-request" } };
    await act(async () => {
      const result = await dispatchUiFrame(actions, frame);
      expect(result).toMatchObject({ status: "result", outcome: { value: { ui_handler_returned: true, runtime_verification: "not established" } } });
      await dispatchUiFrame(actions, frame);
    });
    expect(callback).toHaveBeenCalledExactlyOnceWith(card, "run", 0);
    fireEvent.click(screen.getByRole("button", { name: /click to run it/ }));
    await waitFor(() => expect(callback).toHaveBeenCalledTimes(2));
    expect(receipts.filter(r => r.stage === "requested").map(r => r.actor.kind)).toEqual(["agent", "human"]);
    rerender(<UiActionContext.Provider value={actions}><CardRail {...props} stateOf={() => "running"} /></UiActionContext.Provider>);
    const running = actions.observe(parent).actions[0];
    expect(running.enabled).toBe(false);
    expect(await dispatchUiFrame(actions, { ...frame, arguments: { ...frame.arguments, handle: running.handle, request_id: "disabled" } })).toMatchObject({ status: "disabled" });
    expect(callback).toHaveBeenCalledTimes(2);
  });

  it("uses latest callbacks and unregisters closed controls", async () => {
    const { actions } = registry(); const first = vi.fn(); const next = vi.fn();
    const card: DocCard = { key: "one", kind: "diagram", index: 0, at: 0, from: 0, to: 1, label: "Diagram" };
    const props = { view: null, cards: [card], stateOf: () => "idle" as const, openKey: null, renderedAt: [], actionsOf: () => ["source" as const], replayingAt: [] };
    const { rerender, unmount } = render(<UiActionContext.Provider value={actions}><CardRail {...props} onAction={first} /></UiActionContext.Provider>);
    const descriptor = actions.observe(actions.groups().groups[0]).actions[0];
    rerender(<UiActionContext.Provider value={actions}><CardRail {...props} onAction={next} /></UiActionContext.Provider>);
    await act(async () => { await actions.act(scope, agent, { handle: descriptor.handle, arguments: {}, request_id: "toggle" }); });
    expect(first).not.toHaveBeenCalled(); expect(next).toHaveBeenCalledOnce();
    unmount();
    expect(actions.groups().groups).toEqual([]);
    expect(await actions.act(scope, agent, { handle: descriptor.handle, arguments: {}, request_id: "closed" })).toMatchObject({ status: "unavailable" });
  });

  it("rejects forged actor/scope fields and unstructured action text", async () => {
    const { actions } = registry();
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_act", arguments: { handle: "action:forged", arguments: {}, request_id: "r", window: "another" } })).rejects.toThrow("Invalid");
    await expect(dispatchUiFrame(actions, { id: "1", actor: { id: "human", kind: "human" }, method: "ui_observe", arguments: {} })).rejects.toThrow("agent");
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "quoted action:forged", arguments: {} })).rejects.toThrow("Unknown");
    await expect(dispatchUiFrame(actions, { id: "1", actor: agent, method: "ui_observe", arguments: { limit: 100000 } })).rejects.toThrow("bounds");
  });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reports the known UTF-8 result payload size separately from provider token cost", async () => {
  const { actions } = registry();
  actions.register("unicode", { parent: "reading", subject: "reading", label: "Read Unicode", parameters: {}, prerequisites: [], read: () => ({ revision: "one", enabled: true }), execute: async () => ({ status: "completed", value: { text: "λ😀" } }) });
  const handle = actions.observe("reading").actions[0].handle;
  const result = await dispatchUiFrame(actions, { id: "unicode", actor: agent, method: "ui_act", arguments: { handle, arguments: {}, request_id: "unicode" } });
  expect(result).toMatchObject({ status: "result", result_bytes: 78, token_cost: "Provider-specific; not measured here", outcome: { value: { text: "λ😀" } } });
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("keeps an unchanged command registered while a sibling changes during intent persistence", async () => {
  let release!: () => void;
  const retained = new Promise<void>(resolve => { release = resolve; });
  const actions = new UiActions(scope, () => true, receipt => receipt.stage === "requested" ? retained : Promise.resolve());
  const execute = vi.fn(async () => ({ status: "completed" as const, value: null }));
  function Controls({ sibling }: { sibling: boolean }) {
    const command = { parent: "same", subject: "same", label: "Keep", parameters: {}, prerequisites: [], read: () => ({ revision: "unchanged", enabled: true }), execute };
    useUiCommands([{ key: "keep", command }, ...(sibling ? [{ key: "sibling", command: { ...command, label: "Sibling" } }] : [])]);
    return null;
  }
  const content = (sibling: boolean) => <UiActionContext.Provider value={actions}><Controls sibling={sibling} /></UiActionContext.Provider>;
  const view = render(content(false));
  const handle = actions.descriptor("keep")!.handle;
  const pending = actions.act(scope, agent, { handle, arguments: {}, request_id: "intent" });
  await act(async () => { await Promise.resolve(); });
  view.rerender(content(true));
  expect(actions.descriptor("keep")!.handle).toBe(handle);
  release();
  expect(await pending).toMatchObject({ status: "result", outcome: { status: "completed" } });
  expect(execute).toHaveBeenCalledOnce();
  view.unmount();
  expect(actions.groups().groups).toEqual([]);
});
</book:entry>

</book:doc>
