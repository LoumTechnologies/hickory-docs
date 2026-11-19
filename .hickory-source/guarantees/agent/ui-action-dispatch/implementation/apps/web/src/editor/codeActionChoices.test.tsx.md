<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/codeActionChoices.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7d80c08f42fb"></a>
## `apps/web/src/editor/codeActionChoices.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-7d80c08f42fb-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-7d80c08f42fb-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { cmKeyOf } from "../lib/keymap";
import { EditorView, keymap } from "@codemirror/view";
import { lspFeatures, type ProviderPreviewRequest } from "../lsp/cmLspFeatures";
import type { LspClient } from "../lsp/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-7d80c08f42fb-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-7d80c08f42fb-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("retains real source-bound code choices and refuses forged, stale or unsupported actions", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const offered = [
    { title: "Fix name", edit: { changes: { "file.py": [{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } }, newText: "fixed" }] } } },
    { title: "Requires server command", command: { title: "command", command: "unsupported" } },
    { title: "Workspace change", edit: { changes: { "file.py": [{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } }, newText: "partial" }], "other.py": [{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } }, newText: "partial" }] } } },
    { title: "Incomplete projection", disabled: { reason: "Cannot map all edit targets" }, edit: { changes: { "file.py": [{ range: { start: { line: 0, character: 0 }, end: { line: 0, character: 3 } }, newText: "partial" }] } } },
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
    expect(choices.map((c: { title: string }) => c.title)).toEqual(["Fix name", "Requires server command", "Workspace change", "Incomplete projection"]);
    expect(await run("Apply captured editor code action (file.py)", { snapshot, choice: choices[2].id })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: expect.stringContaining("workspace preview") } });
    expect(choices[3].disabled_reason).toBe("Cannot map all edit targets");
    expect(await run("Apply captured editor code action (file.py)", { snapshot, choice: choices[3].id })).toMatchObject({ status: "result", outcome: { status: "unavailable", reason: "Cannot map all edit targets" } });
    expect(view.state.doc.toString()).toBe("bad");
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
</storycode:copy>

### Preview guards survive the chooser

The chooser hands Main a retained action and the range used to query it. The preview still needs the originating editor after the chooser closes. This test checks that selecting a choice leaves the local text untouched and that losing the originating workspace context invalidates the preview.

<storycode:copy id="main-code-action-preview">
it("hands Main the original range and invalidates the preview when its editor context changes", async () => {
  const request = vi.fn();
  const action = {title: "Fix name", kind: "quickfix", data: {hickory_action: "retained"}};
  const codeActions = vi.fn().mockResolvedValue([action]);
  const client = {codeActions, request, foldingRanges: async () => []} as unknown as LspClient;
  let context = 0;
  const preview = vi.fn(async (_preview: ProviderPreviewRequest) => ({status: "completed" as const, value: {preview_ready: true}}));
  const host = document.createElement("div");
  document.body.append(host);
  const view = new EditorView({parent: host, state: EditorState.create({
    doc: "bad", selection: {anchor: 0, head: 2},
    extensions: lspFeatures({
      client, uri: "file.py", positionAt: p => ({line: 0, character: p}), offsetAt: p => p.character,
      onCodeActions: actions => actions[0], onProviderPreview: preview,
      captureChangeContext: () => {
        const original = context;
        return () => context === original;
      },
    }),
  })});
  try {
    const binding = view.state.facet(keymap).flat().find(b => b.key === cmKeyOf("editor.codeAction"))!;
    await act(async () => { expect(binding.run!(view)).toBe(true); });
    expect(preview).toHaveBeenCalledOnce();
    const captured = preview.mock.calls[0][0];
    if (captured.kind !== "code action") throw Error("Expected the captured code action");
    expect(captured.range).toEqual({start: {line: 0, character: 0}, end: {line: 0, character: 2}});
    expect(captured.action).toEqual(action);
    expect(captured.current()).toBe(true);
    expect(request).not.toHaveBeenCalled();
    expect(view.state.doc.toString()).toBe("bad");
    context++;
    expect(captured.current()).toBe(false);
  } finally { view.destroy(); host.remove(); }
});
</storycode:copy>

### Assemble `apps/web/src/editor/codeActionChoices.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/codeActionChoices.test.tsx"><storycode:paste select="#f-7d80c08f42fb-expression_statement"/><storycode:paste select="#f-7d80c08f42fb-expression_statement-2"/><storycode:paste select="#main-code-action-preview"/></storycode:file>

</storycode:doc>
