// @vitest-environment jsdom
//
// Protects docs/guarantees/authoring/inserting-an-element-writes-hick-you-could-have-typed.md

import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorState, Transaction } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

import { InsertMenu } from "./InsertMenu";
import { insertElement } from "../editor/insertElement";
import { elementById, initialValues, type InsertElement } from "../lib/insertCatalog";
import {
  activeEditor,
  forgetEditor,
  markActiveEditor,
  resetActiveEditor,
} from "../editor/activeEditor";

afterEach(() => {
  cleanup();
  resetActiveEditor();
});

function panel(props: Partial<React.ComponentProps<typeof InsertMenu>> = {}) {
  const onInsert = vi.fn();
  const onClose = vi.fn();
  render(<InsertMenu selectedText="" onInsert={onInsert} onClose={onClose} {...props} />);
  return { onInsert, onClose };
}

describe("choosing an element", () => {
  it("shows the whole vocabulary, grouped", () => {
    panel();
    expect(screen.getByRole("heading", { name: "Execution" })).toBeTruthy();
    expect(screen.getByRole("option", { name: /Exec Cell/i })).toBeTruthy();
    expect(screen.getByRole("option", { name: /^Paste hick:paste/i })).toBeTruthy();
  });

  it("opens on the element a native menu item named", () => {
    panel({ initialId: "allow-network" });
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Allow network");
  });

  it("falls back to the first element when the shell names one it does not have", () => {
    // A newer desktop menu against an older page: the panel opens, which is
    // a better answer than a menu item that does nothing.
    panel({ initialId: "teleport" });
    expect(screen.getByRole("heading", { level: 2 })).toBeTruthy();
  });

  it("narrows the list as you type, and moves the choice with it", () => {
    panel();
    fireEvent.change(screen.getByLabelText("Find an element"), { target: { value: "network" } });
    expect(screen.getByRole("heading", { level: 2 }).textContent).toBe("Allow network");
    expect(screen.queryByRole("option", { name: /Exec Cell/i })).toBeNull();
  });

  it("says so when nothing matches, and points at the guide", () => {
    panel();
    fireEvent.change(screen.getByLabelText("Find an element"), { target: { value: "zzz" } });
    expect(screen.getByText(/hick-guide/)).toBeTruthy();
  });
});

describe("filling it in", () => {
  it("previews the exact bytes as the fields change", () => {
    panel({ initialId: "copy" });
    fireEvent.change(screen.getByLabelText(/^Id/), { target: { value: "version" } });
    expect(screen.getByText('<hick:copy id="version">', { exact: false })).toBeTruthy();
  });

  it("names each attribute beside its label, so the next one can be typed by hand", () => {
    panel({ initialId: "exec" });
    expect(screen.getByText("container")).toBeTruthy();
    expect(screen.getByText("timeout")).toBeTruthy();
  });

  it("seeds the body from the buffer's selection", () => {
    panel({ initialId: "cut", selectedText: "a secret paragraph" });
    expect((screen.getByLabelText(/Content/) as HTMLTextAreaElement).value).toBe(
      "a secret paragraph",
    );
  });

  it("swaps the diagram's starter body with its renderer — mermaid text or a scene", () => {
    panel({ initialId: "diagram" });
    const bodyInput = () => screen.getByLabelText(/Diagram source/) as HTMLTextAreaElement;
    expect(bodyInput().value).toContain("graph TD");
    // Choosing the graph renderer replaces an UNTOUCHED starter with a scene
    // the canvas can open — mermaid text under renderer="graph" would greet
    // the author with a parse error.
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "graph" } });
    expect(bodyInput().value).toContain('"nodes"');
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "mermaid" } });
    expect(bodyInput().value).toContain("graph TD");
    // A body the person already edited is theirs, whatever the renderer says.
    fireEvent.change(bodyInput(), { target: { value: "flowchart LR\n  a --> b" } });
    fireEvent.change(screen.getByLabelText(/Renderer/), { target: { value: "graph" } });
    expect(bodyInput().value).toBe("flowchart LR\n  a --> b");
  });

  it("refuses to insert while a required attribute is empty, and says which", () => {
    const { onInsert } = panel({ initialId: "copy" });
    fireEvent.click(screen.getByRole("button", { name: "Insert" }));
    expect(onInsert).not.toHaveBeenCalled();
    expect(screen.getByRole("alert").textContent).toContain("Copy needs an id");
  });

  it("hands back the element and its values, then closes", () => {
    const { onInsert, onClose } = panel({ initialId: "paste" });
    fireEvent.change(screen.getByLabelText(/^Select/), { target: { value: "version" } });
    fireEvent.click(screen.getByRole("button", { name: "Insert" }));
    expect(onInsert).toHaveBeenCalledTimes(1);
    const [element, values] = onInsert.mock.calls[0];
    expect(element.id).toBe("paste");
    expect(values.select).toBe("version");
    expect(onClose).toHaveBeenCalled();
  });

  it("edits one existing element without showing the element picker", () => {
    const { onInsert } = panel({
      initialId: "copy",
      edit: { values: { id: "draft" }, body: "Existing text." },
    });
    expect(screen.getByRole("dialog", { name: "Edit a hick element" })).toBeTruthy();
    expect(screen.queryByLabelText("Find an element")).toBeNull();
    expect((screen.getByLabelText(/^Id/) as HTMLInputElement).value).toBe("draft");
    expect((screen.getByLabelText(/Content/) as HTMLTextAreaElement).value).toBe("Existing text.");
    fireEvent.change(screen.getByLabelText(/^Id/), { target: { value: "published" } });
    fireEvent.click(screen.getByRole("button", { name: "Apply" }));
    expect(onInsert.mock.calls[0][1].id).toBe("published");
  });

  it("closes on Escape without inserting", () => {
    const { onInsert, onClose } = panel();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onClose).toHaveBeenCalled();
    expect(onInsert).not.toHaveBeenCalled();
  });
});

// ---------------------------------------------------------------------------
// The other half: which buffer it lands in, and what lands there.
// ---------------------------------------------------------------------------

const views: EditorView[] = [];
afterEach(() => {
  for (const view of views.splice(0)) view.destroy();
});

function editor(doc: string): EditorView {
  const view = new EditorView({ parent: document.body, state: EditorState.create({ doc }) });
  views.push(view);
  return view;
}

function insert(view: EditorView, id: string, values: Record<string, string>, body?: string) {
  const element = elementById(id) as InsertElement;
  insertElement(view, element, { ...initialValues(element), ...values }, body);
}

describe("the target buffer", () => {
  it("is the one focused last, not the one focused now", () => {
    // The panel takes the focus the moment it opens, so "has focus" is
    // already the wrong question by the time an insert happens.
    const first = editor("one");
    const second = editor("two");
    markActiveEditor(first);
    markActiveEditor(second);
    expect(activeEditor()).toBe(second);
  });

  it("is forgotten when its pane goes away", () => {
    const view = editor("one");
    markActiveEditor(view);
    forgetEditor(view);
    expect(activeEditor()).toBeNull();
  });

  it("keeps the newer target when an older pane closes behind it", () => {
    const old = editor("one");
    const current = editor("two");
    markActiveEditor(old);
    markActiveEditor(current);
    forgetEditor(old);
    expect(activeEditor()).toBe(current);
  });

  it("is not a destroyed editor, which would swallow the edit in silence", () => {
    const view = editor("one");
    markActiveEditor(view);
    view.destroy();
    expect(activeEditor()).toBeNull();
  });
});

describe("what lands in the buffer", () => {
  it("writes the element where the caret is", () => {
    const view = editor("Version: ");
    view.dispatch({ selection: { anchor: 9 } });
    insert(view, "paste", { select: "version" });
    expect(view.state.doc.toString()).toBe('Version: <hick:paste select="#version" />');
  });

  it("leaves the body selected so the first keystroke replaces it", () => {
    const view = editor("");
    insert(view, "exec", { container: "py" });
    const { from, to } = view.state.selection.main;
    expect(view.state.sliceDoc(from, to)).toBe("echo hello");
  });

  it("wraps the selection when there is one, and opens the block a line", () => {
    const view = editor("keep this\n");
    view.dispatch({ selection: { anchor: 0, head: 9 } });
    insert(view, "copy", { id: "kept" }, "keep this");
    expect(view.state.doc.toString()).toBe('<hick:copy id="kept">\nkeep this\n</hick:copy>\n\n');
  });

  it("arrives as something the person did, so undo can reach it", () => {
    // DocumentEditor keeps every transaction WITHOUT a userEvent out of the
    // undo history — that is how a collaborator's edits and the room's first
    // sync stay un-undoable. An insert dispatched without one would be
    // permanent.
    const seen: (string | undefined)[] = [];
    const view = new EditorView({
      parent: document.body,
      state: EditorState.create({
        doc: "",
        extensions: [
          EditorView.updateListener.of((update) => {
            for (const tr of update.transactions) {
              if (tr.docChanged) seen.push(tr.annotation(Transaction.userEvent));
            }
          }),
        ],
      }),
    });
    views.push(view);
    insert(view, "confirm", { message: "Go?" });
    expect(seen).toEqual(["input.insertElement"]);
  });
});
