// Guarantee: docs/guarantees/editing/a-comparison-keeps-current-code-editable.md
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { UiActions, type UiValue } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
import { comparisonField, setComparison } from "./comparison";
const views: EditorView[] = [];
afterEach(() => { cleanup(); for (const view of views.splice(0)) { const host = view.dom.parentElement; view.destroy(); host?.remove(); } });
function setup(removed: string, editable = true) {
  const events: string[] = [], host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "same\n", extensions: [comparisonField, EditorView.updateListener.of(update => { for (const tr of update.transactions) { const event = tr.annotation(Transaction.userEvent); if (event) events.push(event); } })] }) });
  views.push(view); view.dispatch({ effects: setComparison.of({ base: `same\n${removed}`, editable }) });
  const registry = new UiActions({ workspace: "comparison", window: "one" }, () => true, async () => {});
  function Bridge() { useEditorActions(view, "comparison"); return null; }
  render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args: Record<string, UiValue> = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  return { view, events, registry, actions, run };
}
it("discloses bounded historical bytes only after opening and restores through the real buffer", async () => {
  const removed = "HIDDEN-REMOVED-SOURCE " + "λ".repeat(6000) + "\n";
  const { view, events, registry, actions, run } = setup(removed);
  expect(JSON.stringify(actions())).not.toContain("HIDDEN-REMOVED-SOURCE");
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(false);
  const markers = await run("Read comparison removed-line marker page", { first: 0, count: 64 }) as { outcome: { value: { markers: { position: number }[] } } };
  const position = markers.outcome.value.markers[0].position;
  await run("Open comparison removed-line preview", { position });
  expect(JSON.stringify(actions())).not.toContain("HIDDEN-REMOVED-SOURCE");
  const read = await run("Read comparison removed-source bytes", { offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[]; next: number } } };
  expect(read.outcome.value.bytes).toHaveLength(8192);
  expect(read.outcome.value.next).toBe(8192);
  expect(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes))).toContain("HIDDEN-REMOVED-SOURCE");
  const retired = actions().find(action => action.label === "Read comparison removed-source bytes")!;
  await run("Close comparison removed-line preview");
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(false);
  expect(await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: retired.handle, arguments: { offset: 0, limit: 20 }, request_id: "closed-read" })).not.toMatchObject({ outcome: { status: "completed" } });
  await run("Open comparison removed-line preview", { position });
  expect(await run("Restore comparison removed-source bytes")).toMatchObject({ outcome: { status: "completed", value: { source_changed: true, saved: false, verified: false } } });
  expect(view.state.doc.toString()).toBe(`same\n${removed}`);
  expect(events).toEqual(["input.uiAgentRestore"]);
});
it("refuses a stale native restore callback and keeps historical comparisons read-only", async () => {
  const { view, registry, actions, run } = setup("historical\n");
  await run("Open comparison removed-line preview", { position: 5 });
  const restoreAction = actions().find(action => action.label === "Restore comparison removed-source bytes")!;
  // Retain the actual native button across a source replacement.
  let button!: HTMLButtonElement;
  await waitFor(() => { button = [...document.querySelectorAll<HTMLButtonElement>(".comparison-removed button")].find(button => button.textContent === "Restore these lines")!; expect(button).toBeTruthy(); });
  await act(async () => view.dispatch({ changes: { from: 0, to: 4, insert: "changed" } }));
  const before = view.state.doc.toString();
  await act(async () => button.click());
  expect(view.state.doc.toString()).toBe(before);
  expect(await registry.act(registry.scope, { id: "acp:comparison", kind: "agent" }, { handle: restoreAction.handle, arguments: {}, request_id: "old-restore" })).not.toMatchObject({ outcome: { status: "completed" } });
  await act(async () => view.dispatch({ effects: setComparison.of({ base: `${before}historical\n`, editable: false }) }));
  const markers = await run("Read comparison removed-line marker page", { first: 0, count: 64 }) as { outcome: { value: { markers: { position: number }[] } } };
  await run("Open comparison removed-line preview", { position: markers.outcome.value.markers[0].position });
  expect(actions().some(action => action.label === "Read comparison removed-source bytes")).toBe(true);
  expect(actions().some(action => action.label === "Restore comparison removed-source bytes")).toBe(false);
});
