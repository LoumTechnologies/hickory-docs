<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/WorktreeRegionEditor.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `b06d8804b6e2df549069f9d70f9ae0e7155854011637e1b9fca5edb3067b18ce`.

<book:entry path="apps/web/src/components/WorktreeRegionEditor.test.tsx">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { WorktreeRegionEditor } from "./WorktreeRegionEditor";
afterEach(cleanup);
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label && action.enabled)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  return { registry, actions, run };
}
it("shares a real comparison draft, native apply and actual per-target callback replies without duplicate writes", async () => {
  const { registry, actions, run } = tools();
  let complete!: (result: unknown) => void;
  const apply = vi.fn(() => new Promise(resolve => { complete = resolve; }));
  render(<UiActionContext.Provider value={registry}><WorktreeRegionEditor path="file.txt" values={{ a: "old", b: "old" }} writable={new Set(["a", "b"])} shared sourceEpoch="snapshot-one:region-zero" onApply={apply} /></UiActionContext.Provider>);
  await waitFor(() => expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true));
  expect(await run("Replace comparison draft range", { from: 0, to: 3, text: "edited" })).toMatchObject({ status: "result", outcome: { value: { draft_changed: true } } });
  await run("Set worktree region write route", { route: "just-here" });
  expect((screen.getByLabelText("Write route") as HTMLSelectElement).value).toBe("just-here");
  fireEvent.change(screen.getByLabelText("Write route"), { target: { value: "shared" } });
  await act(async () => { fireEvent.click(screen.getByRole("button", { name: "Apply region to selected targets" })); });
  expect(apply).toHaveBeenCalledExactlyOnceWith("edited", "a", "shared");
  const applyAction = actions().find(action => action.label === "Apply worktree region draft")!;
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: applyAction.handle, arguments: {}, request_id: "duplicate" })).toMatchObject({ status: "disabled" });
  expect((screen.getByLabelText("Write route") as HTMLSelectElement).disabled).toBe(true);
  const actual = { targets: [{ source: "a", ok: true }, { source: "b", ok: false, error: "fixture conflict" }] };
  await act(async () => { complete(actual); });
  await run("Set evidence visibility: Region apply reply", { open: true });
  const reply = await run("Read disclosed evidence bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(Uint8Array.from(reply.outcome.value.bytes)))).toMatchObject({ response: actual, callback_completed: true });
});
it("retains a stale draft read-only and requires explicit reset before using a replaced snapshot", async () => {
  const { registry, actions, run } = tools();
  const oldApply = vi.fn(), newApply = vi.fn();
  const panel = (epoch: string, text: string, apply = oldApply) => <UiActionContext.Provider value={registry}><WorktreeRegionEditor path="file.txt" values={{ a: text }} writable={new Set(["a"])} shared={false} sourceEpoch={epoch} onApply={apply} /></UiActionContext.Provider>;
  const view = render(panel("first", "old"));
  await waitFor(() => expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true));
  await run("Replace comparison draft range", { from: 0, to: 3, text: "keep this draft" });
  const old = actions().find(action => action.label === "Apply worktree region draft")!.handle;
  view.rerender(panel("second", "new", newApply));
  expect(screen.getByText(/draft is retained read-only/)).toBeTruthy();
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
  expect(await registry.act(registry.scope, { id: "acp:fixture", kind: "agent" }, { handle: old, arguments: {}, request_id: "old-snapshot" })).not.toMatchObject({ status: "result" });
  expect(newApply).not.toHaveBeenCalled(); expect(oldApply).not.toHaveBeenCalled();
  await run("Reset worktree region draft to current input");
  expect(screen.queryByText(/draft is retained read-only/)).toBeNull();
  expect(actions().some(action => action.label === "Replace comparison draft range" && action.enabled)).toBe(true);
  expect((screen.getByRole("button", { name: "Apply region to selected targets" }) as HTMLButtonElement).disabled).toBe(true);
});
</book:entry>

</book:doc>
