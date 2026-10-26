<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/documentSaveRender.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-555185246554"></a>
## `apps/web/src/views/documentSaveRender.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-555185246554-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement">
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
</storycode:copy>
<a id="f-555185246554-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement-2">
vi.mock("mermaid", () => ({ default: { initialize: vi.fn(), render: vi.fn() } }));
</storycode:copy>
<a id="f-555185246554-debug"></a>
### `debug`

<storycode:copy id="f-555185246554-debug">
const debug = { status: "idle", breakpoints: [], frames: [], variables: [], watches: [], pausedLine: null };
</storycode:copy>
<a id="f-555185246554-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement-3">
vi.mock("../debug/useDebugger", () => ({ useDebugger: () => debug }));
</storycode:copy>
<a id="f-555185246554-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement-4">
afterEach(() => { cleanup(); installMockHandler(null as never); vi.unstubAllGlobals(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-555185246554-prefix"></a>
### `prefix`

<storycode:copy id="f-555185246554-prefix">
const prefix = '<hick:container name="shell" image="alpine:3.20" />\n';
</storycode:copy>
<a id="f-555185246554-one"></a>
### `one`

<storycode:copy id="f-555185246554-one">
const one = prefix + '<hick:exec container="shell">echo one</hick:exec>\n';
</storycode:copy>
<a id="f-555185246554-two"></a>
### `two`

<storycode:copy id="f-555185246554-two">
const two = one + '<hick:exec container="shell">echo two</hick:exec>\n';
</storycode:copy>
<a id="f-555185246554-receipt"></a>
### `receipt`

<storycode:copy id="f-555185246554-receipt">
async function receipt(source: string) {
  const digest = await webcrypto.subtle.digest("SHA-256", new TextEncoder().encode(source));
  const source_hash = `sha256:${[...new Uint8Array(digest)].map(byte => byte.toString(16).padStart(2, "0")).join("")}`;
  const blocks = [...source.matchAll(/<hick:exec[^>]*>(.*?)<\/hick:exec>/g)].map((m, index) => ({ kind: "exec", id: `cell${index + 1}`, command: m[1], container: "shell", span: [m.index!, m.index! + m[0].length], status: "unrecorded", transcript: [] }));
  return { document: "d", source_hash, blocks };
}
</storycode:copy>
<a id="f-555185246554-openGenerated"></a>
### `openGenerated`

<storycode:copy id="f-555185246554-openGenerated">
const openGenerated = () => {};
</storycode:copy>
<a id="f-555185246554-Harness"></a>
### `Harness`

<storycode:copy id="f-555185246554-Harness">
function Harness({ registry }: { registry: SessionRegistry }) {
  useSessionVersion(registry);
  const session = registry.get("d");
  return <><DocSessionHost registry={registry} docId="d" openGenerated={openGenerated} />
    {session?.doc && <DocumentEditor docId="d" initialSource={session.doc.source} realtime={session.realtime} execBlocks={session.execBlocks} runningCells={session.runningCells}
      onRunCell={session.runCell} onChange={session.setDirtySource} onViewReady={session.onDocViewReady} />}</>;
}

</storycode:copy>
<a id="f-555185246554-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement-5">
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


</storycode:copy>
<a id="f-555185246554-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-555185246554-expression_statement-6">
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
</storycode:copy>

### Assemble `apps/web/src/views/documentSaveRender.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/documentSaveRender.test.tsx"><storycode:paste select="#f-555185246554-expression_statement"/><storycode:paste select="#f-555185246554-expression_statement-2"/><storycode:paste select="#f-555185246554-debug"/><storycode:paste select="#f-555185246554-expression_statement-3"/><storycode:paste select="#f-555185246554-expression_statement-4"/><storycode:paste select="#f-555185246554-prefix"/><storycode:paste select="#f-555185246554-one"/><storycode:paste select="#f-555185246554-two"/><storycode:paste select="#f-555185246554-receipt"/><storycode:paste select="#f-555185246554-openGenerated"/><storycode:paste select="#f-555185246554-Harness"/><storycode:paste select="#f-555185246554-expression_statement-5"/><storycode:paste select="#f-555185246554-expression_statement-6"/></storycode:file>

</storycode:doc>
