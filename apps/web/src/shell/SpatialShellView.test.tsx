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
import { publishReading, forgetReading, openReading } from "../lib/readingViews";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
afterEach(() => { cleanup(); localStorage.clear(); forgetReading("proposal"); });
const live = new Map<string, EditorView>();
function Buffer({ entry }: { entry: Tab }) {
  const parent = useRef<HTMLDivElement>(null);
  useEffect(() => { const view = new EditorView({ parent: parent.current!, state: EditorState.create({ doc: `${entry.target}\nsecond line`, extensions: [lineNumbers(), history()] }) }); live.set(entry.target, view); return () => { view.destroy(); live.delete(entry.target); }; }, [entry.id]);
  return <div ref={parent} />;
}
function Harness({ mixed = false, related = false, files = false }: { mixed?: boolean; related?: boolean; files?: boolean }) {
  const [layout, setLayout] = useState<Layout>(() => { const pane = emptyPane(); return { root: { ...pane, tabs: [tab("document", "main.md"), tab("document", "related.md"), ...(mixed ? [tab("file", "app.py"), tab("chat", "conversation", "Agent")] : []), tab("tree", "folder")], active: 0 }, focus: pane.id }; });
  return <><button onClick={() => { publishReading("proposal", { title: "Review", path: "note.md", before: "old", after: "new", status: "pending" }); setLayout(current => open(current, tab("tool", "reading:proposal", "Review"))); }}>Open review</button><SpatialShellView layout={layout} onLayout={setLayout} sources={[]} files={[]} hasFiles={files} links={related ? [{ key: "related", title: "Related file", family: "declared", from: { path: "main.md", lines: [1, 1] }, to: { path: "related.md" } }] : []} render={entry => <Buffer entry={entry} />} /></>;
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
  expect(tabs().getByRole("tab", { name: "related.md" }).getAttribute("aria-selected")).toBe("true");
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

it("uses the full width and shows no drawer handles when only unrelated tabs are open", async () => {
  const ui = render(<Harness />);
  const shell = ui.container.querySelector<HTMLElement>(".spatial-shell")!;
  Object.defineProperties(shell, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  await waitFor(() => expect(ui.container.querySelector<HTMLElement>('[data-spatial-center=true]')!.style.width).toBe("1200px"));
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!.hidden).toBe(true);
});

it("opens related content as an overlay, reserves space only when pinned, and keeps drafts and selection", async () => {
  const ui = render(<Harness related />);
  const shell = ui.container.querySelector<HTMLElement>(".spatial-shell")!;
  Object.defineProperties(shell, { clientWidth: { value: 1200 }, clientHeight: { value: 800 } });
  const center = ui.container.querySelector<HTMLElement>('[data-spatial-center=true]')!;
  const related = ui.container.querySelector<HTMLElement>('[data-spatial-key="document:related.md"]')!;
  const original = live.get("related.md")!;
  expect(related.hidden).toBe(true);
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  await waitFor(() => expect(related.hidden).toBe(false));
  await waitFor(() => expect(center.style.width).toBe("1200px"));
  act(() => original.dispatch({ changes: { from: 0, insert: "draft " }, selection: { anchor: 3 } }));
  fireEvent.click(ui.getByRole("button", { name: "Pin left drawer" }));
  await waitFor(() => expect(parseFloat(center.style.width)).toBeLessThan(1200));
  fireEvent.click(ui.getByRole("button", { name: "Unpin left drawer" }));
  await waitFor(() => expect(center.style.width).toBe("1200px"));
  fireEvent.keyDown(shell, { key: "Escape" });
  await waitFor(() => expect(related.hidden).toBe(true));
  expect(live.get("related.md")).toBe(original); expect(original.state.doc.toString()).toContain("draft "); expect(original.state.selection.main.anchor).toBe(3);
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  fireEvent.pointerDown(center);
  await waitFor(() => expect(related.hidden).toBe(true));
  ui.rerender(<Harness />);
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
});

it("keeps Files collapsed, restores drawer pins, and respects the independent side settings", async () => {
  const ui = render(<Harness files />);
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.queryByRole("button", { name: "Show right drawer" })).toBeNull();
  fireEvent.click(ui.getByRole("button", { name: "Show left drawer" }));
  fireEvent.click(ui.getByRole("button", { name: "Pin left drawer" }));
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(false);
  const editor = live.get("folder")!;
  act(() => saveSpatialSides({ left: false, right: true }));
  expect(ui.container.querySelector<HTMLElement>('[data-spatial-files=left]')!.hidden).toBe(true);
  expect(ui.getByRole("button", { name: "Show right drawer" })).toBeTruthy();
  act(() => saveSpatialSides({ left: true, right: false }));
  expect(live.get("folder")).toBe(editor);
  ui.unmount();
  const restored = render(<Harness files />);
  expect(restored.getByRole("button", { name: "Unpin left drawer" })).toBeTruthy();
});

it("centers a revealed file when its context side is hidden", async () => {
  const ui = render(<Harness />);
  act(() => revealLine("related.md", 1));
  await waitFor(() => expect(ui.container.querySelector('[data-spatial-key="document:related.md"]')?.getAttribute("data-spatial-center")).toBe("true"));
});

it("announces a pending review in a collapsed drawer without moving the editor", async () => {
  saveSpatialSides({ left: true, right: true });
  const ui = render(<Harness />);
  fireEvent.click(ui.getByRole("button", { name: "Open review" }));
  const show = ui.getByRole("button", { name: /Show .* drawer/ });
  const review = ui.container.querySelector<HTMLElement>('[data-spatial-key="tool:reading:proposal"]')!;
  expect(review.hidden).toBe(true);
  fireEvent.click(show);
  expect(review.hidden).toBe(false);
  act(() => publishReading("proposal", { title: "Review", path: "note.md", before: "old", after: "new", status: "applied" }));
  expect(review.hidden).toBe(true);
  expect(ui.queryByRole("button", { name: /Show .* drawer/ })).toBeNull();
});

it("centers an explicitly requested review while background proposals remain collapsed", async () => {
  const ui = render(<Harness />);
  fireEvent.click(ui.getByRole("button", { name: "Open review" }));
  expect(ui.container.querySelector('[data-spatial-key="tool:reading:proposal"]')?.getAttribute("data-spatial-center")).toBe("false");
  act(() => openReading("proposal", "Review"));
  expect(ui.container.querySelector('[data-spatial-key="tool:reading:proposal"]')?.getAttribute("data-spatial-center")).toBe("true");
});
