import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorSelection, EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";

import { installMockHandler } from "../api/client";
import { FilesystemTreeEditor } from "./FilesystemTreeEditor";

afterEach(cleanup);

function editor(): EditorView {
  const found = EditorView.findFromDOM(document.querySelector(".filesystem-editor .cm-editor")!);
  if (!found) throw new Error("filesystem editor did not mount");
  return found;
}

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
it("keeps reordered rows clean and accepts ordinary multiple selections", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  const opened = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "one.txt", path: "one.txt", dir: false },
      { name: "two.txt", path: "two.txt", dir: false },
    ]}
    onChanged={changed}
    onOpenPath={opened}
    onContextPath={() => {}}
  />);
  const view = editor();
  expect(view.state.facet(EditorState.allowMultipleSelections)).toBe(true);
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "two.txt\none.txt" } });
  await screen.findByText("No filesystem changes — row order is presentation only.");
  expect(screen.queryByRole("toolbar", { name: "Unsaved Files changes" })).toBeNull();
  const firstLine = view.contentDOM.querySelector<HTMLElement>(".cm-line")!;
  expect(firstLine.dataset.treePath).toBe("two.txt");
  fireEvent.doubleClick(firstLine);
  expect(opened).toHaveBeenCalledWith("two.txt");
  expect(view.state.sliceDoc(
    view.state.selection.main.from,
    view.state.selection.main.to,
  )).toBe("two.txt");
  expect(view.state.selection.ranges).toHaveLength(1);

  view.dispatch({ selection: EditorSelection.create([
    EditorSelection.cursor(view.state.doc.line(1).from),
    EditorSelection.cursor(view.state.doc.line(2).from),
  ]) });
  expect(view.state.selection.ranges).toHaveLength(2);
  view.dispatch(view.state.replaceSelection("renamed-"));
  expect(view.state.doc.toString()).toBe("renamed-two.txt\nrenamed-one.txt");
  expect(await screen.findByRole("toolbar", { name: "Unsaved Files changes" })).toBeTruthy();
  expect(operations).toEqual([]);
  fireEvent.keyDown(view.contentDOM, { key: "s", ctrlKey: true });
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([
    { op: "rename", path: "two.txt", to: "renamed-two.txt" },
    { op: "rename", path: "one.txt", to: "renamed-one.txt" },
  ]);
});

it("shows Apply and a non-mutating exact Dry Run only while the buffer is dirty", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] },
      { name: "notes.txt", path: "notes.txt", dir: false },
    ]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  expect(screen.queryByRole("toolbar", { name: "Unsaved Files changes" })).toBeNull();
  editor().dispatch({ changes: { from: 0, to: editor().state.doc.length, insert: "source/\n  first.ts\n  notes.txt" } });
  const toolbar = await screen.findByRole("toolbar", { name: "Unsaved Files changes" });
  fireEvent.click(screen.getByRole("button", { name: "Dry Run" }));
  expect(operations).toEqual([]);
  expect(screen.getByLabelText("Files dry run").textContent).toContain("nothing has been applied");
  expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual([
    "Rename or move src → source",
    "Rename or move source/one.ts → source/first.ts",
    "Rename or move notes.txt → source/notes.txt",
  ]);
  fireEvent.click(toolbar.querySelector<HTMLButtonElement>("button.primary")!);
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toHaveLength(3);
});

it("shows a validation refusal in Dry Run without writing", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  render(<FilesystemTreeEditor
    nodes={[{ name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] }]}
    onChanged={() => {}}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const line = editor().state.doc.line(2);
  editor().dispatch({ changes: { from: line.from, to: line.to, insert: " one.ts" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(screen.getByLabelText("Files dry run").textContent).toContain("indentation must use two spaces");
  expect(operations).toEqual([]);
});

// Guarantee: docs/guarantees/authoring/the-workspace-tree-edits-like-an-editor.md
it("is an ordinary editor whose saved text renames and moves filesystem entries", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (method, path, body) => {
    if (method === "POST" && path === "/api/files/op") { operations.push(body); return {}; }
    throw new Error(`unexpected ${method} ${path}`);
  });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] },
      { name: "notes.txt", path: "notes.txt", dir: false },
    ]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const view = editor();
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "source/\n  first.ts\n  notes.txt" } });
  fireEvent.keyDown(view.contentDOM, { key: "s", ctrlKey: true });
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([
    { op: "rename", path: "src", to: "source" },
    { op: "rename", path: "source/one.ts", to: "source/first.ts" },
    { op: "rename", path: "notes.txt", to: "source/notes.txt" },
  ]);
  expect(screen.getByText("Applied 3 filesystem edits.")).toBeTruthy();
});

it("creates an inserted file line at the path expressed by indentation", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[{ name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] }]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const editor = EditorView.findFromDOM(document.querySelector(".filesystem-editor .cm-editor")!);
  if (!editor) throw new Error("filesystem editor did not mount");
  editor.dispatch({ changes: { from: editor.state.doc.length, insert: "\n  two.ts" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(screen.getByLabelText("Files dry run").textContent).toContain("Create file src/two.ts");
  expect(operations).toEqual([]);
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([{ op: "create", path: "src/two.ts" }]);
});

it("reviews a removed subtree before sending one destructive delete", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] },
      { name: "notes.txt", path: "notes.txt", dir: false },
    ]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const editor = EditorView.findFromDOM(document.querySelector(".filesystem-editor .cm-editor")!);
  if (!editor) throw new Error("filesystem editor did not mount");
  editor.dispatch({ changes: { from: 0, to: editor.state.doc.line(3).from, insert: "" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(operations).toEqual([]);
  expect(screen.getByLabelText("Files dry run").textContent).toContain(
    "Delete src — confirmation required; there is no trash",
  );
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  expect(screen.getByText(/Delete src\? There is no trash/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Delete" }));
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([{ op: "delete", path: "src" }]);
});

it("routes an edited associated-object title through its semantic capability", async () => {
  const apply = vi.fn(async () => undefined);
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[{ name: "src", path: "src", dir: true, children: [] }]}
    extras={[{
      key: "github:acme/widget#7",
      parentPath: "src",
      label: "issue #7: Old title · open",
      edit: { prefix: "issue #7: ", value: "Old title", suffix: " · open", apply },
    }]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const editor = EditorView.findFromDOM(document.querySelector(".filesystem-editor .cm-editor")!);
  if (!editor) throw new Error("filesystem editor did not mount");
  const line = editor.state.doc.line(2);
  editor.dispatch({ changes: { from: line.from, to: line.to, insert: "  issue #7: New title · open" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(screen.getByLabelText("Files dry run").textContent).toContain(
    'Edit github:acme/widget#7: "Old title" → "New title"',
  );
  expect(apply).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(apply).toHaveBeenCalledWith("New title");
});
