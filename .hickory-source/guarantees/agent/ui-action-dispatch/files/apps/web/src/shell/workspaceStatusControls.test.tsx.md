<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/workspaceStatusControls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `83a18bbd4c734a4e9822d88e41956e03a5fcd2da1ad8494a87b9067a338d69ec`.

<book:entry path="apps/web/src/shell/workspaceStatusControls.test.tsx">
import { useState } from "react";
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { StatusBar } from "./StatusBar";
import { ProvenanceToggles } from "./ProvenanceToggles";
import type { ProvenanceLayer } from "../lib/provenanceLayers";
import { WorkspaceStatusExtras } from "../environments/WorkspaceStatusExtras";
import { SpatialFileEdges } from "./SpatialFileEdges";
afterEach(cleanup);
function controls() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let id = 0;
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  const run = (label: string) => registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action(label).handle, arguments: {}, request_id: String(++id) });
  return { registry, action, run };
}
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares status and project navigation handlers and retires replaced status handles", async () => {
  const c = controls(), git = vi.fn(), problems = vi.fn(), attention = vi.fn(), environments = vi.fn(), graph = vi.fn();
  const ui = (path: string, projectId: string | null) => <UiActionContext.Provider value={c.registry}>
    <StatusBar problems={{ errors: 1, warnings: 0 }} needsAttention={1} path={path} zoom={1} git={{ branch: "main", dirty: 2 }} onGit={git} onProblems={problems} onAttention={attention} />
    <WorkspaceStatusExtras provenance={{ layers: new Set(), onToggle: () => {} }} environmentCount={2} onEnvironments={environments} projectId={projectId} onGraph={graph} />
  </UiActionContext.Provider>;
  const rendered = render(ui("a.md", "first"));
  await act(async () => { expect(await c.run("Show status branch history")).toMatchObject({ outcome: { value: { view_requested: true } } }); await c.run("Show status problems"); await c.run("Go to next waiting terminal"); await c.run("Show workspace environments"); await c.run("Show workspace lineage graph"); });
  expect([git.mock.calls.length, problems.mock.calls.length, attention.mock.calls.length, environments.mock.calls.length, graph.mock.calls.length]).toEqual([1, 1, 1, 1, 1]);
  await act(async () => fireEvent.click(screen.getByRole("button", { name: "Branch main, 2 changed" })));
  expect(git).toHaveBeenCalledTimes(2);
  const old = c.action("Show status branch history"), oldGraph = c.action("Show workspace lineage graph");
  rendered.rerender(ui("b.md", null));
  expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "stale" });
  expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: oldGraph.handle, arguments: {}, request_id: "old-graph" })).toMatchObject({ status: "unavailable" });
});
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares provenance toggles and never invents an unavailable project continuity control", async () => {
  const c = controls(), continuity = vi.fn();
  function Harness({ enabled }: { enabled?: boolean }) {
    const [layers, setLayers] = useState<ReadonlySet<ProvenanceLayer>>(new Set(["lineage"]));
    return <ProvenanceToggles layers={layers} onToggle={layer => setLayers(new Set(layers.has(layer) ? [...layers].filter(value => value !== layer) : [...layers, layer]))} continuity={enabled} onContinuity={continuity} />;
  }
  const ui = (enabled?: boolean) => <UiActionContext.Provider value={c.registry}><Harness enabled={enabled} /></UiActionContext.Provider>;
  const rendered = render(ui());
  expect(c.action("Toggle Continuity provenance")).toBeUndefined();
  const old = c.action("Toggle Lineage provenance");
  await act(async () => { await c.run("Toggle Lineage provenance"); });
  expect(screen.getByRole("button", { name: "Lineage" }).getAttribute("aria-pressed")).toBe("false");
  expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "stale" });
  rendered.rerender(ui(false));
  await act(async () => { await c.run("Toggle Continuity provenance"); });
  expect(continuity).toHaveBeenCalledWith(true);
  await act(async () => fireEvent.click(screen.getByRole("button", { name: "Context" })));
  expect(screen.getByRole("button", { name: "Context" }).getAttribute("aria-pressed")).toBe("true");
});
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("scrolls the actual mounted file editor and refuses old viewport and replaced owner handles", async () => {
  const c = controls();
  const rendered = render(<UiActionContext.Provider value={c.registry}><div data-spatial-files><div className="filesystem-editor" data-testid="editor" /><SpatialFileEdges /></div></UiActionContext.Provider>);
  const create = () => {
    const view = new EditorView({ parent: screen.getByTestId("editor"), state: EditorState.create({ doc: "x\n".repeat(100) }) });
    Object.defineProperty(view.scrollDOM, "clientHeight", { configurable: true, value: 100 });
    vi.spyOn(view, "lineBlockAtHeight").mockImplementation(height => ({ from: Math.min(200, Math.floor(height / 10) * 2), to: Math.min(200, Math.floor(height / 10) * 2) }) as ReturnType<EditorView["lineBlockAtHeight"]>);
    return view;
  };
  let view = create();
  try {
    const down = c.action("Scroll file names down");
    await act(async () => { expect(await c.run("Scroll file names down")).toMatchObject({ outcome: { value: { before: 0, after: 108 } } }); });
    expect(view.scrollDOM.scrollTop).toBe(108);
    expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: down.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "stale" });
    const oldOwner = c.action("Scroll file names down"); view.destroy(); view = create(); view.scrollDOM.scrollTop = 108;
    expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: oldOwner.handle, arguments: {}, request_id: "old-owner" })).toMatchObject({ status: "stale" });
    view.destroy();
    expect(c.action("Scroll file names down").enabled).toBe(false);
  } finally { if (view.dom.isConnected) view.destroy(); rendered.unmount(); }
});
</book:entry>

</book:doc>
