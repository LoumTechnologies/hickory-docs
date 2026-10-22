// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import type { LspClient } from "../lsp/client";
import { lspSupport } from "../lsp/cmLsp";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("tracks late language providers, shares mapped navigation and retains bounded hover captures", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const compartment = new Compartment();
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "value", extensions: [compartment.of([])] }), parent: host });
  const location = { uri: "file:///actual.py", range: { start: { line: 2, character: 1 }, end: { line: 2, character: 6 } } };
  const client = { definition: vi.fn().mockResolvedValue([location]), references: vi.fn().mockResolvedValue([location]), hover: vi.fn().mockResolvedValue({ contents: "TYPE " + "λ".repeat(6000) }) } as unknown as LspClient;
  const navigate = vi.fn(), references = vi.fn(), runtimeValue = vi.fn().mockResolvedValue("captured 42");
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    expect(actions().some(a => a.label.startsWith("Go to definition"))).toBe(false);
    await act(async () => view.dispatch({ effects: compartment.reconfigure(lspSupport({ client, uri: "mapped.py", positionAt: offset => ({ line: 2, character: offset + 1 }), onNavigate: navigate, onReferences: references, runtimeValue })) }));
    await waitFor(() => expect(actions().some(a => a.label.startsWith("Go to definition"))).toBe(true));
    let id = 0;
    const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
    expect(await run("Go to definition at editor cursor (mapped.py)")).toMatchObject({ status: "result", outcome: { status: "completed", value: { total: 1, navigation_requested: true } } });
    expect(client.definition).toHaveBeenCalledWith("mapped.py", { line: 2, character: 1 }); expect(navigate).toHaveBeenCalledWith(location);
    await run("Find references at editor cursor (mapped.py)"); expect(references).toHaveBeenCalledWith([location], expect.objectContaining({ uri: "mapped.py" }));
    const capture = await run("Inspect editor hover (mapped.py)", { position: 1 }) as { outcome: { value: { snapshot: string; total_bytes: number } } };
    const snapshot = capture.outcome.value.snapshot;
    expect(capture.outcome.value.total_bytes).toBeGreaterThan(8192);
    const first = await run("Read editor hover snapshot bytes (mapped.py)", { snapshot, offset: 0, limit: 8192 }) as { outcome: { value: { bytes: number[]; next_offset: number } } };
    expect(first.outcome.value.bytes.length).toBe(8192); expect(first.outcome.value.next_offset).toBe(8192);
    await run("Read editor hover snapshot bytes (mapped.py)", { snapshot, offset: 8192, limit: 8192 }); expect(client.hover).toHaveBeenCalledTimes(1); expect(runtimeValue).toHaveBeenCalledTimes(1);
    const oldRead = actions().find(a => a.label === "Read editor hover snapshot bytes (mapped.py)")!;
    let resolve!: (locations: typeof location[]) => void;
    vi.mocked(client.definition).mockImplementationOnce(() => new Promise(done => { resolve = done; }));
    const inflight = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === "Go to definition at editor cursor (mapped.py)")!.handle, arguments: {}, request_id: "late-navigation" });
    await waitFor(() => expect(client.definition).toHaveBeenCalledTimes(2));
    await act(async () => view.dispatch({ changes: { from: 0, to: 5, insert: "changed" } }));
    resolve([location]);
    expect(await inflight).toMatchObject({ status: "result", outcome: { status: "unavailable" } });
    expect(navigate).toHaveBeenCalledTimes(1);
    expect(await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: oldRead.handle, arguments: { snapshot, offset: 0, limit: 100 }, request_id: "stale-hover" })).toMatchObject({ status: "stale" });
    await act(async () => view.dispatch({ effects: compartment.reconfigure([]) }));
    await waitFor(() => expect(actions().some(a => a.label.startsWith("Go to definition"))).toBe(false));
  } finally { cleanup(); view.destroy(); host.remove(); }
});

it("keeps provider revisions opaque and forwards the actual actor", async () => {
  const { editorLanguageActions } = await import("./languageActions");
  const host = document.body.appendChild(document.createElement("div"));
  const view = new EditorView({ parent: host, state: EditorState.create({ doc: "current", extensions: [editorLanguageActions.of({ id: "private-revision", label: "Read public provider status", parameters: {}, enabled: () => true, revision: () => "HIDDEN-PROVIDER-REVISION", execute: async (_view, _args, _signal, actor) => ({ status: "completed", value: { actor: actor?.id ?? null } }) })] }) });
  const registry = new UiActions({ workspace: "p", window: "revision" }, () => true, async () => {});
  function Bridge() { useEditorActions(view, "opaque"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Bridge /></UiActionContext.Provider>);
    const actions = registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
    expect(JSON.stringify(actions)).not.toContain("HIDDEN-PROVIDER-REVISION");
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "acp:actual", kind: "agent" }, { handle: actions.find(action => action.label === "Read public provider status")!.handle, arguments: {}, request_id: "actor" }); });
    expect(result).toMatchObject({ outcome: { status: "completed", value: { actor: "acp:actual" } } });
  } finally { cleanup(); view.destroy(); host.remove(); }
});
