// Context and declared links use the same rails and geometry as byte lineage.
import type { EditorView } from "@codemirror/view";
import type { Draft, Terminal, RibbonFamily, RibbonLink, RibbonSource, RibbonFile } from "./Ribbons";
import { samePath } from "../lib/paths";
import { anchorEdges, clampBand, braceNub, bracePath, braceLinkPath, ribbonPath } from "../lib/ribbonGeometry";

type Geometry = Pick<typeof import("./Ribbons"), "bandBetween" | "paneEdges" | "findTerminal" | "attachTerminal" | "revealStrip">;
export function linkShapes(container: HTMLElement, sources: readonly RibbonSource[], files: readonly RibbonFile[], links: readonly RibbonLink[], on: (family: RibbonFamily) => boolean, braces: boolean, geometry: Geometry): Draft[] {
  const { bandBetween, paneEdges, findTerminal, attachTerminal, revealStrip } = geometry;
  const box = container.getBoundingClientRect();
  const out: Draft[] = [];
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
      const targetView = link.to.lines ? sources.find(s => s.view && samePath(s.docPath, link.to.path))?.view
        ?? files.find(f => f.view && samePath(f.file.path, link.to.path))?.view : undefined;
      if (!terminal && !targetView) continue;
      const total = view.state.doc.lines;
      const a = Math.min(Math.max(link.from.lines[0], 1), total);
      const b = Math.min(Math.max(link.from.lines[1], a), total);
      const from = view.state.doc.line(a).from;
      const to = view.state.doc.line(b).to;
      const rect = link.anchor?.getBoundingClientRect();
      const band = rect ? [rect.top, rect.bottom] : bandBetween(view, from, to);
      if (!band || band[1] <= band[0]) continue;
      const pane = editorViewport(view);
      const edges = paneEdges(view);
      if (targetView && targetView !== view && link.to.lines) {
        const range = lineRange(targetView, link.to.lines);
        // Do not clamp an invalid citation onto some other paragraph.
        if (!range) continue;
        const targetBand = bandBetween(targetView, range.from, range.to);
        if (!targetBand) continue;
        const targetPane = editorViewport(targetView);
        const targetEdges = paneEdges(targetView);
        const { forward, x0, x1 } = anchorEdges(
          { left: edges.left - box.left, right: edges.right - box.left },
          { left: targetEdges.left - box.left, right: targetEdges.right - box.left });
        const fromBand = clampBand(band[0] - box.top, band[1] - box.top, pane.top - box.top, pane.bottom - box.top);
        const into = clampBand(targetBand[0] - box.top, targetBand[1] - box.top, targetPane.top - box.top, targetPane.bottom - box.top);
        const shape: Draft = { key: `${link.family}:${link.key}:text`, family: link.family,
          color: link.family === "context" ? 0 : 1, alwaysVisible: link.alwaysVisible,
          ends: "text", clamped: fromBand.clamped || into.clamped,
          hl: [{ view, from, to }, { view: targetView, ...range }],
          target: { kind: "path", path: link.to.path, lines: link.to.lines, family: link.family } };
        if (braces) {
          const dir = forward ? 1 : -1, back = forward ? -1 : 1;
          const a = bracePath(x0, fromBand.yTop, fromBand.yBot, dir, undefined,
            { top: fromBand.clampedTop, bottom: fromBand.clampedBottom }, forward ? edges.rightRailW : edges.leftRailW);
          const b = bracePath(x1, into.yTop, into.yBot, back, undefined,
            { top: into.clampedTop, bottom: into.clampedBottom }, forward ? targetEdges.leftRailW : targetEdges.rightRailW);
          const na = braceNub(x0, fromBand.yTop, fromBand.yBot, dir), nb = braceNub(x1, into.yTop, into.yBot, back);
          const join = braceLinkPath(na.x, na.y, dir, nb.x, nb.y, back);
          shape.brace = { a, b, link: join, hit: `${a} ${join} ${b}` };
        } else shape.band = ribbonPath(x0, fromBand.yTop, fromBand.yBot, x1, into.yTop, into.yBot);
        out.push(shape);
        continue;
      }
      if (!terminal) continue;
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

  return out;
}

export function lineRange(view: EditorView, lines: [number, number]) {
  const [a, b] = lines;
  if (a < 1 || b < a || b > view.state.doc.lines) return null;
  return { from: view.state.doc.line(a).from, to: view.state.doc.line(b).to };
}

/** A conversation can scroll in a parent while CodeMirror's own scroller grows. */
export function editorViewport(view: EditorView): { top: number; bottom: number } {
  let { top, bottom } = view.scrollDOM.getBoundingClientRect();
  for (let parent = view.scrollDOM.parentElement; parent; parent = parent.parentElement) {
    if (!/^(auto|scroll|hidden|clip)$/.test(getComputedStyle(parent).overflowY)) continue;
    const bounds = parent.getBoundingClientRect();
    top = Math.max(top, bounds.top);
    bottom = Math.min(bottom, bounds.bottom);
  }
  return { top, bottom: Math.max(top, bottom) };
}
