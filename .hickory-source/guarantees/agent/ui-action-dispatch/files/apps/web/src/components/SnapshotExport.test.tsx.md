<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/SnapshotExport.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `dacbb006ce28ff885d2d5a9e147a59f930e2c53d0ad11397ebbfead5aaeaff40`.

<book:entry path="apps/web/src/components/SnapshotExport.test.tsx">
// @ts-expect-error Node crypto supplies the browser digest in this focused test.
import { webcrypto } from "node:crypto";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { request } from "../api/client";
import { SnapshotExport } from "./SnapshotExport";
vi.mock("../api/client", () => ({ request: vi.fn() }));
afterEach(() => { cleanup(); vi.restoreAllMocks(); vi.unstubAllGlobals(); vi.clearAllMocks(); });
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares export, exposes exact captured bytes and never claims download or publication completed", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const create = vi.fn(() => "blob:export"), revoke = vi.fn();
  vi.stubGlobal("URL", Object.assign(class extends URL {}, { createObjectURL: create, revokeObjectURL: revoke }));
  const click = vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
  vi.mocked(request).mockResolvedValue({ source: "秘密 😀", result: { status: "failed" } });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const content = (docId: string) => <UiActionContext.Provider value={registry}><SnapshotExport docId={docId} /></UiActionContext.Provider>;
  const { rerender } = render(content("first"));
  const parent = registry.groups().groups[0];
  const action = registry.observe(parent).actions.find(a => a.label === "Export viewer snapshot")!;
  let exported;
  await act(async () => { exported = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: {}, request_id: "export" }); });
  expect(exported).toMatchObject({ outcome: { value: { browser_download_requested: true, file_saved: "Browser-owned; not observed", publication: "not requested" } } });
  expect(click).toHaveBeenCalledOnce();
  expect(JSON.stringify(registry.observe(parent))).not.toContain("秘密");
  const read = registry.observe(parent).actions.find(a => a.label === "Read exported snapshot bytes")!;
  const result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (result.status !== "result" || result.outcome.status !== "completed") throw Error("Expected bytes");
  const bytes = (result.outcome.value as { bytes: number[] }).bytes;
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(bytes)))).toEqual({ source: "秘密 😀", result: { status: "failed" } });
  rerender(content("second"));
  expect(registry.observe(parent).actions.some(a => a.label === "Read exported snapshot bytes")).toBe(false);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old-doc" })).toMatchObject({ status: "unavailable" });
  fireEvent.click(screen.getByText("Export viewer snapshot"));
  await act(async () => { await new Promise(resolve => setTimeout(resolve, 20)); });
  expect(request).toHaveBeenLastCalledWith("GET", "/api/docs/second/snapshot");
  expect(click).toHaveBeenCalledTimes(2);
});
</book:entry>

</book:doc>
