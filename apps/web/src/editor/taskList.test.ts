import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { parseMarkdownProse } from "./markdownStyling";
import { parseHickDoc } from "./hickDoc";
import { taskCheckboxes, toggleTaskAt } from "./taskList";

describe("task lists — the scan", () => {
  it("finds bullet and ordered items, checked and not", () => {
    const src = "- [ ] open\n* [x] closed\n1. [X] numbered\n";
    const { tasks } = parseMarkdownProse(src);
    expect(tasks.map((t) => t.checked)).toEqual([false, true, true]);
    expect(src.slice(tasks[0].boxFrom, tasks[0].boxTo)).toBe("[ ]");
    expect(src.slice(tasks[2].boxFrom, tasks[2].boxTo)).toBe("[X]");
  });

  it("ignores a bare bracket pair that is prose about brackets", () => {
    // Without the bullet in the match, a sentence mentioning `[ ]` would grow
    // a checkbox in the middle of a paragraph.
    expect(parseMarkdownProse("the empty box [ ] means nothing here\n").tasks).toEqual([]);
  });

  it("wants whitespace after the box, so `[x]y` is a link-ish literal", () => {
    expect(parseMarkdownProse("- [x]done\n").tasks).toEqual([]);
  });

  it("reads a task nested inside a quote", () => {
    const src = "> - [ ] quoted todo\n";
    const { tasks, quotes } = parseMarkdownProse(src);
    expect(quotes).toHaveLength(1);
    expect(tasks).toHaveLength(1);
    expect(src.slice(tasks[0].boxFrom, tasks[0].boxTo)).toBe("[ ]");
  });

  it("does not see a checkbox inside an exec cell's payload", () => {
    // The payload of a cell is a command, and `- [ ]` in it is an argument.
    const src = '<hick:exec>\n- [ ] not a checkbox\n</hick:exec>\n\n- [ ] a checkbox\n';
    const { tasks } = parseHickDoc(src);
    expect(tasks).toHaveLength(1);
    expect(src.slice(tasks[0].from, tasks[0].to)).toBe("- [ ] a checkbox");
  });
});

describe("task lists — toggling", () => {
  const src = "- [ ] one\n- [x] two\n";

  it("checks an empty box with a lowercase x", () => {
    expect(toggleTaskAt(src, 3)).toEqual({ from: 3, to: 4, insert: "x" });
  });

  it("clears a checked box, wherever on the line the click landed", () => {
    // Position 14 is inside the word "two", not on the box itself.
    expect(toggleTaskAt(src, 14)).toEqual({ from: 13, to: 14, insert: " " });
  });

  it("normalises a capital X to an empty box rather than to lowercase", () => {
    expect(toggleTaskAt("- [X] shout\n", 0)).toEqual({ from: 3, to: 4, insert: " " });
  });

  it("answers null off a task line, so a stray click edits nothing", () => {
    expect(toggleTaskAt("just prose\n", 4)).toBeNull();
  });
});

describe("task lists — the checkbox decoration", () => {
  const mount = (doc: string, cursor: number) => {
    const view = new EditorView({
      state: EditorState.create({
        doc,
        selection: { anchor: cursor },
        extensions: [taskCheckboxes()],
      }),
      parent: document.body,
    });
    return view;
  };

  it("replaces the box everywhere except the line holding the cursor", () => {
    // A person who put the caret on a task line is editing its text, and a
    // box they cannot see is a box they cannot fix.
    const view = mount("- [ ] one\n- [x] two\n", 2);
    const boxes = view.dom.querySelectorAll(".cm-md-checkbox");
    expect(boxes).toHaveLength(1);
    expect(boxes[0].getAttribute("aria-checked")).toBe("true");
    expect(view.state.doc.toString()).toContain("- [ ] one");
    view.destroy();
  });

  it("draws an unchecked box with no glyph and a checked one with a tick", () => {
    const view = mount("- [ ] one\n- [x] two\n", 20);
    const boxes = [...view.dom.querySelectorAll(".cm-md-checkbox")];
    expect(boxes).toHaveLength(2);
    expect(boxes[0].textContent).toBe("");
    expect(boxes[1].textContent).toBe("\u2713");
    view.destroy();
  });

  it("edits the document when a box is pressed", () => {
    const view = mount("- [ ] one\n", 10);
    const box = view.dom.querySelector(".cm-md-checkbox") as HTMLElement;
    box.dispatchEvent(new MouseEvent("mousedown", { bubbles: true }));
    expect(view.state.doc.toString()).toBe("- [x] one\n");
    view.destroy();
  });

  it("offers no checkbox in a buffer that has no tasks", () => {
    const view = mount("# just prose\n", 0);
    expect(view.dom.querySelectorAll(".cm-md-checkbox")).toHaveLength(0);
    view.destroy();
  });
});
