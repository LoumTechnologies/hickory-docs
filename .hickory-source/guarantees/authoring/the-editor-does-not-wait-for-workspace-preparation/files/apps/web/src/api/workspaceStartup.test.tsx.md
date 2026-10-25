<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/api/workspaceStartup.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [The Editor Does Not Wait For Workspace Preparation](../../../../../contract.md)

Captured SHA-256: `fc8f1ed793d018a43b9bd480ca9e5889b038d8636897eb3a4281c97f6c6acd9d`.

<book:entry path="apps/web/src/api/workspaceStartup.test.tsx">
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler, request } from "./client";
import { StartupNotice } from "../components/StartupNotice";

afterEach(() => { cleanup(); vi.unstubAllGlobals(); vi.useRealTimers(); });

// Guarantee: docs/guarantees/authoring/the-editor-does-not-wait-for-workspace-preparation.md
it("shares one preparation wait and submits refused writes only after readiness", async () => {
  installMockHandler(null as never);
  let ready!: (response: Response) => void;
  const gate = new Promise<Response>((resolve) => { ready = resolve; });
  let dispatched = false;
  const fetch = vi.fn(async (path: string, _options?: RequestInit) => {
    if (path === "/api/startup") return gate;
    return dispatched ? new Response('{"ok":true}') : new Response('{"error":"preparing"}', {
      status: 503, headers: { "x-hickory-startup": "pending" },
    });
  });
  vi.stubGlobal("fetch", fetch);
  const app = render(<StartupNotice />);
  const payload = { content: "one" };
  const edits = [request("PUT", "/api/file", payload), request("POST", "/api/projects/local/docs", { path: "two.md" })];
  await app.findByText(/You can keep writing your note/);
  expect(fetch.mock.calls.filter(([path]) => path === "/api/startup")).toHaveLength(1);
  expect(fetch.mock.calls.filter(([path]) => path !== "/api/startup")).toHaveLength(2);
  payload.content = "later form edit";
  await act(async () => { dispatched = true; ready(new Response('{"ready":true}')); await Promise.all(edits); });
  expect(fetch.mock.calls.filter(([path]) => path !== "/api/startup")).toHaveLength(4);
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file").map(([, options]) => options?.body)).toEqual(['{"content":"one"}', '{"content":"one"}']);
  expect(app.queryByRole("status")).toBeNull();
});

it("does not replay a failed mutation without an explicit pre-dispatch marker", async () => {
  installMockHandler(null as never);
  const fetch = vi.fn(async () => new Response('{"error":"save failed"}', { status: 503 }));
  vi.stubGlobal("fetch", fetch);
  await expect(request("PUT", "/api/file", { content: "unsaved" })).rejects.toThrow("save failed");
  expect(fetch).toHaveBeenCalledTimes(1);
});

it("keeps a preparation failure visible and never submits the blocked write", async () => {
  installMockHandler(null as never);
  const fetch = vi.fn(async (path: string) => path === "/api/startup"
    ? new Response('{"error":"Workspace preparation stopped. Reopen the folder."}', { status: 503 })
    : new Response('{}', { status: 503, headers: { "x-hickory-startup": "pending" } }));
  vi.stubGlobal("fetch", fetch);
  const app = render(<StartupNotice />);
  await act(async () => { await expect(request("PUT", "/api/file", {})).rejects.toThrow("Reopen the folder"); });
  await waitFor(() => expect(app.getByRole("status").textContent).toContain("Reopen the folder"));
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file")).toHaveLength(1);
});

it("recovers from a hung status probe after an engine restart and replays only pre-dispatch writes", async () => {
  vi.useFakeTimers();
  installMockHandler(null as never);
  let probes = 0, writes = 0;
  let oldSignal: AbortSignal | undefined;
  const fetch = vi.fn((path: string, options?: RequestInit): Promise<Response> => {
    if (path === "/api/startup") {
      if (++probes === 1) {
        oldSignal = options?.signal as AbortSignal;
        // Simulate a stuck old-generation fetch, even if it ignores abort.
        return new Promise(() => {});
      }
      return Promise.resolve(new Response('{"ready":true}'));
    }
    writes++;
    return Promise.resolve(writes === 1
      ? new Response('{}', { status: 503, headers: { "x-hickory-startup": "pending" } })
      : new Response('{"ok":true}'));
  });
  vi.stubGlobal("fetch", fetch);
  const { startupNotice } = await import("./workspaceStartup");
  const saving = request("PUT", "/api/file", { content: "retained bytes" });
  await vi.advanceTimersByTimeAsync(0);
  expect(probes).toBe(1);
  expect(writes).toBe(1);
  expect(startupNotice()).toContain("Preparing generated files");
  await vi.advanceTimersByTimeAsync(5100);
  await expect(saving).resolves.toEqual({ ok: true });
  expect(oldSignal?.aborted).toBe(true);
  expect(probes).toBe(2);
  expect(writes).toBe(2);
  expect(startupNotice()).toBeNull();
  expect(fetch.mock.calls.filter(([path]) => path === "/api/file").map(([, options]) => options?.body))
    .toEqual(['{"content":"retained bytes"}', '{"content":"retained bytes"}']);
});

it("bounds unreachable status probes, retains an actionable notice, and permits a fresh readiness attempt", async () => {
  vi.useFakeTimers();
  const fetch = vi.fn((): Promise<Response> => new Promise(() => {}));
  vi.stubGlobal("fetch", fetch);
  const { waitForWorkspace, startupNotice } = await import("./workspaceStartup");
  const waiting = waitForWorkspace();
  const rejected = expect(waiting).rejects.toThrow("Reopen the folder to reconnect");
  await vi.advanceTimersByTimeAsync(15200);
  await rejected;
  expect(fetch).toHaveBeenCalledTimes(3);
  expect(startupNotice()).toContain("Cannot reach workspace preparation status");
  fetch.mockImplementation(() => Promise.resolve(new Response('{"ready":true}')));
  await waitForWorkspace();
  expect(startupNotice()).toBeNull();
});

it("shows source-free preparation progress and an honest long-stage notice until ready", async () => {
  vi.useFakeTimers();
  let ready = false;
  vi.stubGlobal("fetch", vi.fn(async () => new Response(JSON.stringify(ready ? {ready:true} : {
    ready:false,phase:"weaving",completed:12,total:17,document:"sessions/large.md",elapsed_ms:62000,stage_elapsed_ms:31000,
  }))));
  const { waitForWorkspace, startupNotice } = await import("./workspaceStartup");
  const waiting=waitForWorkspace();
  await vi.advanceTimersByTimeAsync(0);
  expect(startupNotice()).toContain("12 of 17 documents finished");
  expect(startupNotice()).toContain("sessions/large.md");
  expect(startupNotice()).toContain("Elapsed 62s");
  expect(startupNotice()).toContain("31s; preparation is still pending, not complete");
  ready=true;
  await vi.advanceTimersByTimeAsync(100);
  await waiting;
  expect(startupNotice()).toBeNull();
});
</book:entry>

</book:doc>
