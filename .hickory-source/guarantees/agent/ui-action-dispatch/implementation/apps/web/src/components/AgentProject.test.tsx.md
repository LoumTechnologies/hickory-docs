<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AgentProject.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-5a918691d171"></a>
## `apps/web/src/components/AgentProject.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-5a918691d171-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { installMockHandler } from "../api/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { AgentProject } from "./AgentProject";
vi.mock("./ComparisonEditor", () => ({ ComparisonEditor: () => null }));
</storycode:copy>
<a id="f-5a918691d171-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement-2">
vi.mock("./ChangesetDocument", () => ({ ChangesetDocument: () => <p>Actual artifact rendering is covered separately.</p> }));
</storycode:copy>
<a id="f-5a918691d171-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement-3">
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-5a918691d171-capture"></a>
### `capture`

<storycode:copy id="f-5a918691d171-capture">
const capture = (ambiguous: string[] = [], response: string | null = "PRIVATE ARTIFACT") => ({ entry: "project.md", ambiguous, error: null, response, starting: { entries: { "app.py": { bytes: [65], kind: "File", executable: false } } }, completed: { entries: { "app.py": { bytes: [66], kind: "File", executable: false } } } });
</storycode:copy>
<a id="f-5a918691d171-setup"></a>
### `setup`

<storycode:copy id="f-5a918691d171-setup">
function setup() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  return { registry, actions, run };
}
</storycode:copy>
<a id="f-5a918691d171-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement-4">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("requires real human attribution and shares actual finalization without leaking artifact bytes", async () => {
  const post = vi.fn();
  installMockHandler(async (method, _path, body) => { if (method === "GET") return capture(["app.py"], null); post(body); return { response: "PRIVATE VERIFIED ARTIFACT" }; });
  const { registry, actions, run } = setup();
  render(<UiActionContext.Provider value={registry}><AgentProject doc="doc" turn="turn" /></UiActionContext.Provider>);
  await screen.findByRole("checkbox");
  expect(await run("Set human project attribution choice", { path: "app.py", selected: true })).toMatchObject({ status: "result", outcome: { status: "requires_input", request: expect.any(String) } });
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(false); expect(post).not.toHaveBeenCalled();
  await act(async () => fireEvent.click(screen.getByRole("checkbox")));
  await waitFor(() => expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(true));
  expect(await run("Verify project A-to-B explanation")).toMatchObject({ status: "result", outcome: { status: "completed", value: { source_equality_verified: true, runtime_verified: false } } });
  expect(post).toHaveBeenCalledWith({ resolve: ["app.py"] });
  expect(JSON.stringify(actions())).not.toContain("PRIVATE");
});

</storycode:copy>
<a id="f-5a918691d171-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement-5">
it("retains partial disk/editor publication and receipt-recording failures without calling it success", async () => {
  const reports = vi.fn();
  installMockHandler(async (method, path, body) => {
    if (method === "GET") return capture();
    if (path.endsWith("/draft/apply")) return { applied: ["app.py"], error: null, undo: [{ path: "app.py", before: "PRIVATE UNDO" }], editors: [{ name: "first", buffer: "first" }, { name: "second", buffer: "second" }] };
    if (path.endsWith("/draft/result")) { reports(body); throw new Error("receipt storage offline"); }
    return {};
  });
  const apply = vi.fn().mockRejectedValue(new Error("captured editor changed"));
  const { registry, actions, run } = setup();
  render(<UiActionContext.Provider value={registry}><AgentProject doc="doc" turn="turn" getContext={async () => ({ buffers: [] })} applyChange={apply} /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Revise New" });
  await run("Revise completed project New");
  expect(await run("Apply authored project revision")).toMatchObject({ status: "result", outcome: { status: "completed", value: { application_succeeded: false, applied_disk: ["app.py"], record_error: expect.stringContaining("receipt storage offline"), editor_results: [{ buffer: "first", ok: false }, { buffer: "second", ok: false }], runtime_verified: false } } });
  expect(apply).toHaveBeenCalledTimes(1); expect(reports).toHaveBeenCalledTimes(1);
  expect(actions().some(a => a.label === "Read disclosed evidence bytes")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain("PRIVATE UNDO");
  await run("Set evidence visibility: Application and recovery receipt", { open: true });
  const reply = await run("Read disclosed evidence bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))).toContain("PRIVATE UNDO");
  await run("Discard authored project draft");
  expect(apply).toHaveBeenCalledTimes(1); // Discard is not rollback.
});

</storycode:copy>
<a id="f-5a918691d171-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-5a918691d171-expression_statement-6">
it("refuses application when the owning conversation changes while capturing editor context", async () => {
  const applied = vi.fn();
  installMockHandler(async (method, path) => { if (method === "GET") return capture(); if (path.endsWith("/draft/apply")) applied(); return {}; });
  let complete!: (value: { buffers: [] }) => void;
  const context = vi.fn(() => new Promise<{ buffers: [] }>(done => { complete = done; }));
  const { registry, actions, run } = setup();
  const mounted = render(<UiActionContext.Provider value={registry}><AgentProject doc="doc" turn="turn" getContext={context} /></UiActionContext.Provider>);
  await screen.findByRole("button", { name: "Revise New" });
  await run("Revise completed project New");
  const apply = actions().find(a => a.label === "Apply authored project revision")!;
  let pending!: ReturnType<typeof registry.act>;
  await act(async () => { pending = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: apply.handle, arguments: {}, request_id: "pending" }); });
  await waitFor(() => expect(context).toHaveBeenCalledTimes(1));
  mounted.rerender(<UiActionContext.Provider value={registry}><AgentProject doc="other" turn="other-turn" getContext={context} /></UiActionContext.Provider>);
  await act(async () => complete({ buffers: [] }));
  expect(await pending).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("owning conversation changed") } });
  expect(applied).not.toHaveBeenCalled();
  await waitFor(() => expect((screen.getByRole("button", { name: "Revise New" }) as HTMLButtonElement).disabled).toBe(false));
});
</storycode:copy>

### Assemble `apps/web/src/components/AgentProject.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AgentProject.test.tsx"><storycode:paste select="#f-5a918691d171-expression_statement"/><storycode:paste select="#f-5a918691d171-expression_statement-2"/><storycode:paste select="#f-5a918691d171-expression_statement-3"/><storycode:paste select="#f-5a918691d171-capture"/><storycode:paste select="#f-5a918691d171-setup"/><storycode:paste select="#f-5a918691d171-expression_statement-4"/><storycode:paste select="#f-5a918691d171-expression_statement-5"/><storycode:paste select="#f-5a918691d171-expression_statement-6"/></storycode:file>

</storycode:doc>
