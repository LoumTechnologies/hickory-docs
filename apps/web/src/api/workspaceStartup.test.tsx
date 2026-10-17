import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler, request } from "./client";
import { StartupNotice } from "../components/StartupNotice";

afterEach(() => { cleanup(); vi.unstubAllGlobals(); });

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
