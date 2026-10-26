<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/PlainFileActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-ce14f95f9a34"></a>
## `apps/web/src/components/PlainFileActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-ce14f95f9a34-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-ce14f95f9a34-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { openLiterate } from "../lib/openRepresentation";
import { MoreActions } from "./MoreActions";
import { PlainFileActions } from "./PlainFileActions";
vi.mock("../lib/openRepresentation", () => ({ openLiterate: vi.fn().mockResolvedValue(undefined) }));
</storycode:copy>
<a id="f-ce14f95f9a34-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-ce14f95f9a34-expression_statement-2">
afterEach(() => { cleanup(); vi.clearAllMocks(); });
</storycode:copy>
<a id="f-ce14f95f9a34-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-ce14f95f9a34-expression_statement-3">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses menu actions, binds the owning buffer and shares awaited navigation/adoption replies", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  let source = "PRIVATE BUFFER";
  const adopt = vi.fn().mockResolvedValue({ doc_id: "actual-doc", path: "a.md" });
  render(<UiActionContext.Provider value={registry}><MoreActions label="File options"><PlainFileActions path="a.py" loaded generatedBy={null} saveKind="idle" adopting={false} sourceRevision={() => source} adopt={adopt} onError={vi.fn()} /></MoreActions></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  expect(actions().map(a => a.label)).toEqual(["File options"]);
  let id = 0;
  const run = async (label: string) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: {}, request_id: String(++id) }); }); return result; };
  await run("File options");
  const old = actions().find(a => a.label === "Make current file literate")!;
  expect(JSON.stringify(actions())).not.toContain(source);
  source = "CHANGED BUFFER";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old" })).toMatchObject({ status: "stale" });
  expect(adopt).not.toHaveBeenCalled();
  await run("Review current file changes");
  expect(openLiterate).toHaveBeenCalledWith({ kind: "files", paths: ["a.py"] }, "HEAD");
  expect(await run("Make current file literate")).toMatchObject({ status: "result", outcome: { status: "completed", value: { owner_reply: JSON.stringify({ doc_id: "actual-doc", path: "a.md" }) } } });
  fireEvent.click(screen.getByRole("button", { name: "Open literate view" }));
  await act(async () => {});
  expect(actions().map(a => a.label)).toEqual(["File options"]);
  expect(openLiterate).toHaveBeenLastCalledWith({ kind: "files", paths: ["a.py"] });
});
</storycode:copy>

### Assemble `apps/web/src/components/PlainFileActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/PlainFileActions.test.tsx"><storycode:paste select="#f-ce14f95f9a34-expression_statement"/><storycode:paste select="#f-ce14f95f9a34-expression_statement-2"/><storycode:paste select="#f-ce14f95f9a34-expression_statement-3"/></storycode:file>

</storycode:doc>
