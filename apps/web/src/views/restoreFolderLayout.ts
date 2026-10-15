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
