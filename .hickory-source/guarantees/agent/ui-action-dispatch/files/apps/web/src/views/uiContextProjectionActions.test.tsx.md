<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/uiContextProjectionActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `29b827876f1052c37edcd505a9d8a2eca37475d3f9b9bd2d63f7e800499abb03`.

<book:entry path="apps/web/src/views/uiContextProjectionActions.test.tsx">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useContextProjectionActions } from "./uiContextProjectionActions";
afterEach(cleanup);
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
</book:entry>

</book:doc>
