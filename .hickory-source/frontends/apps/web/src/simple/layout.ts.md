<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Move tab identities through a binary split tree

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

Edge drops classify normalized pane coordinates using the outer twenty percent; left/right own corners, and the middle is not a split target. Docking validates the source, target and fresh IDs before removing a tab, then creates one equal split in the requested orientation. Existing buffers stay outside the layout. After a successful close, move or dock, remove empty groups and promote their surviving sibling. Keep survivor identities and split ratios, and retain one empty leaf only if every tab is closed. Explicit Split creates a temporary empty destination until the next tab operation. Collapse after insertion, so moving within one group cannot delete its destination.

A layout contains only identities and proportions. Text, selection and undo do not belong to the tree. A leaf is a tab group; a branch divides its available rectangle.

<ui:copy id="simple-f98ffc6999-0">
export type Edge = "left" | "right" | "top" | "bottom";
export function dropEdge(x: number, y: number): Edge | null {
  if (!Number.isFinite(x) || !Number.isFinite(y) || x < 0 || x > 1 || y < 0 || y > 1) return null;
  return x < .2 ? "left" : x > .8 ? "right" : y < .2 ? "top" : y > .8 ? "bottom" : null;
}
export type Axis = "horizontal" | "vertical";
export interface Group { id: string; tabs: string[]; active: string | null }
export type Layout = Group | { id: string; axis: Axis; ratio: number; first: Layout; second: Layout };
export const isGroup = (node: Layout): node is Group => "tabs" in node;
export const groups = (node: Layout): Group[] => isGroup(node) ? [node] : [...groups(node.first), ...groups(node.second)];
export function initialLayout(): Layout {
  return { id: "root", axis: "horizontal", ratio: .22,
    first: { id: "files-group", tabs: ["files"], active: "files" },
    second: { id: "document-group", tabs: [], active: null } };
}
export function update(node: Layout, id: string, transform: (node: Layout) => Layout): Layout {
  if (node.id === id) return transform(node);
  return isGroup(node) ? node : { ...node, first: update(node.first, id, transform), second: update(node.second, id, transform) };
}
export function activate(node: Layout, group: string, tab: string): Layout {
  return update(node, group, leaf => isGroup(leaf) ? { ...leaf, tabs: leaf.tabs.includes(tab) ? leaf.tabs : [...leaf.tabs, tab], active: tab } : leaf);
}
</ui:copy>

A drop on a tab inserts before that identity; a drop on unused strip space appends. Validate the destination and anchor before removing anything, so an obsolete drop cannot lose a tab. Resolve the insertion position after removal to avoid shifting a forward move by one. Dropping on itself is a no-op. Moving a tab removes its old membership before inserting it at the target. After the full move, empty groups disappear and their siblings expand. Split operations wrap a leaf and preserve its existing identity.

<ui:copy id="simple-f98ffc6999-1">
// Remove first, insert next, and collapse only after the complete operation.
// Otherwise a same-pane reorder could delete its own destination.
function collapse(node: Layout): Layout {
  function live(tree: Layout): Layout | null {
    if (isGroup(tree)) return tree.tabs.length ? tree : null;
    const first = live(tree.first), second = live(tree.second);
    return first && second ? { ...tree, first, second } : first ?? second;
  }
  return live(node) ?? { id: node.id, tabs: [], active: null };
}
export function remove(node: Layout, tab: string): Layout {
  return groups(node).some(group => group.tabs.includes(tab)) ? (tab !== "files" && groups(node).some(group => group.tabs.includes("files")) && groups(node).filter(group => !group.tabs.includes("files")).every(group => group.tabs.every(value => value === tab)) ? detach(node, tab) : collapse(detach(node, tab))) : node;
}
function detach(node: Layout, tab: string): Layout {
  if (!isGroup(node)) return { ...node, first: detach(node.first, tab), second: detach(node.second, tab) };
  const tabs = node.tabs.filter(value => value !== tab);
  return { ...node, tabs, active: node.active === tab ? tabs.at(-1) ?? null : node.active };
}
export function move(node: Layout, tab: string, target: string, before?: string): Layout {
  const destination = groups(node).find(group => group.id === target);
  if (!destination || !groups(node).some(group => group.tabs.includes(tab)) ||
      (before !== undefined && !destination.tabs.includes(before)) || before === tab) return node;
  return collapse(update(detach(node, tab), target, leaf => {
    if (!isGroup(leaf)) return leaf;
    const tabs = [...leaf.tabs];
    tabs.splice(before === undefined ? tabs.length : tabs.indexOf(before), 0, tab);
    return { ...leaf, tabs, active: tab };
  }));
}
export function dock(node: Layout, tab: string, target: string, edge: Edge, newId: string): Layout {
  const ids = (tree: Layout): string[] => isGroup(tree) ? [tree.id] : [tree.id, ...ids(tree.first), ...ids(tree.second)];
  const existing = ids(node);
  if (!["left", "right", "top", "bottom"].includes(edge) || !groups(node).some(group => group.id === target) ||
      !groups(node).some(group => group.tabs.includes(tab)) || !newId || existing.includes(newId) || existing.includes(`split-${newId}`)) return node;
  return collapse(update(detach(node, tab), target, retained => {
    const added: Group = { id: newId, tabs: [tab], active: tab };
    const leading = edge === "left" || edge === "top";
    return { id: `split-${newId}`, axis: edge === "left" || edge === "right" ? "horizontal" : "vertical", ratio: .5,
      first: leading ? added : retained, second: leading ? retained : added };
  }));
}
export function split(node: Layout, id: string, axis: Axis, newId: string): Layout {
  return update(node, id, leaf => isGroup(leaf) ? { id: `split-${newId}`, axis, ratio: .5,
    first: leaf, second: { id: newId, tabs: [], active: null } } : leaf);
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/layout.ts"><ui:paste select="#simple-f98ffc6999-0"/><ui:paste select="#simple-f98ffc6999-1"/></ui:file>

</ui:doc>
