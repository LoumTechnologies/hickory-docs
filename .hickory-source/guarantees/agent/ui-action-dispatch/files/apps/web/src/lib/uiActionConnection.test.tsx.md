<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/uiActionConnection.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `e267b8f6309d13e1a686889b1196a6a2e5ef803a26c40f3d8d8cfac5190f4df0`.

<book:entry path="apps/web/src/lib/uiActionConnection.test.tsx">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { captureUiWindow, UiActionsProvider, uiWindow } from "./uiActionBridge";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.clearAllMocks(); });
const never = new Promise<never>(() => {});
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("captures a pending originating handshake before Send and never adopts its replacement", async () => {
  let finish!: (scope: { workspace: string; window: string }) => void;
  const handshake = new Promise(resolve => { finish = resolve; });
  vi.mocked(request).mockImplementation(async (method, _path) => method === "POST" ? handshake : method === "GET" ? never : {});
  const view = render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>);
  expect(uiWindow()).toBeUndefined();
  let settled = false;
  const capture = captureUiWindow().then(window => { settled = true; return window; });
  await Promise.resolve(); expect(settled).toBe(false);
  await act(async () => { finish({ workspace: "p", window: "origin" }); });
  expect(await capture).toBe("origin");
  expect(uiWindow()).toBe("origin");
  view.unmount();
  expect(uiWindow()).toBeUndefined();
});
it("rejects a closed pending origin even when a new connection succeeds", async () => {
  vi.mocked(request).mockImplementation(async method => method === "POST" || method === "GET" ? never : {});
  const first = render(<UiActionsProvider><span>First</span></UiActionsProvider>);
  const captured = captureUiWindow();
  const rejection = expect(captured).rejects.toThrow("originating UI window closed");
  first.unmount();
  vi.mocked(request).mockImplementation(async method => method === "POST" ? { workspace: "p", window: "replacement" } : method === "GET" ? never : {});
  await act(async () => { render(<UiActionsProvider><span>Second</span></UiActionsProvider>); });
  await rejection;
  expect(await captureUiWindow()).toBe("replacement");
});
it("fails a bounded pending capture without submitting a turn and preserves hostless use", async () => {
  expect(await captureUiWindow()).toBeUndefined();
  vi.mocked(request).mockImplementation(async method => method === "POST" || method === "GET" ? never : {});
  render(<UiActionsProvider><span>Connecting</span></UiActionsProvider>);
  await expect(captureUiWindow(1)).rejects.toThrow("Retry Send");
  expect(vi.mocked(request).mock.calls.every(([, path]) => path === "/api/v1/ui/windows")).toBe(true);
});

it("reports a failed attached handshake instead of silently omitting tools", async () => {
  vi.mocked(request).mockRejectedValue(Error("host offline"));
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("UI connection unavailable: Error: host offline");
});

it("rejects an unsupported engine protocol without falling back to a legacy window route", async () => {
  vi.mocked(request).mockRejectedValue(Object.assign(new Error("not found"), { status: 404 }));
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("Shared UI protocol v1 is unavailable; update both the app and engine");
  expect(vi.mocked(request).mock.calls).toHaveLength(1);
  expect(vi.mocked(request).mock.calls[0][1]).toBe("/api/v1/ui/windows");
});
it("rejects a malformed protocol scope before making it the active originating window", async () => {
  vi.mocked(request).mockResolvedValue({ workspace: "p", window: "../not-a-window" });
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("Invalid shared UI protocol v1 scope");
  expect(uiWindow()).toBeUndefined();
  expect(vi.mocked(request).mock.calls).toHaveLength(1);
});
</book:entry>

</book:doc>
