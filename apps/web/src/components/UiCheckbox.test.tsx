// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiCheckbox } from "./UiCheckbox";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
afterEach(cleanup);
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
