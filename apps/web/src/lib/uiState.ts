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
import type { TableLayout } from "../components/TablePanel";
import { ZOOM_DEFAULT, clampZoom } from "./zoom";

/** Bumped when a stored blob would be misread by this version. */
export const UI_STATE_VERSION = 1;

export interface WorkspaceUi {
  version: number;
  /** The arrangement, or null when there is nothing worth restoring. */
  layout: Layout | null;
  /** Zoom level per tab, keyed the same way as `wrap`. The WHOLE-UI level
   * is not here: it follows the screen and the eyes in front of it rather
   * than the project, so it lives in localStorage with the other appearance
   * preferences (lib/zoom.ts). */
  zoom: Record<string, number>;
  /** Prose measure per tab, keyed by the tab's target — its path.
   *
   * Keyed by path rather than by tab id on purpose: tab ids are per-session
   * (see `reid`), so a measure stored against one would be orphaned the
   * moment it was read back. A path is what the reader thinks of as "this
   * document" anyway. */
  wrap: Record<string, number>;
  /** How big each table was left, keyed by `tableKey`.
   *
   * Here rather than in the document because a column width is presentation
   * and a `.md` document is a dataset somebody diffs. Two people opening the
   * same table are allowed to want different amounts of room for it, and
   * neither should show up in the other's `git status`. */
  tables: Record<string, TableLayout>;
}

/** The name a table's remembered size is stored under.
 *
 * Its `path` when it has one, because that is the table's own identity and it
 * survives being moved down the document, split into a second file, or having
 * prose written above it. Failing that, the nth table of this document — which
 * is stable against everything except reordering the tables themselves. */
export function tableKey(documentPath: string | null, index: number, path?: string): string {
  return path ? `path:${path}` : `${documentPath ?? "untitled"}#${index}`;
}

/** A stored size, with anything unrecognisable dropped. */
function readTableLayout(raw: unknown): TableLayout | null {
  if (!isRecord(raw)) return null;
  const out: TableLayout = {};
  if (typeof raw.fitProse === "boolean") out.fitProse = raw.fitProse;
  if (typeof raw.height === "number" && Number.isFinite(raw.height)) {
    out.height = Math.max(64, Math.min(4000, Math.round(raw.height)));
  }
  const measures = (raw: unknown, least: number, most: number) => {
    if (!isRecord(raw)) return undefined;
    const kept: Record<string, number> = {};
    for (const [index, value] of Object.entries(raw)) {
      if (typeof value === "number" && Number.isFinite(value)) {
        kept[index] = Math.max(least, Math.min(most, Math.round(value)));
      }
    }
    return Object.keys(kept).length > 0 ? kept : undefined;
  };
  out.widths = measures(raw.widths, 40, 2000);
  // Fitted rows must retain enough height to show all their wrapped text.
  out.heights = measures(raw.heights, 16, Number.MAX_SAFE_INTEGER);
  if (out.widths === undefined) delete out.widths;
  if (out.heights === undefined) delete out.heights;
  return out.fitProse === undefined && out.height === undefined && out.widths === undefined && out.heights === undefined
    ? null
    : out;
}

export function emptyUi(): WorkspaceUi {
  return { version: UI_STATE_VERSION, layout: null, wrap: {}, zoom: {}, tables: {} };
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
  const zoom: Record<string, number> = {};
  if (isRecord(raw.zoom)) {
    for (const [path, value] of Object.entries(raw.zoom)) {
      if (typeof value === "number") zoom[path] = clampZoom(value);
    }
  }
  const tables: Record<string, TableLayout> = {};
  if (isRecord(raw.tables)) {
    for (const [key, value] of Object.entries(raw.tables)) {
      const size = readTableLayout(value);
      if (size) tables[key] = size;
    }
  }
  return { version: UI_STATE_VERSION, layout, wrap, zoom, tables };
}

/** The measure for one tab, defaulting where none was stored. */
export function wrapFor(ui: WorkspaceUi, target: string): number {
  return ui.wrap[target] ?? WRAP_DEFAULT;
}

/** The same state with one tab's measure changed. */
export function withWrap(ui: WorkspaceUi, target: string, column: number): WorkspaceUi {
  return { ...ui, wrap: { ...ui.wrap, [target]: clampWrapColumn(column) } };
}

/** How big a table was left, or nothing when it has never been resized. */
export function tableLayoutFor(ui: WorkspaceUi, key: string): TableLayout | undefined {
  return ui.tables[key];
}

/** The same state with one table's size changed. */
export function withTableLayout(
  ui: WorkspaceUi,
  key: string,
  size: TableLayout,
): WorkspaceUi {
  return { ...ui, tables: { ...ui.tables, [key]: size } };
}

/** The zoom level for one tab, defaulting to actual size. */
export function zoomFor(ui: WorkspaceUi, target: string): number {
  return ui.zoom[target] ?? ZOOM_DEFAULT;
}

/** The same state with one tab's zoom changed. */
export function withZoom(ui: WorkspaceUi, target: string, level: number): WorkspaceUi {
  return { ...ui, zoom: { ...ui.zoom, [target]: clampZoom(level) } };
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
