import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext, useUiCommands } from "../lib/uiActionBridge";
import { MoreActions } from "./MoreActions";
afterEach(cleanup);
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
