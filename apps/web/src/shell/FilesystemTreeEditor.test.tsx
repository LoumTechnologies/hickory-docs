import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { EditorView } from "@codemirror/view";
import { afterEach, expect, it, vi } from "vitest";

import { installMockHandler } from "../api/client";
import { FilesystemTreeEditor } from "./FilesystemTreeEditor";

afterEach(cleanup);

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
  const editor = EditorView.findFromDOM(document.querySelector(".filesystem-editor .cm-editor")!);
  if (!editor) throw new Error("filesystem editor did not mount");
  editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: "source/\n  first.ts\n  notes.txt" } });
  fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
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
  fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
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
  fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
  expect(operations).toEqual([]);
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
  fireEvent.keyDown(editor.contentDOM, { key: "s", ctrlKey: true });
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(apply).toHaveBeenCalledWith("New title");
});
