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
export function remove(node: Layout, tab: string): Layout {
  if (!isGroup(node)) return { ...node, first: remove(node.first, tab), second: remove(node.second, tab) };
  const tabs = node.tabs.filter(value => value !== tab);
  return { ...node, tabs, active: node.active === tab ? tabs.at(-1) ?? null : node.active };
}
export function move(node: Layout, tab: string, target: string): Layout {
  if (!groups(node).some(group => group.id === target) || !groups(node).some(group => group.tabs.includes(tab))) return node;
  return activate(remove(node, tab), target, tab);
}
export function split(node: Layout, id: string, axis: Axis, newId: string): Layout {
  return update(node, id, leaf => isGroup(leaf) ? { id: `split-${newId}`, axis, ratio: .5,
    first: leaf, second: { id: newId, tabs: [], active: null } } : leaf);
}
