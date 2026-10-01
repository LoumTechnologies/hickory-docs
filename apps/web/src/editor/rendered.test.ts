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

// A generated picture is shown in place of the code that draws it, and the
// cell nested inside that block must not try to render at the same time.
const PICTURE =
  '<hick:file path="chart.svg">\n<hick:exec container="r">\nplot(x)\n</hick:exec>\n</hick:file>\n';

describe("a picture block", () => {
  it("renders as one slot, swallowing the cell that draws it", async () => {
    const { registry, view } = open(`Intro.\n\n${PICTURE}`);
    const doc = view.state.doc.toString();
    const file = doc.indexOf("<hick:file");
    const exec = doc.indexOf("<hick:exec");
    // Both are renderable and both are asked for — which is what a
    // freshly-opened document does. Two replacements over the same rows is
    // something CodeMirror refuses outright, so the outer block must win.
    view.dispatch({ effects: setRenderedBlocks.of([file, exec]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    const slots = registry.list();
    expect(slots).toHaveLength(1);
    expect(slots[0].kind).toBe("picture");
    expect(slots[0].picture?.path).toBe("chart.svg");
    view.destroy();
  });

  it("shows editable source — not a read-only cell — when the picture is off", async () => {
    // The two states are the picture and the code that draws it. A rendered
    // cell in between would be a display of source you cannot type in.
    const { registry, view } = open(PICTURE);
    const doc = view.state.doc.toString();
    const exec = doc.indexOf("<hick:exec");
    view.dispatch({ effects: setRenderedBlocks.of([exec]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    expect(registry.list()).toHaveLength(0);
    view.destroy();
  });

  it("still renders a cell that is not inside a picture", async () => {
    const { registry, view } = open('<hick:exec container="a">\nls\n</hick:exec>\n');
    view.dispatch({ effects: setRenderedBlocks.of([0]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    expect(registry.list().map((s) => s.kind)).toEqual(["exec"]);
    view.destroy();
  });
});

// A cell that owns ingested files must not hide them.
//
// Found by dogfooding: scaffolding.md's whole subject is the `Program.cs`
// that `dotnet new` wrote, sitting inside the cell as ordinary editable
// bytes — and the app showed a command, a transcript, and no files at all.
const INGESTING = [
  '<hick:exec container="sdk">',
  "<hick:copy id=\"scaffold\">",
  "dotnet new console -o out",
  "</hick:copy>",
  '<hick:ingested from="#scaffold" sha256="abc" at="2026-08-26" files="1" skipped="0">',
  '<hick:file path="app/Program.cs">',
  'Console.WriteLine("Hello");',
  "</hick:file>",
  "</hick:ingested>",
  "</hick:exec>",
  "",
].join("\n");

describe("a cell that owns ingested files", () => {
  it("stops rendering where the ingested bytes begin", async () => {
    const { registry, view } = open(INGESTING);
    view.dispatch({ effects: setRenderedBlocks.of([0]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    expect(registry.list().map((s) => s.kind)).toEqual(["exec"]);

    // What is on SCREEN is the test: a replacement removes the rows it
    // stands for, so the command is gone (the panel shows it) and the file
    // the cell owns is still there, as text.
    const shown = view.dom.textContent ?? "";
    expect(shown, "the command is inside the rendered panel").not.toContain(
      "dotnet new console",
    );
    expect(shown, "the ingested file must stay visible").toContain("Program.cs");
    expect(shown).toContain('Console.WriteLine("Hello");');
    view.destroy();
  });

  it("still renders the whole cell when it owns nothing", async () => {
    const { registry, view } = open('<hick:exec container="a">\nls\n</hick:exec>\n');
    view.dispatch({ effects: setRenderedBlocks.of([0]) });
    await new Promise((r) => queueMicrotask(() => r(null)));
    expect(registry.list()).toHaveLength(1);
    expect(view.dom.textContent ?? "").not.toContain("ls");
    view.destroy();
  });
});
