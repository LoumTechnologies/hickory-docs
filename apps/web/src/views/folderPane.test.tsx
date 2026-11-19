import { act, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, tab, withTree } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { observeManagedStartup, observePlainStartup, openSelectedFile, useFolderPane } from "./folderPane";

afterEach(() => vi.restoreAllMocks());

it("keeps the raw startup bytes readable without granting missing or stale authority", async () => {
  const path = "nested/code #é.py", namespace = "/workspace", workspace = "a".repeat(32);
  const content = "value = 1\n";
  vi.spyOn(api, "startupSource").mockResolvedValue({version: 1, path, content, root_path: namespace, workspace_id: workspace});
  vi.spyOn(api, "files").mockResolvedValue({root: "workspace", root_path: namespace, workspace_id: workspace,
    tree: [{name: "code #é.py", path, dir: false}]});
  const read = vi.spyOn(api, "file").mockResolvedValue({path, content, language: "python", hash: "base"});
  const open = Promise.resolve({path});
  const preview = await observePlainStartup(namespace + "/" + path, path, open, () => true);
  expect(preview?.source.content).toBe(content);
  await expect(preview?.authority).rejects.toThrow(/saved source authority/);

  read.mockResolvedValue({path, content, language: "python", hash: "base", save_guard: "current"});
  let current = true;
  let complete!: (opened: {path: string}) => void;
  const later = new Promise<{path: string}>(resolve => { complete = resolve; });
  const stale = await observePlainStartup(namespace + "/" + path, path, later, () => current);
  current = false;
  complete({path});
  await expect(stale?.authority).rejects.toThrow(/no longer matches/);
  const foreign = await observePlainStartup("/elsewhere/" + path, path, open, () => true);
  expect(foreign).toBeNull();
  vi.mocked(api.startupSource).mockResolvedValueOnce({version: 1, path, content, root_path: namespace, workspace_id: null});
  const unbound = await observePlainStartup(namespace + "/" + path, path, open, () => true);
  expect(unbound?.source.content).toBe(content);
  await expect(unbound?.authority).rejects.toThrow(/no longer matches/);
});

it("retries a refused open against the same startup bytes and workspace", async () => {
  const path = "nested/wanted.txt", root = "/workspace", workspace = "a".repeat(32);
  const content = "Exact late startup text\n";
  const raw = {version: 1 as const, path, content, root_path: root, workspace_id: workspace};
  vi.spyOn(api, "startupSource").mockResolvedValue(raw);
  const tree = vi.spyOn(api, "files").mockResolvedValue({root, root_path: root, workspace_id: workspace,
    tree: [{name: "wanted.txt", path, dir: false}]});
  vi.spyOn(api, "file").mockResolvedValue({path, content, language: "text", hash: "base", save_guard: "guard"});
  vi.spyOn(api, "openFile").mockResolvedValue({path});
  const selected = Promise.reject(Error("Open File was unavailable"));
  void selected.catch(() => {});
  const preview = await observePlainStartup(root + "/" + path, path, selected, () => true);
  expect(preview?.source).toBe(raw);
  await expect(preview?.authority).rejects.toThrow("Open File was unavailable");
  preview?.retry?.();
  await expect(preview?.authority).resolves.toMatchObject({file: {save_guard: "guard"}});
  expect(preview?.source).toBe(raw);
  tree.mockResolvedValue({root, root_path: "/foreign", workspace_id: workspace, tree: []});
  preview?.retry?.();
  await expect(preview?.authority).rejects.toThrow(/no longer matches/);
});

it("rechecks the original managed guard after an equal-byte room wait", async () => {
  const path = "nested/new #é.md", docId = "b".repeat(16), root = "/workspace", workspace = "a".repeat(32);
  const content = "# Readable\n";
  vi.spyOn(api, "startupSource").mockResolvedValue({version: 1, path, doc_id: docId, content, root_path: root, workspace_id: workspace});
  vi.spyOn(api, "files").mockResolvedValue({root, root_path: root, workspace_id: workspace,
    tree: [{name: "new #é.md", path, dir: false, doc_id: docId}]});
  const file = {id: docId, path, source: content, updated_at: "", save_guard: "first physical source"};
  const read = vi.spyOn(api, "doc").mockResolvedValue(file);
  let open!: (selected: {path: string; doc_id: string}) => void;
  const selected = new Promise<{path: string; doc_id: string}>(resolve => { open = resolve; });
  const preview = await observeManagedStartup(root + "/" + path, path, selected, () => true);
  expect(preview?.source.content).toBe(content);
  expect(read).not.toHaveBeenCalled();
  open({path, doc_id: docId});
  await expect(preview?.authority).resolves.toEqual(file);
  read.mockResolvedValue({...file, save_guard: "equal-byte replacement"});
  await expect(preview?.confirmRoom()).rejects.toThrow(/saved authority changed/);
});

it("removes restored Files furniture without removing the editor", async () => {
  const notice = vi.fn();
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState(() => withTree(openUntitledTab(initialWorkspace()), tab("tree", "folder", "Files")));
    const [, requestFocus] = useState(0);
    const focusFiles = useFolderPane(false, true, layout, setLayout, requestFocus, notice);
    return { layout, focusFiles };
  });
  await waitFor(() => expect(panes(result.current.layout.root).flatMap((pane) => pane.tabs).map((entry) => entry.kind)).toEqual(["untitled"]));
  act(() => result.current.focusFiles());
  expect(notice).toHaveBeenCalledWith("Open a folder to show Files.");
});

it("opens the selected document independently of folder visibility", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/notes/note.md", doc_id: "note" });
  const openDoc = vi.fn();
  const openPlain = vi.fn();
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", openDoc, openPlain, notice);
  expect(openDoc).toHaveBeenCalledWith("note", "/notes/note.md");
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

it("ignores a selected-file identity that arrives after the route changes", async () => {
  let deliver!: (value: { path: string; doc_id: string }) => void;
  vi.spyOn(api, "openFile").mockReturnValueOnce(new Promise(resolve => { deliver = resolve; }));
  const openDoc = vi.fn(), openPlain = vi.fn(), notice = vi.fn();
  let current = true;
  const opening = openSelectedFile("/notes/old.md", openDoc, openPlain, notice, () => current);
  current = false;
  deliver({ path: "/notes/old.md", doc_id: "retired" });
  await opening;
  expect(openDoc).not.toHaveBeenCalled();
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

it("opens an ordinary text file without requiring a document session", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/outside/script.py" });
  const openPlain = vi.fn();
  await openSelectedFile("/notes/script.py", vi.fn(), openPlain, vi.fn());
  expect(openPlain).toHaveBeenCalledWith("/outside/script.py");
});

it("explains a failed file lookup", async () => {
  vi.spyOn(api, "openFile").mockRejectedValue(new Error("File unavailable"));
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", vi.fn(), vi.fn(), notice);
  expect(notice).toHaveBeenCalledWith("File unavailable");
});

it("adds the first folder's Files pane while preserving existing editor tabs", async () => {
  const { result, rerender } = renderHook(({ folderOpen }) => {
    const [layout, setLayout] = useState(() => openUntitledTab(openUntitledTab(initialWorkspace()), true));
    const [, requestFocus] = useState(0);
    useFolderPane(folderOpen, true, layout, setLayout, requestFocus, vi.fn());
    return layout;
  }, { initialProps: { folderOpen: false } });
  const before = panes(result.current.root).flatMap(pane => pane.tabs).map(tab => tab.id);
  rerender({ folderOpen: true });
  await waitFor(() => expect(panes(result.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind === "tree")).toHaveLength(1));
  expect(panes(result.current.root).flatMap(pane => pane.tabs).filter(tab => tab.kind !== "tree").map(tab => tab.id)).toEqual(before);
});

it("adds Files even when the stored layout left it closed", async () => {
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState(initialWorkspace);
    useFolderPane(true, true, layout, setLayout, vi.fn(), vi.fn(), true);
    return layout;
  });
  await waitFor(() => expect(panes(result.current.root).flatMap(pane => pane.tabs).map(entry => entry.kind)).toEqual(["tree"]));
});
