// Provenance ribbons, drawn between panes — and out to files that are not
// open yet.
//
// The old Split view drew these between three fixed columns and carried its
// own file tree to do it, which made the chrome depend on the arrangement.
// Here they are an overlay over whatever panes exist.
//
// Three kinds of band:
//
//   * A **ribbon** runs from a block in the document to the text it produced,
//     in a pane that is showing it.
//   * A ribbon to a file that is open but NOT the active tab terminates ON
//     that tab — a connector under the tab header, where the file actually
//     is, instead of a label floating over whatever the pane is showing.
//   * A ribbon to a file that is not open at all terminates at an "open
//     here" port: a small button the shell lays out in the divider between
//     panes. A relationship you cannot see is a relationship you will not
//     look for, and requiring the file to be open first means never
//     discovering that it exists.
//
// Endpoints are EXACT: a connection starts at the pixel row of the first
// involved line and ends at the pixel row of the last, anchored on the outer
// edge of the line-number rail facing the channel (each pane has numbers on
// both sides — CodeMirror's gutter on the left, editor/RightRail.tsx on the
// right), so the reader can say WHICH lines a ribbon spans by reading the
// numbers beside it. Hovering tints those numbers in the ribbon's colour on
// both rails (editor/lineHighlight.ts).
//
// A connection is painted where it is being ASKED about: by default only
// while the caret is in one of the blocks it joins (lib/ribbonVisibility.ts).
// Every one is still measured, still hover-revealed, and the old always-on
// reading of a whole document is a setting away.
//
// Two renderings of the same geometry: filled Sankey **bands** (default) or
// **braces** — a curly brace per side spanning exactly the involved lines,
// joined nub-to-nub by a thin line (lib/ribbonStyle.ts persists the choice).
//
// All are clickable, and clicking is the navigation: a port band opens the
// file and reveals the text; a tab band brings the tab forward; a full
// ribbon reveals the text in the pane already showing it. Nothing this
// overlay draws sits over pane content — endpoints land on chrome (tabs,
// dividers), and everywhere off a painted band the overlay ignores the
// pointer entirely.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useRef, useState } from "react";
import { BlockType } from "@codemirror/view";
import type { EditorView } from "@codemirror/view";

import { preciseTargetBeneath } from "../lib/ribbonClickThrough";
import { highlightLines } from "../editor/lineHighlight";
import { structureOf } from "../editor/wysiwyg";
import { HoverReveal } from "../lib/hoverReveal";
import {
  anchorEdges,
  atLeast,
  bracePath,
  braceLinkPath,
  braceLinkToEdgePath,
  braceNub,
  clampBand,
  ribbonPath,
  ribbonTerminalPath,
  terminalEdge,
} from "../lib/ribbonGeometry";
import { pickTerminal } from "../lib/terminalPriority";
import type { RibbonStyle } from "../lib/ribbonStyle";
import { caretTouches, type RibbonVisibility } from "../lib/ribbonVisibility";
import {
  deriveRibbons,
  drawnRange,
  fragmentKey,
  groupBySourceBlock,
  RIBBON_PALETTE_SIZE,
  type Ribbon,
} from "../lib/ribbons";
import type { OutputFile } from "../api/types";
import { samePath } from "../lib/paths";
import { attrValue as attr } from "../lib/attrSelector";

/**
 * The document side.
 *
 * The editor is optional on purpose: the bands that point BACK to the
 * document are drawn from a generated pane, and need only the document's
 * text to know what came from where. Requiring the view meant closing the
 * document's pane erased the very stubs that exist to find it again.
 */
export interface RibbonSource {
  view?: EditorView;
  docPath: string;
  /** The document's text — or "" when the document is not loaded, in which
   * case its ribbons can only terminate on chrome (a tab, a tree row, a
   * port), never on its prose. */
  docSource: string;
}

/**
 * Which provenance a shape belongs to. Lineage is the weave's byte-exact
 * derivation and is computed here from file provenance; context and
 * declared arrive as `links` — already derived elsewhere (the session
 * record; the document's own `cites=`) — and are drawn in their own stroke
 * so no reader can mistake one for another.
 */
export type RibbonFamily = "lineage" | "context" | "declared";

/**
 * A connection handed to the overlay ready-made: from a run of lines in a
 * source document to a place — another document, a plain file, a session
 * element, a generated file — named by path and optional lines. The overlay
 * finds the far end wherever it is (a tab, a tree row, a port) and draws.
 */
export interface RibbonLink {
  key: string;
  family: RibbonFamily;
  from: { path: string; lines: [number, number] };
  to: {
    path: string;
    lines?: [number, number];
    kind?: "document" | "generated" | "file";
  };
  title: string;
  anchor?: HTMLElement;
  alwaysVisible?: boolean;
}

/** A generated file: always its content, and its editor when one is open. */
export interface RibbonFile {
  file: OutputFile;
  view?: EditorView;
}

/** Where a click on a band wants to go. */
export type RibbonTarget =
  | { kind: "generated"; path: string; range: [number, number] }
  /**
   * Back to the document, at the BYTES this text came from — the unit the
   * span-selection path uses, and the one provenance is recorded in.
   */
  | { kind: "document"; path: string; span: [number, number] }
  /** A context or declared link's far end: a path, maybe lines in it. */
  | {
      kind: "path";
      path: string;
      lines?: [number, number];
      family: RibbonFamily;
    };

/** One side's involved text, for the hovered line-number tint. */
interface HlSide {
  view: EditorView;
  from: number;
  to: number;
}

interface Shape {
  key: string;
  family: RibbonFamily;
  color: number;
  clamped: boolean;
  /** Where the band ends: real output text, a tab header, a folder-tree row,
   * or a divider port. Everything except "text" is chrome, and
   * chrome-terminated connections are hover-revealed (see `reveal`). */
  ends: "text" | "tab" | "tree" | "port";
  /** Chrome-terminated connections only: the invisible hover region — a slim
   * strip along the source's anchor column spanning the band's y-extent —
   * that reveals the connection. Lines over text all the time would bury the
   * text; lines while you hover the involved block answer the question being
   * asked. */
  reveal?: { x: number; y: number; width: number; height: number };
  /** Band mode: the single filled Sankey path. */
  band?: string;
  /** Brace mode: a brace per anchored side, the thin nub-to-nub (or
   * nub-to-terminal) link, and one generous invisible hit path over all of
   * them — braces are thin, and a 1.75px line is not a click target. */
  brace?: { a: string; b?: string; link: string; hit: string };
  /** A small bar drawn on a tab terminal's attached edge — under a top tab,
   * along the facing side of a vertical one — marking WHICH tab the ribbon
   * means. Chrome-adjacent, never over pane content. */
  connector?: { x: number; y: number; width: number; height: number };
  /** The line ranges to tint while hovered, one entry per anchored editor. */
  hl: HlSide[];
  caret: boolean;
  alwaysVisible?: boolean;
  target: RibbonTarget;
  /** Whitespace-only attribution: drawn only while the pointer is over the
   * involved lines (see `hoverZones`), never by default. */
  whitespaceOnly?: boolean;
  /** Container-relative regions whose hover reveals a whitespace-only
   * connection: the involved lines' extent on EACH anchored pane. Checked
   * from a container mousemove — never pointer-taking elements, because a
   * rect over blank lines would steal the caret clicks that place a cursor
   * there. */
  hoverZones?: { x: number; y: number; width: number; height: number }[];
}

/**
 * A shape as the geometry pass builds it: everything except whether the caret
 * is in it, which is one predicate over the finished list rather than a line
 * repeated at all four places a shape is made.
 */
type Draft = Omit<Shape, "caret">;

/** Thickness of the connector bar on a tab terminal's attached edge. */
const CONNECTOR = 3;

function sameBox(
  a: { x: number; y: number; width: number; height: number } | undefined,
  b: { x: number; y: number; width: number; height: number } | undefined,
): boolean {
  if (!a || !b) return a === b;
  return (
    a.x === b.x && a.y === b.y && a.width === b.width && a.height === b.height
  );
}

/** Whether two measurements would draw the same overlay. */
function sameShapes(a: readonly Shape[], b: readonly Shape[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    const x = a[i];
    const y = b[i];
    if (
      x.key !== y.key ||
      x.color !== y.color ||
      x.clamped !== y.clamped ||
      x.ends !== y.ends ||
      x.band !== y.band ||
      x.whitespaceOnly !== y.whitespaceOnly ||
      x.caret !== y.caret ||
      x.alwaysVisible !== y.alwaysVisible ||
      x.brace?.hit !== y.brace?.hit ||
      !sameBox(x.reveal, y.reveal) ||
      !sameBox(x.connector, y.connector) ||
      x.hl.length !== y.hl.length ||
      x.hl.some(
        (side, j) =>
          side.view !== y.hl[j].view ||
          side.from !== y.hl[j].from ||
          side.to !== y.hl[j].to,
      ) ||
      (x.hoverZones?.length ?? 0) !== (y.hoverZones?.length ?? 0) ||
      (x.hoverZones ?? []).some((zone, j) => !sameBox(zone, y.hoverZones?.[j]))
    ) {
      return false;
    }
  }
  return true;
}

/**
 * The screen element of a terminal — a tab header or a divider port button —
 * found by attribute. Rendered chrome is the source of truth for where it
 * is; the shell tags it, this overlay only measures.
 */
function terminalEl(
  container: HTMLElement,
  selector: string,
): HTMLElement | null {
  const el = container.querySelector(selector);
  return el instanceof HTMLElement ? el : null;
}


interface Terminal {
  rect: DOMRect;
  ends: "tab" | "tree" | "port";
  /** A tab rendered vertically — a side-tree row, a collapsed strip's icon,
   * or a folder-tree row. Ribbons land on its facing vertical edge, not its
   * underside. */
  vertical: boolean;
}

/**
 * A folder-tree row for `target`, but only when the row is actually visible:
 * a row scrolled out of its pane (or inside a collapsed directory, which is
 * simply not rendered) is not a terminal anyone can see, and the connection
 * must fall through to the divider port instead.
 */
function visibleTreeRow(
  container: HTMLElement,
  target: string,
): HTMLElement | null {
  const row = terminalEl(container, `[data-tree-path=${attr(target)}]`);
  if (!row) return null;
  const rect = row.getBoundingClientRect();
  const clip = row.closest(".shell-pane__body")?.getBoundingClientRect();
  if (clip && (rect.bottom <= clip.top || rect.top >= clip.bottom)) return null;
  return row;
}

/**
 * Where a connection to `target` should terminate when its text is not on
 * screen. Priority is lib/terminalPriority.ts's: the file's tab when a pane
 * holds it, else a visible folder-tree row naming its path, else the "open
 * here" divider port. (The "text" case never reaches here — a visible
 * editor is handled by the caller before terminals are consulted.)
 */
function findTerminal(
  container: HTMLElement,
  kind: "document" | "generated" | "file",
  target: string,
): Terminal | null {
  const tab = terminalEl(
    container,
    `[data-shell-tab-kind=${attr(kind)}][data-shell-tab-target=${attr(target)}]`,
  );
  const row = visibleTreeRow(container, target);
  const port = terminalEl(
    container,
    `[data-ribbon-port=${attr(`${kind}:${target}`)}]`,
  );
  switch (pickTerminal({ tab: !!tab, tree: !!row, port: !!port })) {
    case "tab": {
      const rect = tab!.getBoundingClientRect();
      // The shell marks side tabs and strip icons explicitly; the geometric
      // test catches anything else that renders a tab taller than it is wide.
      const vertical =
        tab!.hasAttribute("data-shell-tab-vertical") ||
        rect.height > rect.width;
      return { rect, ends: "tab", vertical };
    }
    case "tree":
      // A tree row is a wide, short row: connections land on its near
      // vertical edge, exactly the way a side tab takes them.
      return {
        rect: row!.getBoundingClientRect(),
        ends: "tree",
        vertical: true,
      };
    case "port":
      return {
        rect: port!.getBoundingClientRect(),
        ends: "port",
        vertical: false,
      };
    default:
      return null;
  }
}

/**
 * How a band or brace-link reaches a terminal, and the connector bar that
 * marks the attachment.
 *
 * A top tab takes the ribbon on its UNDERSIDE (the pane is below the strip).
 * A vertical terminal — a side-tree tab, a strip icon, a folder-tree row —
 * takes it on whichever vertical edge faces the source, band-to-band like a
 * pane would: it is taller than wide (or explicitly marked), so its side is
 * the honest edge. A port keeps the old rule: whichever horizontal edge
 * faces the source.
 *
 * `dir` is the source nub's outward direction (+1 rightward): brace links
 * must LEAVE the nub along it — C1 at the brace tip, no corner — which is
 * why it rides along even though the band form never needs it.
 */
function attachTerminal(
  terminal: Terminal,
  box: DOMRect,
  x0: number,
  from: { yTop: number; yBot: number },
  braces: boolean,
  nub: { x: number; y: number } | null,
  dir: 1 | -1,
): { band?: string; link?: string; connector?: Shape["connector"] } {
  const tLeft = terminal.rect.left - box.left;
  const tRight = terminal.rect.right - box.left;
  const tTop = terminal.rect.top - box.top;
  const tBot = terminal.rect.bottom - box.top;

  if (terminal.vertical) {
    const edgeX = x0 <= (tLeft + tRight) / 2 ? tLeft : tRight;
    // The link arrives along the terminal's own outward direction too: out
    // of its facing edge, toward the channel the source sits across.
    const arrive: 1 | -1 = edgeX === tLeft ? -1 : 1;
    return {
      band: braces
        ? undefined
        : ribbonPath(x0, from.yTop, from.yBot, edgeX, tTop, tBot),
      link: nub
        ? braceLinkPath(nub.x, nub.y, dir, edgeX, (tTop + tBot) / 2, arrive)
        : undefined,
      connector: {
        x: edgeX === tLeft ? tLeft : tRight - CONNECTOR,
        y: tTop,
        width: CONNECTOR,
        height: tBot - tTop,
      },
    };
  }

  const edgeY =
    terminal.ends === "tab"
      ? tBot
      : terminalEdge(from.yTop, from.yBot, tTop, tBot) === "top"
        ? tTop
        : tBot;
  return {
    band: braces
      ? undefined
      : ribbonTerminalPath(x0, from.yTop, from.yBot, tLeft, tRight, edgeY),
    link: nub
      ? braceLinkToEdgePath(nub.x, nub.y, dir, (tLeft + tRight) / 2, edgeY)
      : undefined,
    connector:
      terminal.ends === "tab"
        ? {
            x: tLeft,
            y: tBot - CONNECTOR,
            width: tRight - tLeft,
            height: CONNECTOR,
          }
        : undefined,
  };
}

/**
 * The invisible hover region that reveals a chrome-terminated connection: a
 * slim strip hugging the source's anchor column over the band's y-extent —
 * the involved lines' rail edge, NOT the terminal — grown to a minimum
 * height so a one-line source is still hoverable. It leans toward the
 * channel (the nub side), where the revealed shapes begin.
 */
function revealStrip(
  x0: number,
  dir: 1 | -1,
  from: { yTop: number; yBot: number },
): NonNullable<Shape["reveal"]> {
  const { yTop, yBot } = atLeast(from.yTop, from.yBot, 10);
  const width = 18;
  return {
    x: dir === 1 ? x0 - 4 : x0 - width + 4,
    y: yTop,
    width,
    height: yBot - yTop,
  };
}

/**
 * A pane's full horizontal extent INCLUDING both line-number rails: from the
 * outer edge of CodeMirror's left gutter (the scroller's left edge) to the
 * outer edge of the right rail the editor wrapper mounts beside it. This is
 * what ribbons anchor on — the numbers, not the text, are the ribbon's
 * x-axis vocabulary.
 */
function paneEdges(view: EditorView): {
  left: number;
  right: number;
  /** Width of the left (CodeMirror gutter) rail: outer edge to the text-side
   * boundary. Brace horns reach this far back in, wrapping the numbers. */
  leftRailW: number;
  /** Width of the right rail (0 when the pane has none). */
  rightRailW: number;
} {
  const scroller = view.scrollDOM.getBoundingClientRect();
  const content = view.contentDOM.getBoundingClientRect();
  const wrapper = view.dom.closest(".with-right-rail");
  // A pane can carry TWO rails on its right: the line numbers, and outside
  // them the action rail of card icons. A ribbon must reach the pane's real
  // outer edge — stopping at the numbers would run it under the icon column,
  // which paints a background over it.
  const outer = [".cm-right-rail", ".cm-card-rail"]
    .map((selector) => wrapper?.querySelector(`:scope > ${selector}`))
    .filter((el): el is HTMLElement => el instanceof HTMLElement)
    .reduce(
      (edge, el) => Math.max(edge, el.getBoundingClientRect().right),
      scroller.right,
    );
  return {
    left: scroller.left,
    right: outer,
    leftRailW: Math.max(0, content.left - scroller.left),
    // Everything past the text is rail, so the horn wraps both of them and
    // the numbers stay inside the brace exactly as before.
    rightRailW: Math.max(0, outer - scroller.right),
  };
}

export function RibbonOverlay({
  container,
  sources,
  files,
  links = [],
  layers,
  ribbonStyle = "bands",
  visibility = "caret",
  onNavigate,
}: {
  container: HTMLElement | null;
  /**
   * Every document the files' provenance reaches: the focused document
   * first, then any document whose bytes the files carry — a meeting note
   * two hops upstream that a message quotes. A source with a `view` is on
   * screen and anchors pane-to-pane; one without gets stubs reaching back to
   * its tab, tree row, or port. Lineage crosses documents, so the overlay
   * does too.
   */
  sources: readonly RibbonSource[];
  files: readonly RibbonFile[];
  /** Context and declared connections, ready-made (see RibbonLink). */
  links?: readonly RibbonLink[];
  /** Which families to draw; absent means all. */
  layers?: ReadonlySet<RibbonFamily>;
  /** Bands (filled Sankey) or braces (curly braces joined by a thin line). */
  ribbonStyle?: RibbonStyle;
  /**
   * Whether a connection is painted only while the caret is in one of its
   * blocks (the default) or all the time (lib/ribbonVisibility.ts).
   */
  visibility?: RibbonVisibility;
  onNavigate?: (target: RibbonTarget) => void;
}) {
  const [shapes, setShapes] = useState<Shape[]>([]);
  const frame = useRef<number | null>(null);
  // Which chrome-terminated connections are revealed right now. Tab, tree,
  // and port shapes draw only while hovered (plus a grace period so the
  // pointer can travel the line to click its terminal); full pane-to-pane
  // shapes between visible text stay always-on.
  const [revealedKeys, setRevealedKeys] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const reveal = useRef<HoverReveal | null>(null);
  if (reveal.current === null)
    reveal.current = new HoverReveal(setRevealedKeys);
  useEffect(() => () => reveal.current?.dispose(), []);

  /**
   * Tint the tree row a revealed ribbon is pointing at.
   *
   * A tree row is wide and its file name is left-aligned, so the ribbon lands
   * on the row's near vertical EDGE — inches from the name it means. The line
   * says "one of these rows"; without this it does not say which. Marking the
   * row itself is the only way to close that gap without moving either the
   * name or the line.
   *
   * Written straight onto the DOM rather than lifted into the tree pane's
   * state: the tree is a different component in a different subtree, this
   * changes on every hover, and a class toggle is exactly what a class toggle
   * looks like.
   */
  useEffect(() => {
    if (!container) return;
    const marked: HTMLElement[] = [];
    for (const shape of shapes) {
      // Both kinds of target have a path to find a row by: a generated file,
      // or a document that is not on screen — the meeting note a message
      // quoted, say — whose tree row is where its ribbons end.
      if (shape.ends !== "tree") continue;
      if (!revealedKeys.has(shape.key)) continue;
      const row = terminalEl(
        container,
        `[data-tree-path=${attr(shape.target.path)}]`,
      );
      if (!row) continue;
      row.classList.add("folder-tree__row--aimed");
      // The colour is the ribbon's own, so several revealed at once stay
      // told apart — the row and the line that reaches it match.
      row.dataset.ribbonColor = String(shape.color);
      marked.push(row);
    }
    return () => {
      for (const row of marked) {
        row.classList.remove("folder-tree__row--aimed");
        delete row.dataset.ribbonColor;
      }
    };
  }, [container, shapes, revealedKeys]);

  // Whitespace-only connections reveal from the pointer's POSITION over
  // their involved lines, tested here on container mousemove rather than
  // with pointer-taking elements — a hit rect over a blank line would steal
  // the click that places a caret there.
  useEffect(() => {
    if (!container) return;
    const zoned = shapes.filter((s) => s.hoverZones && s.hoverZones.length > 0);
    if (zoned.length === 0) return;
    let frame: number | null = null;
    let last: MouseEvent | null = null;
    const inside = new Set<string>();
    const test = () => {
      frame = null;
      if (!last) return;
      const box = container.getBoundingClientRect();
      const x = last.clientX - box.left;
      const y = last.clientY - box.top;
      for (const shape of zoned) {
        const hit = (shape.hoverZones ?? []).some(
          (z) =>
            x >= z.x && x <= z.x + z.width && y >= z.y && y <= z.y + z.height,
        );
        if (hit && !inside.has(shape.key)) {
          inside.add(shape.key);
          reveal.current?.enter(shape.key);
        } else if (!hit && inside.has(shape.key)) {
          inside.delete(shape.key);
          reveal.current?.leave(shape.key);
        }
      }
    };
    const move = (e: MouseEvent) => {
      last = e;
      if (frame === null) frame = requestAnimationFrame(test);
    };
    const out = () => {
      for (const key of inside) reveal.current?.leave(key);
      inside.clear();
    };
    container.addEventListener("mousemove", move, { passive: true });
    container.addEventListener("mouseleave", out);
    return () => {
      container.removeEventListener("mousemove", move);
      container.removeEventListener("mouseleave", out);
      if (frame !== null) cancelAnimationFrame(frame);
      out();
    };
  }, [container, shapes]);
  // The shape currently under the pointer, for the line-number tint. A ref,
  // not state: hover changes tint OTHER components' DOM (via dispatches at
  // the editors); this overlay itself renders the same either way.
  const hovered = useRef<Shape | null>(null);
  // The editor that had focus most recently, so the caret still has a home
  // while the pointer is off in the chrome.
  const lastFocused = useRef<EditorView | null>(null);

  const setHovered = useCallback((shape: Shape | null) => {
    const previous = hovered.current;
    if (previous)
      for (const side of previous.hl) highlightLines(side.view, null);
    hovered.current = shape;
    if (shape) {
      for (const side of shape.hl) {
        highlightLines(side.view, {
          from: side.from,
          to: side.to,
          color: shape.color,
        });
      }
    }
  }, []);

  // A pane that closes mid-hover would strand its tint; clear on unmount.
  useEffect(() => () => setHovered(null), [setHovered]);

  const measure = useCallback(() => {
    const on = (family: RibbonFamily) => !layers || layers.has(family);
    const anyLineage = on("lineage") && files.length > 0;
    const anyLinks = links.some((l) => on(l.family));
    if (!container || sources.length === 0 || (!anyLineage && !anyLinks)) {
      setShapes((current) => (current.length === 0 ? current : []));
      return;
    }
    const box = container.getBoundingClientRect();
    const braces = ribbonStyle === "braces";
    const out: Draft[] = [];
    const colours = new Map<string, number>();
    const colourFor = (fragment: string) => {
      let assigned = colours.get(fragment);
      if (assigned === undefined) {
        assigned = colours.size % RIBBON_PALETTE_SIZE;
        colours.set(fragment, assigned);
      }
      return assigned;
    };

    for (const source of anyLineage ? sources : []) {
      // A fragment is identified by its span IN ITS DOCUMENT: the same byte
      // range in two documents is two fragments, two colours.
      const fragmentIn = (key: string) => `${source.docPath}\u0000${key}`;
      const documentView = source.view ?? null;
      const documentVisible = documentView !== null;
      // The channel is the space between the two RAILS: connections anchor on
      // the outer edge of the line-number rail facing the gap, so a ribbon
      // visibly spans from number column to number column.
      const left = documentView?.scrollDOM.getBoundingClientRect() ?? null;
      const docEdges = documentView ? paneEdges(documentView) : null;
      const structure = documentView ? structureOf(documentView.state) : null;
      const sourceLength = documentView?.state.doc.length ?? 0;

      for (const entry of files) {
        const ribbons = deriveRibbons(
          entry.file,
          source.docPath,
          source.docSource,
        );
        if (ribbons.length === 0) continue;

        if (entry.view && documentView && left && docEdges && structure) {
          const right = entry.view.scrollDOM.getBoundingClientRect();
          const outEdges = paneEdges(entry.view);
          // Panes can be in any order: a generated file to the LEFT of its
          // document is an arrangement someone is allowed to build. Whichever
          // pane is left anchors on its RIGHT rail's outer edge, the other on
          // its LEFT gutter's outer edge.
          const { forward, x0, x1 } = anchorEdges(
            {
              left: docEdges.left - box.left,
              right: docEdges.right - box.left,
            },
            {
              left: outEdges.left - box.left,
              right: outEdges.right - box.left,
            },
          );
          const targetLength = entry.view.state.doc.length;

          for (const ribbon of ribbons) {
            const fullRange = sourceRange(structure, ribbon, sourceLength);
            // Drawn extents cover only lines the span VISIBLY owns: a newline
            // or indentation on a line whose characters belong to another
            // span must not inflate a line-granular brace into claiming that
            // line (lib/ribbons.ts::drawnRange). A side owning no visible
            // line keeps its full extent but joins the hover-only shapes.
            const trimmedSrc = drawnRange(
              source.docSource,
              fullRange[0],
              fullRange[1],
            );
            const range = trimmedSrc ?? fullRange;
            const band = bandBetween(documentView, range[0], range[1]);
            const fullOutFrom = Math.min(ribbon.outputRange[0], targetLength);
            const fullOutTo = Math.min(ribbon.outputRange[1], targetLength);
            const trimmedOut = drawnRange(
              entry.file.content,
              fullOutFrom,
              fullOutTo,
            );
            const [outFrom, outTo] = trimmedOut ?? [fullOutFrom, fullOutTo];
            const hoverOnlySide = trimmedSrc === null || trimmedOut === null;
            const to = bandBetween(entry.view, outFrom, outTo);
            if (!band || !to) continue;
            // EXACT: the band's vertical extent is [top of first involved
            // line, bottom of last], clamped to the pane — no minimum-height
            // inflation, because a line is already a readable, clickable row.
            const from = clampBand(
              band[0] - box.top,
              band[1] - box.top,
              left.top - box.top,
              left.bottom - box.top,
            );
            const into = clampBand(
              to[0] - box.top,
              to[1] - box.top,
              right.top - box.top,
              right.bottom - box.top,
            );
            const hl: HlSide[] = [
              { view: documentView, from: range[0], to: range[1] },
              { view: entry.view, from: outFrom, to: outTo },
            ];
            const shape: Draft = {
              key: `${entry.file.path}:${ribbon.key}`,

              family: "lineage",
              color: colourFor(fragmentIn(fragmentKey(ribbon))),
              clamped: from.clamped || into.clamped,
              ends: "text",
              hl,
              target: {
                kind: "generated",
                path: entry.file.path,
                range: ribbon.outputRange,
              },
            };
            if (ribbon.whitespaceOnly || hoverOnlySide) {
              const pad = 4; // a blank line's band is a few pixels tall
              shape.whitespaceOnly = true;
              shape.hoverZones = [
                {
                  x: docEdges.left - box.left,
                  y: from.yTop - pad,
                  width: docEdges.right - docEdges.left,
                  height: from.yBot - from.yTop + pad * 2,
                },
                {
                  x: outEdges.left - box.left,
                  y: into.yTop - pad,
                  width: outEdges.right - outEdges.left,
                  height: into.yBot - into.yTop + pad * 2,
                },
              ];
            }
            if (braces) {
              // Orientation is a property of the SIDE, not the link: a brace
              // on a file's right edge is a closing `}` (nub bulging right,
              // away from the text), on its left edge an opening `{`. The
              // two braces embrace their own file's text: { text } .
              const dir: 1 | -1 = forward ? 1 : -1;
              const back: 1 | -1 = dir === 1 ? -1 : 1;
              // Horns reach back across the anchored rail so the brace wraps
              // the line numbers of the included range; a clamped end stays
              // OPEN — no horn at all — so the spine visibly runs off the
              // viewport edge where the range continues.
              const hornA = forward ? docEdges.rightRailW : docEdges.leftRailW;
              const hornB = forward ? outEdges.leftRailW : outEdges.rightRailW;
              const a = bracePath(
                x0,
                from.yTop,
                from.yBot,
                dir,
                undefined,
                { top: from.clampedTop, bottom: from.clampedBottom },
                hornA,
              );
              const b = bracePath(
                x1,
                into.yTop,
                into.yBot,
                back,
                undefined,
                { top: into.clampedTop, bottom: into.clampedBottom },
                hornB,
              );
              const nubA = braceNub(x0, from.yTop, from.yBot, dir);
              const nubB = braceNub(x1, into.yTop, into.yBot, back);
              // Each end leaves its nub along that nub's own outward
              // direction: C1 at both tips, so the line reads as growing out
              // of the braces rather than cornering off them.
              const link = braceLinkPath(
                nubA.x,
                nubA.y,
                dir,
                nubB.x,
                nubB.y,
                back,
              );
              shape.brace = { a, b, link, hit: `${a} ${link} ${b}` };
            } else {
              shape.band = ribbonPath(
                x0,
                from.yTop,
                from.yBot,
                x1,
                into.yTop,
                into.yBot,
              );
            }
            out.push(shape);
          }
          continue;
        }

        // No editor for this file: the ribbon still says where its text went,
        // it just terminates on chrome instead of floating over content — the
        // file's own tab when a pane holds it inactive, or the "open here"
        // port the shell lays out in the divider when nothing does. One band
        // per source block rather than per fragment: one file that a block
        // feeds is one statement, not fifty.
        if (!documentView || !left || !docEdges || !structure) continue;
        const terminal = findTerminal(container, "generated", entry.file.path);
        if (!terminal) continue;
        const tMid = (terminal.rect.left + terminal.rect.right) / 2 - box.left;
        // Leave from whichever rail of the document faces the terminal: a tab
        // can be in a pane on either side.
        const docL = docEdges.left - box.left;
        const docR = docEdges.right - box.left;
        const x0 = tMid >= (docL + docR) / 2 ? docR : docL;

        for (const [key, group] of groupBySourceBlock(ribbons)) {
          const fullRange = sourceRange(structure, group.ribbon, sourceLength);
          const range =
            drawnRange(source.docSource, fullRange[0], fullRange[1]) ??
            fullRange;
          const band = bandBetween(documentView, range[0], range[1]);
          if (!band) continue;
          const from = clampBand(
            band[0] - box.top,
            band[1] - box.top,
            left.top - box.top,
            left.bottom - box.top,
          );
          // `+1` iff x0 is the pane's RIGHT edge (the same predicate that
          // chose x0): the brace always bulges away from its file's text —
          // left edge = `{`, right edge = `}` — never toward it, wherever
          // the tab or port happens to sit.
          const dir: 1 | -1 = x0 === docR ? 1 : -1;
          // Horn reaches back across whichever rail the brace anchors on.
          const horn = x0 === docR ? docEdges.rightRailW : docEdges.leftRailW;
          const nub = braces ? braceNub(x0, from.yTop, from.yBot, dir) : null;
          // A top tab is attached from below; a vertical terminal (side tree,
          // collapsed strip, folder-tree row) on its facing edge; a port on
          // whichever horizontal edge faces the source. attachTerminal decides.
          const attach = attachTerminal(
            terminal,
            box,
            x0,
            from,
            braces,
            nub,
            dir,
          );
          const shape: Draft = {
            key: `${source.docPath}:${entry.file.path}:${terminal.ends}:${key}`,

            family: "lineage",
            color: colourFor(fragmentIn(key)),
            clamped: from.clamped,
            ends: terminal.ends,
            reveal: revealStrip(x0, dir, from),
            connector: attach.connector,
            hl: [{ view: documentView, from: range[0], to: range[1] }],
            target: {
              kind: "generated",
              path: entry.file.path,
              range: group.ribbon.outputRange,
            },
          };
          if (braces && attach.link) {
            const a = bracePath(
              x0,
              from.yTop,
              from.yBot,
              dir,
              undefined,
              { top: from.clampedTop, bottom: from.clampedBottom },
              horn,
            );
            shape.brace = { a, link: attach.link, hit: `${a} ${attach.link}` };
          } else {
            shape.band = attach.band;
          }
          out.push(shape);
        }
      }

      // The other direction: a generated pane whose document is not on screen
      // gets bands of its own, reaching back the way they came — to the
      // document's inactive tab when a pane holds it, or to its divider port
      // when none does. Provenance is symmetrical and so is the question —
      // "where did this come from" is asked from the generated side at least
      // as often.
      if (!documentVisible) {
        const terminal = findTerminal(container, "document", source.docPath);
        for (const entry of files) {
          if (!entry.view || !terminal) continue;
          const pane = entry.view.scrollDOM.getBoundingClientRect();
          const genEdges = paneEdges(entry.view);
          const tMid =
            (terminal.rect.left + terminal.rect.right) / 2 - box.left;
          const genL = genEdges.left - box.left;
          const genR = genEdges.right - box.left;
          const x0 = tMid >= (genL + genR) / 2 ? genR : genL;
          const byBlock = groupBySourceBlock(
            deriveRibbons(entry.file, source.docPath, source.docSource),
          );
          for (const [key, group] of byBlock) {
            const targetLength = entry.view.state.doc.length;
            const fullOutFrom = Math.min(
              group.ribbon.outputRange[0],
              targetLength,
            );
            const fullOutTo = Math.min(
              group.ribbon.outputRange[1],
              targetLength,
            );
            const [outFrom, outTo] = drawnRange(
              entry.file.content,
              fullOutFrom,
              fullOutTo,
            ) ?? [fullOutFrom, fullOutTo];
            const band = bandBetween(entry.view, outFrom, outTo);
            if (!band) continue;
            const from = clampBand(
              band[0] - box.top,
              band[1] - box.top,
              pane.top - box.top,
              pane.bottom - box.top,
            );
            // Same rule as the document side: orientation follows the
            // anchored edge, bulging away from this file's text.
            const dir: 1 | -1 = x0 === genR ? 1 : -1;
            // Horn reaches back across whichever rail the brace anchors on.
            const horn = x0 === genR ? genEdges.rightRailW : genEdges.leftRailW;
            const nub = braces ? braceNub(x0, from.yTop, from.yBot, dir) : null;
            const attach = attachTerminal(
              terminal,
              box,
              x0,
              from,
              braces,
              nub,
              dir,
            );
            const shape: Draft = {
              key: `${source.docPath}:${entry.file.path}:back:${key}`,

              family: "lineage",
              color: colourFor(fragmentIn(key)),
              clamped: from.clamped,
              ends: terminal.ends,
              reveal: revealStrip(x0, dir, from),
              connector: attach.connector,
              hl: [{ view: entry.view, from: outFrom, to: outTo }],
              target: {
                kind: "document",
                path: source.docPath,
                span: group.ribbon.sourceByteSpan,
              },
            };
            if (braces && attach.link) {
              const a = bracePath(
                x0,
                from.yTop,
                from.yBot,
                dir,
                undefined,
                { top: from.clampedTop, bottom: from.clampedBottom },
                horn,
              );
              shape.brace = {
                a,
                link: attach.link,
                hit: `${a} ${attach.link}`,
              };
            } else {
              shape.band = attach.band;
            }
            out.push(shape);
          }
        }
      }
    }

    for (const link of links) {
      if (!on(link.family)) continue;
      const source = sources.find(
        (s) => s.view && samePath(s.docPath, link.from.path),
      );
      const view = source?.view;
      if (!source || !view) continue;
      const kinds: ("document" | "generated" | "file")[] = link.to.kind
        ? [link.to.kind]
        : ["document", "generated", "file"];
      let terminal: Terminal | null = null;
      for (const kind of kinds) {
        terminal = findTerminal(container, kind, link.to.path);
        if (terminal) break;
      }
      if (!terminal) continue;
      const total = view.state.doc.lines;
      const a = Math.min(Math.max(link.from.lines[0], 1), total);
      const b = Math.min(Math.max(link.from.lines[1], a), total);
      const from = view.state.doc.line(a).from;
      const to = view.state.doc.line(b).to;
      const rect = link.anchor?.getBoundingClientRect();
      const band = rect ? [rect.top, rect.bottom] : bandBetween(view, from, to);
      if (!band || band[1] <= band[0]) continue;
      const pane = view.scrollDOM.getBoundingClientRect();
      const edges = paneEdges(view);
      const tMid = (terminal.rect.left + terminal.rect.right) / 2 - box.left;
      const docL = edges.left - box.left;
      const docR = edges.right - box.left;
      const x0 = tMid >= (docL + docR) / 2 ? docR : docL;
      const clamped = clampBand(
        band[0] - box.top,
        band[1] - box.top,
        pane.top - box.top,
        pane.bottom - box.top,
      );
      const dir: 1 | -1 = x0 === docR ? 1 : -1;
      const horn = x0 === docR ? edges.rightRailW : edges.leftRailW;
      const nub = braces ? braceNub(x0, clamped.yTop, clamped.yBot, dir) : null;
      const attach = attachTerminal(
        terminal,
        box,
        x0,
        clamped,
        braces,
        nub,
        dir,
      );
      const shape: Draft = {
        key: `${link.family}:${link.key}:${terminal.ends}`,
        alwaysVisible: link.alwaysVisible,
        family: link.family,
        color: link.family === "context" ? 0 : 1,
        clamped: clamped.clamped,
        ends: terminal.ends,
        reveal: revealStrip(x0, dir, clamped),
        connector: attach.connector,
        hl: [{ view, from, to }],
        target: {
          kind: "path",
          path: link.to.path,
          lines: link.to.lines,
          family: link.family,
        },
      };
      if (braces && attach.link) {
        const brace = bracePath(
          x0,
          clamped.yTop,
          clamped.yBot,
          dir,
          undefined,
          { top: clamped.clampedTop, bottom: clamped.clampedBottom },
          horn,
        );
        shape.brace = {
          a: brace,
          link: attach.link,
          hit: `${brace} ${attach.link}`,
        };
      } else {
        shape.band = attach.band;
      }
      out.push(shape);
    }

    // Whose caret counts: the editor that has focus, and when focus has gone
    // to something that is not an editor — a menu, the tree, the toolbar —
    // the last editor that had it. Ribbons that vanish because you reached
    // for a button are ribbons you cannot click.
    const views = [...sources.map((s) => s.view), ...files.map((f) => f.view)];
    const focusedNow = views.find((view) => view?.hasFocus) ?? null;
    if (focusedNow) lastFocused.current = focusedNow;
    const caretIn = lastFocused.current;
    const shaped: Shape[] = out.map((shape) => ({
      ...shape,
      caret:
        caretIn !== null &&
        shape.hl.some(
          (side) =>
            side.view === caretIn &&
            caretTouches(
              { from: side.from, to: side.to },
              caretIn.state.selection.main,
            ),
        ),
    }));

    // Measured twice a second whether or not anything moved; commit only a
    // real change, or the SVG re-renders on every tick.
    setShapes((current) => (sameShapes(current, shaped) ? current : shaped));
  }, [container, sources, files, links, layers, ribbonStyle]);

  // Measurement follows the things that move: scrolling either pane, editing
  // either buffer, and the window changing shape. Throttled to a frame,
  // because all three can happen at once and the answer is the same.
  useEffect(() => {
    const schedule = () => {
      if (frame.current !== null) return;
      frame.current = requestAnimationFrame(() => {
        frame.current = null;
        measure();
      });
    };
    schedule();

    const views = [
      ...sources.map((s) => s.view),
      ...files.map((f) => f.view),
    ].filter(Boolean) as EditorView[];
    const scrollers = views.map((view) => view.scrollDOM);
    for (const scroller of scrollers) {
      scroller.addEventListener("scroll", schedule, { passive: true });
    }
    window.addEventListener("resize", schedule);
    // The caret is a thing that moves, so it belongs on this list. One
    // document-level listener catches every way of moving it in every pane —
    // typing, arrows, a click, a drag — where per-view key handlers would
    // catch some of them.
    document.addEventListener("selectionchange", schedule);
    const observer = new ResizeObserver(schedule);
    if (container) observer.observe(container);
    const timer = window.setInterval(schedule, 500);

    return () => {
      for (const scroller of scrollers)
        scroller.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      document.removeEventListener("selectionchange", schedule);
      observer.disconnect();
      window.clearInterval(timer);
      if (frame.current !== null) cancelAnimationFrame(frame.current);
      frame.current = null;
    };
  }, [measure, container, sources, files, links]);

  if (shapes.length === 0) return null;
  return (
    <svg className="ribbon-layer">
      {shapes.map((shape) => {
        const title = (
          <title>
            {shape.target.kind === "path"
              ? linkTitle(shape.target, links)
              : shape.target.kind === "document"
                ? shape.ends === "text"
                  ? "Show the prose this came from"
                  : `Open ${shape.target.path} — this text came from it`
                : shape.ends === "tab"
                  ? `Show ${shape.target.path} — its tab is right here`
                  : shape.ends === "tree"
                    ? `Open ${shape.target.path} — this row in the tree`
                    : shape.ends === "port"
                      ? `Open ${shape.target.path} at what this block produced`
                      : `Show this in ${shape.target.path}`}
          </title>
        );
        const modifier = `${shape.clamped ? " ribbon-clamped" : ""}${
          shape.ends === "text" ? "" : ` ribbon-to-${shape.ends}`
        }${shape.family === "lineage" ? "" : ` ribbon-family-${shape.family}`}`;
        // Chrome-terminated connections draw only while revealed: their
        // lines would otherwise sit over pane text all the time. The hover
        // strip below is what reveals them; the revealed shapes themselves
        // keep the reveal alive so the pointer can travel to the terminal.
        // Whitespace-only attribution is hover-revealed too (via the
        // container mousemove over its involved lines): a band whose whole
        // meaning is "this blank region came from that blank region" is
        // noise until someone is working exactly there.
        const chrome = shape.ends !== "text";
        const hoverOnly = chrome || shape.whitespaceOnly === true;
        const shown = shape.alwaysVisible || (
          visibility === "caret"
            ? shape.caret || revealedKeys.has(shape.key)
            : !hoverOnly || revealedKeys.has(shape.key));
        return (
          <g
            key={shape.key}
            className={`ribbon-group${hoverOnly && shown ? " ribbon-group--revealed" : ""}${
              shape.family === "lineage" ? "" : ` ribbon-group--${shape.family}`
            }`}
            onMouseEnter={() => {
              setHovered(shape);
              if (hoverOnly) reveal.current?.enter(shape.key);
            }}
            onMouseLeave={() => {
              setHovered(null);
              if (hoverOnly) reveal.current?.leave(shape.key);
            }}
            onClick={(e) => {
              // Precise controls under the band win the click: the ribbon can
              // be hit anywhere along its length, the button beneath it only
              // exactly where it is. See lib/ribbonClickThrough.ts.
              const layer = (e.currentTarget as SVGGElement).ownerSVGElement;
              const beneath = preciseTargetBeneath(e.clientX, e.clientY, layer);
              if (beneath) {
                beneath.click();
                return;
              }
              onNavigate?.(shape.target);
            }}
          >
            {chrome && shape.reveal && (
              // The always-present hover region over the SOURCE band's rail
              // edge: invisible, pointer-events on this strip only, so the
              // hidden connection can be found without lines over text.
              <rect
                className="ribbon-reveal-hit"
                x={shape.reveal.x}
                y={shape.reveal.y}
                width={shape.reveal.width}
                height={shape.reveal.height}
              >
                {title}
              </rect>
            )}
            {shown && shape.brace ? (
              <>
                {/* The forgiving click/hover target: a wide invisible stroke
                    over the braces and the link, because the drawn strokes
                    are two pixels of curve nobody could aim at. */}
                <path d={shape.brace.hit} className="ribbon-brace-hit">
                  {title}
                </path>
                <path
                  d={shape.brace.a}
                  className={`ribbon-brace ribbon-bc${shape.color}${modifier}`}
                />
                {shape.brace.b && (
                  <path
                    d={shape.brace.b}
                    className={`ribbon-brace ribbon-bc${shape.color}${modifier}`}
                  />
                )}
                <path
                  d={shape.brace.link}
                  className={`ribbon-brace ribbon-brace-link ribbon-bc${shape.color}${modifier}`}
                />
              </>
            ) : shown ? (
              <path
                d={shape.band}
                className={`ribbon ribbon-c${shape.color}${modifier}`}
              >
                {title}
              </path>
            ) : null}
            {shown && shape.connector && (
              <rect
                className={`ribbon-connector ribbon-connector-c${shape.color}`}
                x={shape.connector.x}
                y={shape.connector.y}
                width={shape.connector.width}
                height={shape.connector.height}
                rx={1.5}
              />
            )}
          </g>
        );
      })}
    </svg>
  );
}

/**
 * The char range a ribbon's source claims on screen: the whole enclosing
 * block, not the fragment. A band that covers the copy block says "this
 * block became that text", which is the claim; a band over three characters
 * of it says nothing anyone can read at a glance.
 */
function sourceRange(
  structure: ReturnType<typeof structureOf>,
  ribbon: Ribbon,
  length: number,
): [number, number] {
  let from = Math.min(ribbon.sourceSpan[0], length);
  let to = Math.min(ribbon.sourceSpan[1], length);
  for (const block of structure.blocks) {
    if (
      (block.name === "copy" ||
        block.name === "cut" ||
        block.name === "file") &&
      ribbon.sourceSpan[0] >= block.from &&
      ribbon.sourceSpan[1] <= block.to
    ) {
      from = Math.min(block.from, length);
      to = Math.min(block.to, length);
      break;
    }
  }
  return [from, to];
}

/**
 * Screen extent of a range, including one scrolled out of sight.
 *
 * `coordsAtPos` answers for rendered text only — everything outside the
 * viewport returns null, and a ribbon that cannot be measured is a ribbon
 * that is not drawn: six relationships showed one. The height map knows where
 * every line is whether or not it is on screen, and the content box moves
 * with the scroll, so the two together answer for any position; the caller
 * clamps the result to the visible pane. The answer is EXACT: the top of the
 * first line holding `from` to the bottom of the last line holding `to`.
 */
function bandBetween(
  view: EditorView,
  from: number,
  to: number,
): [number, number] | null {
  // Blocks are document-relative; the screen conversion is documentTop —
  // NOT contentDOM's rect, which sits a content-padding away, and by a
  // different amount per editor. The rails convert through documentTop
  // (editor/RightRail.tsx), and a band converted through anything else
  // lands its horns a pane-specific offset away from the numbers.
  const base = view.documentTop;
  const first = textOnly(view.lineBlockAt(Math.max(0, from)));
  const last = textOnly(view.lineBlockAt(Math.max(0, to)));
  const start = { top: base + first.top, bottom: base + first.bottom };
  const end = { top: base + last.top, bottom: base + last.bottom };
  return [Math.min(start.top, end.top), Math.max(start.bottom, end.bottom)];
}

/**
 * The TEXT extent of a line block. A line carrying block widgets (a copy
 * chip, a cell panel) reports a composite block spanning the widgets too;
 * a band measured against that lands its horns offset from the number rows
 * — braces visibly crossing OUT the very numbers they mean to include. The
 * rails already align numbers to the text portion (lib/rightRail.ts
 * textExtent); the bands must measure the same thing, so a brace's edges
 * sit exactly on line boundaries, between numbers.
 */
function textOnly(block: { top: number; bottom: number; type: unknown }): {
  top: number;
  bottom: number;
} {
  if (Array.isArray(block.type)) {
    const text = (
      block.type as { type: BlockType; top: number; bottom: number }[]
    ).find((c) => c.type === BlockType.Text);
    if (text) return { top: text.top, bottom: text.bottom };
  }
  return { top: block.top, bottom: block.bottom };
}

/** The tooltip of a context/declared connection: the link's own words. */
function linkTitle(
  target: Extract<RibbonTarget, { kind: "path" }>,
  links: readonly RibbonLink[],
): string {
  const link = links.find(
    (l) =>
      l.family === target.family &&
      samePath(l.to.path, target.path) &&
      (l.to.lines?.[0] ?? 0) === (target.lines?.[0] ?? 0),
  );
  return link?.title ?? `${target.family}: ${target.path}`;
}
