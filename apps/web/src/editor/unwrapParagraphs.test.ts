// @vitest-environment jsdom
// Guarantee: docs/guarantees/authoring/paragraphs-unwrap-by-default.md
import { afterEach, describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { history, undo } from "@codemirror/commands";
import { paragraphUnwrapChanges, unwrapParagraphs } from "./unwrapParagraphs";
import { loadUnwrapParagraphs, saveUnwrapParagraphs, UNWRAP_PARAGRAPHS_KEY } from "../lib/unwrapParagraphs";

const reflow = (doc: string) => EditorState.create({ doc }).update({
  changes: paragraphUnwrapChanges(doc),
}).state.doc.toString();

const views: EditorView[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
  localStorage.removeItem(UNWRAP_PARAGRAPHS_KEY);
});
function editor(doc: string, readOnly = false, onOpen = true) {
  const view = new EditorView({ state: EditorState.create({
    doc, extensions: [history(), unwrapParagraphs({ onOpen }), EditorState.readOnly.of(readOnly)],
  }) });
  views.push(view);
  return view;
}

describe("paragraph unwrapping", () => {
  it("keeps seeded Untitled bytes clean through sync, then reflows a real paste", async () => {
    const original = "An introduction\nwith a soft break.";
    const view = editor(original, false, false);
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe(original);
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: original } });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe(original);
    view.dispatch({ changes: { from: view.state.doc.length, insert: "\n\nPasted prose\ncontinues." }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("An introduction with a soft break.\n\nPasted prose continues.");
  });

  it("joins soft breaks without merging paragraphs or changing inline markup", () => {
    const input = "# Heading\n\nA **long** paragraph\nwith a [link](https://example.com)\nand more text.\n\nAnother paragraph\nwith text.\n";
    const output = "# Heading\n\nA **long** paragraph with a [link](https://example.com) and more text.\n\nAnother paragraph with text.\n";
    expect(reflow(input)).toBe(output);
    expect(reflow(output)).toBe(output);
  });

  it("preserves explicit breaks, lists, quotes, tables, code, HTML and YAML", () => {
    const input = [
      "---\nsummary: |\n  first line\n  second line\n---",
      "First  \nsecond\\\nthird",
      "- item\n  continuation\n- next item",
      "> quote\n> continuation",
      "a | b\n--- | ---\nx | y\nz | w",
      "```md\ncode paragraph\ncontinues\n```",
      "    indented code\n    more code",
      "<div>\nHTML content\nmore content\n</div>",
      "Text with `multi\nline code`.",
      "Text [link\ntext](url).",
      "$$\nx = 1\ny = 2\n$$",
    ].join("\n\n");
    expect(reflow(input)).toBe(input);
  });

  it("unwraps prose in containers but leaves payload bytes and tag attributes intact", () => {
    const input = '<hick:doc>\nA paragraph\ncontinues.\n\n<hick:copy id="source">\nOriginal transcript\ncontinues.\n</hick:copy>\n<hick:file path="out.md">\nFile content\ncontinues.\n</hick:file>\n<hick:exec container="shell">\necho first\necho second\n</hick:exec>\n</hick:doc>';
    expect(reflow(input)).toBe(input.replace("A paragraph\ncontinues.", "A paragraph continues."));
    expect(reflow('<hick:doc title="first\nsecond">\nProse\ncontinues.\n</hick:doc>')).toContain('title="first\nsecond"');
  });

  it("is on by default, persists off, and leaves read-only documents alone", async () => {
    expect(loadUnwrapParagraphs()).toBe(true);
    saveUnwrapParagraphs(false);
    expect(loadUnwrapParagraphs()).toBe(false);
    const off = editor("A paragraph\ncontinues.");
    await Promise.resolve();
    expect(off.state.doc.toString()).toBe("A paragraph\ncontinues.");
    saveUnwrapParagraphs(true);
    const readOnly = editor("A paragraph\ncontinues.", true);
    await Promise.resolve();
    expect(readOnly.state.doc.toString()).toBe("A paragraph\ncontinues.");
  });

  it("reflows initial and synced text as an undoable edit without undo loops", async () => {
    const view = editor("A paragraph\ncontinues.");
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("A paragraph continues.");
    expect(undo(view)).toBe(true);
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("A paragraph\ncontinues.");
    view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "New text\nfrom disk." } });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("New text from disk.");
  });

  it("reflows pasted paragraphs, maps the caret, and does not fight Enter", async () => {
    const view = editor("");
    await Promise.resolve();
    view.dispatch({ changes: { from: 0, insert: "Pasted text\ncontinues." }, selection: { anchor: 22 }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toBe("Pasted text continues.");
    expect(view.state.selection.main.head).toBe(view.state.doc.length);
    view.dispatch({ changes: { from: 11, insert: "\n" }, userEvent: "input.type" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toContain("\n");
    saveUnwrapParagraphs(false);
    view.dispatch({ changes: { from: view.state.doc.length, insert: "\nmore pasted text" }, userEvent: "input.paste" });
    await Promise.resolve();
    expect(view.state.doc.toString()).toContain("\nmore pasted text");
  });
});
