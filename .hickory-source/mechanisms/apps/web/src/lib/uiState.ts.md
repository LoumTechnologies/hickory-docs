<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/lib/uiState.ts

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-7183338fb20a"></a>
## `apps/web/src/lib/uiState.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Folder Remembers Its Window](../../../../../guarantees/authoring/a-folder-remembers-its-window/contract.md), [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md).

<a id="f-7183338fb20a-UI_STATE_VERSION"></a>
### `UI_STATE_VERSION`

What the window looked like when you closed it.
Presentation lives here: open tabs, pane and spatial arrangements, prose
measures, zoom and table sizes. Deliberately
not buffer contents — those are drafts, they are large, and they need a
merge on the way back in; see lib/drafts.ts.
The blob is stored under the user's own data directory (api.md, "Workspace
state and drafts"), so nothing here can end up in anybody's git history.
## Reading it back is the hard half
Everything in this file is written for the case where the stored state is
WRONG: hand-edited, written by a newer build, truncated by a full disk,
or carrying a layout whose shape this version no longer understands. A
window that refuses to open because its remembered layout is damaged is
far worse than one that opens with default tabs, so `normalizeUi` is total:
it takes `unknown`, never throws, and drops anything it does not recognise
rather than trusting it into the shell.

<storycode:copy id="f-7183338fb20a-UI_STATE_VERSION">
// What the window looked like when you closed it.
//
// Presentation lives here: open tabs, pane and spatial arrangements, prose
// measures, zoom and table sizes. Deliberately
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
import { readSpatialState, type SpatialState } from "../shell/spatialLayout";
import { WRAP_DEFAULT, clampWrapColumn } from "../editor/wrapColumn";
import type { TableLayout } from "../components/TablePanel";
import { ZOOM_DEFAULT, clampZoom } from "./zoom";

/** Bumped when a stored blob would be misread by this version. */
export const UI_STATE_VERSION = 1;

</storycode:copy>
<a id="f-7183338fb20a-WorkspaceUi"></a>
### `WorkspaceUi`

<storycode:copy id="f-7183338fb20a-WorkspaceUi">
export interface WorkspaceUi {
  version: number;
  /** Spatial tiles, centered view, drawer pins and widths. */
  spatial?: SpatialState;
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

</storycode:copy>
<a id="f-7183338fb20a-tableKey"></a>
### `tableKey`

The name a table's remembered size is stored under.
Its `path` when it has one, because that is the table's own identity and it
survives being moved down the document, split into a second file, or having
prose written above it. Failing that, the nth table of this document — which
is stable against everything except reordering the tables themselves.

<storycode:copy id="f-7183338fb20a-tableKey">
/** The name a table's remembered size is stored under.
 *
 * Its `path` when it has one, because that is the table's own identity and it
 * survives being moved down the document, split into a second file, or having
 * prose written above it. Failing that, the nth table of this document — which
 * is stable against everything except reordering the tables themselves. */
export function tableKey(documentPath: string | null, index: number, path?: string): string {
  return path ? `path:${path}` : `${documentPath ?? "untitled"}#${index}`;
}

</storycode:copy>
<a id="f-7183338fb20a-readTableLayout"></a>
### `readTableLayout`

A stored size, with anything unrecognisable dropped.

<storycode:copy id="f-7183338fb20a-readTableLayout">
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

</storycode:copy>
<a id="f-7183338fb20a-emptyUi"></a>
### `emptyUi`

<storycode:copy id="f-7183338fb20a-emptyUi">
export function emptyUi(): WorkspaceUi {
  return { version: UI_STATE_VERSION, layout: null, wrap: {}, zoom: {}, tables: {} };
}

</storycode:copy>
<a id="f-7183338fb20a-VIEW_KINDS"></a>
### `VIEW_KINDS`

<storycode:copy id="f-7183338fb20a-VIEW_KINDS">
const VIEW_KINDS: ReadonlySet<string> = new Set<ViewKind>([
  "document",
  "generated",
  "file",
  "tool",
  "tree",
  "untitled",
  "scratchpad",
  "terminal",
  "chat",
]);

</storycode:copy>
<a id="f-7183338fb20a-isRecord"></a>
### `isRecord`

<storycode:copy id="f-7183338fb20a-isRecord">
const isRecord = (v: unknown): v is Record<string, unknown> =>
  typeof v === "object" && v !== null && !Array.isArray(v);

</storycode:copy>
<a id="f-7183338fb20a-str"></a>
### `str`

<storycode:copy id="f-7183338fb20a-str">
const str = (v: unknown): string | undefined => (typeof v === "string" ? v : undefined);

</storycode:copy>
<a id="f-7183338fb20a-readTab"></a>
### `readTab`

One tab, or null when the entry is not one.
A tab whose kind this build does not know is dropped rather than rendered:
the shell would have nothing to draw for it, and an empty pane is a better
answer than a blank rectangle with a title.

<storycode:copy id="f-7183338fb20a-readTab">
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

</storycode:copy>
<a id="f-7183338fb20a-readNode"></a>
### `readNode`

<storycode:copy id="f-7183338fb20a-readNode">
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

</storycode:copy>
<a id="f-7183338fb20a-normalizeUi"></a>
### `normalizeUi`

A stored blob, made safe to hand to the shell.
Total by construction: any input at all produces a `WorkspaceUi`, and the
worst case is `emptyUi()` — the window opens with default tabs.

<storycode:copy id="f-7183338fb20a-normalizeUi">
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
  return { version: UI_STATE_VERSION, layout, wrap, zoom, tables, ...(isRecord(raw.spatial) ? { spatial: readSpatialState(JSON.stringify({ ...raw.spatial, version: 1 })) } : {}) };
}

</storycode:copy>
<a id="f-7183338fb20a-wrapFor"></a>
### `wrapFor`

The measure for one tab, defaulting where none was stored.

<storycode:copy id="f-7183338fb20a-wrapFor">
/** The measure for one tab, defaulting where none was stored. */
export function wrapFor(ui: WorkspaceUi, target: string): number {
  return ui.wrap[target] ?? WRAP_DEFAULT;
}

</storycode:copy>
<a id="f-7183338fb20a-withWrap"></a>
### `withWrap`

The same state with one tab's measure changed.

<storycode:copy id="f-7183338fb20a-withWrap">
/** The same state with one tab's measure changed. */
export function withWrap(ui: WorkspaceUi, target: string, column: number): WorkspaceUi {
  return { ...ui, wrap: { ...ui.wrap, [target]: clampWrapColumn(column) } };
}

</storycode:copy>
<a id="f-7183338fb20a-tableLayoutFor"></a>
### `tableLayoutFor`

How big a table was left, or nothing when it has never been resized.

<storycode:copy id="f-7183338fb20a-tableLayoutFor">
/** How big a table was left, or nothing when it has never been resized. */
export function tableLayoutFor(ui: WorkspaceUi, key: string): TableLayout | undefined {
  return ui.tables[key];
}

</storycode:copy>
<a id="f-7183338fb20a-withTableLayout"></a>
### `withTableLayout`

The same state with one table's size changed.

<storycode:copy id="f-7183338fb20a-withTableLayout">
/** The same state with one table's size changed. */
export function withTableLayout(
  ui: WorkspaceUi,
  key: string,
  size: TableLayout,
): WorkspaceUi {
  return { ...ui, tables: { ...ui.tables, [key]: size } };
}

</storycode:copy>
<a id="f-7183338fb20a-zoomFor"></a>
### `zoomFor`

The zoom level for one tab, defaulting to actual size.

<storycode:copy id="f-7183338fb20a-zoomFor">
/** The zoom level for one tab, defaulting to actual size. */
export function zoomFor(ui: WorkspaceUi, target: string): number {
  return ui.zoom[target] ?? ZOOM_DEFAULT;
}

</storycode:copy>
<a id="f-7183338fb20a-withZoom"></a>
### `withZoom`

<storycode:copy id="f-7183338fb20a-withZoom">
/** The same state with one tab's zoom changed. */
export function withZoom(ui: WorkspaceUi, target: string, level: number): WorkspaceUi {
  return { ...ui, zoom: { ...ui.zoom, [target]: clampZoom(level) } };
}

</storycode:copy>
<a id="f-7183338fb20a-worthStoring"></a>
### `worthStoring`

Whether an arrangement is worth writing down.
An empty workspace — nothing but the folder tree — is what a fresh project
opens as anyway, and storing it would only turn "we have never seen this
project" into "this project was left empty", which reads the same and
costs a write on every start.

<storycode:copy id="f-7183338fb20a-worthStoring">
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
</storycode:copy>

### Assemble `apps/web/src/lib/uiState.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiState.ts"><storycode:paste select="#f-7183338fb20a-UI_STATE_VERSION"/><storycode:paste select="#f-7183338fb20a-WorkspaceUi"/><storycode:paste select="#f-7183338fb20a-tableKey"/><storycode:paste select="#f-7183338fb20a-readTableLayout"/><storycode:paste select="#f-7183338fb20a-emptyUi"/><storycode:paste select="#f-7183338fb20a-VIEW_KINDS"/><storycode:paste select="#f-7183338fb20a-isRecord"/><storycode:paste select="#f-7183338fb20a-str"/><storycode:paste select="#f-7183338fb20a-readTab"/><storycode:paste select="#f-7183338fb20a-readNode"/><storycode:paste select="#f-7183338fb20a-normalizeUi"/><storycode:paste select="#f-7183338fb20a-wrapFor"/><storycode:paste select="#f-7183338fb20a-withWrap"/><storycode:paste select="#f-7183338fb20a-tableLayoutFor"/><storycode:paste select="#f-7183338fb20a-withTableLayout"/><storycode:paste select="#f-7183338fb20a-zoomFor"/><storycode:paste select="#f-7183338fb20a-withZoom"/><storycode:paste select="#f-7183338fb20a-worthStoring"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-43efcbce9427"></a>
## `apps/web/src/lib/uiState.test.ts`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [A Table Is A Dataset And A Paragraph](../../../../../guarantees/authoring/a-table-is-a-dataset-and-a-paragraph/contract.md), [Trace a ribbon to the visible part of its source](../../../../../guarantees/lineage/a-ribbon-crosses-documents/contract.md).

<a id="f-43efcbce9427-stored"></a>
### `stored`

<storycode:copy id="f-43efcbce9427-stored">
import { describe, expect, it } from "vitest";

import {
  UI_STATE_VERSION,
  emptyUi,
  normalizeUi,
  tableKey,
  withTableLayout,
  withWrap,
  worthStoring,
  wrapFor,
} from "./uiState";
import { WRAP_DEFAULT, WRAP_MAX, WRAP_MIN } from "../editor/wrapColumn";
import { freeform, open as openInLayout, panes, tab, withTree } from "../shell/layout";

const stored = (layout: unknown, wrap: Record<string, unknown> = {}) => ({
  version: UI_STATE_VERSION,
  layout,
  wrap,
});

</storycode:copy>
<a id="f-43efcbce9427-onePane"></a>
### `onePane`

<storycode:copy id="f-43efcbce9427-onePane">
const onePane = {
  root: {
    type: "pane",
    id: "pane-1",
    tabs: [{ id: "tab-1", kind: "document", target: "notes.md", docId: "d1" }],
    active: 0,
  },
  focus: "pane-1",
};

</storycode:copy>
<a id="f-43efcbce9427-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement">
describe("reading a stored layout back", () => {
  it("restores tabs, in their panes", () => {
    const ui = normalizeUi(stored(onePane));
    const restored = panes(ui.layout!.root);
    expect(restored).toHaveLength(1);
    expect(restored[0].tabs.map((t) => t.target)).toEqual(["notes.md"]);
    expect(restored[0].tabs[0].docId).toBe("d1");
  });

  it("mints fresh ids, so the session's counter cannot hand one out twice", () => {
    // Ids come from a counter that restarts at zero every session. A restored
    // `pane-1` plus the next split's `pane-1` is two panes with one id, and
    // every lookup finding whichever came first.
    const ui = normalizeUi(stored(onePane));
    const restored = panes(ui.layout!.root)[0];
    expect(restored.id).not.toBe("pane-1");
    expect(restored.tabs[0].id).not.toBe("tab-1");
    expect(ui.layout!.focus).toBe(restored.id);
  });

  it("restores a split with its proportions", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s1",
          direction: "row",
          sizes: [0.3, 0.7],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { type: "pane", id: "p2", tabs: [], active: 0 },
          ],
        },
        focus: "p2",
      }),
    );
    const root = ui.layout!.root;
    expect(root.type).toBe("split");
    expect(root.type === "split" && root.sizes).toEqual([0.3, 0.7]);
  });
});

</storycode:copy>
<a id="f-43efcbce9427-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement-2">
describe("reading a stored layout that is WRONG", () => {
  // Every case here is a window that must still open. A damaged layout is an
  // annoyance; a window that will not start because of one is a disaster.
  it("opens default tabs for a blob that is not an object", () => {
    for (const junk of [null, undefined, 42, "layout", []]) {
      expect(normalizeUi(junk)).toEqual(emptyUi());
    }
  });

  it("opens default tabs for a version this build does not know", () => {
    // The same field names may mean something else there; guessing is how a
    // layout gets silently mangled.
    expect(normalizeUi({ version: 99, layout: onePane })).toEqual(emptyUi());
  });

  it("drops a tab whose kind this build cannot draw", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "pane",
          id: "p1",
          active: 0,
          tabs: [
            { id: "t1", kind: "hologram", target: "x" },
            { id: "t2", kind: "file", target: "README.md" },
          ],
        },
        focus: "p1",
      }),
    );
    expect(panes(ui.layout!.root)[0].tabs.map((t) => t.target)).toEqual(["README.md"]);
  });

  it("clamps an active index past the end of a shortened tab list", () => {
    // The exact shape of a crash on first render.
    const ui = normalizeUi(
      stored({
        root: { type: "pane", id: "p1", active: 9, tabs: [{ id: "t", kind: "file", target: "a" }] },
        focus: "p1",
      }),
    );
    expect(panes(ui.layout!.root)[0].active).toBe(0);
  });

  it("collapses a split left with one child rather than drawing a lone divider", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s",
          direction: "row",
          sizes: [0.5, 0.5],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { nonsense: true },
          ],
        },
        focus: "p1",
      }),
    );
    expect(ui.layout!.root.type).toBe("pane");
  });

  it("renormalises sizes so a layout is never 70% wide", () => {
    const ui = normalizeUi(
      stored({
        root: {
          type: "split",
          id: "s",
          direction: "column",
          sizes: [3, 1],
          children: [
            { type: "pane", id: "p1", tabs: [], active: 0 },
            { type: "pane", id: "p2", tabs: [], active: 0 },
          ],
        },
        focus: "p1",
      }),
    );
    const root = ui.layout!.root;
    const sizes = root.type === "split" ? root.sizes : [];
    expect(sizes.reduce((a, b) => a + b, 0)).toBeCloseTo(1);
  });

  it("repairs a focus naming a pane that is not there", () => {
    // Otherwise every "open here" has nowhere to go.
    const ui = normalizeUi(stored({ ...onePane, focus: "pane-gone" }));
    expect(panes(ui.layout!.root).some((p) => p.id === ui.layout!.focus)).toBe(true);
  });
});

</storycode:copy>
<a id="f-43efcbce9427-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement-3">
describe("the prose measure, per tab", () => {
  it("keys on the path, because tab ids do not survive a session", () => {
    const ui = withWrap(emptyUi(), "notes.md", 64);
    expect(wrapFor(ui, "notes.md")).toBe(64);
    expect(wrapFor(ui, "other.md")).toBe(WRAP_DEFAULT);
  });

  it("clamps a stored measure that would not lay out", () => {
    const ui = normalizeUi(stored(onePane, { "a.md": 1, "b.md": 9999, "c.md": "wide" }));
    expect(ui.wrap["a.md"]).toBe(WRAP_MIN);
    expect(ui.wrap["b.md"]).toBe(WRAP_MAX);
    expect(ui.wrap["c.md"]).toBeUndefined();
  });
});

</storycode:copy>
<a id="f-43efcbce9427-expression_statement-4"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement-4">
describe("what is worth writing down", () => {
  it("does not store a workspace holding nothing but furniture", () => {
    // A fresh project opens like this anyway; storing it only turns "never
    // seen" into "left empty", which reads the same and costs a write.
    expect(worthStoring(withTree(freeform(), tab("tree", "folder", "Files")))).toBe(false);
  });

  it("stores a workspace with a document open", () => {
    const layout = withTree(freeform(), tab("tree", "folder", "Files"));
    const withDoc = openInLayout(layout, tab("document", "notes.md", undefined, "d1"), layout.focus);
    expect(worthStoring(withDoc)).toBe(true);
  });
});

</storycode:copy>
<a id="f-43efcbce9427-expression_statement-5"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement-5">
describe("a table's remembered size", () => {
  it("is named by the table's own path where it has one", () => {
    // A path survives prose being written above the table, the table moving
    // down the document, or another table appearing before it.
    expect(tableKey("notes/sales.md", 2, "data/sales.csv")).toBe("path:data/sales.csv");
  });

  it("falls back to the nth table of the document when it writes no file", () => {
    expect(tableKey("notes/sales.md", 2)).toBe("notes/sales.md#2");
    expect(tableKey(null, 0)).toBe("untitled#0");
  });

  it("comes back from storage, and is dropped when it is nonsense", () => {
    const stored = normalizeUi({
      version: UI_STATE_VERSION,
      layout: null,
      wrap: {},
      zoom: {},
      tables: {
        "path:a.csv": { height: 300, widths: { "0": 220 } },
        "path:b.csv": { height: "tall" },
        "path:c.csv": "not a size",
        "path:d.csv": { height: 9_000_000 },
      },
    });
    expect(stored.tables["path:a.csv"]).toEqual({ height: 300, widths: { "0": 220 } });
    // A malformed height leaves nothing behind rather than a broken entry.
    expect(stored.tables["path:b.csv"]).toBeUndefined();
    expect(stored.tables["path:c.csv"]).toBeUndefined();
    // A height that would push the table off the screen is clamped, not kept.
    expect(stored.tables["path:d.csv"]).toEqual({ height: 4000 });
  });

  it("remembers explicit prose fitting settings, including false, and drops invalid values", () => {
    const stored = normalizeUi({ version: UI_STATE_VERSION, tables: {
      a: { fitProse: true }, b: { fitProse: false }, c: { fitProse: "yes" },
    } });
    expect(stored.tables).toEqual({ a: { fitProse: true }, b: { fitProse: false } });
  });

  it("remembers tall fitted rows without cutting off their wrapped text", () => {
    const stored = normalizeUi({
      version: UI_STATE_VERSION,
      layout: null,
      wrap: {},
      zoom: {},
      tables: {
        "path:a.csv": { heights: { "0": 60, "3": 9_000, "4": "tall" } },
        "path:b.csv": { heights: {} },
      },
    });
    // A fitted row must still show all its words after reopening.
    expect(stored.tables["path:a.csv"]).toEqual({ heights: { "0": 60, "3": 9_000 } });
    // Nothing worth keeping is nothing kept, rather than an empty record.
    expect(stored.tables["path:b.csv"]).toBeUndefined();
  });

  it("survives a round trip through withTableLayout", () => {
    const ui = withTableLayout(emptyUi(), "path:a.csv", { height: 240 });
    expect(ui.tables).toEqual({ "path:a.csv": { height: 240 } });
    expect(emptyUi().tables).toEqual({});
  });
});

</storycode:copy>
<a id="f-43efcbce9427-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-43efcbce9427-expression_statement-6">
// Guarantee: docs/guarantees/lineage/a-ribbon-crosses-documents.md.
it("restores the Agent pane in a custom arrangement", () => {
  const layout = { root: { type: "split", id: "split", direction: "row", sizes: [0.5, 0.5], children: [
    onePane.root, { type: "pane", id: "agent", active: 0, tabs: [{ id: "chat", kind: "chat", target: "chat" }] },
  ] }, focus: "agent" };
  expect(panes(normalizeUi(stored(layout)).layout!.root).map(p => p.tabs[0].kind)).toEqual(["document", "chat"]);
});
</storycode:copy>

### Assemble `apps/web/src/lib/uiState.test.ts`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/lib/uiState.test.ts"><storycode:paste select="#f-43efcbce9427-stored"/><storycode:paste select="#f-43efcbce9427-onePane"/><storycode:paste select="#f-43efcbce9427-expression_statement"/><storycode:paste select="#f-43efcbce9427-expression_statement-2"/><storycode:paste select="#f-43efcbce9427-expression_statement-3"/><storycode:paste select="#f-43efcbce9427-expression_statement-4"/><storycode:paste select="#f-43efcbce9427-expression_statement-5"/><storycode:paste select="#f-43efcbce9427-expression_statement-6"/></storycode:file>


</storycode:doc>
