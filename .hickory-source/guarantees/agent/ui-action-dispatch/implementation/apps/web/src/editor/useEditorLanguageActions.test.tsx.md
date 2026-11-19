<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/useEditorLanguageActions.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-a7aead7d56cd"></a>
## `apps/web/src/editor/useEditorLanguageActions.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-a7aead7d56cd-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-a7aead7d56cd-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { Compartment, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import type { LspClient } from "../lsp/client";
import { lspSupport, type RelatedLocationReading } from "../lsp/cmLsp";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-a7aead7d56cd-expression_statement-2"></a>
### `expression_statement`

Provider actions can finish before a person picks a reference. Exercise the
callback after the action resolves, then move the caret away and back. The
workspace can also retire a provider request before its reply arrives.

<storycode:copy id="f-a7aead7d56cd-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("tracks late language providers, shares mapped navigation and retains bounded hover captures", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const compartment = new Compartment();
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "value", extensions: [compartment.of([])] }), parent: host });
  const location = { uri: "file:///actual.py", range: { start: { line: 2, character: 1 }, end: { line: 2, character: 6 } } };
  const client = { definition: vi.fn().mockResolvedValue([location]), references: vi.fn().mockResolvedValue([location]), hover: vi.fn().mockResolvedValue({ contents: "TYPE " + "λ".repeat(6000) }) } as unknown as LspClient;
  const navigate = vi.fn(), references = vi.fn(), runtimeValue = vi.fn().mockResolvedValue("captured 42");
  let foreground = 0;
  const captureNavigationContext = () => {
    const captured = foreground;
    return () => foreground === captured;
  };
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    expect(actions().some(a => a.label.startsWith("Go to definition"))).toBe(false);
    await act(async () => view.dispatch({ effects: compartment.reconfigure(lspSupport({
      client, uri: "mapped.py", positionAt: offset => ({ line: 2, character: offset + 1 }),
      navigationIntent: true, captureNavigationContext, onNavigate: navigate, onReferences: references, runtimeValue,
    })) }));
    await waitFor(() => expect(actions().some(a => a.label.startsWith("Go to definition"))).toBe(true));
    let id = 0;
    const run = async (label: string, args = {}) => { let reply: unknown; await act(async () => { reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: actions().find(a => a.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return reply; };
    expect(await run("Go to definition at editor cursor (mapped.py)")).toMatchObject({ status: "result", outcome: { status: "completed", value: { total: 1, navigation_requested: true } } });
    expect(client.definition).toHaveBeenCalledWith("mapped.py", { line: 2, character: 1 });
    expect(navigate).toHaveBeenCalledWith(location, expect.any(Function), view);
    await run("Find references at editor cursor (mapped.py)");
    expect(references).toHaveBeenCalledWith([location], expect.objectContaining({ uri: "mapped.py" }), expect.any(Function));
    const resultIsCurrent = references.mock.calls[0][2] as () => boolean;
    expect(resultIsCurrent()).toBe(true);
    await act(async () => view.dispatch({ selection: { anchor: 1 } }));
    expect(resultIsCurrent()).toBe(false);
    await act(async () => view.dispatch({ selection: { anchor: 0 } }));
    expect(resultIsCurrent()).toBe(false);
    let resolveReferences!: (locations: typeof location[]) => void;
    vi.mocked(client.references).mockImplementationOnce(() => new Promise(done => { resolveReferences = done; }));
    const lateReferences = registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
      handle: actions().find(action => action.label.startsWith("Find references"))!.handle,
      arguments: {}, request_id: "late-references",
    });
    await waitFor(() => expect(client.references).toHaveBeenCalledTimes(2));
    foreground++;
    resolveReferences([location]);
    expect(await lateReferences).toMatchObject({ outcome: { status: "unavailable" } });
    expect(references).toHaveBeenCalledTimes(1);
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

</storycode:copy>
<a id="f-a7aead7d56cd-expression_statement-3"></a>
### `expression_statement`

The Structure action tests the normalized declaration location in the
current editor. A qualified child is selectable, a foreign or split-scalar
location is skipped, and a valid choice becomes stale when the caret moves.
Hierarchy uses the mapped authored caret and keeps its prepare item opaque.
Exercise both the explicit action and the palette entry so a stale prepare
cannot issue a follow-up and a fresh result preserves its selection range.

<storycode:copy id="f-a7aead7d56cd-expression_statement-3">
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

it("reveals only a current authored Structure position", async () => {
  const { editorLanguageActions } = await import("./languageActions");
  const host = document.body.appendChild(document.createElement("div"));
  const uri = "hick:///source.py";
  const rangeAt = (character: number) => ({start: {line: 0, character}, end: {line: 0, character}});
  const client = {documentSymbols: vi.fn().mockResolvedValue([
    {name: "Outer", selectionRange: rangeAt(0), children: [
      {name: "member", selectionRange: rangeAt(3)},
      {name: "split", selectionRange: rangeAt(2)},
    ]},
    {name: "foreign", location: {uri: "hick:///other.py", range: rangeAt(4)}},
  ])} as unknown as LspClient;
  let choose!: (offset: number) => void;
  const chooseOutline = vi.fn((_options, _current) => new Promise<number | null>(resolve => { choose = resolve; }));
  const onLocalNavigation = vi.fn((view: EditorView, departure: number) => [departure, view.state.selection.main.head]);
  const view = new EditorView({parent: host, state: EditorState.create({doc: "x😀member", extensions: lspSupport({
    client, uri, navigationIntent: true, chooseOutline, onLocalNavigation, positionAt: () => null, onNavigate: vi.fn(),
  })})});
  try {
    const action = view.state.facet(editorLanguageActions).find(action => action.id.endsWith(":outline"))!;
    const run = () => action.execute(view, {}, new AbortController().signal);
    expect(await run()).toMatchObject({status: "completed", value: {total: 2}});
    expect(chooseOutline.mock.calls[0][0]).toEqual([{label: "Outer  1:1", value: 0}, {label: "Outer.member  1:4", value: 3}]);
    view.dispatch({selection: {anchor: 1}});
    choose(3);
    await Promise.resolve();
    expect(view.state.selection.main.head).toBe(1);
    expect(onLocalNavigation).not.toHaveBeenCalled();
    await run();
    choose(3);
    await Promise.resolve();
    expect(view.state.selection.main.head).toBe(3);
    expect(onLocalNavigation.mock.results.map(result => result.value)).toEqual([[1, 3]]);
    expect(view.state.doc.toString()).toBe("x😀member");
  } finally { view.destroy(); host.remove(); }
});

it("uses the mapped hierarchy item and skips follow-up after the editor moves", async () => {
  const {editorLanguageActions} = await import("./languageActions");
  const host = document.body.appendChild(document.createElement("div"));
  const uri = "hick:///authored.md";
  const rangeAt = (character: number) => ({
    start: {line: 5, character}, end: {line: 5, character: character + 6},
  });
  const prepared = {name: "callee", uri, selectionRange: rangeAt(3), data: {providerToken: "opaque"}};
  const caller = {name: "caller", uri: "hick:///caller.py", range: rangeAt(0), selectionRange: rangeAt(4)};
  let release!: (value: unknown) => void;
  const request = vi.fn()
    .mockImplementationOnce(() => new Promise(resolve => { release = resolve; }))
    .mockResolvedValueOnce([prepared])
    .mockResolvedValueOnce([{from: caller, fromRanges: [rangeAt(22)]}]);
  const client = {request} as unknown as LspClient;
  const onRelatedLocations = vi.fn<(reading: RelatedLocationReading) => void>();
  const view = new EditorView({parent: host, state: EditorState.create({doc: "callee", extensions: lspSupport({
    client, uri: "hick-output:///generated.py", navigationIntent: true, onRelatedLocations,
    positionAt: () => null, targetAt: offset => ({uri, position: {line: 5, character: offset + 3}}),
    onNavigate: vi.fn(),
  })})});
  try {
    const action = view.state.facet(editorLanguageActions).find(action => action.command?.id === "callers")!;
    const late = action.execute(view, {}, new AbortController().signal);
    view.dispatch({selection: {anchor: 1}});
    release([prepared]);
    expect(await late).toMatchObject({status: "unavailable"});
    expect(request).toHaveBeenCalledOnce();
    expect(request).toHaveBeenCalledWith("textDocument/prepareCallHierarchy", {
      textDocument: {uri}, position: {line: 5, character: 3},
    });
    expect(onRelatedLocations).not.toHaveBeenCalled();
    action.command!.run(view, () => true);
    await waitFor(() => expect(onRelatedLocations).toHaveBeenCalledOnce());
    expect(request).toHaveBeenLastCalledWith("callHierarchy/incomingCalls", {item: prepared});
    const reading = onRelatedLocations.mock.calls[0][0];
    expect(reading.locations).toEqual([{name: "caller", uri: caller.uri, range: caller.selectionRange}]);
    expect(reading.current()).toBe(true);
    view.dispatch({selection: {anchor: 0}});
    expect(reading.current()).toBe(false);
  } finally { view.destroy(); host.remove(); }
});
</storycode:copy>

### Assemble `apps/web/src/editor/useEditorLanguageActions.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/useEditorLanguageActions.test.tsx"><storycode:paste select="#f-a7aead7d56cd-expression_statement"/><storycode:paste select="#f-a7aead7d56cd-expression_statement-2"/><storycode:paste select="#f-a7aead7d56cd-expression_statement-3"/></storycode:file>

</storycode:doc>
