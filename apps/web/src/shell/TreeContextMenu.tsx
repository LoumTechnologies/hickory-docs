// The tree's right-click menu: the items treeMenu.ts computed, at the pointer.
//
// The behaviour — Escape, click-away, staying on screen, the first item taking
// focus — is components/ContextMenu.tsx, shared with the table's menu. What is
// left here is the tree's own vocabulary: it deals in `TreeMenuItem`s and
// hands one back, so callers never have to look an id back up.

import { ContextMenu } from "../components/ContextMenu";

import type { TreeMenuItem } from "./treeMenu";

export interface TreeContextMenuProps {
  /** Viewport coordinates of the click that opened it. */
  x: number;
  y: number;
  /** What this row offers. */
  items: readonly TreeMenuItem[];
  /** The row it belongs to, for the heading that says what will be acted on. */
  subject: string;
  onPick: (item: TreeMenuItem) => void;
  onClose: () => void;
}

export function TreeContextMenu({ x, y, items, subject, onPick, onClose }: TreeContextMenuProps) {
  return (
    <ContextMenu
      x={x}
      y={y}
      subject={subject}
      items={items}
      onClose={onClose}
      onPick={(id) => {
        const item = items.find((candidate) => candidate.id === id);
        if (item) onPick(item);
      }}
    />
  );
}
