import { act, cleanup, render } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiResult, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { TablePanel } from "./TablePanel";
import { tableView } from "../elements/table/view";
import type { RenderedSlot } from "../editor/rendered";
import type { SlotContext } from "../elements/types";
afterEach(cleanup);
function tools() {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {}); let n = 0;
  return { registry, invoke: async (parent: string, label: string, args: Record<string, UiValue>) => {
    const action = registry.observe(parent).actions.find(a => a.label === label)!; let result: UiResult | undefined;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++n) }); });
    return result as UiResult;
  } };
}
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares guarded cell writes, selection, context menu and fill without leaking data in discovery", async () => {
  const { registry, invoke } = tools(); const writes = vi.fn(); let current = true;
  function Harness() { const [source, setSource] = useState("name,count\nPRIVATE λ,2\nOther,4\n"); return <TablePanel source={source} sourceCurrent={() => current} onChange={(text, formats, actor) => { writes(text, formats, actor); setSource(text); }} />; }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("table:"))!;
  expect(JSON.stringify(registry.observe(parent))).not.toContain("PRIVATE λ");
  await invoke(parent, "Write table cell contents", { row: 1, column: 1, value: "3" });
  expect(writes).toHaveBeenLastCalledWith("name,count\nPRIVATE λ,3\nOther,4\n", undefined, { id: "python:turn", kind: "agent" });
  await invoke(parent, "Select table cell or range", { row: 1, column: 1, extend: false });
  await invoke(parent, "Set table context menu visibility", { open: true });
  expect(registry.observe(parent).actions.some(a => a.label === "Delete row")).toBe(true);
  await invoke(parent, "Fill table selection to cell", { row: 2, column: 1 });
  expect(writes.mock.calls.at(-1)![0]).toContain("Other,3");
  const old = registry.observe(parent).actions.find(a => a.label === "Write table cell contents")!;
  current = false;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { row: 1, column: 1, value: "99" }, request_id: "stale" })).toMatchObject({ status: "stale" });
  expect(writes).toHaveBeenCalledTimes(2);
  const read = await invoke(parent, "Read table cell region bytes", { row: 1, column: 0, rows: 2, columns: 2, offset: 0, limit: 8192 });
  if (read.status !== "result" || read.outcome.status !== "completed") throw Error("Expected cells");
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array((read.outcome.value as { bytes: number[] }).bytes)))).toMatchObject({ write_scope_current: false, total_rows: 3 });
});
it("guards real Markdown-table ownership against an editor change before React rerenders", async () => {
  const { registry, invoke } = tools();
  const source = "| Name | Count |\n| --- | --- |\n| North | 2 |";
  const editor = new EditorView({ parent: document.body, state: EditorState.create({ doc: source }) });
  const replace = vi.fn();
  const slot = { key: "table-0", kind: "table", index: 0, at: 0, span: [0, source.length], text: source, table: { format: "markdown", header: true } } as unknown as RenderedSlot;
  const cx = { view: editor, path: "doc.md", replaceBlockContent: replace } as unknown as SlotContext;
  try {
    render(<UiActionContext.Provider value={registry}>{tableView.render(slot, cx)}</UiActionContext.Provider>);
    const parent = registry.groups().groups.find(g => g.startsWith("table:"))!;
    await invoke(parent, "Write table cell contents", { row: 1, column: 1, value: "3" });
    expect(replace).toHaveBeenCalledWith(slot, expect.stringContaining("3"), "input.uiAgentTable");
    const old = registry.observe(parent).actions.find(a => a.label === "Write table cell contents")!;
    editor.dispatch({ changes: { from: 0, insert: "Changed\n" } });
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { row: 1, column: 1, value: "4" }, request_id: "changed" })).toMatchObject({ status: "stale" });
    expect(replace).toHaveBeenCalledOnce();
  } finally { editor.destroy(); editor.dom.remove(); }
});

it("writes an owned CSV table through real CodeMirror and retires its original source scope", async () => {
  const { registry, invoke } = tools();
  const source = '<hick:table>\nname,count\nNorth,2\n</hick:table>';
  const editor = new EditorView({ parent: document.body, state: EditorState.create({ doc: source }) });
  const text = '\nname,count\nNorth,2\n';
  const slot = { key: "table-0", kind: "table", index: 0, at: 0, span: [0, source.length], text, table: { header: true } } as unknown as RenderedSlot;
  const cx = { view: editor, path: "doc.md", replaceBlockContent: vi.fn() } as unknown as SlotContext;
  try {
    render(<UiActionContext.Provider value={registry}>{tableView.render(slot, cx)}</UiActionContext.Provider>);
    const parent = registry.groups().groups.find(g => g.startsWith("table:"))!;
    await invoke(parent, "Write table cell contents", { row: 1, column: 1, value: "3" });
    expect(editor.state.doc.toString()).toBe(source.replace("North,2", "North,3"));
    expect(registry.observe(parent).actions.find(a => a.label === "Write table cell contents")!.enabled).toBe(false);
  } finally { editor.destroy(); editor.dom.remove(); }
});

it("links nested format/size/settings controls to the owning table and guards resize submission", async () => {
  const { registry, invoke } = tools(); const writes = vi.fn(); let current = true;
  function Harness() {
    const [source, setSource] = useState("price\n123.456\n"), [formats, setFormats] = useState("");
    return <TablePanel source={source} formats={formats} sourceCurrent={() => current} onChange={(text, nextFormats = "", actor) => { writes(text, nextFormats, actor); setSource(text); setFormats(nextFormats); }} />;
  }
  render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
  const parent = registry.groups().groups.find(g => g.startsWith("table:"))!;
  await invoke(parent, "Select table cell or range", { row: 1, column: 0, extend: false });
  const format = registry.groups().groups.find(g => g.startsWith("table-format:"))!;
  expect(registry.observe(format).actions[0].value).toMatchObject({ owner: parent });
  await invoke(format, "Set table number format", { format: "USD" });
  expect(writes.mock.calls.at(-1)).toEqual(["price\n123.456\n", "A2:USD", { id: "python:turn", kind: "agent" }]);
  expect(registry.observe(parent).actions.some(a => a.label.includes("prose wrapping"))).toBe(false);
  const gear = registry.observe("feature-settings").actions.find(a => a.label === "Set Table settings visibility")!;
  expect(gear.value).toMatchObject({ owner: parent });
  await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: gear.handle, arguments: { open: true }, request_id: "gear" }); });
  await invoke(`${parent}/settings`, "Set table prose wrapping", { checked: true });
  await invoke(parent, "Set table size dialog visibility", { open: true });
  const size = registry.groups().groups.find(g => g.startsWith("table-size:"))!;
  await invoke(size, "Set table size draft", { rows: "3", columns: "1" });
  const old = registry.observe(size).actions.find(a => a.label === "Apply table size")!;
  expect(old.value).toMatchObject({ owner: parent });
  current = false;
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "old-size" })).toMatchObject({ status: "stale" });
  expect(writes).toHaveBeenCalledOnce();
  current = true;
  await invoke(size, "Apply table size", {});
  expect(writes.mock.calls.at(-1)![0]).toBe("price\n123.456\n\n");
  expect(writes.mock.calls.at(-1)![2]).toEqual({ id: "python:turn", kind: "agent" });
  expect(registry.observe(size).actions).toEqual([]);
});
