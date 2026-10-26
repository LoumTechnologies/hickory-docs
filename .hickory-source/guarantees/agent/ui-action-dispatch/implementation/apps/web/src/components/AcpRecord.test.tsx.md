<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AcpRecord.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-4fad025b1410"></a>
## `apps/web/src/components/AcpRecord.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-4fad025b1410-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-4fad025b1410-expression_statement">
import { act, cleanup, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { AcpRecord } from "./AcpRecord";
afterEach(cleanup);
</storycode:copy>
<a id="f-4fad025b1410-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-4fad025b1410-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses recorded tool evidence explicitly and retires nested readings when collapsed", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const body = JSON.stringify({ toolCallId: "tool", title: "Read file", status: "completed", rawOutput: "PRIVATE EVIDENCE λ" });
  render(<UiActionContext.Provider value={registry}><AcpRecord kind="acp-activity" body={body} /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0];
  expect(registry.groups().groups).toEqual([parent]);
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE EVIDENCE");
  const open = registry.observe(parent).actions[0];
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: open.handle, arguments: { open: true }, request_id: "open" }); });
  expect(registry.groups().groups.some(g => g.startsWith("adapter-tool-reading:"))).toBe(true);
  const read = registry.observe(parent).actions.find(a => a.label === "Read disclosed evidence bytes")!;
  const reading = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "read" });
  if (reading.status !== "result" || reading.outcome.status !== "completed") throw Error("Expected bytes");
  expect(new TextDecoder().decode(new Uint8Array((reading.outcome.value as { bytes: number[] }).bytes))).toContain("PRIVATE EVIDENCE λ");
  const close = registry.observe(parent).actions.find(a => a.label.startsWith("Set evidence visibility"))!;
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: close.handle, arguments: { open: false }, request_id: "close" }); });
  expect(registry.groups().groups).toEqual([parent]);
  expect(registry.observe(parent).actions).toHaveLength(1);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: read.handle, arguments: { offset: 0, limit: 8192 }, request_id: "old" })).toMatchObject({ status: "unavailable" });
});
</storycode:copy>
<a id="f-4fad025b1410-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-4fad025b1410-expression_statement-3">
it("treats malformed/non-object recorded payloads as inspectable text and labels stopped neutrally", () => {
  const view = render(<AcpRecord kind="acp-activity" body="null" />);
  expect(screen.getByText("Agent activity")).toBeTruthy();
  view.rerender(<AcpRecord kind="acp-turn-status" body='{"status":"stopped"}' />);
  expect(screen.getByText("Turn stopped")).toBeTruthy();
  expect(screen.queryByText("Stopped by you")).toBeNull();
});
</storycode:copy>

### Assemble `apps/web/src/components/AcpRecord.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AcpRecord.test.tsx"><storycode:paste select="#f-4fad025b1410-expression_statement"/><storycode:paste select="#f-4fad025b1410-expression_statement-2"/><storycode:paste select="#f-4fad025b1410-expression_statement-3"/></storycode:file>

</storycode:doc>
