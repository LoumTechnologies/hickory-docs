<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/shell/TreeContextMenu.tsx

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Table Is A Dataset And A Paragraph](../../../../../../a-table-is-a-dataset-and-a-paragraph/contract.md)
- [A Tree Row Hands Its File To The Rest Of The Machine](../../../../../contract.md)

Captured SHA-256: `7114b7db3a6014a4d320134f7f7639dfc61352f6c176e3cab541f8f3312d687c`.

<book:entry path="apps/web/src/shell/TreeContextMenu.tsx">
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
</book:entry>

</book:doc>
