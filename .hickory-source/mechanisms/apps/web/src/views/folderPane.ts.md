<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/folderPane.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-c66b482f3cc6"></a>
## `apps/web/src/views/folderPane.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [The File Menu Preserves The Originating Window](../../../../../guarantees/authoring/file-menu-preserves-the-originating-window/contract.md).

<a id="f-c66b482f3cc6-useFolderPane"></a>
### `useFolderPane`

<storycode:copy id="f-c66b482f3cc6-useFolderPane">
import { useCallback, useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { api } from "../api/client";
import { activate, closeTab, panes, tab, treePane, withTree, type Layout } from "../shell/layout";

/** Folder visibility belongs to this window, independent of the engine's working directory. */
export function useFolderPane(
  folderOpen: boolean | null,
  hydrated: boolean,
  layout: Layout,
  setLayout: Dispatch<SetStateAction<Layout>>,
  requestFocus: Dispatch<SetStateAction<number>>,
  notice: (message: string) => void,
  restored = false,
) {
  const openedByDefault = useRef(false);
  useEffect(() => {
    if (folderOpen === false) openedByDefault.current = false;
    if (!hydrated || folderOpen !== true || openedByDefault.current) return;
    openedByDefault.current = true;
    if (restored) return;
    // Wait for restoration, then add Files without taking editor focus.
    // Apply the default once so closing the pane remains a person's choice.
    setLayout((current) => treePane(current) ? current : withTree(current, tab("tree", "folder", "Files")));
  }, [folderOpen, hydrated, restored, setLayout]);
  useEffect(() => {
    if (folderOpen !== false) return;
    // Restored furniture may have come from a folder view of the same engine.
    setLayout((current) => panes(current.root).reduce((next, pane) =>
      pane.tabs.filter((entry) => entry.kind === "tree").reduce(
        (result, entry) => closeTab(result, pane.id, entry.id), next), current));
  }, [folderOpen, hydrated, layout, setLayout]);
  return useCallback(() => {
    if (folderOpen !== true) {
      notice("Open a folder to show Files.");
      return;
    }
    setLayout((current) => {
      const pane = treePane(current);
      if (!pane) return withTree(current, tab("tree", "folder", "Files"));
      const index = pane.tabs.findIndex((entry) => entry.kind === "tree");
      return activate(current, pane.id, Math.max(0, index));
    });
    requestFocus((request) => request + 1);
  }, [folderOpen, setLayout, requestFocus, notice]);
}

</storycode:copy>
<a id="f-c66b482f3cc6-openSelectedFile"></a>
### `openSelectedFile`

<storycode:copy id="f-c66b482f3cc6-openSelectedFile">
export async function openSelectedFile(
  path: string,
  openDoc: (id: string) => void,
  openPlain: (path: string) => void,
  notice: (message: string) => void,
) {
  try {
    const selected = await api.openFile(path);
    if (selected.doc_id) openDoc(selected.doc_id);
    else openPlain(selected.path);
  } catch (error) {
    notice(error instanceof Error ? error.message : String(error));
  }
}
</storycode:copy>

### Assemble `apps/web/src/views/folderPane.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/folderPane.ts"><storycode:paste select="#f-c66b482f3cc6-useFolderPane"/><storycode:paste select="#f-c66b482f3cc6-openSelectedFile"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-c29616a4a7b1"></a>
## `apps/web/src/views/folderPane.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md).

<a id="f-c29616a4a7b1-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement">
import { act, renderHook, waitFor } from "@testing-library/react";
import { useState } from "react";
import { afterEach, expect, it, vi } from "vitest";
import { api } from "../api/client";
import { panes, tab, withTree } from "../shell/layout";
import { initialWorkspace, openUntitledTab } from "./workspaceState";
import { openSelectedFile, useFolderPane } from "./folderPane";

afterEach(() => vi.restoreAllMocks());

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-2">
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

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-3">
it("opens the selected document independently of folder visibility", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/notes/note.md", doc_id: "note" });
  const openDoc = vi.fn();
  const openPlain = vi.fn();
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", openDoc, openPlain, notice);
  expect(openDoc).toHaveBeenCalledWith("note");
  expect(openPlain).not.toHaveBeenCalled();
  expect(notice).not.toHaveBeenCalled();
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-4">
it("opens an ordinary text file without requiring a document session", async () => {
  vi.spyOn(api, "openFile").mockResolvedValue({ path: "/outside/script.py" });
  const openPlain = vi.fn();
  await openSelectedFile("/notes/script.py", vi.fn(), openPlain, vi.fn());
  expect(openPlain).toHaveBeenCalledWith("/outside/script.py");
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-5">
it("explains a failed file lookup", async () => {
  vi.spyOn(api, "openFile").mockRejectedValue(new Error("File unavailable"));
  const notice = vi.fn();
  await openSelectedFile("/notes/note.md", vi.fn(), vi.fn(), notice);
  expect(notice).toHaveBeenCalledWith("File unavailable");
});

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-6">
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

</storycode:copy>
<a id="f-c29616a4a7b1-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-c29616a4a7b1-expression_statement-7">
it("keeps Files closed when the stored layout left it closed", async () => {
  const { result } = renderHook(() => {
    const [layout, setLayout] = useState(initialWorkspace);
    useFolderPane(true, true, layout, setLayout, vi.fn(), vi.fn(), true);
    return layout;
  });
  expect(panes(result.current.root).flatMap(pane => pane.tabs)).toEqual([]);
});
</storycode:copy>

### Assemble `apps/web/src/views/folderPane.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/folderPane.test.tsx"><storycode:paste select="#f-c29616a4a7b1-expression_statement"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-2"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-3"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-4"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-5"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-6"/><storycode:paste select="#f-c29616a4a7b1-expression_statement-7"/></storycode:file>


</storycode:doc>
