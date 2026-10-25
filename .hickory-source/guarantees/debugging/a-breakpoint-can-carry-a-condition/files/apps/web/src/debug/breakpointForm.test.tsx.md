<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/breakpointForm.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Breakpoint Can Carry A Condition](../../../../../contract.md)

Captured SHA-256: `ba28fc0585b1d401259dc460e215945e1deca2646337169d67939ec866aa5d4a`.

<book:entry path="apps/web/src/debug/breakpointForm.test.tsx">
// Guarantee: docs/guarantees/debugging/a-breakpoint-can-carry-a-condition.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { debugEditor, setBreakpointMarks } from "./cmDebug";
let view: EditorView | undefined;
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
async function setup() {
  const toggle = vi.fn(), set = vi.fn(), host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: "value = 1\nprint(value)\n", extensions: debugEditor({ language: "python", onToggleBreakpoint: toggle, onSetBreakpointCondition: set }) }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "breakpoint" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "breakpoint"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:condition", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  return { current, registry, actions, run, set, toggle };
}
it("shares actual drafts and native/agent apply while allowing the new breakpoint's adapter acknowledgement", async () => {
  const { current, actions, run, set, toggle } = await setup();
  expect(actions().some(action => action.label === "Read gutter breakpoint editor bytes")).toBe(false);
  await run("Open gutter breakpoint condition editor", { line: 1 }); expect(toggle).toHaveBeenCalledWith(0);
  const input = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await run("Set gutter breakpoint editor field", { field: "condition", text: "  value > 2  " }); expect(input.value).toBe("  value > 2  ");
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([{ line: 0, state: "bound", conditional: false }]) }));
  expect(input.isConnected).toBe(true);
  const reply = await run("Read gutter breakpoint editor bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
  expect(JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes)))).toMatchObject({ line: 1, fields: { condition: "  value > 2  " } });
  expect(await run("Apply gutter breakpoint editor")).toMatchObject({ outcome: { value: { callback_requested: true, adapter_acceptance_unknown: true } } });
  expect(set).toHaveBeenCalledWith(0, { condition: "value > 2", hit_condition: "", log_message: "" });
  await run("Open gutter breakpoint condition editor", { line: 2 });
  const native = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await act(async () => { fireEvent.input(native, { target: { value: "native condition" } }); fireEvent.keyDown(native, { key: "Enter" }); });
  expect(set).toHaveBeenLastCalledWith(1, { condition: "native condition", hit_condition: "", log_message: "" });
  expect(current.state.doc.toString()).toBe("value = 1\nprint(value)\n");
});
it("preserves another focused field and refuses stale native/agent application after source or target replacement", async () => {
  const { current, registry, actions, run, set } = await setup();
  const external = document.body.appendChild(document.createElement("input")); external.focus();
  expect(await run("Open gutter breakpoint condition editor", { line: 1 })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(external); external.remove();
  await run("Open gutter breakpoint condition editor", { line: 1 });
  const native = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  const old = actions().find(action => action.label === "Apply gutter breakpoint editor")!;
  await act(async () => current.dispatch({ changes: { from: 0, insert: "# shifted\n" } }));
  expect(current.dom.querySelector(".cm-bp-editor")).toBeNull();
  await act(async () => fireEvent.keyDown(native, { key: "Enter" })); expect(set).not.toHaveBeenCalled();
  expect(await registry.act(registry.scope, { id: "acp:condition", kind: "agent" }, { handle: old.handle, arguments: {}, request_id: "stale" })).not.toMatchObject({ outcome: { status: "completed" } });
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([{ line: 1, state: "bound", conditional: false }]) }));
  await run("Open gutter breakpoint condition editor", { line: 2 });
  await act(async () => current.dispatch({ effects: setBreakpointMarks.of([]) }));
  expect(current.dom.querySelector(".cm-bp-editor")).toBeNull();
  await run("Open gutter breakpoint condition editor", { line: 2 });
  const latest = current.dom.querySelector<HTMLInputElement>(".cm-bp-editor input")!;
  await act(async () => { fireEvent.input(latest, { target: { value: "blur condition" } }); latest.blur(); });
  await waitFor(() => expect(set).toHaveBeenCalledWith(1, { condition: "blur condition", hit_condition: "", log_message: "" }));
});
</book:entry>

</book:doc>
