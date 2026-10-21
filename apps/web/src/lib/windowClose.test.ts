// Guarantee: docs/guarantees/authoring/unsaved-work-survives-closing-the-app.md
import { expect, it, vi } from "vitest";
import { requestApplicationClose, requestWorkspaceClose, WORKSPACE_CLOSE, type WorkspaceCloseRequest } from "./windowClose";

it("requires a mounted workspace owner and forwards its real result and cancellation scope", async () => {
  expect(await requestWorkspaceClose()).toMatchObject({ status: "unavailable", reason: expect.stringContaining("No fallback") });
  const controller = new AbortController(); let actual: AbortSignal | undefined;
  const listener = (event: Event) => {
    const request = (event as CustomEvent<WorkspaceCloseRequest>).detail;
    request.accept(); actual = request.signal;
    request.respond({ status: "unavailable", reason: "actual source changed" });
  };
  window.addEventListener(WORKSPACE_CLOSE, listener);
  try { expect(await requestWorkspaceClose(controller.signal)).toMatchObject({ status: "unavailable", reason: "actual source changed" }); expect(actual).toBe(controller.signal); }
  finally { window.removeEventListener(WORKSPACE_CLOSE, listener); }
});
it("reveals the retained workspace and never falls back past its refusal", async () => {
  const reveal = vi.fn(), hostClose = vi.fn(async () => ({ ok: true as const }));
  const workspaceClose = vi.fn(async () => ({ status: "unavailable" as const, reason: "save refused" }));
  expect(await requestApplicationClose({ workspace: "/docs/kept", inWorkspace: false, reveal, workspaceClose, hostClose })).toMatchObject({ status: "unavailable", reason: "save refused" });
  expect(reveal).toHaveBeenCalledExactlyOnceWith("/docs/kept"); expect(workspaceClose).toHaveBeenCalledTimes(1); expect(hostClose).not.toHaveBeenCalled();
  workspaceClose.mockRejectedValue(Error("owner retired"));
  expect(await requestApplicationClose({ workspace: "/docs/kept", inWorkspace: true, reveal, workspaceClose, hostClose })).toMatchObject({ status: "unavailable", reason: expect.stringContaining("without fallback") });
  expect(hostClose).not.toHaveBeenCalled(); expect(reveal).toHaveBeenCalledTimes(1);
});
it("reports only a valid host acknowledgement when no workspace exists", async () => {
  const options = { workspace: null, inWorkspace: false, reveal: vi.fn(), workspaceClose: vi.fn(), hostClose: vi.fn(async () => ({ ok: true as const })) };
  expect(await requestApplicationClose(options)).toMatchObject({ status: "completed", value: { host_close_request_acknowledged: true, window_destruction_observed: false } });
  options.hostClose.mockResolvedValue({ ok: false } as unknown as { ok: true });
  expect(await requestApplicationClose(options)).toMatchObject({ status: "unavailable", reason: expect.stringContaining("Invalid host close acknowledgement") });
  const controller = new AbortController(); controller.abort();
  expect(await requestApplicationClose({ ...options, signal: controller.signal })).toMatchObject({ status: "completed", value: { cancelled: true, host_close_submitted: false } });
  expect(options.hostClose).toHaveBeenCalledTimes(2); expect(options.workspaceClose).not.toHaveBeenCalled();
});
