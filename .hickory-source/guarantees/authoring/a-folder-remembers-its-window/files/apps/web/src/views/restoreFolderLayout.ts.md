<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/views/restoreFolderLayout.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [A Folder Remembers Its Window](../../../../../contract.md)

Captured SHA-256: `29a87326d282960a8decab58adf05e8c7b0bce747f1e91479adf998076d52798`.

<book:entry path="apps/web/src/views/restoreFolderLayout.ts">
import { open, panes, type Layout } from "../shell/layout";
import { isWorkspaceEmpty } from "./workspaceState";

/** Restore folder geometry while keeping editors opened before the folder. */
export function restoreFolderLayout(current: Layout, restored: Layout, folder: boolean, introduction?: string): Layout {
  if (!folder) return introduction || !isWorkspaceEmpty(current) ? current : restored;
  const existing = panes(restored.root).flatMap(pane => pane.tabs);
  const carried = panes(current.root).flatMap(pane => pane.tabs).filter(tab =>
    tab.kind !== "tree" && tab.id !== introduction && !existing.some(saved =>
      saved.kind === tab.kind && (tab.kind === "untitled" ? saved.id === tab.id : saved.target === tab.target)));
  return carried.reduce((layout, tab) => open(layout, tab), restored);
}
</book:entry>

</book:doc>
