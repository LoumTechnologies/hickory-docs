<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lsp/macOptionFormat.test.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.


Captured SHA-256: `2f611a1edda00cd91d3c05047b0e797401316fac491164ff1420363f2450f16b`.

<book:entry path="apps/web/src/lsp/macOptionFormat.test.ts">
import { vi, expect, it, afterEach } from "vitest";
vi.hoisted(() => { Object.defineProperty(navigator, "platform", { value: "MacIntel", configurable: true }); });
import { EditorState } from "@codemirror/state";
import { EditorView, runScopeHandlers } from "@codemirror/view";
import { lspFeatures } from "./cmLspFeatures";
import type { LspClient } from "./client";
import { setKeymapSettings } from "../lib/keymap";
afterEach(() => setKeymapSettings({ profile: "hickory", overrides: {} }));

function fixture(readOnly = false) {
  const client = { serverCapabilities: null, semanticTokens: async () => null, inlayHints: async () => [], documentHighlight: async () => [], foldingRanges: async () => [], signatureHelp: async () => null, formatting: vi.fn().mockResolvedValue([]) } as unknown as LspClient;
  const view = new EditorView({ state: EditorState.create({ doc: "int main() {}", extensions: [EditorState.readOnly.of(readOnly), lspFeatures({ client, uri: "main.cpp", positionAt: p => ({ line: 0, character: p }), offsetAt: p => p.character })] }) });
  return { client, view };
}

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


it("keeps the actual formatter read-only guard when the Option fallback handles a key", async () => {
  const { client, view } = fixture(true);
  try {
    expect(runScopeHandlers(view, new KeyboardEvent("keydown", { key: "Ï", code: "KeyF", keyCode: 70, altKey: true, shiftKey: true }), "editor")).toBe(true);
    await Promise.resolve();
    expect(client.formatting).not.toHaveBeenCalled();
    expect(view.state.doc.toString()).toBe("int main() {}");
  } finally { view.destroy(); }
});
</book:entry>

</book:doc>
