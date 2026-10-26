<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/shell/useTreeNavigation.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-d184bfff53d1"></a>
## `apps/web/src/shell/useTreeNavigation.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Every Shortcut Is A Setting](../../../../../guarantees/editor-intelligence/every-shortcut-is-a-setting/contract.md).

<a id="f-d184bfff53d1-ROW"></a>
### `ROW`

<storycode:copy id="f-d184bfff53d1-ROW">
import { useRef, type FocusEvent, type KeyboardEvent } from "react";

import { isAction } from "../lib/keymap";
import { treeNavigationIntent, treeSelectionRange, type NavigableTreeRow } from "./treeNavigation";

const ROW = "[data-workspace-node-key]";

</storycode:copy>
<a id="f-d184bfff53d1-DomTreeRow"></a>
### `DomTreeRow`

<storycode:copy id="f-d184bfff53d1-DomTreeRow">
interface DomTreeRow extends NavigableTreeRow {
  element: HTMLElement;
  path?: string;
}
</storycode:copy>
<a id="f-d184bfff53d1-rowsOf"></a>
### `rowsOf`

<storycode:copy id="f-d184bfff53d1-rowsOf">
function rowsOf(tree: HTMLElement): DomTreeRow[] {
  return [...tree.querySelectorAll<HTMLElement>(ROW)].map((element) => ({
    element,
    key: element.dataset.workspaceNodeKey ?? "",
    parent: element.dataset.workspaceParent || null,
    expandable: element.dataset.workspaceExpandable === "true",
    expanded: element.dataset.workspaceExpanded === "true",
    selectable: element.dataset.treePath !== undefined,
    path: element.dataset.treePath,
  }));
}

</storycode:copy>
<a id="f-d184bfff53d1-useTreeNavigation"></a>
### `useTreeNavigation`

<storycode:copy id="f-d184bfff53d1-useTreeNavigation">
export function useTreeNavigation({
  replacePaths,
  addPaths,
}: {
  replacePaths: (paths: readonly string[]) => void;
  addPaths: (paths: readonly string[]) => void;
}) {
  const active = useRef<string | null>(null);
  const anchor = useRef<string | null>(null);

  const focus = (row: DomTreeRow) => {
    row.element.tabIndex = 0;
    row.element.focus();
    active.current = row.key;
  };

  const onFocusCapture = (event: FocusEvent<HTMLElement>) => {
    const row = (event.target as HTMLElement).closest<HTMLElement>(ROW);
    if (!row) return;
    const tree = event.currentTarget;
    for (const other of tree.querySelectorAll<HTMLElement>(ROW)) {
      if (other !== row) other.tabIndex = -1;
    }
    row.tabIndex = 0;
    active.current = row.dataset.workspaceNodeKey ?? null;
  };

  const onFocus = (event: FocusEvent<HTMLElement>) => {
    if (event.target !== event.currentTarget) return;
    const rows = rowsOf(event.currentTarget);
    const remembered = rows.find((row) => row.key === active.current);
    if (remembered ?? rows[0]) focus(remembered ?? rows[0]);
  };

  const onKeyDown = (event: KeyboardEvent<HTMLElement>): boolean => {
    const tree = event.currentTarget;
    const element = (event.target as HTMLElement).closest<HTMLElement>(ROW);
    if (!element) return false;
    const rows = rowsOf(tree);
    const currentKey = element.dataset.workspaceNodeKey ?? "";
    const at = rows.findIndex((row) => row.key === currentKey);

    const columnDirection = isAction(event, "tree.addSelectionAbove")
      ? -1
      : isAction(event, "tree.addSelectionBelow")
        ? 1
        : 0;
    if (columnDirection !== 0) {
      const target = rows[at + columnDirection];
      if (!target) return true;
      event.preventDefault();
      event.stopPropagation();
      addPaths([rows[at].path, target.path].filter((path): path is string => path !== undefined));
      focus(target);
      anchor.current ??= currentKey;
      return true;
    }

    if ((event.ctrlKey || event.metaKey || event.altKey) && !event.shiftKey) return false;
    const intent = treeNavigationIntent(rows, currentKey, event.key);
    if (!intent) return false;
    event.preventDefault();
    event.stopPropagation();

    if (intent.kind === "toggle" || intent.kind === "activate") {
      rows.find((row) => row.key === intent.key)?.element.click();
      return true;
    }

    const target = rows.find((row) => row.key === intent.key);
    if (!target) return true;
    if (event.shiftKey && (event.key === "ArrowUp" || event.key === "ArrowDown")) {
      anchor.current ??= currentKey;
      const selected = new Set(treeSelectionRange(rows, anchor.current, target.key));
      replacePaths(rows.filter((row) => selected.has(row.key)).flatMap((row) => row.path ? [row.path] : []));
    } else {
      anchor.current = target.key;
    }
    focus(target);
    return true;
  };

  return { onFocus, onFocusCapture, onKeyDown };
}
</storycode:copy>

### Assemble `apps/web/src/shell/useTreeNavigation.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/shell/useTreeNavigation.ts"><storycode:paste select="#f-d184bfff53d1-ROW"/><storycode:paste select="#f-d184bfff53d1-DomTreeRow"/><storycode:paste select="#f-d184bfff53d1-rowsOf"/><storycode:paste select="#f-d184bfff53d1-useTreeNavigation"/></storycode:file>

</storycode:doc>
