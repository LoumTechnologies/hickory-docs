<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/uiContextProjectionActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-75b9b8324e45"></a>
## `apps/web/src/views/uiContextProjectionActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-75b9b8324e45-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-75b9b8324e45-expression_statement">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useContextProjectionActions } from "./uiContextProjectionActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-75b9b8324e45-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-75b9b8324e45-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares selection, discloses bounded preview explicitly and retires closed choices", async () => {
  const registry = new UiActions({ workspace: "practice", window: "one" }, () => true, async () => {});
  const capture = vi.fn();
  function Harness() {
    const [open, setOpen] = useState(false);
    const [selection, setSelection] = useState<Record<string, "expanded" | "pinned">>({});
    const actions = useContextProjectionActions({ open, setOpen, tabs: [{ id: "doc", target: "a.md", kind: "document" }], selection, choose: (id, mode) => setSelection(mode === "off" ? {} : { [id]: mode }), preview: { text: "SECRET SELECTED TEXT 😀" }, error: "", refresh: async () => { capture(selection); } });
    return <><output>{selection.doc ?? "off"}</output><button onClick={() => void actions.invoke("choose", { id: "doc", mode: "expanded" })}>Choose expanded</button></>;
  }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups[0]; let n = 0;
  async function invoke(label: string, args: Record<string, UiValue>) {
    const descriptor = registry.observe(parent).actions.find(a => a.label === label)!;
    let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: descriptor.handle, arguments: args, request_id: String(++n) }); });
    return result as UiResult;
  }
  expect(registry.observe(parent).actions).toHaveLength(1);
  await invoke("Set context projection visibility", { open: true });
  expect(JSON.stringify(registry.observe(parent))).not.toContain("SECRET");
  expect(await invoke("Select document context", { id: "closed", mode: "pinned" })).toMatchObject({ outcome: { status: "unavailable" } });
  await invoke("Select document context", { id: "doc", mode: "pinned" });
  expect(screen.getByRole("status").textContent).toBe("pinned");
  await invoke("Capture context projection preview", {});
  expect(capture).toHaveBeenLastCalledWith({ doc: "pinned" });
  const reading = await invoke("Read context projection preview bytes", { offset: 0, limit: 8192 });
  expect(reading.status).toBe("result");
  if (reading.status === "result" && reading.outcome.status === "completed") {
    const value = reading.outcome.value as { bytes: number[] };
    expect(new TextDecoder().decode(new Uint8Array(value.bytes))).toContain("SECRET SELECTED TEXT 😀");
  }
  fireEvent.click(screen.getByText("Choose expanded"));
  await act(async () => {});
  expect(screen.getByRole("status").textContent).toBe("expanded");
  const old = registry.observe(parent).actions.find(a => a.label === "Select document context")!.handle;
  await invoke("Set context projection visibility", { open: false });
  expect(registry.observe(parent).actions).toHaveLength(1);
  expect(await registry.act(registry.scope, { id: "acp:session", kind: "agent" }, { handle: old, arguments: { id: "doc", mode: "off" }, request_id: "closed" })).toMatchObject({ status: "unavailable" });
});
</storycode:copy>

### Assemble `apps/web/src/views/uiContextProjectionActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/uiContextProjectionActions.test.tsx"><storycode:paste select="#f-75b9b8324e45-expression_statement"/><storycode:paste select="#f-75b9b8324e45-expression_statement-2"/></storycode:file>

</storycode:doc>
