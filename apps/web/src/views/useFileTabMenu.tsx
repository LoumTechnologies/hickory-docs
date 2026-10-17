import { useState, type MouseEvent } from "react";
import type { FileNode } from "../api/types";
import { docPathOf, isLikelyBinaryPath, type FolderTree } from "../shell/FolderTreePane";
import { FileActionPrompts, runFileMenuItem } from "../shell/fileMenuActions";
import { TreeContextMenu } from "../shell/TreeContextMenu";
import { treeMenuItems } from "../shell/treeMenu";
import { useDired } from "../shell/useDired";
import type { Tab } from "../shell/layout";

function findFile(nodes: readonly FileNode[], path: string): FileNode | undefined {
  for (const node of nodes) {
    if (node.path === path) return node;
    const found = node.children && findFile(node.children, path);
    if (found) return found;
  }
}

export function useFileTabMenu(roots: readonly FolderTree[], activeDocId: string | null | undefined, openable: ReadonlyMap<string, string>, openDocument: (id: string) => void) {
  const [menu, setMenu] = useState<{ x: number; y: number; path: string; folder: FolderTree; plainText: boolean } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const dired = useDired(setNotice);
  const onTabContextMenu = (event: MouseEvent, tab: Tab) => {
    if (!["document", "generated", "file"].includes(tab.kind)) return;
    for (const folder of roots) {
      const path = tab.kind === "document" && tab.docId ? docPathOf(folder.tree, tab.docId) ?? tab.target : tab.target;
      const node = findFile(folder.tree, path);
      // Tabs can outlive a truncated listing. Their kind still identifies
      // document and generated files, which must never offer adoption.
      if (!node && roots.length !== 1) continue;
      event.preventDefault();
      event.stopPropagation();
      setNotice(null);
      setMenu({ x: event.clientX, y: event.clientY, path, folder,
        plainText: tab.kind === "file" && !node?.doc_id && !node?.generated_by && !openable.has(path) && !isLikelyBinaryPath(path) });
      return;
    }
  };
  const activePath = menu && activeDocId ? docPathOf(menu.folder.tree, activeDocId) : null;
  return {
    onTabContextMenu,
    fileTabMenu: <>
      {menu && <TreeContextMenu x={menu.x} y={menu.y} subject={menu.path}
        items={treeMenuItems({ rootPath: menu.folder.root_path, separator: menu.folder.separator, fileManager: menu.folder.file_manager }, menu.path, false, {
          plainText: menu.plainText,
          activeDoc: activePath ? { path: activePath, name: activePath.split("/").pop() ?? activePath } : undefined,
        })}
        onClose={() => setMenu(null)} onPick={item => {
          setMenu(null);
          runFileMenuItem(item, dired, setNotice, openDocument);
        }} />}
      {notice && <p role="alert">{notice}</p>}
      <FileActionPrompts dired={dired} />
    </>,
  };
}
