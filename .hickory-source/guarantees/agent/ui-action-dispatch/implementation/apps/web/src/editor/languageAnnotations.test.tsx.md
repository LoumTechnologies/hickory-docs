<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/editor/languageAnnotations.test.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-54c8647e6438"></a>
## `apps/web/src/editor/languageAnnotations.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Explicit UI action dispatch checks scope, state, policy and retries](../../../../../contract.md).

<a id="f-54c8647e6438-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-54c8647e6438-expression_statement">
// @vitest-environment jsdom
import { act, cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { inlayHintField, lspFeatures, signatureField } from "../lsp/cmLspFeatures";
import type { LspClient } from "../lsp/client";
import { UiActions } from "../lib/uiActions";
import { UiActionContext } from "../lib/uiActionBridge";
import { useEditorActions } from "./uiEditorActions";
afterEach(cleanup);
</storycode:copy>
<a id="f-54c8647e6438-quiet"></a>
### `quiet`

<storycode:copy id="f-54c8647e6438-quiet">
const quiet = { serverCapabilities: null, semanticTokens: async () => null, inlayHints: async () => [], documentHighlight: async () => [], foldingRanges: async () => [], signatureHelp: async () => null };
</storycode:copy>
<a id="f-54c8647e6438-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-54c8647e6438-expression_statement-2">
// Guarantee: docs/guarantees/agent/ui-action-dispatch.md
it("reads actual rendered hints and signature state, and clears signatures after editing", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const client = { ...quiet, inlayHints: vi.fn().mockResolvedValue([{ position: { line: 0, character: 1 }, label: ": int" }]), signatureHelp: vi.fn().mockResolvedValue({ signatures: [{ label: "f(value: int)" }] }) } as unknown as LspClient;
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "f", selection: { anchor: 1 }, extensions: lspFeatures({ client, uri: "file.py", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character, inlayHints: true }) }), parent: host });
  function Harness() { useEditorActions(view, "source"); return null; }
  try {
    render(<UiActionContext.Provider value={registry}><Harness /></UiActionContext.Provider>);
    await act(async () => view.dispatch({ changes: { from: 1, insert: "(" }, selection: { anchor: 2 } }));
    await waitFor(() => expect(view.state.field(signatureField)?.label).toBe("f(value: int)"));
    expect(view.dom.querySelectorAll(".cm-lsp-signature")).toHaveLength(1);
    expect(view.dom.querySelector(".cm-lsp-signature")?.textContent).toBe("f(value: int)");
    await waitFor(() => expect(view.state.field(inlayHintField).size).toBe(1));
    const actions = () => registry.groups().groups.flatMap(group => registry.observe(group).actions);
    const reading = actions().find(a => a.label === "Read editor language annotations (file.py)")!;
    const reply = await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, { handle: reading.handle, arguments: { offset: 0, limit: 8192 }, request_id: "annotations" });
    if (reply.status !== "result" || reply.outcome.status !== "completed") throw new Error(JSON.stringify(reply));
    const readingValue = reply.outcome.value as { bytes: number[] };
    const value = JSON.parse(new TextDecoder().decode(new Uint8Array(readingValue.bytes)));
    expect(value.signature).toEqual({ label: "f(value: int)", at: 2 });
    expect(value.inlay_hints).toEqual([{ from: 1, to: 1, class: null, text: ": int" }]);
    await act(async () => view.dispatch({ changes: { from: 0, insert: "changed" } }));
    expect(view.state.field(signatureField)).toBeNull();
    expect(view.dom.querySelector(".cm-lsp-signature")).toBeNull();
  } finally { cleanup(); view.destroy(); host.remove(); }
});

</storycode:copy>
<a id="f-54c8647e6438-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-54c8647e6438-expression_statement-3">
it("ignores signature and inlay responses whose source changed before arrival", async () => {
  let hint!: (value: { position: { line: number; character: number }; label: string }[]) => void;
  let signature!: (value: { signatures: { label: string }[] }) => void;
  const hints = vi.fn().mockImplementation(() => new Promise(done => { hint = done; }));
  const signatures = vi.fn().mockImplementation(() => new Promise(done => { signature = done; }));
  const client = { ...quiet, inlayHints: hints, signatureHelp: signatures } as unknown as LspClient;
  const host = document.createElement("div"); document.body.append(host);
  const view = new EditorView({ state: EditorState.create({ doc: "f", selection: { anchor: 1 }, extensions: lspFeatures({ client, uri: "file.py", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character, inlayHints: true }) }), parent: host });
  try {
    await act(async () => view.dispatch({ changes: { from: 1, insert: "(" }, selection: { anchor: 2 } }));
    await waitFor(() => expect(hints).toHaveBeenCalledTimes(1));
    expect(signatures).toHaveBeenCalledTimes(1);
    await act(async () => {
      view.dispatch({ changes: { from: 0, to: 2, insert: "different" } });
      hint([{ position: { line: 0, character: 1 }, label: "OLD HINT" }]);
      signature({ signatures: [{ label: "OLD SIGNATURE" }] });
    });
    expect(view.state.field(inlayHintField).size).toBe(0);
    expect(view.state.field(signatureField)).toBeNull();
  } finally { view.destroy(); host.remove(); }
});
</storycode:copy>

### Assemble `apps/web/src/editor/languageAnnotations.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/editor/languageAnnotations.test.tsx"><storycode:paste select="#f-54c8647e6438-expression_statement"/><storycode:paste select="#f-54c8647e6438-quiet"/><storycode:paste select="#f-54c8647e6438-expression_statement-2"/><storycode:paste select="#f-54c8647e6438-expression_statement-3"/></storycode:file>

</storycode:doc>
