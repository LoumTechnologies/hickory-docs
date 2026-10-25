<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/editor/taskList.controls.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md)

Captured SHA-256: `cf63f28ecb981b199d594c39e8c9c8c0ac9c404f2c47b4b2b996a6388aa466e0`.

<book:entry path="apps/web/src/editor/taskList.controls.test.tsx">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
import { act, cleanup, render } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { Compartment, EditorState, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { codeFolding, foldEffect } from "@codemirror/language";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { taskCheckboxes } from "./taskList";
const views: EditorView[] = [];
afterEach(() => { cleanup(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
function Bridge({ view }: { view: EditorView }) { useEditorActions(view, "tasks"); return null; }
it("shares the real task checkbox, retains actors, and retires folded/readonly mutations", async () => {
  const source = "# Tasks\n\n- [ ] first\n- [X] second\n";
  const access = new Compartment(), events: string[] = [], host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [taskCheckboxes(), codeFolding(), access.of(EditorState.readOnly.of(false)), EditorView.updateListener.of(update => { for (const tr of update.transactions) { const event = tr.annotation(Transaction.userEvent); if (event) events.push(event); } })] }) });
  views.push(view);
  const registry = new UiActions({ workspace: "tasks", window: "one" }, () => true, async () => {});
  render(<UiActionContext.Provider value={registry}><Bridge view={view} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, import("../lib/uiActions").UiValue>) => {
    let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:tasks", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result;
  };
  const page = await run("Read Markdown task checkbox page", { first: 0, count: 64 }) as { outcome: { value: { tasks: { position: number; checked: boolean }[] } } };
  expect(page.outcome.value.tasks.map(task => task.checked)).toEqual([false, true]);
  expect(await run("Set Markdown task checkbox", { position: page.outcome.value.tasks[0].position, checked: true })).toMatchObject({ outcome: { status: "completed", value: { source_changed: true, saved: false } } });
  expect(view.state.doc.toString()).toContain("- [x] first");
  await act(async () => { view.dom.querySelectorAll(".cm-md-checkbox")[1].dispatchEvent(new MouseEvent("mousedown", { bubbles: true })); });
  expect(view.state.doc.toString()).toContain("- [ ] second");
  expect(events).toEqual(["input.uiAgentTask", "input.toggleTask"]);
  await act(async () => view.dispatch({ effects: access.reconfigure(EditorState.readOnly.of(true)) }));
  expect(actions().find(action => action.label === "Set Markdown task checkbox")!.enabled).toBe(false);
  const before = view.state.doc.toString();
  await act(async () => { view.dom.querySelector(".cm-md-checkbox")!.dispatchEvent(new MouseEvent("mousedown", { bubbles: true })); });
  expect(view.state.doc.toString()).toBe(before);
  expect(view.dom.querySelector(".cm-md-checkbox")!.getAttribute("aria-disabled")).toBe("true");
  const retired = actions().find(action => action.label === "Read Markdown task checkbox page")!;
  await act(async () => view.dispatch({ effects: foldEffect.of({ from: source.indexOf("- [ ] first"), to: source.length - 1 }) }));
  expect(actions().some(action => action.label.includes("Markdown task checkbox"))).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:tasks", kind: "agent" }, { handle: retired.handle, arguments: { first: 0, count: 1 }, request_id: "folded" })).not.toMatchObject({ outcome: { status: "completed" } });
});
</book:entry>

</book:doc>
