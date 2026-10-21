// Guarantees: docs/guarantees/embedding/a-viewer-snapshot-keeps-recorded-results-historical.md
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { SnapshotPage } from "./SnapshotPage";
import { UiActionContext } from "../lib/uiActionBridge";
import { UiActions } from "../lib/uiActions";
import { sha256 } from "../lib/reviews";
vi.mock("./PublishedSnapshot", async importOriginal => {
  const actual = await importOriginal<typeof import("./PublishedSnapshot")>();
  return { ...actual, PublishedSnapshot: ({ snapshot }: { snapshot: { path: string } }) => <p>Captured {snapshot.path}</p> };
});
afterEach(() => { cleanup(); vi.unstubAllGlobals(); window.history.replaceState(null, "", "/"); });
async function fixture(path = "spec.md") {
  const source = `# ${path}`;
  return { format: "hickory-viewer-snapshot", version: 1, path, source, source_hash: await sha256(source), captured_at_ms: 1, execution: "fixture", cells: [] };
}
function setup() {
  const registry = new UiActions({ workspace: "static", window: "viewer" }, () => true, async () => {});
  const app = render(<UiActionContext.Provider value={registry}><SnapshotPage /></UiActionContext.Provider>);
  const run = async (label: string, args: Record<string, string> = {}) => {
    const action = registry.observe("publication-viewer", 64).actions.find(action => action.label === label)!;
    let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  return { registry, app, run };
}
it("shares validated native/agent import and refuses invalid source without replacing retained evidence", async () => {
  const captured = await fixture(), { run } = setup();
  expect(await run("Import exported snapshot JSON", { json: JSON.stringify(captured) })).toMatchObject({ status: "result", outcome: { status: "completed", value: { historical: true, execution_requested: false } } });
  expect(screen.getByText("Captured spec.md")).toBeTruthy();
  expect(await run("Import exported snapshot JSON", { json: JSON.stringify({ ...captured, source: "tampered" }) })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(screen.getByText("Captured spec.md")).toBeTruthy();
  const next = await fixture("native.md");
  fireEvent.change(screen.getByLabelText("Open exported snapshot"), { target: { files: [{ text: async () => JSON.stringify(next) }] } });
  await screen.findByText("Captured native.md");
});
it("cancels a pending fetch without adopting a late response and rejects external URLs before fetching", async () => {
  let finish!: (value: Response) => void;
  const fetcher = vi.fn(() => new Promise<Response>(resolve => { finish = resolve; })); vi.stubGlobal("fetch", fetcher);
  const { registry, run } = setup();
  expect(await run("Load same-origin publication snapshot", { path: "https://external.example/capture.json" })).toMatchObject({ outcome: { status: "unavailable" } });
  expect(fetcher).not.toHaveBeenCalled();
  const action = registry.observe("publication-viewer", 64).actions.find(action => action.label === "Load same-origin publication snapshot")!;
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "parent", kind: "agent" }, { handle: action.handle, arguments: { path: "/capture.json" }, request_id: "pending" }); });
  expect(await run("Cancel publication loading")).toMatchObject({ outcome: { value: { cancellation_requested: true } } });
  const snapshot = await fixture();
  await act(async () => { finish(new Response(JSON.stringify(snapshot))); expect(await pending).toMatchObject({ outcome: { status: "unavailable" } }); });
  expect(screen.queryByText("Captured spec.md")).toBeNull();
});

it("binds index navigation to the selected source and tested revision", async () => {
  const captured = await fixture(), revision = "a".repeat(40);
  const index = { version: 1, revision, documents: [{ path: "spec.md", status: "passed", snapshot: "capture.json", revision, source_hash: captured.source_hash }] };
  const wrong = { ...captured, verification: { status: "passed", revision: "b".repeat(40), source_hash: captured.source_hash, dependencies: {}, assets: {}, environment: {}, commands: [], requirements: [], origin: "fixture", scope: "fixture" } };
  const fetcher = vi.fn().mockResolvedValueOnce(new Response(JSON.stringify(index))).mockResolvedValueOnce(new Response(JSON.stringify(wrong))); vi.stubGlobal("fetch", fetcher);
  const { run } = setup();
  expect(await run("Load same-origin publication index", { path: "/publications/index.json" })).toMatchObject({ outcome: { status: "completed" } });
  expect(await run("Open published document 0: spec.md")).toMatchObject({ outcome: { status: "unavailable", reason: expect.stringContaining("identities differ") } });
  expect(screen.queryByText("Captured spec.md")).toBeNull();
});
