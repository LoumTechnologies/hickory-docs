<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/BisectPane.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-8c59f0b8becf"></a>
## `apps/web/src/views/BisectPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-8c59f0b8becf-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-8c59f0b8becf-expression_statement">
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { bisects, type BisectSession } from "../api/representations";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { BisectPane } from "./BisectPane";
afterEach(() => { cleanup(); vi.restoreAllMocks(); });
</storycode:copy>
<a id="f-8c59f0b8becf-session"></a>
### `session`

<storycode:copy id="f-8c59f0b8becf-session">
const session: BisectSession = { id: "s1", worktree: "/tmp/candidate", good: "a", bad: "c", candidate: "b", outcome: null, modified: false, said: "Host bisect output", commits: "PRIVATE REMAINING COMMITS", history: [], graph: [{ sha: "b", parents: [], subject: "candidate" }] };
</storycode:copy>
<a id="f-8c59f0b8becf-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-8c59f0b8becf-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("discloses bisect state progressively and shares real candidate requests while preserving verdict authority", async () => {
  vi.spyOn(bisects, "list").mockResolvedValue({ sessions: [session] });
  const mark = vi.spyOn(bisects, "mark").mockResolvedValue(session);
  const open = vi.spyOn(bisects, "open").mockResolvedValue({ opened: true });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><BisectPane /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  await waitFor(() => expect(actions()).toHaveLength(1));
  expect(screen.queryByText("Host bisect output")).toBeNull();
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result as { outcome: { value: { bytes: number[] } } }; };
  await run("Set visual bisect visibility", { open: true });
  await screen.findByText("Host bisect output");
  const reading = await run("Read visual bisect state bytes", { offset: 0, limit: 8192 });
  const text = new TextDecoder().decode(new Uint8Array(reading.outcome.value.bytes));
  expect(text).toContain("Host bisect output"); expect(text).not.toContain("PRIVATE REMAINING COMMITS");
  expect(await run("Classify bisect candidate b as good")).toMatchObject({ status: "result", outcome: { status: "requires_input" } });
  expect(mark).not.toHaveBeenCalled();
  await run("Open bisect candidate b"); expect(open).toHaveBeenCalledWith("s1");
  await run("Classify bisect candidate b as skip"); expect(mark).toHaveBeenCalledWith(session, "skip");
  fireEvent.click(screen.getByRole("button", { name: "Bad" }));
  await waitFor(() => expect(mark).toHaveBeenCalledWith(session, "bad"));
  const old = actions().find(a => a.label === "Open bisect candidate b")!;
  await run("Set visual bisect visibility", { open: false });
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "closed" })).toMatchObject({ status: "unavailable" });
});

</storycode:copy>
<a id="f-8c59f0b8becf-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-8c59f0b8becf-expression_statement-3">
it("refuses candidate classification and ending while a modified experiment needs preserving", async () => {
  const modified = { ...session, modified: true };
  vi.spyOn(bisects, "list").mockResolvedValue({ sessions: [modified] });
  const restore = vi.spyOn(bisects, "restore").mockResolvedValue({ patch: "/tmp/experiment.patch", session });
  render(<BisectPane />);
  fireEvent.click(screen.getByText("Visual bisect"));
  await screen.findByText("Host bisect output");
  expect((screen.getByRole("button", { name: "Good" }) as HTMLButtonElement).disabled).toBe(true);
  expect((screen.getByRole("button", { name: "End bisect" }) as HTMLButtonElement).disabled).toBe(true);
  fireEvent.click(screen.getByRole("button", { name: "Keep experiment as patch and restore candidate" }));
  await screen.findByText("Experiment saved at /tmp/experiment.patch; candidate restored.");
  expect(restore).toHaveBeenCalledWith("s1");
});
</storycode:copy>

### Assemble `apps/web/src/views/BisectPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/BisectPane.test.tsx"><storycode:paste select="#f-8c59f0b8becf-expression_statement"/><storycode:paste select="#f-8c59f0b8becf-session"/><storycode:paste select="#f-8c59f0b8becf-expression_statement-2"/><storycode:paste select="#f-8c59f0b8becf-expression_statement-3"/></storycode:file>

</storycode:doc>
