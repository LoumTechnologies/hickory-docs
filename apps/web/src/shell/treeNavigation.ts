// Structural keyboard movement for the workspace tree, kept pure so provider
// nodes and filesystem rows cannot quietly learn different arrow semantics.

export interface NavigableTreeRow {
  key: string;
  parent: string | null;
  expanded?: boolean;
  expandable?: boolean;
  selectable?: boolean;
}
export type TreeNavigationIntent =
  | { kind: "focus"; key: string }
  | { kind: "toggle"; key: string }
  | { kind: "activate"; key: string };

export function treeNavigationIntent(
  rows: readonly NavigableTreeRow[],
  currentKey: string,
  key: string,
): TreeNavigationIntent | null {
  const at = rows.findIndex((row) => row.key === currentKey);
  if (at < 0) return null;
  const current = rows[at];
  if (key === "ArrowUp" && at > 0) return { kind: "focus", key: rows[at - 1].key };
  if (key === "ArrowDown" && at + 1 < rows.length) return { kind: "focus", key: rows[at + 1].key };
  if (key === "Home" && rows.length > 0) return { kind: "focus", key: rows[0].key };
  if (key === "End" && rows.length > 0) return { kind: "focus", key: rows[rows.length - 1].key };
  if (key === "ArrowRight" && current.expandable) {
    if (!current.expanded) return { kind: "toggle", key: current.key };
    const child = rows.find((row) => row.parent === current.key);
    return child ? { kind: "focus", key: child.key } : null;
  }
  if (key === "ArrowLeft") {
    if (current.expandable && current.expanded) return { kind: "toggle", key: current.key };
    return current.parent ? { kind: "focus", key: current.parent } : null;
  }
  if (key === "Enter") return { kind: "activate", key: current.key };
  return null;
}

/** Select the compatible rows between an anchor and the new point. */
export function treeSelectionRange(
  rows: readonly NavigableTreeRow[],
  anchorKey: string,
  pointKey: string,
): string[] {
  const anchor = rows.findIndex((row) => row.key === anchorKey);
  const point = rows.findIndex((row) => row.key === pointKey);
  if (anchor < 0 || point < 0) return [];
  const from = Math.min(anchor, point);
  const to = Math.max(anchor, point);
  return rows.slice(from, to + 1).filter((row) => row.selectable).map((row) => row.key);
}
