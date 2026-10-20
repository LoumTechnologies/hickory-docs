import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { useTerminals } from "./useTerminals";
import type { TerminalSession } from "../api/types";
const response = (sessions: TerminalSession[] = []) => ({ sessions, attention: [], turbo: false });
beforeEach(() => {
  vi.restoreAllMocks();
  vi.spyOn(api, "terminals").mockResolvedValue(response());
  vi.spyOn(api, "terminalAnchors").mockResolvedValue({ anchors: {} });
});
afterEach(cleanup);
it("reports actual terminal refusals and keeps them visible across successful supervision refreshes", async () => {
  const close = vi.spyOn(api, "closeTerminal").mockRejectedValueOnce(Error("actual close refusal")).mockResolvedValueOnce(undefined);
  const hook = renderHook(() => useTerminals());
  await act(async () => { expect(await hook.result.current.close("one")).toBe(false); });
  await act(async () => { hook.result.current.refresh(); });
  expect(hook.result.current.error).toContain("actual close refusal");
  await act(async () => { expect(await hook.result.current.close("one")).toBe(true); });
  expect(hook.result.current.error).toBeNull();
  expect(close).toHaveBeenCalledTimes(2);
});
it("refuses stale supervision replies and closed-controller mutations", async () => {
  let finishOld!: (value: ReturnType<typeof response>) => void;
  vi.mocked(api.terminals).mockImplementationOnce(() => new Promise(resolve => { finishOld = resolve; })).mockResolvedValue(response());
  const close = vi.spyOn(api, "closeTerminal");
  const hook = renderHook(() => useTerminals());
  await act(async () => { hook.result.current.refresh(); });
  await act(async () => { finishOld({ ...response(), turbo: true }); });
  expect(hook.result.current.turbo).toBe(false);
  const cachedClose = hook.result.current.close;
  hook.unmount();
  expect(await cachedClose("one")).toBe(false);
  expect(close).not.toHaveBeenCalled();
});
it("does not optimistically enable automatic answers when the host refuses", async () => {
  vi.spyOn(api, "setTurbo").mockRejectedValue(Error("host policy refusal"));
  const hook = renderHook(() => useTerminals());
  await act(async () => { expect(await hook.result.current.setTurbo(true)).toBe(false); });
  expect(hook.result.current.turbo).toBe(false);
  await waitFor(() => expect(hook.result.current.error).toContain("host policy refusal"));
});
