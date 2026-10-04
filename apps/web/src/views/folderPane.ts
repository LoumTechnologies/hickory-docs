import { useCallback, useEffect, useRef, type Dispatch, type SetStateAction } from "react";
import { api } from "../api/client";
import { nodeForAbsolutePath } from "../lib/openPath";
import { isLikelyBinaryPath } from "../shell/FolderTreePane";
import { activate, closeTab, panes, tab, treePane, withTree, type Layout } from "../shell/layout";

/** Folder visibility belongs to this window, independent of the engine's working directory. */
export function useFolderPane(
  folderOpen: boolean | null,
  hydrated: boolean,
  layout: Layout,
  setLayout: Dispatch<SetStateAction<Layout>>,
  requestFocus: Dispatch<SetStateAction<number>>,
  notice: (message: string) => void,
) {
  const openedByDefault = useRef(false);
  useEffect(() => {
    if (folderOpen === false) openedByDefault.current = false;
    if (!hydrated || folderOpen !== true || openedByDefault.current) return;
    openedByDefault.current = true;
    // Wait for restoration, then add Files without taking editor focus.
    // Apply the default once so closing the pane remains a person's choice.
    setLayout((current) => treePane(current) ? current : withTree(current, tab("tree", "folder", "Files")));
  }, [folderOpen, hydrated, setLayout]);
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

export async function openSelectedFile(
  path: string,
  openDoc: (id: string) => void,
  openPlain: (path: string) => void,
  notice: (message: string) => void,
) {
  try {
    const files = await api.files();
    const node = nodeForAbsolutePath(files.tree, path);
    if (node?.doc_id) openDoc(node.doc_id);
    else if (node && !node.dir && !isLikelyBinaryPath(node.path)) openPlain(node.path);
    else notice("This file cannot be opened as text.");
  } catch (error) {
    notice(error instanceof Error ? error.message : String(error));
  }
}
