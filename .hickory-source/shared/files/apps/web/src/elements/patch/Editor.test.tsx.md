<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/elements/patch/Editor.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Captured reading surfaces share owned native actions](../../../../../../../guarantees/agent/scoped-reading-surfaces-share-native-actions/contract.md)
- [Patch regeneration revalidates downstream construction](../../../../../../../guarantees/literate-projects/patch-edits-revalidate-downstream-construction/contract.md)

Captured SHA-256: `a4ad3e62b41f4a9a0cfae7d8333e9391ca879f5846da2c7e211c2b48841610c8`.

<book:entry path="apps/web/src/elements/patch/Editor.test.tsx">
// Guarantee: docs/guarantees/agent/scoped-reading-surfaces-share-native-actions.md
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorState, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../../api/client";
import { parseHickDoc } from "../../editor/hickDoc";
import { materializeProject } from "../../editor/hickLang";
import type { RenderedSlot } from "../../editor/rendered";
import type { SlotContext } from "../types";
import { UiActions } from "../../lib/uiActions";
import { UiActionContext } from "../../lib/uiActionBridge";
import { PatchEditor } from "./Editor";

vi.mock("../../api/client", () => ({ request: vi.fn() }));
vi.mock("../../components/ComparisonTools", () => ({ ComparisonTools: () => null }));
const views: EditorView[] = [];
afterEach(() => { cleanup(); views.splice(0).forEach(view => view.destroy()); vi.clearAllMocks(); });
const source = `<hick:project/><hick:file id="base" path="app.py">old\n</hick:file>
<hick:patch id="change" after="base">
--- a/app.py
+++ b/app.py
@@ -1 +1 @@
-old
+new
</hick:patch>`;
function mount(text = source, readOnly = false, registry?: UiActions) {
  const view = new EditorView({ state: EditorState.create({ doc: text, extensions: [EditorState.readOnly.of(readOnly)] }) }); views.push(view);
  const block = parseHickDoc(text).blocks.find(block => block.name === "patch")!;
  const slot: RenderedSlot = { key: "patch-0", el: document.createElement("div"), kind: "patch", index: 0,
    at: block.from, span: [block.from, block.to], text: text.slice(block.contentFrom, block.contentTo), renderer: "", asserts: [] };
  const cx: SlotContext = { view, path: "project.md", docId: "untitled", execBlocks: [], diagramBlocks: [], runningCells: new Set(), replaying: [], replaceBlockContent: vi.fn() };
  const ui = render(registry ? <UiActionContext.Provider value={registry}><PatchEditor slot={slot} cx={cx} /></UiActionContext.Provider> : <PatchEditor slot={slot} cx={cx} />);
  const inner = () => EditorView.findFromDOM(ui.container.querySelector<HTMLElement>(".comparison-side:last-child .cm-editor")!)!;
  return { ui, view, inner, slot, cx };
}

// Guarantee: docs/guarantees/literate-projects/patch-edits-revalidate-downstream-construction.md
it("opens as a normal code editor and automatically regenerates the patch without losing selection", async () => {
  const { ui, view, inner } = mount();
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect(screen.queryByText("Inspect and edit patch")).toBeNull();
  const editor = inner();
  act(() => editor.dispatch({ changes: { from: 0, to: 3, insert: "héllo 😀" }, selection: { anchor: 8 } }));
  await waitFor(() => expect(view.state.doc.toString()).toContain("+héllo 😀"));
  expect(inner()).toBe(editor); expect(editor.state.selection.main.anchor).toBe(8);
  expect(new TextDecoder().decode(new Uint8Array(materializeProject("project.md", { "project.md": view.state.doc.toString() }).new[0].bytes))).toBe("héllo 😀\n");
  expect(screen.getByText("app.py · Up to date")).toBeTruthy(); expect(request).not.toHaveBeenCalled();
});

it("keeps a downstream-breaking edit as an unsaved draft and saves a corrected draft", async () => {
  const text = `${source}\n<hick:patch id="later" after="change">
--- a/app.py
+++ b/app.py
@@ -1 +1 @@
-new
+final
</hick:patch>`;
  const { ui, view, inner } = mount(text);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  act(() => inner().dispatch({ changes: { from: 0, to: 3, insert: "broken" } }));
  await waitFor(() => expect(screen.getByRole("alert").textContent).toMatch(/invalidates/));
  expect(view.state.doc.toString()).toBe(text); expect(inner().state.doc.toString()).toBe("broken\n");
  expect(screen.getByText("app.py · Unsaved draft")).toBeTruthy();
  act(() => inner().dispatch({ changes: { from: 0, to: 6, insert: "new" } }));
  await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
  await waitFor(() => expect(screen.getByText("app.py · Up to date")).toBeTruthy());
});

it("makes historical code read-only while the New code is editable", async () => {
  const { ui } = mount();
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect((screen.getByLabelText("Comparison layout") as HTMLSelectElement).value).toBe("side-by-side");
  const editors = ui.container.querySelectorAll(".comparison-side .cm-content");
  expect(editors[0].getAttribute("contenteditable")).toBe("false");
  expect(editors[1].getAttribute("contenteditable")).toBe("true");
});

it("respects an immutable owning document", async () => {
  const { ui } = mount(source, true);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect(ui.container.querySelector(".comparison-side .cm-content")?.getAttribute("contenteditable")).toBe("false");
});

it("discloses patch choices and preserves agent edit markers through regeneration", async () => {
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const { ui, view } = mount(source, false, registry);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  const markers: string[] = [];
  view.dispatch({ effects: StateEffect.appendConfig.of(EditorView.updateListener.of(update => {
    for (const transaction of update.transactions) if (transaction.isUserEvent("input.uiAgentProjectPatch")) markers.push("agent");
  })) });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  const run = async (label: string, args: Record<string, string | number | boolean>) => {
    const action = actions().find(action => action.label === label && action.enabled)!;
    let result: unknown;
    await act(async () => { result = await registry.act(registry.scope, { id: "python:fixture", kind: "agent" }, { handle: action.handle, arguments: args, request_id: crypto.randomUUID() }); });
    return result;
  };
  expect(actions().some(action => action.label === "Select constructed patch file")).toBe(false);
  await run("Set Patch options visibility", { open: true });
  expect(await run("Select constructed patch editable side", { side: "old" })).toMatchObject({ status: "result" });
  expect((screen.getByLabelText("Editable side") as HTMLSelectElement).value).toBe("old");
  fireEvent.change(screen.getByLabelText("Editable side"), { target: { value: "new" } });
  expect(await run("Replace comparison draft range", { from: 0, to: 3, text: "agent code" })).toMatchObject({ status: "result", outcome: { value: { draft_changed: true, saved: false } } });
  await waitFor(() => expect(view.state.doc.toString()).toContain("+agent code"));
  expect(markers).toEqual(["agent"]);
  await run("Set Patch options visibility", { open: false });
  expect(actions().some(action => action.label === "Select constructed patch file")).toBe(false);
});
it("does not adopt an inspection error from a retired document owner", async () => {
  const { ui, slot, cx } = mount();
  let reject!: (error: Error) => void;
  vi.mocked(request).mockImplementation(async (_method, path) => path.includes("first") ? new Promise((_resolve, fail) => { reject = fail; }) : { entry: "project.md", sources: { "project.md": source } });
  ui.rerender(<PatchEditor slot={slot} cx={{ ...cx, docId: "first" }} />);
  await waitFor(() => expect(reject).toBeTruthy());
  ui.rerender(<PatchEditor slot={slot} cx={{ ...cx, docId: "second" }} />);
  await screen.findByText("app.py · Up to date");
  await act(async () => { reject(Error("retired inspection failed")); });
  expect(screen.queryByText("retired inspection failed")).toBeNull();
});
</book:entry>

</book:doc>
