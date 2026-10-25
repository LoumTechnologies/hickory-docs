<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/uiConversationActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `cefe6303b99c35a7f25980a026d9a6531b7927654a349ee50719cf85dfb7a797`.

<book:entry path="apps/web/src/components/uiConversationActions.test.tsx">
import { act, cleanup, render } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useConversationActions } from "./uiConversationActions";
afterEach(cleanup);
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
</book:entry>

</book:doc>
