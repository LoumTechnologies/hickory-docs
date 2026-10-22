import { useEffect, useId, useMemo, useState, type MouseEvent } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
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

export function useFileTabMenu(roots: readonly FolderTree[], activeDocId: string | null | undefined, openable: ReadonlyMap<string, string>, openDocument: (id: string) => void, tabs: readonly Tab[] = []) {
  const ownerSignature = JSON.stringify([roots, activeDocId, [...openable], tabs]);
  const owner = useMemo(() => crypto.randomUUID(), [ownerSignature]);
  const [menu, setMenu] = useState<{ owner: string; x: number; y: number; path: string; folder: FolderTree; plainText: boolean } | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const dired = useDired(setNotice);
  useEffect(() => { if (menu && menu.owner !== owner) setMenu(null); }, [menu, owner]);
  const open = (tab: Tab, position: { x: number; y: number }) => {
    if (!["document", "generated", "file"].includes(tab.kind) || (tabs.length > 0 && !tabs.some(current => current.id === tab.id && current.kind === tab.kind && current.target === tab.target && current.docId === tab.docId))) return false;
    for (const folder of roots) {
      const path = tab.kind === "document" && tab.docId ? docPathOf(folder.tree, tab.docId) ?? tab.target : tab.target;
      const node = findFile(folder.tree, path);
      // Tabs can outlive a truncated listing. Their kind still identifies
      // document and generated files, which must never offer adoption.
      if (!node && roots.length !== 1) continue;
      setNotice(null);
      setMenu({ owner, ...position, path, folder,
        plainText: tab.kind === "file" && !node?.doc_id && !node?.generated_by && !openable.has(path) && !isLikelyBinaryPath(path) });
      return true;
    }
    return false;
  };
  const onTabContextMenu = (event: MouseEvent, tab: Tab) => {
    if (open(tab, { x: event.clientX, y: event.clientY })) { event.preventDefault(); event.stopPropagation(); }
  };
  const parent = `file-tab-menus:${useId()}`;
  const targets = tabs.filter(tab => ["document", "generated", "file"].includes(tab.kind));
  const revision = useUiRevision(() => [roots, activeDocId, [...openable], targets, menu]);
  useUiCommands([{ key: `${parent}:read`, command: { parent, subject: `${parent}:reading`, label: "Read file tab menu targets", parameters: { first: { type: "number", integer: true, minimum: 0, maximum: 100000000 }, count: { type: "number", integer: true, minimum: 1, maximum: 64 } }, prerequisites: [],
    read: () => ({ revision: revision(), enabled: true }), execute: async args => {
      const first = Number(args.first), count = Number(args.count);
      if (first > targets.length) return { status: "unavailable", reason: "Index exceeds current file tabs" };
      const value = { tabs: targets.slice(first, first + count).map(tab => ({ id: tab.id, kind: tab.kind, path: tab.target })), total: targets.length, next: first + count < targets.length ? first + count : null };
      if (new TextEncoder().encode(JSON.stringify(value)).length > 8192) return { status: "unavailable", reason: "Tab page exceeds 8192 UTF-8 bytes; request fewer tabs" };
      return { status: "completed", value };
    } } }, { key: `${parent}:open`, command: { parent, subject: parent, label: "Open file tab context menu", parameters: { tab: { type: "string", maxLength: 4096 } }, prerequisites: ["Current document/generated/plain-file tab in its owning folder"],
      read: () => ({ revision: revision(), enabled: targets.length > 0 }), execute: async args => {
        const matches = targets.filter(tab => tab.id === args.tab);
        if (matches.length !== 1 || !open(matches[0], { x: 16, y: 16 })) return { status: "unavailable", reason: "Tab closed, ambiguous or its owning folder is unavailable" };
        return { status: "completed", value: { menu_open_requested: true, source_changed: false } };
      } } }]);
  const activePath = menu && activeDocId ? docPathOf(menu.folder.tree, activeDocId) : null;
  return {
    onTabContextMenu,
    fileTabMenu: <>
      {menu && menu.owner === owner && <TreeContextMenu x={menu.x} y={menu.y} subject={menu.path}
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
