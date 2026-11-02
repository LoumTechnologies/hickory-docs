<static:doc xmlns:static="http://www.hickorydocs.com/1.0">
# Keep touch inspection answers bound to the current gesture

A live language service can finish requests out of order, unlike the immediate frozen client. Drive the shared extension with deferred hover answers: the second gesture must win, and a destroyed editor must never publish a late result. A missing reference provider must not hide available documentation or definitions.

<static:copy id="static-apps-web-src-lsp-symbolInspection-test-ts-0">
import { it, expect, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { symbolInspection, type SymbolInspection } from "./symbolInspection";
import type { HoverResult } from "./client";

function editor(onInspect: (value: SymbolInspection) => void, hover: () => Promise<HoverResult | null>, failReferences = false) {
  const parent = document.createElement("div"); document.body.append(parent);
  const view = new EditorView({ parent, doc: "first second", extensions: [symbolInspection({
    client: { hover, definition: async () => [], references: async () => { if (failReferences) throw Error("unavailable"); return []; } },
    targetAt: at => ({ uri: "hick:///book.md", position: { line: 0, character: at } }), onInspect,
  })] });
  return { view, inspect(at: number) {
    view.dispatch({ selection: { anchor: at } });
    view.contentDOM.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", altKey: true, bubbles: true, cancelable: true }));
  }, destroy() { view.destroy(); parent.remove(); } };
}

it("keeps newer inspections and discards replies after editor destruction", async () => {
  const replies: ((value: HoverResult) => void)[] = [], inspect = vi.fn();
  const fixture = editor(inspect, () => new Promise(resolve => replies.push(resolve)));
  try {
    fixture.inspect(1); fixture.inspect(7);
    replies[1]({ contents: "second documentation" });
    await vi.waitFor(() => expect(inspect).toHaveBeenCalledTimes(1));
    expect(inspect.mock.calls[0][0]).toMatchObject({ label: "second", documentation: "second documentation" });
    replies[0]({ contents: "stale first documentation" });
    await Promise.resolve(); await Promise.resolve();
    expect(inspect).toHaveBeenCalledTimes(1);
    fixture.inspect(1); fixture.destroy();
    replies[2]({ contents: "destroyed" });
    await Promise.resolve(); await Promise.resolve();
    expect(inspect).toHaveBeenCalledTimes(1);
  } finally { if (fixture.view.dom.isConnected) fixture.destroy(); }
});

it("retains documentation when another provider answer is unavailable", async () => {
  const inspect = vi.fn(), fixture = editor(inspect, async () => ({ contents: [{ value: "type" }, "docs"] }), true);
  try {
    fixture.inspect(1);
    await vi.waitFor(() => expect(inspect).toHaveBeenCalledTimes(1));
    expect(inspect.mock.calls[0][0]).toMatchObject({ documentation: "type\n\ndocs", unavailable: ["References"], definitions: [] });
  } finally { fixture.destroy(); }
});
</static:copy>

<static:file path="apps/web/src/lsp/symbolInspection.test.ts"><static:paste select="#static-apps-web-src-lsp-symbolInspection-test-ts-0"/></static:file>
</static:doc>
