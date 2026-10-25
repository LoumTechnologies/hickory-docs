import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { ContextMenu } from "./ContextMenu";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
it("shares menu picks and refuses old handles after an identical-label target replacement", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const first = vi.fn(), second = vi.fn(), close = vi.fn();
  const items = [{ id: "open", label: "Open" }, { id: "blocked", label: "Unavailable", disabled: true, tip: "Current owner refuses" }];
  const menu = (pick: typeof first) => <UiActionContext.Provider value={registry}><ContextMenu x={0} y={0} items={items} subject="file" onPick={pick} onClose={close} /></UiActionContext.Provider>;
  const view = render(menu(first));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (label: string) => {
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: {}, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(await run("Unavailable")).toMatchObject({ status: "disabled" });
  expect(first).not.toHaveBeenCalled();
  expect(await run("Open")).toMatchObject({ status: "result", outcome: { value: { callback_requested: true, item: "open" } } });
  expect(first).toHaveBeenCalledExactlyOnceWith("open");
  const old = actions().find(action => action.label === "Open")!.handle;
  view.rerender(menu(second));
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "stale" })).not.toMatchObject({ status: "result" });
  expect(second).not.toHaveBeenCalled();
  await act(async () => { fireEvent.click(screen.getByRole("menuitem", { name: "Open" })); });
  expect(second).toHaveBeenCalledExactlyOnceWith("open");
  expect(await run("Close menu for file")).toMatchObject({ status: "result", outcome: { value: { dismissal_requested: true, operation_cancelled: false } } });
  expect(close).toHaveBeenCalledTimes(1);
  const retired = actions().find(action => action.label === "Open")!.handle;
  view.unmount();
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: retired, arguments: {}, request_id: "closed" })).not.toMatchObject({ status: "result" });
});
it("excludes duplicate async menu picks and surfaces the callback's actual failure", async () => {
  let reject!: (error: Error) => void;
  const pick = vi.fn(() => new Promise((_resolve, fail) => { reject = fail; }));
  render(<ContextMenu x={0} y={0} items={[{ id: "write", label: "Write" }]} subject="draft" onPick={pick} onClose={() => {}} />);
  fireEvent.click(screen.getByRole("menuitem", { name: "Write" }));
  fireEvent.click(screen.getByRole("menuitem", { name: "Write" }));
  expect(pick).toHaveBeenCalledTimes(1);
  await act(async () => { reject(Error("actual refusal")); });
  expect(screen.getByRole("alert").textContent).toContain("actual refusal");
  expect((screen.getByRole("menuitem", { name: "Write" }) as HTMLButtonElement).disabled).toBe(false);
});
it("keeps dismissal addressable while an item callback is pending without claiming cancellation", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let finish!: () => void;
  const pick = vi.fn(() => new Promise<void>(resolve => { finish = resolve; }));
  const close = vi.fn();
  render(<UiActionContext.Provider value={registry}><ContextMenu x={0} y={0} items={[{ id: "write", label: "Write" }]} subject="draft" onPick={pick} onClose={close} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Write")!.handle, arguments: {}, request_id: "pending-item",
  }); });
  let closed: unknown;
  await act(async () => { closed = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === "Close menu for draft")!.handle, arguments: {}, request_id: "close-pending",
  }); });
  expect(closed).toMatchObject({ status: "result", outcome: { value: { dismissal_requested: true, operation_cancelled: false } } });
  expect(close).toHaveBeenCalledTimes(1);
  await act(async () => { finish(); expect(await pending).toMatchObject({ status: "result", outcome: { value: { callback_requested: true } } }); });
});

it("navigates enabled items like a desktop menu and restores focus on dismissal", () => {
  const opener = document.createElement("button");
  document.body.append(opener);
  opener.focus();
  const view = render(<ContextMenu x={0} y={0} subject="file" onPick={() => {}} onClose={() => {}}
    items={[{ id: "open", label: "Open" }, { id: "disabled", label: "Unavailable", disabled: true }, { id: "rename", label: "Rename" }]} />);
  const open = screen.getByRole("menuitem", { name: "Open" });
  const rename = screen.getByRole("menuitem", { name: "Rename" });
  expect(document.activeElement).toBe(open);
  fireEvent.keyDown(open, { key: "ArrowUp" });
  expect(document.activeElement).toBe(rename);
  fireEvent.keyDown(rename, { key: "ArrowDown" });
  expect(document.activeElement).toBe(open);
  fireEvent.keyDown(open, { key: "End" });
  expect(document.activeElement).toBe(rename);
  fireEvent.keyDown(rename, { key: "Home" });
  expect(document.activeElement).toBe(open);
  fireEvent.keyDown(open, { key: "r" });
  expect(document.activeElement).toBe(rename);
  view.unmount();
  expect(document.activeElement).toBe(opener);
  opener.remove();
});
