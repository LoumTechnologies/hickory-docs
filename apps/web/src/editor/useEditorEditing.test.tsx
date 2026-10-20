import { act, cleanup, renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { Compartment, EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { history } from "@codemirror/commands";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorEditing } from "./useEditorEditing";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("uses the real input/history pipeline and refuses old editor revisions and changed owners", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const input = vi.fn(); let owner = "first";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "λ😀\nnext", extensions: [history(), EditorView.updateListener.of(update => { for (const t of update.transactions) if (t.isUserEvent("input.uiAgentSource")) input(); })] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorEditing(view, { source: "file:a", label: "owned editor", identity: () => owner, current: () => true }), { wrapper });
  const actions = () => registry.groups().groups.flatMap(g => registry.observe(g).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  const old = actions().find(a => a.label === "Replace owned editor range")!;
  expect(await run(old.label, { from: 0, to: 3, text: "edit" })).toMatchObject({ status: "result", outcome: { value: { requested_applied: true, source_changed: true, approved: false, verified: false } } });
  expect(view.state.doc.toString()).toBe("edit\nnext"); expect(input).toHaveBeenCalledOnce();
  await run("Undo owned editor edit"); expect(view.state.doc.toString()).toBe("λ😀\nnext");
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { from: 0, to: 0, text: "x" }, request_id: "old" })).toMatchObject({ status: "unavailable" });
  await run("Redo owned editor edit"); expect(view.state.doc.toString()).toBe("edit\nnext");
  const current = actions().find(a => a.label === old.label)!;
  owner = "different";
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: current.handle, arguments: { from: 0, to: 0, text: "x" }, request_id: "owner" })).toMatchObject({ status: "stale" });
  hook.unmount(); view.destroy(); host.remove();
});

it("reports actual filter rejection and disables writes after the editor becomes read-only", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const access = new Compartment();
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "protected", extensions: [access.of(EditorState.readOnly.of(false)), EditorState.transactionFilter.of(tr => tr.docChanged ? [] : tr)] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorEditing(view, { source: "file:a", label: "owned editor", identity: () => "a", current: () => true }), { wrapper });
  const action = () => registry.observe(registry.groups().groups[0]).actions.find(a => a.label === "Replace owned editor range")!;
  let result: unknown;
  await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action().handle, arguments: { from: 0, to: 9, text: "change" }, request_id: "filtered" }); });
  expect(result).toMatchObject({ status: "result", outcome: { value: { requested_applied: false, source_changed: false } } });
  expect(view.state.doc.toString()).toBe("protected");
  act(() => { view.dispatch({ effects: access.reconfigure(EditorState.readOnly.of(true)) }); });
  expect(action().enabled).toBe(false);
  expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action().handle, arguments: { from: 0, to: 1, text: "x" }, request_id: "readonly" })).toMatchObject({ status: "disabled" });
  hook.unmount(); view.destroy(); host.remove();
});

// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("refuses split-surrogate ranges and malformed input before source changes, while allowing whole Unicode replacement", async () => {
  const host = document.createElement("div"); document.body.append(host);
  const changed = vi.fn();
  const source = "λ😀tail";
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: source, extensions: [EditorView.updateListener.of(update => { if (update.docChanged) changed(); })] }) });
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const wrapper = ({ children }: { children: ReactNode }) => <UiActionContext.Provider value={registry}>{children}</UiActionContext.Provider>;
  const hook = renderHook(() => useEditorEditing(view, { source: "file:unicode", label: "owned editor", identity: () => "unicode", current: () => true }), { wrapper });
  let id = 0;
  const run = async (label: string, args: Record<string, string | number>) => { let result: unknown; await act(async () => { const action = registry.groups().groups.flatMap(g => registry.observe(g).actions).find(a => a.label === label)!; result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: action.handle, arguments: args, request_id: String(++id) }); }); return result; };
  for (const [from, to] of [[2, 3], [1, 2], [2, 2]]) expect(await run("Replace owned editor range", { from, to, text: "X" })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("surrogate pair") } });
  expect(await run("Replace owned editor range", { from: 0, to: 0, text: "\ud800" })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("unpaired UTF-16") } });
  act(() => { view.dispatch({ selection: EditorSelection.cursor(2) }); });
  expect(await run("Replace owned editor selections", { text: "X" })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("surrogate pair") } });
  expect(view.state.doc.toString()).toBe(source); expect(changed).not.toHaveBeenCalled();
  expect(await run("Replace owned editor range", { from: 1, to: 3, text: "🦀" })).toMatchObject({ status: "result", outcome: { value: { requested_applied: true } } });
  expect(view.state.doc.toString()).toBe("λ🦀tail"); expect(changed).toHaveBeenCalledOnce();
  hook.unmount(); view.destroy(); host.remove();
});
