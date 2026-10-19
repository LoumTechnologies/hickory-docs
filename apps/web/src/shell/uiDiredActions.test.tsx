// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
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
