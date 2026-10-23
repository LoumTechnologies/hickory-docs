// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { formatView, lspFeatures, renameSymbol } from "../lsp/cmLspFeatures";
import type { LspClient } from "../lsp/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
const range = (from: number, to: number) => ({ start: { line: 0, character: from }, end: { line: 0, character: to } });
const quiet = { serverCapabilities: null, semanticTokens: async () => null, inlayHints: async () => [], documentHighlight: async () => [], foldingRanges: async () => [], signatureHelp: async () => null };
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("shares actual formatter and typed rename edits while preserving read-only boundaries", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const client = { ...quiet, formatting: vi.fn().mockResolvedValue([{ range: range(1, 3), newText: " " }]), prepareRename: vi.fn().mockResolvedValue(range(0, 1)), rename: vi.fn().mockResolvedValue({ changes: { "file.py": [{ range: range(0, 1), newText: "renamed" }], "other.py": [{ range: range(0, 1), newText: "renamed" }] } }) } as unknown as LspClient;
  const readonly = new Compartment();
  const host = document.createElement("div"); document.body.append(host);
  const onRename = vi.fn(), onMessage = vi.fn();
  const view = new EditorView({ state: EditorState.create({ doc: "a  b", extensions: [readonly.of([]), lspFeatures({ client, uri: "file.py", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character, onRename, onMessage })] }), parent: host });
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    await waitFor(() => expect(actions().some(a => a.label === "Format editor source (file.py)")).toBe(true));
    let id = 0;
    const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
    expect(await run("Format editor source (file.py)")).toMatchObject({ status: "result", outcome: { status: "completed", value: { source_changed: true, saved: false, verified: false } } });
    expect(view.state.doc.toString()).toBe("a b");
    expect(await run("Rename editor symbol (file.py)", { name: "renamed" })).toMatchObject({ status: "result", outcome: { status: "completed", value: { source_changed: true, other_documents: ["other.py"], saved: false } } });
    expect(view.state.doc.toString()).toBe("renamed b"); expect(client.rename).toHaveBeenCalledWith("file.py", { line: 0, character: 0 }, "renamed"); expect(onRename).not.toHaveBeenCalled();
    await act(async () => view.dispatch({ effects: readonly.reconfigure(EditorState.readOnly.of(true)) }));
    expect(actions().find(a => a.label === "Format editor source (file.py)")?.enabled).toBe(false);
    expect(await formatView(view)).toBe(false); expect(client.formatting).toHaveBeenCalledTimes(1);
  } finally { cleanup(); view.destroy(); host.remove(); }
});

it("refuses an entire invalid Unicode edit batch and a rename whose preparation becomes stale", async () => {
  const client = { ...quiet, formatting: vi.fn().mockResolvedValue([{ range: range(2, 3), newText: "valid" }, { range: range(1, 2), newText: "invalid split" }]), prepareRename: vi.fn(), rename: vi.fn() } as unknown as LspClient;
  const onRename = vi.fn().mockResolvedValue("next"), onMessage = vi.fn();
  const opts = { client, uri: "file.py", positionAt: (p: number) => ({ line: 0, character: p }), offsetAt: (p: { character: number }) => p.character, onRename, onMessage };
  const view = new EditorView({ state: EditorState.create({ doc: "😀b", extensions: lspFeatures(opts) }) });
  try {
    expect(await formatView(view)).toBe(false); expect(view.state.doc.toString()).toBe("😀b"); expect(onMessage).toHaveBeenCalledWith(expect.stringContaining("surrogate"));
    let complete!: (value: ReturnType<typeof range>) => void;
    vi.mocked(client.prepareRename).mockImplementationOnce(() => new Promise(done => { complete = done; }));
    const pending = renameSymbol(view, opts, onRename);
    view.dispatch({ changes: { from: 0, to: 3, insert: "new source" } });
    complete(range(0, 2));
    expect(await pending).toMatchObject({ status: "unavailable" }); expect(onRename).not.toHaveBeenCalled(); expect(client.rename).not.toHaveBeenCalled(); expect(view.state.doc.toString()).toBe("new source");
  } finally { view.destroy(); }
});

it("applies a provider rename addressed to the encoded spelling of a spaced document URI", async () => {
  const uri = "hick:///Application Support/main.cpp";
  const client = { ...quiet, prepareRename: vi.fn().mockResolvedValue(range(4, 10)), rename: vi.fn().mockResolvedValue({ changes: { "hick:///Application%20Support/main.cpp": [{ range: range(4, 10), newText: "squared" }] } }) } as unknown as LspClient;
  const onMessage = vi.fn();
  const opts = { client, uri, positionAt: (p: number) => ({ line: 0, character: p }), offsetAt: (p: { character: number }) => p.character, onMessage };
  const view = new EditorView({ state: EditorState.create({ doc: "int square(int value);", selection: { anchor: 5 } }) });
  try {
    expect(await renameSymbol(view, opts, async previous => { expect(previous).toBe("square"); return "squared"; })).toMatchObject({ status: "completed", value: { source_changed: true, other_documents: [] } });
    expect(view.state.doc.toString()).toBe("int squared(int value);");
    expect(onMessage).not.toHaveBeenCalled();
    view.dispatch({ changes: { from: 4, to: 11, insert: "square" } });
    vi.mocked(client.rename).mockRejectedValueOnce(new Error("Index results do not match current content"));
    expect(await renameSymbol(view, opts, async () => "squared")).toMatchObject({ status: "unavailable", reason: "Index results do not match current content" });
    expect(view.state.doc.toString()).toBe("int square(int value);");
    expect(onMessage).toHaveBeenCalledWith("Index results do not match current content");
  } finally { view.destroy(); }
});
