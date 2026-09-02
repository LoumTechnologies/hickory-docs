// The links, drawn over the rail: a bracket around each end and one thin
// thread between them.
//
// The shape divides the labour. The BRACKET says how much text an end covers,
// so a fragment of twenty lines does not look like a fragment of one. The
// THREAD says only which bracket it reaches, so it is stroked at a constant
// width — an earlier version filled the space between two curves, which has
// no constant thickness and visibly swelled along its own length.
//
// An end can be in three states and each has somewhere to dock: visible lines
// (the block's extent), inside a hole (the hole), or in a file this column
// does not have open (that file's row in the tree). A link is never simply
// absent, because absence reads as "no such relationship" — the one thing the
// picture must not say.

import { useEffect, useState, type RefObject } from "react";

import { attrValue } from "../lib/attrSelector";
import type { LineageModel, LinkKind } from "./model";

const BRACE = 7;

interface End {
  left: number;
  right: number;
  top: number;
  bottom: number;
  docked: Element | null;
}

export interface LinkLayerProps {
  model: LineageModel;
  kinds: Set<LinkKind>;
  enabled: boolean;
  selected: string | null;
  relation: (id: string) => "self" | "up" | "down" | null;
  railRef: RefObject<HTMLDivElement | null>;
  /** Bumped by the host after anything that can move an endpoint. */
  tick: number;
}

export function LinkLayer({ model, kinds, enabled, selected, relation, railRef, tick }: LinkLayerProps) {
  const [paths, setPaths] = useState<{ d: string; className: string; key: string }[]>([]);
  const [size, setSize] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const rail = railRef.current;
    if (!rail) return;
    const columns = rail.querySelector(".lineage-columns");
    if (!columns) return;

    setSize({ width: columns.clientWidth, height: columns.clientHeight });
    for (const el of rail.querySelectorAll("[data-docked]")) el.removeAttribute("data-docked");
    if (!enabled) {
      setPaths([]);
      return;
    }

    const docked = new Map<Element, number>();
    const next: { d: string; className: string; key: string }[] = [];

    for (const link of model.links) {
      if (!kinds.has(link.kind)) continue;
      if (selected && !(relation(link.from) && relation(link.to))) continue;
      const from = model.nodes.get(link.from);
      const to = model.nodes.get(link.to);
      if (!from || !to) continue;

      const fromCol = columnFor(rail, from.file);
      if (fromCol === null) continue;
      // A link whose ends share a stage — a fragment and the file it weaves —
      // lives in ONE column, and that is the commonest link there is. Drawing
      // only left-to-right between columns silently dropped every one of them.
      const after = columnFor(rail, to.file, fromCol);
      const toCol = after ?? fromCol;
      if (toCol < fromCol) continue;

      const left = endpoint(rail, columns, fromCol, from, docked);
      const right = endpoint(rail, columns, toCol, to, docked);
      if (!left || !right) continue;
      if (toCol > fromCol && right.left < left.right) continue;

      const current = selected === link.from || selected === link.to;
      const cls = [
        `k-${link.kind}`,
        left.docked || right.docked ? "docked" : "",
        current ? "current" : "",
      ]
        .filter(Boolean)
        .join(" ");

      const sameColumn = toCol === fromCol;
      next.push({
        key: `t-${link.from}-${link.to}`,
        className: `link-thread ${cls}`,
        d: sameColumn ? loop(left, right) : thread(left, right),
      });
      next.push({ key: `l-${link.from}-${link.to}`, className: `link-brace ${cls}`, d: brace(left, "left") });
      next.push({
        key: `r-${link.from}-${link.to}`,
        className: `link-brace ${cls}`,
        // Both ends of a same-column link are braced from the same side, so
        // the pair reads as one bracket bowing out and back.
        d: brace(right, sameColumn ? "left" : "right"),
      });
    }

    for (const [el, count] of docked) el.setAttribute("data-docked", String(count));
    setPaths(next);
  }, [enabled, kinds, model, railRef, relation, selected, tick]);

  return (
    <svg className="link-layer" width={size.width} height={size.height} aria-hidden="true">
      {paths.map((p) => (
        <path key={p.key} d={p.d} className={p.className} />
      ))}
    </svg>
  );
}

/** The index of the column showing `file`, at or after `after`. */
function columnFor(rail: Element, file: string, after = -1): number | null {
  const columns = [...rail.querySelectorAll("[data-column]")];
  for (const col of columns) {
    const index = Number(col.getAttribute("data-column"));
    if (index <= after) continue;
    if (col.querySelector(`[data-treepath=${attrValue(file)}].open`)) return index;
  }
  // Not open anywhere after `after`: the column whose stage owns it, so the
  // link can still dock to the closed file's row.
  for (const col of columns) {
    const index = Number(col.getAttribute("data-column"));
    if (index <= after) continue;
    if (col.querySelector(`[data-treepath=${attrValue(file)}]`)) return index;
  }
  return null;
}

function endpoint(
  rail: Element,
  columns: Element,
  columnIndex: number,
  node: { id: string; file: string; startLine: number; endLine: number },
  docked: Map<Element, number>,
): End | null {
  const col = rail.querySelector(`[data-column="${columnIndex}"]`);
  if (!col) return null;
  const body = col.querySelector(".stage-body");
  if (!body) return null;
  const railBox = columns.getBoundingClientRect();
  const bodyBox = body.getBoundingClientRect();
  const clamp = (y: number) => Math.min(Math.max(y, bodyBox.top + 1), bodyBox.bottom - 1);
  const frame = (top: number, bottom: number, dock: Element | null): End => {
    if (dock) docked.set(dock, (docked.get(dock) ?? 0) + 1);
    return {
      left: bodyBox.left - railBox.left,
      right: bodyBox.right - railBox.left,
      top: clamp(top) - railBox.top,
      bottom: clamp(bottom) - railBox.top,
      docked: dock,
    };
  };

  const rows = [...body.querySelectorAll(`[data-blockid=${attrValue(node.id)}]`)];
  if (rows.length) {
    const first = rows[0].getBoundingClientRect();
    const last = rows[rows.length - 1].getBoundingClientRect();
    return frame(first.top, last.bottom, null);
  }

  const hole = [...body.querySelectorAll(".hole")].find((h) => {
    const from = Number(h.getAttribute("data-from"));
    const to = Number(h.getAttribute("data-to"));
    return node.startLine <= to && node.endLine >= from;
  });
  if (hole) {
    const box = hole.getBoundingClientRect();
    return frame(box.top, box.bottom, hole);
  }

  const row = body.querySelector(`[data-treepath=${attrValue(node.file)}]`);
  if (row) {
    const box = row.getBoundingClientRect();
    return frame(box.top, box.bottom, row);
  }
  return null;
}

function thread(a: End, b: End): string {
  const x1 = a.right + BRACE * 1.5;
  const x2 = b.left - BRACE * 1.5;
  const y1 = (a.top + a.bottom) / 2;
  const y2 = (b.top + b.bottom) / 2;
  const mx = (x1 + x2) / 2;
  return `M ${x1} ${y1} C ${mx} ${y1}, ${mx} ${y2}, ${x2} ${y2}`;
}

/**
 * A link whose ends share a column: out into the gap beside it and back.
 *
 * The gap between columns already exists for the threads that cross it, so a
 * same-stage link borrows it rather than drawing over the text — which is
 * where the two ends actually are.
 */
function loop(a: End, b: End): string {
  const x = a.right + BRACE * 1.5;
  const y1 = (a.top + a.bottom) / 2;
  const y2 = (b.top + b.bottom) / 2;
  const bulge = x + 26;
  return `M ${x} ${y1} C ${bulge} ${y1}, ${bulge} ${y2}, ${x} ${y2}`;
}

function brace(end: End, side: "left" | "right"): string {
  const x = side === "left" ? end.right : end.left;
  const dir = side === "left" ? 1 : -1;
  const w = BRACE;
  const mid = (end.top + end.bottom) / 2;
  const lip = Math.min(w, Math.max(2, (mid - end.top) / 2));
  return [
    `M ${x} ${end.top}`,
    `q ${dir * w} 0, ${dir * w} ${lip}`,
    `L ${x + dir * w} ${mid - w / 2}`,
    `q 0 ${w / 2}, ${dir * (w / 2)} ${w / 2}`,
    `q ${-dir * (w / 2)} 0, ${-dir * (w / 2)} ${w / 2}`,
    `L ${x + dir * w} ${end.bottom - lip}`,
    `q 0 ${lip}, ${-dir * w} ${lip}`,
  ].join(" ");
}

