<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/FilesystemTreeEditor.test.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Filesystem Acts Keep Dired's Safety](../../../../../../the-tree-is-a-dired/contract.md)
- [The Workspace Tree Is An Editor Buffer](../../../../../contract.md)

Captured SHA-256: `35fc216d4d8cacb281aeb556e2fb877b4970d89c41151e407a4ea3ddb56976c5`.

<book:entry path="apps/web/src/shell/FilesystemTreeEditor.test.tsx">
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
  expect(document.querySelector(".filesystem-editor .cm-lineNumbers")).toBeTruthy();
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

it("moves a root file beneath an existing folder from indentation alone", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "test 1", path: "test 1", dir: true, children: [] },
      { name: "test53.py", path: "test53.py", dir: false },
    ]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const view = editor();
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "test 1/\n  test53.py" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(screen.getByLabelText("Files dry run").textContent)
    .toContain("Rename or move test53.py → test 1/test53.py");
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([{ op: "rename", path: "test53.py", to: "test 1/test53.py" }]);
});

it("creates a folder before moving an unchanged file into it in one Apply", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  const changed = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[{ name: "test53.py", path: "test53.py", dir: false }]}
    onChanged={changed}
    onOpenPath={() => {}}
    onContextPath={() => {}}
  />);
  const view = editor();
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "test 1/\n  test53.py" } });
  fireEvent.click(await screen.findByRole("button", { name: "Dry Run" }));
  expect(screen.getAllByRole("listitem").map((item) => item.textContent)).toEqual([
    "Create folder test 1/",
    "Rename or move test53.py → test 1/test53.py",
  ]);
  fireEvent.click(screen.getByRole("button", { name: "Apply" }));
  await vi.waitFor(() => expect(changed).toHaveBeenCalledOnce());
  expect(operations).toEqual([
    { op: "mkdir", path: "test 1" },
    { op: "rename", path: "test53.py", to: "test 1/test53.py" },
  ]);
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
  const deleted = vi.fn();
  render(<FilesystemTreeEditor
    nodes={[
      { name: "src", path: "src", dir: true, children: [{ name: "one.ts", path: "src/one.ts", dir: false }] },
      { name: "notes.txt", path: "notes.txt", dir: false },
    ]}
    onChanged={changed}
    onDeleted={deleted}
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
  expect(deleted).toHaveBeenCalledWith(["src"]);
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

it("shares Files draft editing, dry run and exact apply receipts with an agent", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return { op: "rename", from: "one.txt", to: "renamed.txt" }; });
  render(<UiActionContext.Provider value={registry}><FilesystemTreeEditor nodes={[{ name: "one.txt", path: "one.txt", dir: false }]}
    onChanged={() => {}} onOpenPath={() => {}} onContextPath={() => {}} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  let id = 0;
  const run = async (label: string, args = {}) => { let result: unknown; await act(async () => { result = await registry.act(registry.scope,
    { id: "python:turn", kind: "agent" }, { handle: actions().find(action => action.label === label)!.handle, arguments: args, request_id: String(++id) }); }); return result; };
  await run("Replace Files draft range", { from: 0, to: 7, text: "renamed.txt" });
  expect(editor().state.doc.toString()).toBe("renamed.txt");
  await run("Dry run Files draft");
  expect(screen.getByLabelText("Files dry run").textContent).toContain("one.txt → renamed.txt");
  expect(operations).toEqual([]);
  expect(await run("Apply Files draft")).toMatchObject({ status: "result", outcome: { value: {
    all_completed: true, completed: 1, atomic: false, rollback: false,
    results: [{ status: "completed", result: { op: "rename", from: "one.txt", to: "renamed.txt" } }],
  } } });
  expect(operations).toEqual([{ op: "rename", path: "one.txt", to: "renamed.txt" }]);
});

it("stops a Files batch after cancellation and blocks duplicate apply and native edits while pending", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const operations: unknown[] = [];
  let complete!: (value: unknown) => void;
  installMockHandler(async (_method, _path, body) => { operations.push(body); return new Promise(done => { complete = done; }); });
  render(<UiActionContext.Provider value={registry}><FilesystemTreeEditor nodes={[
    { name: "one.txt", path: "one.txt", dir: false }, { name: "two.txt", path: "two.txt", dir: false },
  ]} onChanged={() => {}} onOpenPath={() => {}} onContextPath={() => {}} /></UiActionContext.Provider>);
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  await act(async () => { editor().dispatch({ changes: { from: 0, to: editor().state.doc.length, insert: "new-one.txt\nnew-two.txt" } }); });
  let pending!: ReturnType<typeof registry.act>;
  const actor = { id: "acp:turn", kind: "agent" as const };
  await act(async () => { pending = registry.act(registry.scope, actor, { handle: actions().find(a => a.label === "Apply Files draft")!.handle, arguments: {}, request_id: "apply" }); });
  await vi.waitFor(() => expect(operations).toHaveLength(1));
  expect(await registry.act(registry.scope, { id: "python:other", kind: "agent" }, { handle: actions().find(a => a.label === "Apply Files draft")!.handle, arguments: {}, request_id: "other" })).toMatchObject({ status: "disabled" });
  const before = editor().state.doc.toString();
  editor().dispatch({ changes: { from: 0, insert: "oops" } });
  expect(editor().state.doc.toString()).toBe(before);
  expect(registry.cancel(registry.scope, actor, "apply")).toBe(true);
  let result: unknown;
  await act(async () => { complete({ op: "rename", from: "one.txt", to: "new-one.txt" }); result = await pending; });
  expect(result).toMatchObject({ status: "result", outcome: { value: { all_completed: false, completed: 1, requested: 2,
    results: [{ status: "completed" }, { status: "not_started" }], rollback: false } } });
  expect(operations).toHaveLength(1);
});

it("invalidates a Files deletion confirmation when its draft changes", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => { operations.push(body); return {}; });
  render(<FilesystemTreeEditor nodes={[{ name: "one.txt", path: "one.txt", dir: false }, { name: "two.txt", path: "two.txt", dir: false }]}
    onChanged={() => {}} onOpenPath={() => {}} onContextPath={() => {}} />);
  editor().dispatch({ changes: { from: 0, to: 8, insert: "" } });
  fireEvent.click(await screen.findByRole("button", { name: "Apply" }));
  expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
  editor().dispatch({ changes: { from: 0, insert: "one.txt\n" } });
  await vi.waitFor(() => expect(screen.queryByRole("button", { name: "Delete" })).toBeNull());
  expect(operations).toEqual([]);
});

it("reports partial Files failure and never starts later operations", async () => {
  const operations: unknown[] = [];
  installMockHandler(async (_method, _path, body) => {
    operations.push(body);
    if (operations.length === 2) throw Error("Host refused the second rename");
    return { op: "rename", from: "one.txt", to: "new-one.txt" };
  });
  render(<FilesystemTreeEditor nodes={["one", "two", "three"].map(name => ({ name: `${name}.txt`, path: `${name}.txt`, dir: false }))}
    onChanged={() => {}} onOpenPath={() => {}} onContextPath={() => {}} />);
  editor().dispatch({ changes: { from: 0, to: editor().state.doc.length, insert: "new-one.txt\nnew-two.txt\nnew-three.txt" } });
  fireEvent.click(await screen.findByRole("button", { name: "Apply" }));
  await screen.findByText(/Filesystem edits stopped/);
  expect(operations).toEqual([{ op: "rename", path: "one.txt", to: "new-one.txt" }, { op: "rename", path: "two.txt", to: "new-two.txt" }]);
  expect(editor().state.doc.toString()).toBe("new-one.txt\nnew-two.txt\nnew-three.txt");
});

it("shares reordered Files row activation and semantic menu anchors without synthetic events", async () => {
  const { act } = await import("@testing-library/react");
  const { UiActions } = await import("../lib/uiActions");
  const { UiActionContext } = await import("../lib/uiActionBridge");
  const registry = new UiActions({ workspace: "p", window: "w" }, () => true, async () => {});
  const open = vi.fn(), menu = vi.fn();
  render(<UiActionContext.Provider value={registry}><FilesystemTreeEditor nodes={[
    { name: "one.txt", path: "one.txt", dir: false }, { name: "two.txt", path: "two.txt", dir: false },
  ]} onChanged={() => {}} onOpenPath={open} onContextPath={menu} /></UiActionContext.Provider>);
  await act(async () => { editor().dispatch({ changes: { from: 0, to: editor().state.doc.length, insert: "two.txt\none.txt" } }); });
  const actions = () => registry.groups().groups.flatMap(group => registry.observe(group, 64).actions);
  for (const [index, label] of ["Activate Files row", "Open Files row menu"].entries()) {
    await act(async () => { await registry.act(registry.scope, { id: "python:turn", kind: "agent" }, {
      handle: actions().find(action => action.label === label)!.handle, arguments: { line: 1 }, request_id: String(index),
    }); });
  }
  expect(open).toHaveBeenCalledExactlyOnceWith("two.txt");
  expect(editor().state.sliceDoc(editor().state.selection.main.from, editor().state.selection.main.to)).toBe("two.txt");
  expect(menu).toHaveBeenCalledWith({ clientX: expect.any(Number), clientY: expect.any(Number) }, "two.txt", false);
  expect(menu.mock.calls[0][0]).not.toBeInstanceOf(Event);
  fireEvent.doubleClick(editor().contentDOM.querySelector(".cm-line")!);
  expect(open).toHaveBeenLastCalledWith("two.txt");
});
</book:entry>

</book:doc>
