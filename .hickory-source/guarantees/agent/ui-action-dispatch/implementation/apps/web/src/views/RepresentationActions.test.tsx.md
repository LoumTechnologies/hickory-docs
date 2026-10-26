<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/RepresentationActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-92d47e8f9d7f"></a>
## `apps/web/src/views/RepresentationActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-92d47e8f9d7f-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-92d47e8f9d7f-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { representations, type Representation } from "../api/representations";
import { RepresentationActions } from "./RepresentationActions";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-92d47e8f9d7f-record"></a>
### `record`

<storycode:copy id="f-92d47e8f9d7f-record">
const record: Representation = { id: "view", revision: "r1", backing: { kind: "files", paths: ["code.py"] }, source: "PRIVATE SOURCE", files: [], explanation_stale: false };
</storycode:copy>
<a id="f-92d47e8f9d7f-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-92d47e8f9d7f-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses actions progressively, refuses stale buffers and returns actual keep failures/replies", async () => {
  const registry = new UiActions({ workspace: "project", window: "window" }, () => true, async () => {});
  let source = record.source;
  const keep = vi.spyOn(representations, "keep").mockRejectedValueOnce(new Error("disk unavailable")).mockResolvedValue({ path: "/actual/view.md" });
  const notice = vi.fn();
  render(<UiActionContext.Provider value={registry}><RepresentationActions record={record} dirty={false} busy={false} historical={false} agent={false} live={() => source} save={vi.fn()} refresh={vi.fn()} discard={vi.fn()} toggleAgent={vi.fn()} onNotice={notice} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let request = 0;
  const run = async (label: string) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: {}, request_id: String(++request) }); }); return reply; };
  expect(actions().some(a => a.label === "Keep literate view locally")).toBe(false);
  expect(JSON.stringify(actions())).not.toContain(record.source);
  await run("More actions for literate view view");
  const old = actions().find(a => a.label === "Keep literate view locally")!;
  source = "changed";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(keep).not.toHaveBeenCalled();
  expect(await run("Keep literate view locally")).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "disk unavailable" } });
  expect(await run("Keep literate view locally")).toMatchObject({ status: "result", outcome: { status: "completed", value: { path: "/actual/view.md" } } });
  expect(keep).toHaveBeenCalledTimes(2);
});
</storycode:copy>

### Assemble `apps/web/src/views/RepresentationActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/RepresentationActions.test.tsx"><storycode:paste select="#f-92d47e8f9d7f-expression_statement"/><storycode:paste select="#f-92d47e8f9d7f-record"/><storycode:paste select="#f-92d47e8f9d7f-expression_statement-2"/></storycode:file>

</storycode:doc>
