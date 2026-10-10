import { useEffect, useRef, useState } from "react";
import { act, cleanup, fireEvent, render, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView, lineNumbers } from "@codemirror/view";
import { emptyPane, tab, type Layout, type Tab } from "./layout";
import { SpatialShellView } from "./SpatialShellView";

// Guarantee: docs/guarantees/shell/spatial-views-preserve-the-editor.md
afterEach(() => { cleanup(); localStorage.clear(); });
const live = new Map<string, EditorView>();
function Buffer({ entry }: { entry: Tab }) {
  const parent = useRef<HTMLDivElement>(null);
  useEffect(() => { const view = new EditorView({ parent: parent.current!, state: EditorState.create({ doc: `${entry.target}\nsecond line`, extensions: [lineNumbers()] }) }); live.set(entry.target, view); return () => { view.destroy(); live.delete(entry.target); }; }, [entry.id]);
  return <div ref={parent} />;
}
function Harness() {
  const [layout, setLayout] = useState<Layout>(() => { const pane = emptyPane(); return { root: { ...pane, tabs: [tab("document", "main.md"), tab("document", "related.md"), tab("tree", "folder")], active: 0 }, focus: pane.id }; });
  return <SpatialShellView layout={layout} onLayout={setLayout} sources={[]} files={[]} links={[]} render={entry => <Buffer entry={entry} />} />;
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
