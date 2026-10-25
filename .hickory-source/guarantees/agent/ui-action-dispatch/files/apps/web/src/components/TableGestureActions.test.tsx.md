<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/components/TableGestureActions.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `dc6117c9f024695f0660058287e85e0658538ee4a05a16cd181b358cf85da3ef`.

<book:entry path="apps/web/src/components/TableGestureActions.test.tsx">
import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TablePanel } from "./TablePanel";
afterEach(cleanup);
function controls() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {}); let id = 0;
  const action = (label: string) => registry.groups().groups.flatMap(group => registry.observe(group).actions).find(action => action.label === label)!;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result!: Awaited<ReturnType<typeof registry.act>>; await act(async () => { result = await registry.act(registry.scope, { id: "agent", kind: "agent" }, { handle: action(label).handle, arguments: args, request_id: String(++id) }); }); return result; };
  return { registry, action, run };
}
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("uses native table paste parsing with agent attribution and refuses stale/OS cut without deleting", async () => {
  const c = controls(), onChange = vi.fn(); let current = true;
  render(<UiActionContext.Provider value={c.registry}><TablePanel source={"a,b\nc,d\n"} onChange={onChange} sourceCurrent={() => current} /></UiActionContext.Provider>);
  await c.run("Select table cell or range", { row: 0, column: 0, extend: false });
  const payload = { text: "ignored", html: "<table><tr><td>hello</td><td>world</td></tr></table>" };
  expect(await c.run("Paste table clipboard contents", payload)).toMatchObject({ outcome: { value: { source_write_requested: true, saved: false } } });
  expect(onChange).toHaveBeenLastCalledWith("hello,world\nc,d\n", undefined, { id: "agent", kind: "agent" });
  const table = document.querySelector(".table-panel")!;
  fireEvent.paste(table, { clipboardData: { getData: (kind: string) => kind === "text/html" ? payload.html : kind === "text/plain" ? payload.text : "" } });
  expect(onChange).toHaveBeenLastCalledWith("hello,world\nc,d\n");
  const calls = onChange.mock.calls.length;
  expect(await c.run("Cut selected table cells")).toMatchObject({ outcome: { status: "requires_input" } });
  expect(onChange).toHaveBeenCalledTimes(calls);
  const old = c.action("Paste table clipboard contents"); current = false;
  expect(await c.registry.act(c.registry.scope, { id: "agent", kind: "agent" }, { handle: old.handle, arguments: payload, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(onChange).toHaveBeenCalledTimes(calls);
});
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("edits an actual formula draft, points its range, resizes and scrolls the actual viewport", async () => {
  const c = controls(), onChange = vi.fn(), onLayout = vi.fn();
  render(<UiActionContext.Provider value={c.registry}><TablePanel source={"a,b\nc,d\ne,f\n"} language="python" onChange={onChange} onLayout={onLayout} /></UiActionContext.Provider>);
  await c.run("Begin table cell edit", { row: 2, column: 0, input: "grid" });
  await c.run("Change table cell edit draft", { value: "=" });
  await c.run("Select table formula draft range", { anchor: 1, head: 1 });
  expect(await c.run("Point table formula to cell range", { row: 0, column: 0, end_row: 1, end_column: 1 })).toMatchObject({ outcome: { value: { draft_changed: true, committed: false } } });
  expect((document.querySelector(".table-panel__input") as HTMLInputElement).value).toBe("=A1:B2");
  expect(onChange).not.toHaveBeenCalled();
  await c.run("Cancel pending table cell edit");
  await c.run("Resize table viewport height", { pixels: 240 });
  expect(onLayout).toHaveBeenLastCalledWith({ height: 240 });
  const viewport = screen.getByTestId("table-panel-scroll");
  expect(viewport.style.height).toBe("240px");
  expect(await c.run("Scroll table viewport", { top: 30, left: 15 })).toMatchObject({ outcome: { value: { top: 30, left: 15, source_written: false } } });
  expect(viewport.scrollTop).toBe(30); expect(viewport.scrollLeft).toBe(15);
  expect(await c.run("Read table viewport")).toMatchObject({ outcome: { value: { top: 30, left: 15 } } });
  expect(onChange).not.toHaveBeenCalled();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reports real rich clipboard success and denial without mutating table source", async () => {
  const c = controls(), onChange = vi.fn();
  const previous = Object.getOwnPropertyDescriptor(navigator, "clipboard");
  const write = vi.fn().mockRejectedValueOnce(new Error("denied")).mockResolvedValue(undefined);
  class Item { constructor(public data: Record<string, Blob>) {} }
  vi.stubGlobal("ClipboardItem", Item);
  Object.defineProperty(navigator, "clipboard", { configurable: true, value: { write } });
  try {
    render(<UiActionContext.Provider value={c.registry}><TablePanel source={"a,b\nc,d\n"} onChange={onChange} /></UiActionContext.Provider>);
    await c.run("Select table cell or range", { row: 0, column: 0, extend: false });
    expect(await c.run("Copy selected table cells")).toMatchObject({ outcome: { status: "requires_input" } });
    expect(await c.run("Copy selected table cells")).toMatchObject({ outcome: { value: { copied: true, source_written: false } } });
    const item = write.mock.calls[1][0][0] as Item;
    expect(item.data["text/plain"].size).toBe(1);
    expect(item.data["text/html"].type).toBe("text/html");
    expect(onChange).not.toHaveBeenCalled();
  } finally { if (previous) Object.defineProperty(navigator, "clipboard", previous); else Reflect.deleteProperty(navigator, "clipboard"); vi.unstubAllGlobals(); }
});
</book:entry>

</book:doc>
