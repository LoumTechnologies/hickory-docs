// Guarantee: docs/guarantees/authoring/a-table-is-a-dataset-and-a-paragraph.md
// — a grid's edits land in the document, which requires the rendered block's
// slot to know where its block is NOW, not where it was when it was drawn.
import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { RenderedRegistry, renderedBlocks, setRenderedBlocks } from "./rendered";

const TABLE = '<hick:table delimiter="," header="true">\nregion,units\nnorth,120\n</hick:table>\n';

function open(doc: string) {
  const registry = new RenderedRegistry();
  const view = new EditorView({
    state: EditorState.create({ doc, extensions: [renderedBlocks(registry)] }),
    parent: document.body,
  });
  return { registry, view };
}

describe("a rendered block's slot", () => {
  it("follows its block when text is inserted above it", async () => {
    const { registry, view } = open(`Intro.\n\n${TABLE}`);
    const at = view.state.doc.toString().indexOf("<hick:table");
    view.dispatch({ effects: setRenderedBlocks.of([at]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    expect(registry.list()).toHaveLength(1);
    const slot = registry.list()[0];
    expect(slot.at).toBe(at);

    // A heading typed above: the widget is unchanged (same content, same
    // key), so it is never re-mounted — and the slot must move anyway.
    const inserted = "# A heading\n";
    view.dispatch({ changes: { from: 0, insert: inserted } });
    expect(registry.list()).toHaveLength(1);
    expect(registry.list()[0]).toBe(slot); // the same object the UI holds
    expect(slot.at).toBe(at + inserted.length);
    expect(slot.span).toEqual([at + inserted.length, view.state.doc.toString().indexOf("</hick:table>") + "</hick:table>".length]);
    view.destroy();
  });

  it("keeps the same slot, with new text, when its own content changes", async () => {
    const { registry, view } = open(TABLE);
    view.dispatch({ effects: setRenderedBlocks.of([0]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    const slot = registry.list()[0];
    const el = slot.el;
    let notified = 0;
    registry.subscribe(() => notified++);
    // What a grid commit does: rewrite the body between the tags.
    const from = view.state.doc.toString().indexOf("north,120");
    view.dispatch({ changes: { from, to: from + "north,120".length, insert: "north,121" } });
    await new Promise((r) => queueMicrotask(() => r(null)));
    // Same object, same element — the panel inside it was never remounted,
    // so its selection and focus survive — and the new text is announced.
    expect(registry.list()).toHaveLength(1);
    expect(registry.list()[0]).toBe(slot);
    expect(slot.el).toBe(el);
    expect(slot.text).toContain("north,121");
    expect(notified).toBe(1);
    view.destroy();
  });

  it("does not move when the edit is below it", async () => {
    const { registry, view } = open(`${TABLE}\nAfter.`);
    view.dispatch({ effects: setRenderedBlocks.of([0]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    const slot = registry.list()[0];
    view.dispatch({ changes: { from: view.state.doc.length, insert: " More." } });
    expect(slot.at).toBe(0);
    view.destroy();
  });
});
