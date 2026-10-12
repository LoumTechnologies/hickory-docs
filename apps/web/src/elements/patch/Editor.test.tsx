import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";
import { request } from "../../api/client";
import { parseHickDoc } from "../../editor/hickDoc";
import { materializeProject } from "../../editor/hickLang";
import type { RenderedSlot } from "../../editor/rendered";
import type { SlotContext } from "../types";
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
function mount(text = source, readOnly = false) {
  const view = new EditorView({ state: EditorState.create({ doc: text, extensions: [EditorState.readOnly.of(readOnly)] }) }); views.push(view);
  const block = parseHickDoc(text).blocks.find(block => block.name === "patch")!;
  const slot: RenderedSlot = { key: "patch-0", el: document.createElement("div"), kind: "patch", index: 0,
    at: block.from, span: [block.from, block.to], text: text.slice(block.contentFrom, block.contentTo), renderer: "", asserts: [] };
  const cx: SlotContext = { view, path: "project.md", docId: "untitled", execBlocks: [], diagramBlocks: [], runningCells: new Set(), replaying: [], replaceBlockContent: vi.fn() };
  const ui = render(<PatchEditor slot={slot} cx={cx} />);
  const inner = () => EditorView.findFromDOM(ui.container.querySelector<HTMLElement>(".comparison-side .cm-editor")!)!;
  return { ui, view, inner };
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
  fireEvent.change(screen.getByLabelText("Comparison layout"), { target: { value: "side-by-side" } });
  const editors = ui.container.querySelectorAll(".comparison-side .cm-content");
  expect(editors[0].getAttribute("contenteditable")).toBe("false");
  expect(editors[1].getAttribute("contenteditable")).toBe("true");
});

it("respects an immutable owning document", async () => {
  const { ui } = mount(source, true);
  await waitFor(() => expect(ui.container.querySelector(".comparison-side .cm-editor")).not.toBeNull());
  expect(ui.container.querySelector(".comparison-side .cm-content")?.getAttribute("contenteditable")).toBe("false");
});
