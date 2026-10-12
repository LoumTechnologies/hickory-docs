import { useEffect, useRef, useState } from "react";
import { act, cleanup, fireEvent, render, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { emptyPane, open, tab, type Layout, type Tab } from "./layout";
import { SpatialShellView } from "./SpatialShellView";
import { saveSpatialSides } from "../lib/spatialSides";
import { revealLine } from "../lib/revealLine";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
afterEach(() => { cleanup(); localStorage.clear(); });
const live = new Map<string, EditorView>();
function Buffer({ entry }: { entry: Tab }) {
  const parent = useRef<HTMLDivElement>(null);
  useEffect(() => { const view = new EditorView({ parent: parent.current!, state: EditorState.create({ doc: `${entry.target}\nsecond line`, extensions: [lineNumbers(), history()] }) }); live.set(entry.target, view); return () => { view.destroy(); live.delete(entry.target); }; }, [entry.id]);
  return <div ref={parent} />;
}
function Harness({ mixed = false }: { mixed?: boolean }) {
  const [layout, setLayout] = useState<Layout>(() => { const pane = emptyPane(); return { root: { ...pane, tabs: [tab("document", "main.md"), tab("document", "related.md"), ...(mixed ? [tab("file", "app.py"), tab("chat", "conversation", "Agent")] : []), tab("tree", "folder")], active: 0 }, focus: pane.id }; });
  return <><button onClick={() => setLayout(current => open(current, tab("tool", "reading:proposal", "Review")))}>Open review</button><SpatialShellView layout={layout} onLayout={setLayout} sources={[]} files={[]} links={[]} render={entry => <Buffer entry={entry} />} /></>;
}
it("centers and restores the same editor instance with selection and undo state intact", async () => {
  const ui = render(<Harness />);
  await waitFor(() => expect(live.has("related.md")).toBe(true));
  const original = live.get("related.md")!;
  act(() => original.dispatch({ changes: { from: 0, insert: "my edit " }, selection: { anchor: 5 } }));
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  fireEvent.click(related.querySelector('[aria-label="Center"]')!);
  await waitFor(() => expect(related.dataset.spatialCenter).toBe("true"));
  expect(live.get("related.md")).toBe(original);
  expect(original.state.doc.toString()).toContain("my edit "); expect(original.state.selection.main.anchor).toBe(5);
  fireEvent.click(related.querySelector('[aria-label="Restore previous center"]')!);
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:main.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
  expect(live.get("related.md")).toBe(original);
});

it("switches documents, files and agents through center tabs without remounting their buffers", async () => {
  const ui = render(<Harness mixed />);
  await waitFor(() => expect(live.has("conversation")).toBe(true));
  const document = live.get("main.md")!, agent = live.get("conversation")!, file = live.get("app.py")!;
  act(() => { document.dispatch({ changes: { from: 0, insert: "draft " }, selection: { anchor: 4 } }); agent.dispatch({ changes: { from: 0, insert: "unsent prompt " } }); });
  const tabs = () => within(ui.getByRole("tablist", { name: "Center views" }));
  expect(tabs().queryByRole("tab", { name: "folder" })).toBeNull();
  for (const [label, key] of [["main.md", "document:main.md"], ["app.py", "file:app.py"], ["Agent", "chat:conversation"], ["main.md", "document:main.md"]]) {
    fireEvent.click(tabs().getByRole("tab", { name: label }));
    await waitFor(() => expect(ui.container.querySelector(`[data-spatial-key="${key}"]`)?.getAttribute("data-spatial-center")).toBe("true"));
    expect(tabs().getByRole("tab", { name: label }).getAttribute("aria-selected")).toBe("true");
  }
  expect(live.get("main.md")).toBe(document); expect(live.get("app.py")).toBe(file); expect(live.get("conversation")).toBe(agent);
  expect(document.state.selection.main.anchor).toBe(4); expect(agent.state.doc.toString()).toContain("unsent prompt");
  act(() => { expect(undo(document)).toBe(true); });
  expect(document.state.doc.toString()).toBe("main.md\nsecond line");
  fireEvent.click(ui.container.querySelector('[data-spatial-center=true] [aria-label="Close main.md"]')!);
  await waitFor(() => expect(tabs().queryByRole("tab", { name: "main.md" })).toBeNull());
  expect(tabs().getByRole("tab", { name: "Agent" }).getAttribute("aria-selected")).toBe("true");
});

it("supports arrow keys, Home and End in the center tab strip", async () => {
  const ui = render(<Harness />);
  const tabs = () => within(ui.getByRole("tablist", { name: "Center views" }));
  fireEvent.keyDown(tabs().getByRole("tab", { name: "main.md" }), { key: "ArrowRight" });
  await waitFor(() => expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true"));
  await waitFor(() => expect(document.activeElement).toBe(tabs().getByRole("tab", { name: "related.md" })));
  fireEvent.keyDown(document.activeElement!, { key: "ArrowRight" });
  await waitFor(() => expect(tabs().getByRole("tab", { name: "main.md" }).getAttribute("aria-selected")).toBe("true"));
  fireEvent.keyDown(tabs().getByRole("tab", { name: "main.md" }), { key: "End" });
  expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true");
  fireEvent.keyDown(tabs().getByRole("tab", { name: "related.md" }), { key: "Home" });
  expect(tabs().getByRole("tab", { name: "main.md" }).getAttribute("aria-selected")).toBe("true");
});

it("hides right context by default, returns its space to the center and keeps toggled editors mounted", async () => {
  const ui = render(<Harness />);
  const shell = ui.container.querySelector<HTMLElement>(".spatial-shell")!;
  Object.defineProperties(shell, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  const center = ui.container.querySelector<HTMLElement>('[data-spatial-center=true]')!;
  expect(related.hidden).toBe(true);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=right]')!.hidden).toBe(true);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(false);
  const editor = live.get("related.md")!;
  act(() => editor.dispatch({ changes: { from: 0, insert: "keep me " } }));
  await waitFor(() => expect(parseFloat(center.style.width)).toBeGreaterThan(0));
  const defaultWidth = parseFloat(center.style.width);
  act(() => saveSpatialSides({ left: true, right: true }));
  await waitFor(() => expect(related.hidden).toBe(false));
  await waitFor(() => expect(parseFloat(center.style.width)).toBeLessThan(defaultWidth));
  act(() => saveSpatialSides({ left: false, right: false }));
  await waitFor(() => expect(related.hidden).toBe(true));
  await waitFor(() => expect(parseFloat(center.style.width)).toBe(1200));
  expect(center.hidden).toBe(false); expect(live.get("related.md")).toBe(editor);
  fireEvent.click(within(ui.getByRole("tablist", { name: "Center views" })).getByRole("tab", { name: "related.md" }));
  expect(related.hidden).toBe(false); expect(related.dataset.spatialCenter).toBe("true");
  expect(editor.state.doc.toString()).toContain("keep me ");
});

it("centers a revealed file when its context side is hidden", async () => {
  const ui = render(<Harness />);
  act(() => revealLine("related.md", 1));
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
});

it("expands a newly opened change review instead of leaving its approval controls collapsed", async () => {
  saveSpatialSides({ left: true, right: true });
  const ui = render(<Harness />);
  fireEvent.click(ui.getByRole("button", { name: "Open review" }));
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="tool:reading:proposal"]')?.getAttribute("data-expanded")).toBe("true"));
});
