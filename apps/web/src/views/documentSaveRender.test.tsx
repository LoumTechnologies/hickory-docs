import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
// @ts-expect-error Node test runtime only.
import { webcrypto } from "node:crypto";
import { api, installMockHandler } from "../api/client";
import { LocalRealtime } from "../api/realtime";
import { undo, redo } from "@codemirror/commands";
import { DocumentEditor } from "../editor/DocumentEditor";
import { DiskChangeNotice } from "../components/MergeView";
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
function Harness({ registry, hidden = false, retainUnsaved = false, compare = false }: {
  registry: SessionRegistry; hidden?: boolean; retainUnsaved?: boolean; compare?: boolean;
}) {
  useSessionVersion(registry);
  const session = registry.get("d");
  return <><DocSessionHost registry={registry} docId="d" openGenerated={openGenerated} retainUnsaved={retainUnsaved} />
    <div hidden={hidden}>{session?.doc && <DocumentEditor docId="d" initialSource={session.doc.source} realtime={session.realtime} execBlocks={session.execBlocks} runningCells={session.runningCells}
      onRunCell={session.runCell} onChange={session.setDirtySource} onViewReady={session.onDocViewReady} />}</div>
    {compare && session?.doc && <DiskChangeNotice path={session.doc.path} base={session.diskReviewBase ?? session.doc.source}
      source={session.liveSource} reviewRequired={session.needsDraftReconciliation}
      reviewGuard={session.detachedDraft ? disk => disk.save_guard : undefined} read={() => api.doc("d")}
      apply={(disk, text) => session.adoptDiskSource(disk, text, session.liveSource)} />}</>;
}

it("keeps a managed proposal detached through rollback, Undo, tab visibility and guarded Save", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const bound = vi.spyOn(LocalRealtime.prototype, "bindDoc");
  const saved = "First line\n\nvalue = 1\n\nLast line\n";
  const before = saved.replace("First line", "First\nline"), after = before.replace("= 1", "= 2");
  const saves: { version: number; source: string; request_id: string; save_guard: string; expected: { source: string; disk: string } }[] = [];
  let disk = saved;
  const handler = vi.fn(async (method: string, path: string, body?: unknown) => {
    if (path === "/api/files") return {root_path: "/workspace", workspace_id: "a".repeat(32)};
    if (path.startsWith("/api/complete?")) return {suggestions: []};
    if (path === "/api/docs/d/render") return receipt(disk);
    if (path === "/api/docs/d/outputs") return {files: []};
    if (path === "/api/docs/d" && method === "GET") return {id: "d", path: "note.md", source: disk, save_guard: "original-guard"};
    if (path === "/api/source-history/docs/d" && method === "PUT") {
      const save = body as (typeof saves)[number]; saves.push(save);
      disk = save.source;
      const room = bound.mock.calls[0][0].getText("source");
      if (saves.length === 1) room.doc!.transact(() => { room.delete(0, room.length); room.insert(0, disk); });
      const rendered = await receipt(disk);
      return { version: 1, history_protocol: 1, id: "d", path: "note.md", source: disk,
        source_hash: rendered.source_hash.slice(7), request_id: saves.length === 1 ? "wrong-id" : save.request_id,
        save_guard: "replacement-guard", history: {available: true, outcome: "published", id: save.request_id} };
    }
    return {};
  });
  installMockHandler(handler);
  const registry = new SessionRegistry(), mounted = render(<Harness registry={registry} />);
  const session = () => registry.get("d")!;
  await waitFor(() => expect(session()?.recordedPatchSource()?.text).toBe(saved));
  const view = session().docEditor!, room = bound.mock.calls[0][0].getText("source");
  await act(async () => {
    // A typed soft break does not reflow until a fresh editor opens. Rollback
    // has to preserve it rather than reopening over the same Y.Text.
    view.dispatch({changes: {from: 5, to: 6, insert: "\n"}, selection: {anchor: 2, head: 7}, userEvent: "input.type"});
  });
  expect(room.toString()).toBe(before);
  const original = view.state;
  const rolledBack = session().recordedPatchSource()!.prepareDraft!(after)!;
  await act(async () => {
    rolledBack.commit(); rolledBack.rollback();
    expect(view.state).toBe(original);
  });
  expect(view.state.doc).toBe(original.doc);
  expect(room.toString()).toBe(before);

  const stage = session().recordedPatchSource()!.prepareDraft!(after)!;
  await act(async () => { stage.commit(); });
  expect(view.state.doc.toString()).toBe(after);
  expect(view.state.selection.toJSON()).toEqual(original.selection.toJSON());
  await act(async () => { expect(undo(view)).toBe(true); });
  expect(view.state.doc.toString()).toBe(before);
  await act(async () => { expect(redo(view)).toBe(true); });
  mounted.rerender(<Harness registry={registry} hidden />);
  mounted.rerender(<Harness registry={registry} />);
  expect(session().docEditor).toBe(view);
  expect(view.state.doc.toString()).toBe(after);
  expect(room.toString()).toBe(before);
  expect(saves).toEqual([]);

  await act(async () => { expect(await session().menuSave()).toBe(false); });
  expect(session().savedSource).toBe(saved);
  const newer = after + "Still local\n";
  await act(async () => { view.dispatch({changes: {from: view.state.doc.length, insert: "Still local\n"}, userEvent: "input.type"}); });
  expect(room.toString()).toBe(after);
  await act(async () => { expect(await session().menuSave()).toBe(false); });
  expect(saves).toHaveLength(2);
  expect(saves[0]).toEqual({version: 1, request_id: expect.stringMatching(/^[0-9a-f]{32}$/), save_guard: "original-guard", source: after,
    expected: {source: before, disk: saved}});
  expect(saves[1]).toEqual(saves[0]);
  expect(session().savedSource).toBe(after);
  expect(session().dirty).toBe(true);
  expect(view.state.doc.toString()).toBe(newer);
  await act(async () => { room.insert(room.length, "Remote\n"); });
  expect(view.state.doc.toString()).toBe(newer);
  await act(async () => { expect(await session().menuSave()).toBe(false); });
  expect(saves).toHaveLength(2);
  expect(handler.mock.calls.some(([method, path]) => method === "PUT" && path === "/api/docs/d")).toBe(false);
});

it("recovers marked managed drafts and requires exact Compare before explicit guarded Save", async () => {
  vi.stubGlobal("crypto", webcrypto);
  const before = "Current room\n", recovered = "An older private draft\n";
  const bound = vi.spyOn(LocalRealtime.prototype, "bindDoc");
  let guard = "observed-guard", disk = before;
  const guarded: unknown[] = [];
  const handler = vi.fn(async (method: string, path: string, body?: unknown) => {
    if (path === "/api/files") return {root_path: "/workspace", workspace_id: "a".repeat(32)};
    if (path.startsWith("/api/complete?")) return {suggestions: []};
    if (method === "GET" && path === "/api/docs/d") return {id: "d", path: "note.md", source: disk, save_guard: guard};
    if (path === "/api/docs/d/render") return receipt(before);
    if (path === "/api/docs/d/outputs") return {files: []};
    if (method === "GET" && path === "/api/workspace/drafts") {
      return {drafts: [{path: "note.md", base: before, contents: recovered, detached: true, saved_at: 1}]};
    }
    if (method === "PUT" && path === "/api/source-history/docs/d") {
      guarded.push(body);
      const save = body as {source: string; request_id: string};
      disk = save.source;
      return {version: 1, history_protocol: 1, id: "d", path: "note.md", source: disk,
        source_hash: (await receipt(disk)).source_hash.slice(7), request_id: save.request_id,
        save_guard: "published-guard", history: {available: true, outcome: "published", id: save.request_id}};
    }
    return {};
  });
  installMockHandler(handler);
  const registry = new SessionRegistry();
  render(<Harness registry={registry} retainUnsaved compare />);
  const session = () => registry.get("d")!;
  await waitFor(() => expect(session()?.docEditor?.state.doc.toString()).toBe(recovered));
  const view = session().docEditor!, room = bound.mock.calls[0][0].getText("source");
  expect(room.toString()).toBe(before);
  await act(async () => { expect(await session().menuSave()).toBe(false); });

  fireEvent.click(await screen.findByRole("button", {name: "Merge changes"}));
  const oldAccept = await screen.findByRole("button", {name: "Accept merge"});
  guard = "same-bytes-replacement-guard";
  fireEvent.click(oldAccept);
  await screen.findByText(/saved source identity changed/i);
  expect(session().needsDraftReconciliation).toBe(true);
  expect(view.state.doc.toString()).toBe(recovered);

  fireEvent.click(await screen.findByRole("button", {name: "Merge changes"}));
  fireEvent.click(await screen.findByRole("button", {name: "Accept merge"}));
  await waitFor(() => expect(session().needsDraftReconciliation).toBe(false));
  expect(guarded).toEqual([]);
  expect(session().docEditor).toBe(view);
  expect(view.state.doc.toString()).toBe(recovered);
  expect(room.toString()).toBe(before);

  await act(async () => { expect(await session().menuSave()).toBe(true); });
  expect(guarded).toEqual([{version: 1, request_id: expect.stringMatching(/^[0-9a-f]{32}$/), source: recovered,
    save_guard: "same-bytes-replacement-guard", expected: {source: before, disk: before}}]);
  expect(handler.mock.calls.some(([method, path]) => method === "PUT" && path === "/api/docs/d")).toBe(false);
});

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
  // jsdom has no layout: give the rail a viewport so measured controls do
  // not disappear merely because clientHeight defaults to zero.
  Object.defineProperty(view.scrollDOM, "clientHeight", { value: 1000 });
  await act(async () => { view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: two } }); });
  expect(screen.getByRole("button", { name: /save this document to enable running/ })).toBeTruthy();
  await act(async () => { expect(await registry.get("d")!.menuSave()).toBe(true); });
  await waitFor(() => expect(screen.queryByRole("button", { name: /save this document to enable running/ })).toBeNull());
  // Save acknowledgement precedes projection and rail layout readiness.
  const runs = await waitFor(() => {
    const ready = screen.getAllByRole("button", { name: /click to run it/ });
    expect(ready).toHaveLength(2);
    return ready;
  });
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

// Contract: .hickory-source/guarantees/shell/quiet-inline-workspace/contract.md
it("publishes each background output without a fanout or slowest-output barrier", async () => {
  const requested: string[] = [];
  const replies: ((value: unknown) => void)[] = [];
  installMockHandler(async (_method, path) => {
    if (path === "/api/docs/d") return { id: "d", path: "note.md", source: one };
    if (path === "/api/docs/d/render") return new Promise(() => {});
    if (path === "/api/docs/d/outputs") return { files: ["a.rs", "b.toml", "c.txt"].map(path => ({ path, language: "text" })) };
    if (path.includes("/outputs/file")) {
      requested.push(new URL(path, "http://local").searchParams.get("path")!);
      return new Promise(resolve => { replies.push(resolve); });
    }
    return {};
  });
  const registry = new SessionRegistry();
  const mounted = render(<DocSessionHost registry={registry} docId="d" openGenerated={openGenerated} />);
  await waitFor(() => expect(requested).toEqual(["a.rs"]));
  await act(async () => replies[0]({ path: "a.rs", content: "a", language: "rust", provenance: [] }));
  await waitFor(() => expect(requested).toEqual(["a.rs", "b.toml"]));
  expect(registry.get("d")!.outputs.get("a.rs")?.content).toBe("a");
  expect(registry.get("d")!.outputs.has("b.toml")).toBe(false);
  mounted.unmount();
  await act(async () => replies[1]({ path: "b.toml", content: "b", language: "text", provenance: [] }));
  expect(requested).toEqual(["a.rs", "b.toml"]);
});
