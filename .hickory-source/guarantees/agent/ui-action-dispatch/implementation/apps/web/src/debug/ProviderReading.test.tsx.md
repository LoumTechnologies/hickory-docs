<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/debug/ProviderReading.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9b03d99a7bfc"></a>
## `apps/web/src/debug/ProviderReading.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-9b03d99a7bfc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9b03d99a7bfc-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { ProviderReading } from "./ProviderReading";
import { providerObservationContext, type ProviderReading as Reading } from "./providerObservations";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-9b03d99a7bfc-reading"></a>
### `reading`

<storycode:copy id="f-9b03d99a7bfc-reading">
const reading: Reading = { schema_version: 1, source: { id: "source", uri: "doc.md", revision: "revision", bytes: [120, 10] }, unsupported: [], annotations: [{
  evidence: "review-evidence", provider: "github-pr-threads", provider_version: "v1", payload_schema: "raw-review.v1", source_precision: "exact", measurement_semantics: null, uncertainty: null,
  origin: { kind: "review", repository: "owner/repo", pull_request: 1, thread: "thread", base_revision: "base", head_revision: "head", side: "RIGHT", author: "reviewer", resolved: false, outdated: false },
  location: { id: "location", revision: "revision", line: 0, location: { source: "doc.md", bytes: { start: 0, end: 1 }, precision: "exact" } },
  payload: { mapping_state: "exact", thread: { comments: [{ id: "comment", body: "VISIBLE COMMENT", diffHunk: "HIDDEN DIFF", url: "https://github.com/owner/repo/pull/1#comment" }] }, private_lineage: "HIDDEN LINEAGE", reviewed_source: { uri: "https://github.com/owner/repo/blob/head/doc.md" } },
}] };
</storycode:copy>
<a id="f-9b03d99a7bfc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9b03d99a7bfc-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares pinning and nested evidence disclosure without adding hidden lineage to selected context", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const opened = vi.spyOn(window, "open").mockReturnValue(null);
  render(<UiActionContext.Provider value={registry}><ProviderReading reading={reading} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
  let id = 0;
  const run = async (label: string, args = {}, subject?: string) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label && (!subject || a.subject === subject))!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  const read = async (subject?: string) => {
    const reply = await run("Read disclosed evidence bytes", { offset: 0, limit: 8192 }, subject) as { outcome: { value: { bytes: number[] } } };
    return new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes));
  };
  expect(providerObservationContext("doc.md", "x\n")).toEqual([]);
  expect(actions().some(a => a.label.startsWith("Complete external"))).toBe(false);
  await run("Set provider evidence pin", { pinned: true });
  expect((screen.getByRole("checkbox") as HTMLInputElement).checked).toBe(true);
  const context = JSON.stringify(providerObservationContext("doc.md", "x\n"));
  expect(context).toContain("VISIBLE COMMENT"); expect(context).not.toContain("HIDDEN"); expect(context).toContain("hickory.github-review-visible.v1");
  await run("Set provider evidence pin", { pinned: false });
  expect(providerObservationContext("doc.md", "x\n")).toEqual([]);
  await run("Set evidence visibility: Provider observations · captured", { open: true });
  const outer = actions().find(a => a.label === "Set evidence visibility: Provider observations · captured")!.subject;
  expect(await read(outer)).toContain("VISIBLE COMMENT"); expect(await read(outer)).not.toContain("HIDDEN");
  const diff = actions().find(a => a.label === "Set evidence visibility: Original reviewed diff · comment")!.subject;
  await run("Set evidence visibility: Original reviewed diff · comment", { open: true });
  expect(await read(diff)).toContain("HIDDEN DIFF");
  const lineage = actions().find(a => a.label === "Set evidence visibility: Complete external evidence lineage")!.subject;
  await run("Set evidence visibility: Complete external evidence lineage", { open: true });
  expect(await read(lineage)).toContain("HIDDEN LINEAGE");
  expect(JSON.stringify(providerObservationContext("doc.md", "x\n"))).not.toContain("HIDDEN");
  fireEvent.click(screen.getByRole("link", { name: "Exact GitHub discussion · comment" }));
  expect(opened).toHaveBeenCalledWith("https://github.com/owner/repo/pull/1#comment", "_blank", "noopener,noreferrer");
  expect(await run("Exact GitHub discussion · comment")).toMatchObject({ status: "result", outcome: { status: "completed", value: { navigation_requested: true, tab_handle_returned: false } } });
  await run("Set evidence visibility: Provider observations · captured", { open: false });
  expect(actions().some(a => a.label === "Exact GitHub discussion · comment")).toBe(false);
  expect(providerObservationContext("doc.md", "x\n")).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/debug/ProviderReading.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/debug/ProviderReading.test.tsx"><storycode:paste select="#f-9b03d99a7bfc-expression_statement"/><storycode:paste select="#f-9b03d99a7bfc-reading"/><storycode:paste select="#f-9b03d99a7bfc-expression_statement-2"/></storycode:file>

</storycode:doc>
