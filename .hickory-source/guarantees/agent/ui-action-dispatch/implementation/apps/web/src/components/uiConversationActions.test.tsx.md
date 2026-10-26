<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/uiConversationActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-f7ffa11bc8cc"></a>
## `apps/web/src/components/uiConversationActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-f7ffa11bc8cc-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-f7ffa11bc8cc-expression_statement">
import { act, cleanup, render } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useConversationActions } from "./uiConversationActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-f7ffa11bc8cc-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-f7ffa11bc8cc-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("returns a real queued turn, deduplicates Send, observes state explicitly and requests stop", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const send = vi.fn(async () => "actual-turn"), stop = vi.fn(async () => true);
  function Harness() {
    const [draft, setDraft] = useState(""); const [running, setRunning] = useState<string | null>(null);
    const [tip, setTip] = useState<string | null>(null); const [tree, setTree] = useState(false);
    useConversationActions({ doc: "workspace", draft, tip, turns: [{ id: "old-turn" }], running, sending: false, stopping: false, ready: true, tree, collapsed: false, evidence: { answer: "PRIVATE ANSWER λ" }, setDraft, discard: () => setDraft(""), setTip, setTree,
      send: async () => { const id = await send(); setRunning(id); return id; }, stop });
    return null;
  }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0]; let n = 0;
  async function invoke(label: string, args: Record<string, UiValue>) {
    const descriptor = registry.observe(parent).actions.find(a => a.label === label)!;
    let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: descriptor.handle, arguments: args, request_id: String(++n) }); });
    return result;
  }
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE");
  expect(await invoke("Select conversation turn", { id: "forged" })).toMatchObject({ outcome: { status: "unavailable" } });
  await invoke("Set conversation response draft", { text: "Investigate the deadline" });
  const descriptor = registry.observe(parent).actions.find(a => a.label === "Send conversation response")!;
  const invocation = { handle: descriptor.handle, arguments: {}, request_id: "send-once" };
  await act(async () => {
    const first = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, invocation);
    expect(first).toMatchObject({ outcome: { status: "queued", job: "actual-turn" } });
    expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, invocation)).toEqual(first);
  });
  expect(send).toHaveBeenCalledOnce();
  expect(registry.observe(parent).actions.find(a => a.label === "Send conversation response")!.enabled).toBe(false);
  expect(await invoke("Stop conversation agent", {})).toMatchObject({ outcome: { value: { turn: "actual-turn", stop_requested: true, stopped: "Await actual terminal turn state" } } });
  expect(stop).toHaveBeenCalledOnce();
  const read = await invoke("Read conversation state bytes", { offset: 0, limit: 8192 });
  if (read?.status !== "result" || read.outcome.status !== "completed") throw Error("Expected reading");
  expect(new TextDecoder().decode(new Uint8Array((read.outcome.value as { bytes: number[] }).bytes))).toContain("PRIVATE ANSWER λ");
});
</storycode:copy>

### Assemble `apps/web/src/components/uiConversationActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/uiConversationActions.test.tsx"><storycode:paste select="#f-f7ffa11bc8cc-expression_statement"/><storycode:paste select="#f-f7ffa11bc8cc-expression_statement-2"/></storycode:file>

</storycode:doc>
