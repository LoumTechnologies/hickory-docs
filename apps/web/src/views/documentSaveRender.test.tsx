import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { api, installMockHandler } from "../api/client";
import { DocumentEditor } from "../editor/DocumentEditor";
import { DocSessionHost, SessionRegistry, useSessionVersion } from "./documentSession";
vi.mock("../api/realtime", async original => {
  const actual = await original<typeof import("../api/realtime")>();
  return { ...actual, WsRealtime: actual.LocalRealtime };
});
vi.mock("mermaid", () => ({ default: { initialize: vi.fn(), render: vi.fn() } }));
const debug = { status: "idle", breakpoints: [], frames: [], variables: [], watches: [], pausedLine: null };
vi.mock("../debug/useDebugger", () => ({ useDebugger: () => debug }));
afterEach(() => { cleanup(); installMockHandler(null as never); vi.unstubAllGlobals(); vi.restoreAllMocks(); });
const prefix = '<hick:container name="shell" image="alpine:3.20" />\n';
const one = prefix + '<hick:exec container="shell">echo one</hick:exec>\n';
const two = one + '<hick:exec container="shell">echo two</hick:exec>\n';
async function receipt(source: string) {
  const digest = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  const source_hash = `sha256:${[...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("")}`;
  const blocks = [...source.matchAll(/<hick:exec[^>]*>(.*?)<\/hick:exec>/g)].map((m, index) => ({ kind: "exec", id: `cell${index + 1}`, command: m[1], container: "shell", span: [m.index!, m.index! + m[0].length], status: "unrecorded", transcript: [] }));
  return { document: "d", source_hash, blocks };
}
const openGenerated = () => {};
function Harness({ registry }: { registry: SessionRegistry }) {
  useSessionVersion(registry);
  const session = registry.get("d");
  return <><DocSessionHost registry={registry} docId="d" openGenerated={openGenerated} />
    {session?.doc && <DocumentEditor docId="d" initialSource={session.doc.source} realtime={session.realtime} execBlocks={session.execBlocks} runningCells={session.runningCells}
      onRunCell={session.runCell} onChange={session.setDirtySource} onViewReady={session.onDocViewReady} />}</>;
}

it("acknowledged Save makes a newly authored cell runnable through the actual session and editor rail", async () => {
  vi.stubGlobal("crypto", webcrypto);
  let saved = one;
  const handler = vi.fn(async (method: string, path: string, body?: unknown) => {
    if (path === "/api/docs/d" && method === "PUT") { saved = (body as { source: string }).source; return { id: "d", path: "note.md", source: saved }; }
    if (path === "/api/docs/d") return { id: "d", path: "note.md", source: saved };
    if (path === "/api/docs/d/render") return receipt(saved);
    if (path === "/api/docs/d/outputs") return { files: [] };
    if (path.includes("/blocks/")) return { outcome: "run", run_id: "run2", cells: ["cell2"] };
    return {};
  });
  installMockHandler(handler);
  const registry = new SessionRegistry();
  render(<Harness registry={registry} />);
  await screen.findByRole("button", { name: /click to run it/ });
  const view = registry.get("d")!.docEditor!;
  await act(async () => { view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: two } }); });
  expect(screen.getByRole("button", { name: /save this document to enable running/ })).toBeTruthy();
  await act(async () => { expect(await registry.get("d")!.menuSave()).toBe(true); });
  await waitFor(() => expect(screen.queryByRole("button", { name: /save this document to enable running/ })).toBeNull());
  const runs = screen.getAllByRole("button", { name: /click to run it/ });
  expect(runs).toHaveLength(2);
  await act(async () => { runs[1].click(); });
  await waitFor(() => expect(handler.mock.calls.some(([method, path]) => method === "POST" && path.endsWith("/run"))).toBe(true));
  expect(handler.mock.calls.filter(([method, path]) => method === "POST" && path.endsWith("/run"))).toHaveLength(1);
  expect(saved).toBe(two);
});


it("keeps mock startup compatible with the same source-backed render receipt contract", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const { installMockApi } = await import("../mock/mockApi");
  installMockApi();
  const [project] = await api.projects();
  const [summary] = await api.projectDocs(project.id);
  const document = await api.doc(summary.id);
  const rendered = await api.render(summary.id) as unknown as { document: string; source_hash: string; blocks: unknown[] };
  const digest = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(document.source));
  expect(rendered.document).toBe(document.id);
  expect(rendered.source_hash).toBe([...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join(""));
  expect(rendered.blocks.length).toBeGreaterThan(0);
});
