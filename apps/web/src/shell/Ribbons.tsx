// Provenance ribbons, drawn between panes — and out to files that are not
// open yet.
//
// The old Split view drew these between three fixed columns and carried its
// own file tree to do it, which made the chrome depend on the arrangement.
// Here they are an overlay over whatever panes exist.
//
// Two kinds of band:
//
//   * A **ribbon** runs from a block in the document to the text it produced,
//     in a pane that is showing it.
//   * A **stub** runs from a block to the edge of its pane and stops there,
//     labelled with the file it feeds. A relationship you cannot see is a
//     relationship you will not look for, and requiring the file to be open
//     first means never discovering that it exists.
//
// Both are clickable, and clicking is the navigation: a stub opens the file
// and reveals the text; a ribbon reveals it in the pane already showing it.
//
// See docs/specs/freeform/shell-layouts.md.

import { useCallback, useEffect, useRef, useState } from "react";
import type { EditorView } from "@codemirror/view";

import { structureOf } from "../editor/wysiwyg";
import { atLeast, clampBand, ribbonPath, ribbonStubPath, thicknessFor } from "../lib/ribbonGeometry";
import { deriveRibbons, RIBBON_PALETTE_SIZE, type Ribbon } from "../lib/ribbons";
import type { OutputFile } from "../api/types";

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
  docSource: string;
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
  | { kind: "document"; span: [number, number] };

interface Shape {
  key: string;
  path: string;
  color: number;
  clamped: boolean;
  stub: boolean;
  label?: { x: number; y: number; text: string; anchor?: "start" | "end" };
  target: RibbonTarget;
}

/** How far a stub reaches past the pane edge, before its label. */
const STUB = 26;

export function RibbonOverlay({
  container,
  source,
  files,
  documentVisible = true,
  onNavigate,
}: {
  container: HTMLElement | null;
  source: RibbonSource | null;
  files: readonly RibbonFile[];
  /** Whether the document itself is on screen. Decides which way stubs go. */
  documentVisible?: boolean;
  onNavigate?: (target: RibbonTarget) => void;
}) {
  const [shapes, setShapes] = useState<Shape[]>([]);
  const frame = useRef<number | null>(null);

  const measure = useCallback(() => {
    if (!container || !source || files.length === 0) {
      setShapes((current) => (current.length === 0 ? current : []));
      return;
    }
    const box = container.getBoundingClientRect();
    const documentView = source.view ?? null;
    // The channel is the space between the two TEXTS: the gutters on either
    // side belong to it. Anchoring on the scrollers leaves four pixels of
    // divider, which is not a picture anyone can read.
    const left = documentView?.scrollDOM.getBoundingClientRect() ?? null;
    const leftText = documentView?.contentDOM.getBoundingClientRect() ?? null;
    const structure = documentView ? structureOf(documentView.state) : null;
    const sourceLength = documentView?.state.doc.length ?? 0;

    const out: Shape[] = [];
    const colours = new Map<string, number>();
    const colourFor = (fragment: string) => {
      let assigned = colours.get(fragment);
      if (assigned === undefined) {
        assigned = colours.size % RIBBON_PALETTE_SIZE;
        colours.set(fragment, assigned);
      }
      return assigned;
    };

    for (const entry of files) {
      const ribbons = deriveRibbons(entry.file, source.docPath, source.docSource);
      if (ribbons.length === 0) continue;

      if (entry.view && documentView && left && leftText && structure) {
        const right = entry.view.scrollDOM.getBoundingClientRect();
        const rightText = entry.view.contentDOM.getBoundingClientRect();
        // Panes can be in any order: a generated file to the LEFT of its
        // document is an arrangement someone is allowed to build.
        const forward = left.right <= right.left;
        const x0 = (forward ? leftText.right : leftText.left) - box.left;
        const x1 = (forward ? rightText.left : rightText.right) - box.left;
        const targetLength = entry.view.state.doc.length;

        for (const ribbon of ribbons) {
          const band = sourceBand(documentView, structure, ribbon, sourceLength);
          const to = bandBetween(
            entry.view,
            Math.min(ribbon.outputRange[0], targetLength),
            Math.min(ribbon.outputRange[1], targetLength),
          );
          if (!band || !to) continue;
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
          const thickness = Math.max(3, thicknessFor(ribbon.bytes, 1));
          const a = atLeast(from.yTop, from.yBot, thickness);
          const b = atLeast(into.yTop, into.yBot, thickness);
          out.push({
            key: `${entry.file.path}:${ribbon.key}`,
            path: ribbonPath(x0, a.yTop, a.yBot, x1, b.yTop, b.yBot),
            color: colourFor(fragmentKey(ribbon)),
            clamped: from.clamped || into.clamped,
            stub: false,
            target: {
              kind: "generated",
              path: entry.file.path,
              range: ribbon.outputRange,
            },
          });
        }
        continue;
      }

      // Not open, and the document is: one stub per source block rather than
      // per fragment. One file that a block feeds is one statement, not fifty.
      if (!documentView || !left || !leftText || !structure) continue;
      const byBlock = new Map<string, { ribbon: Ribbon; bytes: number }>();
      for (const ribbon of ribbons) {
        const key = fragmentKey(ribbon);
        const seen = byBlock.get(key);
        if (seen) seen.bytes += ribbon.bytes;
        else byBlock.set(key, { ribbon, bytes: ribbon.bytes });
      }

      const x0 = leftText.right - box.left;
      const edge = left.right - box.left;
      for (const [key, group] of byBlock) {
        const band = sourceBand(documentView, structure, group.ribbon, sourceLength);
        if (!band) continue;
        const from = clampBand(
          band[0] - box.top,
          band[1] - box.top,
          left.top - box.top,
          left.bottom - box.top,
        );
        const thickness = Math.max(3, thicknessFor(group.bytes, 1));
        const a = atLeast(from.yTop, from.yBot, thickness);
        const mid = (a.yTop + a.yBot) / 2;
        out.push({
          key: `${entry.file.path}:stub:${key}`,
          path: ribbonStubPath(
            x0,
            a.yTop,
            a.yBot,
            edge + STUB,
            mid - thickness / 2,
            mid + thickness / 2,
          ),
          color: colourFor(key),
          clamped: from.clamped,
          stub: true,
          label: {
            x: edge + STUB + 5,
            y: mid + 3,
            text: entry.file.path.split("/").pop() ?? entry.file.path,
          },
          target: {
            kind: "generated",
            path: entry.file.path,
            range: group.ribbon.outputRange,
          },
        });
      }
    }

    // The other direction: a generated pane whose document is not on screen
    // gets stubs of its own, reaching back the way they came. Provenance is
    // symmetrical and so is the question — "where did this come from" is
    // asked from the generated side at least as often.
    if (!documentVisible) {
      for (const entry of files) {
        if (!entry.view) continue;
        const pane = entry.view.scrollDOM.getBoundingClientRect();
        const text = entry.view.contentDOM.getBoundingClientRect();
        const x0 = text.left - box.left;
        const edge = pane.left - box.left;
        const byBlock = new Map<string, { ribbon: Ribbon; bytes: number }>();
        for (const ribbon of deriveRibbons(entry.file, source.docPath, source.docSource)) {
          const key = fragmentKey(ribbon);
          const seen = byBlock.get(key);
          if (seen) seen.bytes += ribbon.bytes;
          else byBlock.set(key, { ribbon, bytes: ribbon.bytes });
        }
        for (const [key, group] of byBlock) {
          const targetLength = entry.view.state.doc.length;
          const band = bandBetween(
            entry.view,
            Math.min(group.ribbon.outputRange[0], targetLength),
            Math.min(group.ribbon.outputRange[1], targetLength),
          );
          if (!band) continue;
          const from = clampBand(
            band[0] - box.top,
            band[1] - box.top,
            pane.top - box.top,
            pane.bottom - box.top,
          );
          const thickness = Math.max(3, thicknessFor(group.bytes, 1));
          const a = atLeast(from.yTop, from.yBot, thickness);
          const mid = (a.yTop + a.yBot) / 2;
          const label = source.docPath.split("/").pop() ?? source.docPath;
          out.push({
            key: `${entry.file.path}:back:${key}`,
            path: ribbonStubPath(
              x0,
              a.yTop,
              a.yBot,
              edge - STUB,
              mid - thickness / 2,
              mid + thickness / 2,
            ),
            color: colourFor(key),
            clamped: from.clamped,
            stub: true,
            label: {
              x: edge - STUB - 5,
              y: mid + 3,
              text: label,
              anchor: "end",
            },
            target: { kind: "document", span: group.ribbon.sourceByteSpan },
          });
        }
      }
    }

    setShapes(out);
  }, [container, source, files, documentVisible]);

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

    const views = [source?.view, ...files.map((f) => f.view)].filter(Boolean) as EditorView[];
    const scrollers = views.map((view) => view.scrollDOM);
    for (const scroller of scrollers) {
      scroller.addEventListener("scroll", schedule, { passive: true });
    }
    window.addEventListener("resize", schedule);
    const observer = new ResizeObserver(schedule);
    if (container) observer.observe(container);
    const timer = window.setInterval(schedule, 500);

    return () => {
      for (const scroller of scrollers) scroller.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      observer.disconnect();
      window.clearInterval(timer);
      if (frame.current !== null) cancelAnimationFrame(frame.current);
      frame.current = null;
    };
  }, [measure, container, source, files]);

  if (shapes.length === 0) return null;
  return (
    <svg className="ribbon-layer">
      {shapes.map((shape) => (
        <g key={shape.key} className="ribbon-group" onClick={() => onNavigate?.(shape.target)}>
          <path
            d={shape.path}
            className={`ribbon ribbon-c${shape.color}${shape.clamped ? " ribbon-clamped" : ""}${
              shape.stub ? " ribbon-stub" : ""
            }`}
          >
            <title>
              {shape.target.kind === "document"
                ? "Show the prose this came from"
                : shape.stub
                  ? `Open ${shape.target.path} at what this block produced`
                  : `Show this in ${shape.target.path}`}
            </title>
          </path>
          {shape.label && (
            <text
              className="ribbon-label"
              x={shape.label.x}
              y={shape.label.y}
              textAnchor={shape.label.anchor ?? "start"}
            >
              {shape.label.text}
            </text>
          )}
        </g>
      ))}
    </svg>
  );
}

/** Fragments of one source block share a colour and a stub. */
function fragmentKey(ribbon: Ribbon): string {
  return `${ribbon.sourceByteSpan[0]}:${ribbon.sourceByteSpan[1]}`;
}

/**
 * The vertical extent of a ribbon's source, in client coordinates.
 *
 * The whole enclosing block, not the fragment: a band that covers the copy
 * block says "this block became that text", which is the claim. A band over
 * three characters of it says nothing anyone can read at a glance.
 */
function sourceBand(
  view: EditorView,
  structure: ReturnType<typeof structureOf>,
  ribbon: Ribbon,
  length: number,
): [number, number] | null {
  let from = Math.min(ribbon.sourceSpan[0], length);
  let to = Math.min(ribbon.sourceSpan[1], length);
  for (const block of structure.blocks) {
    if (
      (block.name === "copy" || block.name === "cut" || block.name === "file") &&
      ribbon.sourceSpan[0] >= block.from &&
      ribbon.sourceSpan[1] <= block.to
    ) {
      from = Math.min(block.from, length);
      to = Math.min(block.to, length);
      break;
    }
  }
  return bandBetween(view, from, to);
}

/**
 * Screen extent of a range, including one scrolled out of sight.
 *
 * `coordsAtPos` answers for rendered text only — everything outside the
 * viewport returns null, and a ribbon that cannot be measured is a ribbon
 * that is not drawn: six relationships showed one. The height map knows where
 * every line is whether or not it is on screen, and the content box moves
 * with the scroll, so the two together answer for any position; the caller
 * clamps the result to the visible pane.
 */
function bandBetween(view: EditorView, from: number, to: number): [number, number] | null {
  const content = view.contentDOM.getBoundingClientRect();
  const first = view.lineBlockAt(Math.max(0, from));
  const last = view.lineBlockAt(Math.max(0, to));
  const start = { top: content.top + first.top, bottom: content.top + first.bottom };
  const end = { top: content.top + last.top, bottom: content.top + last.bottom };
  return [Math.min(start.top, end.top), Math.max(start.bottom, end.bottom)];
}
