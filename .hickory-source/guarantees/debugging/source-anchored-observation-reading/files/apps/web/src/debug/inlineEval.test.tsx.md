<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/debug/inlineEval.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Observations read beside their captured source](../../../../../contract.md)

Captured SHA-256: `8e779bc464126783da9a3de3b35b4697ae68d22fe74d479c8fe358459cb75aea`.

<book:entry path="apps/web/src/debug/inlineEval.test.tsx">
// Guarantee: docs/guarantees/debugging/source-anchored-observation-reading.md
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "../editor/uiEditorActions";
import { inlineEvaluationOwner } from "./inlineEval";
import type { DebugSession } from "./useDebugger";
import { debugEditor, setPausedLine } from "./cmDebug";
let view: EditorView | undefined;
afterEach(() => { cleanup(); const host = view?.dom.parentElement; view?.destroy(); host?.remove(); });
async function setup(query: (expression: string) => Promise<string | null>) {
  let owner = "stop-one";
  const watch = vi.fn(), host = document.body.appendChild(document.createElement("div"));
  const current = new EditorView({ parent: host, state: EditorState.create({ doc: "print(value)\n", extensions: debugEditor({ language: "python", onToggleBreakpoint: () => {}, onEvaluate: query, onAddWatch: watch, evaluationOwner: () => owner }) }) }); view = current;
  const registry = new UiActions({ workspace: "p", window: "inline-eval" }, () => true, async () => {});
  function Bridge() { useEditorActions(current, "inline-eval"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  await act(async () => current.dispatch({ effects: setPausedLine.of(0) }));
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await waitFor(() => expect(actions().some(action => action.label === "Set paused-line evaluation disclosure")).toBe(true));
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
  const read = async () => { const reply = await run("Read paused-line evaluation form bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } }; return JSON.parse(new TextDecoder().decode(new Uint8Array(reply.outcome.value.bytes))); };
  return { current, registry, actions, run, read, watch, replace: () => { owner = "stop-two"; current.dispatch({ effects: setPausedLine.of(0) }); } };
}
it("shares native/agent disclosure, draft, evaluation and watch without implicit value discovery", async () => {
  const query = vi.fn(async () => "private current value");
  const { current, actions, run, read, watch } = await setup(query);
  expect(actions().some(action => action.label === "Evaluate paused-line draft")).toBe(false);
  await run("Set paused-line evaluation disclosure", { open: true });
  const input = current.dom.querySelector<HTMLInputElement>(".cm-eval-input")!;
  await run("Set paused-line evaluation draft", { text: "value + 1" }); expect(input.value).toBe("value + 1");
  expect(await run("Evaluate paused-line draft")).toMatchObject({ outcome: { value: { evaluation_callback_completed: true, verified: false } } });
  expect(query).toHaveBeenCalledWith("value + 1");
  expect(JSON.stringify(actions())).not.toContain("private current value");
  expect(await read()).toMatchObject({ draft: "value + 1", result: "= private current value" });
  await act(async () => { fireEvent.input(input, { target: { value: "native expression" } }); fireEvent.keyDown(input, { key: "Enter", shiftKey: true }); });
  expect(watch).toHaveBeenCalledWith("native expression");
  await run("Watch paused-line evaluation draft"); expect(watch).toHaveBeenCalledTimes(2);
  await act(async () => fireEvent.keyDown(input, { key: "Escape" }));
  expect(actions().some(action => action.label === "Read paused-line evaluation form bytes")).toBe(false);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "fresh draft" });
  await act(async () => fireEvent.input(input, { target: { value: "detached draft" } }));
  expect(await read()).toMatchObject({ draft: "fresh draft" });
  expect(current.state.doc.toString()).toBe("print(value)\n");
});
it("refuses late values after source/stop replacement and does not steal another field's focus", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { current, registry, actions, run, replace } = await setup(query);
  const field = document.body.appendChild(document.createElement("input")); field.value = "unsubmitted"; field.focus();
  expect(await run("Set paused-line evaluation disclosure", { open: true })).toMatchObject({ outcome: { status: "requires_input" } });
  expect(document.activeElement).toBe(field); field.remove();
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  const nativeInput = current.dom.querySelector<HTMLInputElement>(".cm-eval-input")!;
  const oldToggle = current.dom.querySelector(".cm-eval-toggle");
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "pending" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  await act(async () => replace());
  await act(async () => resolve("RETIRED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(current.dom.textContent).not.toContain("RETIRED VALUE");
  await act(async () => fireEvent.keyDown(nativeInput, { key: "Enter" })); expect(query).toHaveBeenCalledTimes(1);
  expect(oldToggle).toBeNull();
  await run("Set paused-line evaluation disclosure", { open: true });
  const old = actions().find(action => action.label === "Set paused-line evaluation draft")!;
  await act(async () => current.dispatch({ changes: { from: 0, insert: "# changed\n" } }));
  expect(await registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: old.handle, arguments: { text: "other" }, request_id: "old" })).not.toMatchObject({ outcome: { status: "completed" } });
});

it("keeps an evaluation owner stable for value replies and changes it for a new frame or stop", () => {
  const debug = { status: "paused", program: "test.py", selectedFrame: 1, frames: [], reading: { source: { revision: "source" }, scope: { run: "run", stop: "one" } } } as unknown as DebugSession;
  const owner = inlineEvaluationOwner(debug);
  expect(inlineEvaluationOwner({ ...debug, lastValue: { expression: "value", value: "reply", type: null }, watches: [] })).toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, selectedFrame: 2 })).not.toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, frames: [] })).not.toBe(owner);
  expect(inlineEvaluationOwner({ ...debug, status: "running" })).toBeNull();
});
it("retains cancellation uncertainty without adopting a value from the dispatched request", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { current, registry, actions, run, read } = await setup(query);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "cancel" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  expect(await run("Set paused-line evaluation draft", { text: "next expression" })).toMatchObject({ outcome: { status: "completed" } });
  expect(await read()).toMatchObject({ draft: "next expression", requested_expression: "value", pending: true });
  expect(registry.cancel(registry.scope, { id: "acp:eval", kind: "agent" }, "cancel")).toBe(true);
  await act(async () => resolve("CANCELLED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(current.dom.textContent).not.toContain("CANCELLED VALUE");
  expect(await read()).toMatchObject({ pending: false, error: expect.stringContaining("may have executed") });
});

it("allows explicit close while evaluation is pending and reports possible dispatch", async () => {
  let resolve!: (value: string) => void;
  const query = vi.fn(() => new Promise<string>(done => { resolve = done; }));
  const { registry, actions, run } = await setup(query);
  await run("Set paused-line evaluation disclosure", { open: true });
  await run("Set paused-line evaluation draft", { text: "value" });
  let pending!: Promise<unknown>;
  await act(async () => { pending = registry.act(registry.scope, { id: "acp:eval", kind: "agent" }, { handle: actions().find(action => action.label === "Evaluate paused-line draft")!.handle, arguments: {}, request_id: "closing" }); });
  await waitFor(() => expect(query).toHaveBeenCalledTimes(1));
  expect(await run("Set paused-line evaluation disclosure", { open: false })).toMatchObject({ outcome: { value: { possible_pending_request: true, evaluation_cancelled: false } } });
  await act(async () => resolve("CLOSED VALUE"));
  expect(await pending).not.toMatchObject({ outcome: { status: "completed" } });
  expect(actions().some(action => action.label === "Read paused-line evaluation form bytes")).toBe(false);
});
</book:entry>

</book:doc>
