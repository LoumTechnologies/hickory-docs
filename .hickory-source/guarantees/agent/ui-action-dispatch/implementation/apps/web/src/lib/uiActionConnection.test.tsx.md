<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiActionConnection.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-2217191ba72c"></a>
## `apps/web/src/lib/uiActionConnection.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-2217191ba72c-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../api/client";
import { captureUiWindow, UiActionsProvider, uiWindow } from "./uiActionBridge";
vi.mock("../api/client", () => ({ request: vi.fn() }));
</storycode:copy>
<a id="f-2217191ba72c-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-2217191ba72c-never"></a>
### `never`

<storycode:copy id="f-2217191ba72c-never">
const never = new Promise<never>(() => {});
</storycode:copy>
<a id="f-2217191ba72c-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-3">
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
</storycode:copy>
<a id="f-2217191ba72c-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-4">
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
</storycode:copy>
<a id="f-2217191ba72c-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-5">
it("fails a bounded pending capture without submitting a turn and preserves hostless use", async () => {
  expect(await captureUiWindow()).toBeUndefined();
  vi.mocked(request).mockImplementation(async method => method === "POST" || method === "GET" ? never : {});
  render(<UiActionsProvider><span>Connecting</span></UiActionsProvider>);
  await expect(captureUiWindow(1)).rejects.toThrow("Retry Send");
  expect(vi.mocked(request).mock.calls.every(([, path]) => path === "/api/v1/ui/windows")).toBe(true);
});

</storycode:copy>
<a id="f-2217191ba72c-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-6">
it("reports a failed attached handshake instead of silently omitting tools", async () => {
  vi.mocked(request).mockRejectedValue(Error("host offline"));
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("UI connection unavailable: Error: host offline");
});

</storycode:copy>
<a id="f-2217191ba72c-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-7">
it("rejects an unsupported engine protocol without falling back to a legacy window route", async () => {
  vi.mocked(request).mockRejectedValue(Object.assign(new Error("not found"), { status: 404 }));
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("Shared UI protocol v1 is unavailable; update both the app and engine");
  expect(vi.mocked(request).mock.calls).toHaveLength(1);
  expect(vi.mocked(request).mock.calls[0][1]).toBe("/api/v1/ui/windows");
});
</storycode:copy>
<a id="f-2217191ba72c-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-2217191ba72c-expression_statement-8">
it("rejects a malformed protocol scope before making it the active originating window", async () => {
  vi.mocked(request).mockResolvedValue({ workspace: "p", window: "../not-a-window" });
  await act(async () => { render(<UiActionsProvider><span>Workspace</span></UiActionsProvider>); });
  await expect(captureUiWindow()).rejects.toThrow("Invalid shared UI protocol v1 scope");
  expect(uiWindow()).toBeUndefined();
  expect(vi.mocked(request).mock.calls).toHaveLength(1);
});
</storycode:copy>

### Assemble `apps/web/src/lib/uiActionConnection.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiActionConnection.test.tsx"><storycode:paste select="#f-2217191ba72c-expression_statement"/><storycode:paste select="#f-2217191ba72c-expression_statement-2"/><storycode:paste select="#f-2217191ba72c-never"/><storycode:paste select="#f-2217191ba72c-expression_statement-3"/><storycode:paste select="#f-2217191ba72c-expression_statement-4"/><storycode:paste select="#f-2217191ba72c-expression_statement-5"/><storycode:paste select="#f-2217191ba72c-expression_statement-6"/><storycode:paste select="#f-2217191ba72c-expression_statement-7"/><storycode:paste select="#f-2217191ba72c-expression_statement-8"/></storycode:file>

</storycode:doc>
