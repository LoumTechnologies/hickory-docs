<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/RefactorBadge.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `5b2c8480a1ec4f4c974baf2b52b482d67f4ad6527a4de3f53ce9583a96e48df0`.

<book:entry path="apps/web/src/components/RefactorBadge.test.tsx">
import { act, cleanup, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import type { RefactorStatus } from "../api/types";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { RefactorBadge, useBaseline } from "./RefactorBadge";
afterEach(cleanup);
beforeEach(() => { vi.restoreAllMocks(); });
const active: RefactorStatus = { active: true, started_at: "fixture", clean: false, diffs: [{ path: "out.txt", kind: "changed" }] };
it("refuses old-document replies and cached callbacks after changing owners", async () => {
  let finishOld!: (status: RefactorStatus) => void;
  vi.spyOn(api, "refactorStatus").mockImplementation(doc => doc === "a" ? new Promise(resolve => { finishOld = resolve; }) : Promise.resolve({ active: false }));
  const begin = vi.spyOn(api, "refactorBegin").mockResolvedValue(active);
  const hook = renderHook(({ doc }) => useBaseline(doc), { initialProps: { doc: "a" } });
  const oldBegin = hook.result.current.begin, oldScope = hook.result.current.scope;
  hook.rerender({ doc: "b" });
  await waitFor(() => expect(hook.result.current.status).toEqual({ active: false }));
  await act(async () => { finishOld(active); });
  expect(hook.result.current.status).toEqual({ active: false });
  expect(hook.result.current.scope).not.toBe(oldScope);
  await act(async () => { expect(await oldBegin()).toMatchObject({ status: "unavailable" }); });
  expect(begin).not.toHaveBeenCalled();
});
it("excludes duplicate writes and prevents an old status poll from overwriting a mutation", async () => {
  let finishStatus!: (status: RefactorStatus) => void;
  let finishBegin!: (status: RefactorStatus) => void;
  vi.spyOn(api, "refactorStatus").mockImplementation(() => new Promise(resolve => { finishStatus = resolve; }));
  const begin = vi.spyOn(api, "refactorBegin").mockImplementation(() => new Promise(resolve => { finishBegin = resolve; }));
  const end = vi.spyOn(api, "refactorEnd");
  const hook = renderHook(() => useBaseline("a"));
  let pending!: ReturnType<typeof hook.result.current.begin>;
  await act(async () => { pending = hook.result.current.begin(); });
  await act(async () => { expect(await hook.result.current.end()).toMatchObject({ status: "unavailable" }); });
  expect(begin).toHaveBeenCalledTimes(1); expect(end).not.toHaveBeenCalled();
  await act(async () => { finishBegin(active); expect(await pending).toMatchObject({ status: "completed", value: { doc_id: "a", response: active } }); });
  await act(async () => { finishStatus({ active: false }); });
  expect(hook.result.current.status).toEqual(active);
});
it("shares visible verdict reading and stop/refresh with actual mocked replies and failures", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  vi.spyOn(api, "refactorStatus").mockResolvedValue(active);
  const end = vi.spyOn(api, "refactorEnd").mockRejectedValueOnce(Error("actual refusal")).mockResolvedValueOnce({ active: false });
  function Harness() { const baseline = useBaseline("a"); return <RefactorBadge baseline={baseline} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  await screen.findByText(/1 output differs/);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, {
    handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id),
  }); }); return result; };
  const reading = await run("Read output baseline verdict bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(Uint8Array.from(reading.outcome.value.bytes))).toContain("out.txt (changed)");
  expect(await run("Stop comparing outputs to baseline")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("actual refusal") } });
  expect(screen.getByText(/actual refusal/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Done" }));
  await waitFor(() => expect(screen.queryByRole("button", { name: "Done" })).toBeNull());
  expect(end).toHaveBeenCalledTimes(2);
  expect(actions().some(action => action.label === "Stop comparing outputs to baseline")).toBe(false);
});
</book:entry>

</book:doc>
