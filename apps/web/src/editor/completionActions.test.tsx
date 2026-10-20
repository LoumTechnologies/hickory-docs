// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { acceptCompletion, completionStatus, currentCompletions, startCompletion } from "@codemirror/autocomplete";
import { completions } from "../lsp/completion";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads and accepts the real completion popup, and retires selection after source edits", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const project = vi.fn().mockResolvedValue([{ text: "total_units", detail: "a.py:1", score: 1, semantic: false }, { text: "total_retries", detail: "b.py:2", score: 1, semantic: true }]);
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "tot", selection: { anchor: 3 }, extensions: completions({ project }) }), parent: host });
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    await waitFor(() => expect(actions().some(a => a.label === "Open editor completions")).toBe(true));
    let id = 0;
    const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply as { status: string; outcome: { status: string; value: { snapshot: string; bytes: number[] } } }; };
    expect(await run("Open editor completions")).toMatchObject({ status: "result", outcome: { status: "queued", job: expect.any(String) } });
    await waitFor(() => expect(completionStatus(view.state)).toBe("active"));
    const observed = await run("Observe editor completion popup");
    const snapshot = observed.outcome.value.snapshot;
    const read = await run("Read editor completion choices", { snapshot, offset: 0, limit: 8192 });
    const choices = JSON.parse(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes))).choices;
    expect(choices.map((c: { label: string }) => c.label)).toEqual(currentCompletions(view.state).map(c => c.label));
    expect(choices.some((c: { info: string }) => c.info.includes("how often"))).toBe(true);
    const index = choices.findIndex((c: { label: string }) => c.label === "total_units");
    const selected = await run("Select editor completion", { snapshot, index });
    await new Promise(done => setTimeout(done, 90)); // Native popup interaction delay applies to both adapters.
    expect(await run("Accept selected editor completion", { snapshot: selected.outcome.value.snapshot })).toMatchObject({ status: "result", outcome: { status: "completed", value: { accepted: true, source_changed: true, saved: false, verified: false } } });
    expect(view.state.doc.toString()).toBe("total_units");
    await act(async () => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "tot" }, selection: { anchor: 3 } }));
    await run("Open editor completions");
    await waitFor(() => expect(completionStatus(view.state)).toBe("active"));
    const old = actions().find(a => a.label === "Accept selected editor completion")!;
    const before = await run("Observe editor completion popup");
    await act(async () => view.dispatch({ changes: { from: 0, insert: "new_" } }));
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { snapshot: before.outcome.value.snapshot }, request_id: "stale" })).toMatchObject({ status: "stale" });
    await run("Close editor completions");
    expect(completionStatus(view.state)).toBeNull();
  } finally { cleanup(); view.destroy(); host.remove(); }
});

it("refuses malformed provider insertion through the same native completion filter", async () => {
  const client = { completion: async () => [{ label: "valid_label", insertText: "bad\ud800" }] };
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "val", selection: { anchor: 3 }, extensions: completions({ lsp: { client: client as never, uri: "file.py", positionAt: pos => ({ line: 0, character: pos }) } }) }), parent: host });
  try {
    expect(startCompletion(view)).toBe(true);
    await waitFor(() => expect(completionStatus(view.state)).toBe("active"));
    await new Promise(done => setTimeout(done, 90));
    acceptCompletion(view);
    expect(view.state.doc.toString()).toBe("val");
  } finally { view.destroy(); host.remove(); }
});
