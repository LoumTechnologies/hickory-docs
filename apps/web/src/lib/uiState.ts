// What the window looked like when you closed it.
//
// Two things go in here and nothing else: the arrangement (which tabs, in
// which panes, split which way) and each tab's prose measure. Deliberately
// not buffer contents — those are drafts, they are large, and they need a
// merge on the way back in; see lib/drafts.ts.
//
// The blob is stored under the user's own data directory (api.md, "Workspace
// state and drafts"), so nothing here can end up in anybody's git history.
//
// ## Reading it back is the hard half
//
// Everything in this file is written for the case where the stored state is
// WRONG: hand-edited, written by a newer build, truncated by a full disk,
// or carrying a layout whose shape this version no longer understands. A
// window that refuses to open because its remembered layout is damaged is
// far worse than one that opens with default tabs, so `normalizeUi` is total:
// it takes `unknown`, never throws, and drops anything it does not recognise
// rather than trusting it into the shell.

import { panes, reid, type Layout, type Node, type Pane, type Tab, type ViewKind } from "../shell/layout";
import { WRAP_DEFAULT, clampWrapColumn } from "../editor/wrapColumn";

/** Bumped when a stored blob would be misread by this version. */
export const UI_STATE_VERSION = 1;

export interface WorkspaceUi {
  version: number;
  /** The arrangement, or null when there is nothing worth restoring. */
  layout: Layout | null;
  /** Prose measure per tab, keyed by the tab's target — its path.
   *
   * Keyed by path rather than by tab id on purpose: tab ids are per-session
   * (see `reid`), so a measure stored against one would be orphaned the
   * moment it was read back. A path is what the reader thinks of as "this
   * document" anyway. */
  wrap: Record<string, number>;
}

export function emptyUi(): WorkspaceUi {
  return { version: UI_STATE_VERSION, layout: null, wrap: {} };
}

const VIEW_KINDS: ReadonlySet<string> = new Set<ViewKind>([
  "document",
  "generated",
  "file",
  "tool",
  "tree",
  "untitled",
  "scratchpad",
  "terminal",
]);

const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

const str = (v: unknown): string | undefined => (typeof v === "string" ? v : undefined);

/**
 * One tab, or null when the entry is not one.
 *
 * A tab whose kind this build does not know is dropped rather than rendered:
 * the shell would have nothing to draw for it, and an empty pane is a better
 * answer than a blank rectangle with a title.
 */
function readTab(raw: unknown): Tab | null {
  if (!isRecord(raw)) return null;
  const kind = str(raw.kind);
  const target = str(raw.target);
  const id = str(raw.id);
  if (!kind || !VIEW_KINDS.has(kind) || target === undefined || !id) return null;
  return {
    id,
    kind: kind as ViewKind,
    target,
    ...(str(raw.title) !== undefined ? { title: str(raw.title) } : {}),
    ...(str(raw.docId) !== undefined ? { docId: str(raw.docId) } : {}),
  };
}

function readNode(raw: unknown): Node | null {
  if (!isRecord(raw)) return null;
  const id = str(raw.id);
  if (!id) return null;

  if (raw.type === "pane") {
    if (!Array.isArray(raw.tabs)) return null;
    const tabs = raw.tabs.map(readTab).filter((t): t is Tab => t !== null);
    const active =
      typeof raw.active === "number" && Number.isInteger(raw.active) ? raw.active : 0;
    const pane: Pane = {
      type: "pane",
      id,
      tabs,
      // Clamped rather than trusted: an `active` past the end of a shortened
      // tab list is the exact shape of a crash on first render.
      active: tabs.length === 0 ? 0 : Math.min(Math.max(active, 0), tabs.length - 1),
      ...(str(raw.region) !== undefined ? { region: str(raw.region) } : {}),
      ...(raw.collapsed === true ? { collapsed: true } : {}),
    };
    return pane;
  }

  if (raw.type === "split") {
    const direction = raw.direction === "column" ? "column" : "row";
    if (!Array.isArray(raw.children)) return null;
    const children = raw.children.map(readNode).filter((n): n is Node => n !== null);
    if (children.length === 0) return null;
    // A split of one is not a split. Collapsing it here means a dropped child
    // leaves a working arrangement rather than a divider with nothing beside
    // it.
    if (children.length === 1) return children[0];
    const rawSizes = Array.isArray(raw.sizes) ? raw.sizes : [];
    const sizes = children.map((_, i) =>
      typeof rawSizes[i] === "number" && rawSizes[i] > 0 ? (rawSizes[i] as number) : 1,
    );
    const total = sizes.reduce((a, b) => a + b, 0);
    return {
      type: "split",
      id,
      direction,
      children,
      // Renormalised: the fractions must sum to 1, and children dropped above
      // would otherwise leave a layout that is 70% wide.
      sizes: sizes.map((s) => s / total),
    };
  }
  return null;
}

/**
 * A stored blob, made safe to hand to the shell.
 *
 * Total by construction: any input at all produces a `WorkspaceUi`, and the
 * worst case is `emptyUi()` — the window opens with default tabs.
 */
export function normalizeUi(raw: unknown): WorkspaceUi {
  if (!isRecord(raw)) return emptyUi();
  // A blob from a version that did not exist when this build shipped may mean
  // something different by the same field names. Opening default tabs is the
  // honest response; guessing is how a layout gets silently mangled.
  if (raw.version !== UI_STATE_VERSION) return emptyUi();

  let layout: Layout | null = null;
  if (isRecord(raw.layout)) {
    const root = readNode(raw.layout.root);
    if (root) {
      const live = panes(root);
      const focus = str(raw.layout.focus);
      layout = {
        root,
        focus: focus && live.some((p) => p.id === focus) ? focus : (live[0]?.id ?? ""),
      };
      // Ids come back fresh, or the session's counter would hand out one of
      // these again on the next split.
      layout = layout.focus ? reid(layout) : null;
    }
  }

  const wrap: Record<string, number> = {};
  if (isRecord(raw.wrap)) {
    for (const [path, value] of Object.entries(raw.wrap)) {
      if (typeof value === "number") wrap[path] = clampWrapColumn(value);
    }
  }
  return { version: UI_STATE_VERSION, layout, wrap };
}

/** The measure for one tab, defaulting where none was stored. */
export function wrapFor(ui: WorkspaceUi, target: string): number {
  return ui.wrap[target] ?? WRAP_DEFAULT;
}

/** The same state with one tab's measure changed. */
export function withWrap(ui: WorkspaceUi, target: string, column: number): WorkspaceUi {
  return { ...ui, wrap: { ...ui.wrap, [target]: clampWrapColumn(column) } };
}

/**
 * Whether an arrangement is worth writing down.
 *
 * An empty workspace — nothing but the folder tree — is what a fresh project
 * opens as anyway, and storing it would only turn "we have never seen this
 * project" into "this project was left empty", which reads the same and
 * costs a write on every start.
 */
export function worthStoring(layout: Layout): boolean {
  return panes(layout.root).some((pane) =>
    pane.tabs.some((t) => t.kind !== "tree" && t.kind !== "tool"),
  );
}
