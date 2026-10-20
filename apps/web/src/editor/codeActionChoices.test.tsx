// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { cmKeyOf } from "../lib/keymap";
import { EditorView, keymap } from "@codemirror/view";
import { lspFeatures } from "../lsp/cmLspFeatures";
import type { LspClient } from "../lsp/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains real source-bound code choices and refuses forged, stale or unsupported actions", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const offered = [
    { title: "Fix name", edit: { changes: { "file.py": [{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } }, newText: "fixed" }] } } },
    { title: "Requires server command", command: { title: "command", command: "unsupported" } },
  ];
  const client = { serverCapabilities: null, semanticTokens: async () => null, inlayHints: async () => [], documentHighlight: async () => [], foldingRanges: async () => [], signatureHelp: async () => null, codeActions: vi.fn().mockResolvedValue(offered) } as unknown as LspClient;
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "bad", extensions: lspFeatures({ client, uri: "file.py", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character, onCodeActions: actions => actions[0], onMessage: vi.fn() }) }), parent: host });
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    await waitFor(() => expect(actions().some(a => a.label === "Inspect editor code actions (file.py)")).toBe(true));
    let id = 0;
    const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
    const inspect = await run("Inspect editor code actions (file.py)") as { outcome: { value: { snapshot: string } } };
    const snapshot = inspect.outcome.value.snapshot;
    const read = await run("Read editor code action choices (file.py)", { snapshot, offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[] } } };
    const choices = JSON.parse(new TextDecoder().decode(new Uint8Array(read.outcome.value.bytes))).choices;
    expect(choices.map((c: { title: string }) => c.title)).toEqual(["Fix name", "Requires server command"]);
    expect(await run("Apply captured editor code action (file.py)", { snapshot, choice: "Fix name" })).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
    expect(await run("Apply captured editor code action (file.py)", { snapshot, choice: choices[1].id })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("cannot execute") } });
    offered[0].edit!.changes["file.py"][0].newText = "MUTATED PROVIDER OBJECT";
    expect(await run("Apply captured editor code action (file.py)", { snapshot, choice: choices[0].id })).toMatchObject({ status: "result", outcome: { status: "completed", value: { source_changed: true, saved: false, verified: false } } });
    expect(view.state.doc.toString()).toBe("fixed"); expect(client.codeActions).toHaveBeenCalledTimes(1);
    const fresh = await run("Inspect editor code actions (file.py)") as { outcome: { value: { snapshot: string } } };
    const old = actions().find(a => a.label === "Apply captured editor code action (file.py)")!;
    await act(async () => view.dispatch({ selection: { anchor: 2 } }));
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: old.handle, arguments: { snapshot: fresh.outcome.value.snapshot, choice: "anything" }, request_id: "stale" })).toMatchObject({ status: "stale" });
    offered[0].edit!.changes["file.py"][0].newText = "human";
    await act(async () => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "bad" }, selection: { anchor: 0 } }));
    const binding = view.state.facet(keymap).flat().find(b => b.key === cmKeyOf("editor.codeAction"))!;
    await act(async () => { expect(binding.run!(view)).toBe(true); });
    await waitFor(() => expect(view.state.doc.toString()).toBe("human"));
    expect(client.codeActions).toHaveBeenCalledTimes(3);
  } finally { cleanup(); view.destroy(); host.remove(); }
});
