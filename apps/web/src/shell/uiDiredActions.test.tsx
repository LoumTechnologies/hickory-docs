// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { FILES_CHANGED_EVENT } from "./FolderTreePane";
import { FILES_DELETED_EVENT } from "./FolderTreePane";
import { useDired } from "./useDired";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
function fixture() {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const notice = vi.fn(); const hook = renderHook(() => useDired(notice), { wrapper });
  const descriptor = (label: string) => registry.observe(registry.groups().groups[0]).actions.find(action => action.label === label)!;
  let n = 0;
  const invoke = async (label: string, args = {}) => { let result; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: descriptor(label).handle, arguments: args, request_id: String(++n) }); }); return result; };
  return { hook, registry, descriptor, invoke };
}
it("prepares and submits the same copy prompt as the human controller", async () => {
  const fileOp = vi.spyOn(api, "fileOp").mockResolvedValue({} as never);
  const f = fixture();
  await f.invoke("Prepare file copy", { paths: ["source.md"] });
  expect(fileOp).not.toHaveBeenCalled();
  expect(f.hook.result.current.prompt?.verb).toBe("Copy");
  expect(await f.invoke("Submit file prompt: Copy", { value: "target" })).toMatchObject({ status: "result", outcome: { status: "completed" } });
  expect(fileOp).toHaveBeenCalledWith({ op: "copy", path: "source.md", to: "target" });
  expect(f.hook.result.current.prompt).toBeNull();
});
it("requires the explicit delete confirmation and preserves actual host failure", async () => {
  const fileOp = vi.spyOn(api, "fileOp").mockRejectedValue(Error("Host refused deletion"));
  const f = fixture();
  await f.invoke("Prepare file delete", { paths: ["source.md"] });
  expect(await f.invoke("Submit file prompt: Delete", { confirm: false })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(fileOp).not.toHaveBeenCalled();
  expect(await f.invoke("Submit file prompt: Delete", { confirm: true })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
  expect(f.hook.result.current.promptError).toBe("Host refused deletion");
  await f.invoke("Cancel file prompt"); expect(f.hook.result.current.prompt).toBeNull();
});
it("announces each successfully deleted path for open-tab cleanup", async () => {
  const fileOp = vi.spyOn(api, "fileOp").mockResolvedValue({} as never);
  const deleted = vi.fn(); window.addEventListener(FILES_DELETED_EVENT, deleted);
  const f = fixture();
  try {
    await f.invoke("Prepare file delete", { paths: ["source.md"] });
    await f.invoke("Submit file prompt: Delete", { confirm: true });
    expect(fileOp).toHaveBeenCalledWith({ op: "delete", path: "source.md" });
    expect(deleted).toHaveBeenCalledOnce();
    expect((deleted.mock.calls[0][0] as CustomEvent<{ paths: string[] }>).detail.paths).toEqual(["source.md"]);
  } finally { window.removeEventListener(FILES_DELETED_EVENT, deleted); }
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("admits one filesystem submission before paint and keeps its prompt until completion", async () => {
  let release!: () => void;
  const fileOp = vi.spyOn(api, "fileOp").mockImplementation(() => new Promise(resolve => { release = () => resolve({} as never); }));
  const f = fixture();
  await f.invoke("Prepare file copy", { paths: ["source.md"] });
  const firstOwner = f.hook.result.current.prompt?.id;
  const answer = f.hook.result.current.answer;
  let first!: Promise<boolean>, duplicate!: Promise<boolean>;
  await act(async () => { first = answer("target"); duplicate = answer("target"); f.hook.result.current.cancel(); f.hook.result.current.run({ kind: "create", dir: "other" }); });
  expect(await duplicate).toBe(false); expect(fileOp).toHaveBeenCalledTimes(1);
  expect(f.hook.result.current.prompt?.id).toBe(firstOwner);
  expect(f.hook.result.current.busy).toBe(true);
  await act(async () => { release(); expect(await first).toBe(true); });
  expect(f.hook.result.current.prompt).toBeNull();
  await f.invoke("Prepare file copy", { paths: ["source.md"] });
  expect(f.hook.result.current.prompt?.id).not.toBe(firstOwner);
  expect(await answer("wrong-owner")).toBe(false); expect(fileOp).toHaveBeenCalledTimes(1);
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retires a replaced or cancelled prompt synchronously before React paints", async () => {
  const fileOp = vi.spyOn(api, "fileOp").mockResolvedValue({} as never);
  const f = fixture();
  await f.invoke("Prepare file copy", { paths: ["source.md"] });
  const oldAnswer = f.hook.result.current.answer;
  await act(async () => { f.hook.result.current.run({ kind: "create", dir: "new" }); expect(await oldAnswer("wrong-target")).toBe(false); });
  expect(fileOp).not.toHaveBeenCalled();
  const nextAnswer = f.hook.result.current.answer;
  await act(async () => { f.hook.result.current.cancel(); expect(await nextAnswer("cancelled")).toBe(false); });
  expect(fileOp).not.toHaveBeenCalled();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains actual partial filesystem outcomes and refreshes the visible tree after a later failure", async () => {
  const fileOp = vi.spyOn(api, "fileOp").mockResolvedValueOnce({} as never).mockRejectedValueOnce(Error("Host refused second path"));
  const changed = vi.fn(); window.addEventListener(FILES_CHANGED_EVENT, changed);
  const f = fixture();
  try {
    await f.invoke("Prepare file delete", { paths: ["first.md", "second.md"] });
    expect(await f.invoke("Submit file prompt: Delete", { confirm: true })).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("earlier effects") } });
    expect(fileOp).toHaveBeenCalledTimes(2);
    expect(f.hook.result.current.promptError).toContain('Completed 1 of 2 paths before failure: ["first.md"]');
    expect(f.hook.result.current.promptError).toContain("No rollback occurred");
    expect(f.hook.result.current.prompt).not.toBeNull(); expect(f.hook.result.current.busy).toBe(false);
    expect(changed).toHaveBeenCalledTimes(1);
  } finally { window.removeEventListener(FILES_CHANGED_EVENT, changed); }
});
