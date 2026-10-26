<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lsp/macOptionFormat.test.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-9efc23041235"></a>
## `apps/web/src/lsp/macOptionFormat.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-9efc23041235-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement">
import { vi, expect, it, afterEach } from "vitest";
vi.hoisted(() => { Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true }); });
</storycode:copy>
<a id="f-9efc23041235-import_statement"></a>
### `import_statement`

<storycode:copy id="f-9efc23041235-import_statement">
import { EditorState } from "@codemirror/state";
</storycode:copy>
<a id="f-9efc23041235-import_statement-2"></a>
### `import_statement`

<storycode:copy id="f-9efc23041235-import_statement-2">
import { EditorView, runScopeHandlers } from "@codemirror/view";
</storycode:copy>
<a id="f-9efc23041235-import_statement-3"></a>
### `import_statement`

<storycode:copy id="f-9efc23041235-import_statement-3">
import { lspFeatures } from "./cmLspFeatures";
</storycode:copy>
<a id="f-9efc23041235-import_statement-4"></a>
### `import_statement`

<storycode:copy id="f-9efc23041235-import_statement-4">
import type { LspClient } from "./client";
</storycode:copy>
<a id="f-9efc23041235-import_statement-5"></a>
### `import_statement`

<storycode:copy id="f-9efc23041235-import_statement-5">
import { setKeymapSettings } from "../lib/keymap";
</storycode:copy>
<a id="f-9efc23041235-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement-2">
afterEach(() => setKeymapSettings({ profile: "hickory", overrides: {} }));

</storycode:copy>
<a id="f-9efc23041235-fixture"></a>
### `fixture`

<storycode:copy id="f-9efc23041235-fixture">
function fixture(readOnly = false) {
  const client = { serverCapabilities: null, semanticTokens: async () => null, inlayHints: async () => [], documentHighlight: async () => [], foldingRanges: async () => [], signatureHelp: async () => null, formatting: vi.fn().mockResolvedValue([]) } as unknown as LspClient;
  const view = new EditorView({ state: EditorState.create({ doc: "int main() {}", extensions: [EditorState.readOnly.of(readOnly), lspFeatures({ client, uri: "main.cpp", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character })] }) });
  return { client, view };
}

</storycode:copy>
<a id="f-9efc23041235-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement-3">
it("formats macOS Option-generated characters once through the actual configured formatter keybinding", async () => {
  const { client, view } = fixture();
  try {
    const ascii = new KeyboardEvent("keydown", { key: "f", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true, bubbles: true, cancelable: true });
    expect(runScopeHandlers(view, ascii, "editor")).toBe(true);
    await Promise.resolve();
    expect(client.formatting).toHaveBeenCalledTimes(1);
    const optionCharacter = new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true, bubbles: true, cancelable: true });
    expect(runScopeHandlers(view, optionCharacter, "editor")).toBe(true);
    await Promise.resolve();
    expect(client.formatting).toHaveBeenCalledTimes(2);
  } finally { view.destroy(); }
});

</storycode:copy>
<a id="f-9efc23041235-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement-4">
it("preserves other Option text, composition, and AltGraph", async () => {
  const { client, view } = fixture();
  const event = (options: KeyboardEventInit = {}) => new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true, ...options });
  try {
    expect(runScopeHandlers(view, event({ key: "˝", code: "KeyG", keyCode: 71 }), "editor")).toBe(false);
    expect(runScopeHandlers(view, event({ shiftKey: false }), "editor")).toBe(false);
    expect(runScopeHandlers(view, event({ isComposing: true }), "editor")).toBe(false);
    const altGraph = event();
    Object.defineProperty(altGraph, "getModifierState", { value: (modifier: string) => modifier === "AltGraph" });
    expect(runScopeHandlers(view, altGraph, "editor")).toBe(false);
    expect(runScopeHandlers(view, event({ code: "" }), "editor")).toBe(false);
    expect(client.formatting).not.toHaveBeenCalled();
    expect(view.state.doc.toString()).toBe("int main() {}");
  } finally { view.destroy(); }
});

</storycode:copy>
<a id="f-9efc23041235-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement-5">
it("honors configured overrides and unbinding instead of capturing a global Option-F", async () => {
  setKeymapSettings({ profile: "hickory", overrides: { "editor.format": "Shift+Alt+G" } });
  const configured = fixture();
  try {
    expect(runScopeHandlers(configured.view, new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true }), "editor")).toBe(false);
    expect(runScopeHandlers(configured.view, new KeyboardEvent("keydown", { key: "˝", code: "KeyG", keyCode: 71, altKey: true, shiftKey: true }), "editor")).toBe(true);
    await Promise.resolve();
    expect(configured.client.formatting).toHaveBeenCalledTimes(1);
  } finally { configured.view.destroy(); }
  setKeymapSettings({ profile: "hickory", overrides: { "editor.format": "" } });
  const unbound = fixture();
  try {
    expect(runScopeHandlers(unbound.view, new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true }), "editor")).toBe(false);
    expect(unbound.client.formatting).not.toHaveBeenCalled();
  } finally { unbound.view.destroy(); }
});


</storycode:copy>
<a id="f-9efc23041235-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-9efc23041235-expression_statement-6">
it("keeps the actual formatter read-only guard when the Option fallback handles a key", async () => {
  const { client, view } = fixture(true);
  try {
    expect(runScopeHandlers(view, new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true }), "editor")).toBe(true);
    await Promise.resolve();
    expect(client.formatting).not.toHaveBeenCalled();
    expect(view.state.doc.toString()).toBe("int main() {}");
  } finally { view.destroy(); }
});
</storycode:copy>

### Assemble `apps/web/src/lsp/macOptionFormat.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lsp/macOptionFormat.test.ts"><storycode:paste select="#f-9efc23041235-expression_statement"/><storycode:paste select="#f-9efc23041235-import_statement"/><storycode:paste select="#f-9efc23041235-import_statement-2"/><storycode:paste select="#f-9efc23041235-import_statement-3"/><storycode:paste select="#f-9efc23041235-import_statement-4"/><storycode:paste select="#f-9efc23041235-import_statement-5"/><storycode:paste select="#f-9efc23041235-expression_statement-2"/><storycode:paste select="#f-9efc23041235-fixture"/><storycode:paste select="#f-9efc23041235-expression_statement-3"/><storycode:paste select="#f-9efc23041235-expression_statement-4"/><storycode:paste select="#f-9efc23041235-expression_statement-5"/><storycode:paste select="#f-9efc23041235-expression_statement-6"/></storycode:file>

</storycode:doc>
