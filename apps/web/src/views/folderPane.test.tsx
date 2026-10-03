import { act, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, tab, withTree } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { openSelectedFile, useFolderPane } from "./folderPane";

afterEach(() => vi.restoreAllMocks());

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
  vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "notes", tree: [{ name: "note.md", path: "note.md", dir: false, doc_id: "note" }] });
  const openDoc = vi.fn();
  const openPlain = vi.fn();
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", openDoc, openPlain, notice);
  expect(openDoc).toHaveBeenCalledWith("note");
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

it("opens an ordinary text file without requiring a document session", async () => {
  vi.spyOn(api, "files").mockResolvedValue({ folder_open: false, root: "notes", tree: [{ name: "script.py", path: "script.py", dir: false }] });
  const openPlain = vi.fn();
  await openSelectedFile("/notes/script.py", vi.fn(), openPlain, vi.fn());
  expect(openPlain).toHaveBeenCalledWith("script.py");
});

it("explains a failed file lookup", async () => {
  vi.spyOn(api, "files").mockRejectedValue(new Error("File unavailable"));
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", vi.fn(), vi.fn(), notice);
  expect(notice).toHaveBeenCalledWith("File unavailable");
});
