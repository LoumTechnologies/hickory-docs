// The landing page's split-mode editor: the document on the left, the things
// it generates in the middle, the generated text on the right, and Sankey
// ribbons threading all three. Edit either end.
//
// Relationship to views/SplitView.tsx, stated plainly so neither drifts into
// pretending to be the other: SplitView is the signed-in workspace's split —
// server-woven outputs, CRDT rooms per output buffer, LSP bindings, folded
// ranges. This is the same PICTURE computed entirely in the visitor's browser
// (lib/weave), with the same geometry helpers (lib/ribbonGeometry) and the
// same provenance round-trip (lib/weave.mapEditsToSource), so what a stranger
// plays with on the landing page is the real mechanism rather than a mockup.
//
// The overlay rules SplitView documents hold here too: ribbons are an SVG
// layer that cannot affect editor layout, and anchors are measured from live
// geometry on every scroll, resize and change.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorView } from "@codemirror/view";
import type * as Y from "yjs";
import type { Awareness } from "y-protocols/awareness";
import { setRangeHighlights } from "../../editor/rangeHighlight";
import { computeEdits, toByteEdits } from "../../lib/diff";
import { deriveRibbons, RIBBON_PALETTE_SIZE, type Ribbon } from "../../lib/ribbons";
import { atLeast, clampBand, ribbonPathVia, ribbonStubPath } from "../../lib/ribbonGeometry";
import {
  SyntheticRangeViolation,
  applySourceEdits,
  mapEditsToSource,
  weaveOutputs,
} from "../../lib/weave";
import type { SourceEdit } from "../../api/types";
import { DemoEditor } from "./DemoEditor";

export interface DemoSplitProps {
  /** Path the document is known by — provenance points back at it. */
  docPath: string;
  source: string;
  onSourceChange: (next: string) => void;
  /** Heading over the middle column ("Generated files", "Jira issues", …). */
  nodeHeading: string;
  /** Display name for a generated path. Defaults to the path itself. */
  nodeLabel?: (path: string) => string;
  /** Caption over the right pane for the open output. */
  outputCaption?: (path: string) => string;
  /** Caption over the left pane. */
  sourceCaption?: string;
  /** Let the visitor type into the generated side. */
  editableOutput?: boolean;
  /** An output edit was resolved backwards into the document. */
  onOutputMappedBack?: (info: { path: string; edits: SourceEdit[] }) => void;
  /** An output edit was refused because it touched weaver-generated text. */
  onOutputRejected?: (message: string) => void;
  /** Bind the document pane to a CRDT room shared with another pane. */
  collab?: { ytext: Y.Text; awareness: Awareness };
  /** Narrower layout, for two splits side by side. */
  compact?: boolean;
  /**
   * Char range in the document to scroll to and flash. Used to show what a
   * walkthrough step just added: without it a step can append a whole session
   * below the fold and the visitor sees an unchanged screen, which reads as a
   * broken button rather than as work being done.
   */
  revealSpan?: [number, number] | null;
  testId?: string;
}

/** A ribbon plus the generated file it flows into. */
interface FileRibbon extends Ribbon {
  filePath: string;
}

interface RibbonShape {
  key: string;
  color: number;
  path: string;
  clamped: boolean;
  sourceHl: [number, number];
  outputHl: [number, number] | null;
  filePath: string;
}

/** True when two measurements would draw exactly the same picture. */
function sameShapes(a: RibbonShape[], b: RibbonShape[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i].key !== b[i].key || a[i].path !== b[i].path || a[i].clamped !== b[i].clamped) {
      return false;
    }
  }
  return true;
}

function humanBytes(n: number): string {
  return n < 1024 ? `${n} B` : `${(n / 1024).toFixed(1)} KB`;
}

/** Node labels are the file name; the full path stays in the tooltip. In a
 * column this narrow a path ellipsises away exactly the part that identifies
 * the file. */
function basename(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

/** Characters of a change the reveal flash will paint. Roughly a screenful. */
const FLASH_CAP = 700;

/** Block bodies collapsed on arrival — see DemoEditor's `foldBlocks`. */
const FOLD_ON_ARRIVAL = ["file"];

/** Smooth, unless the visitor has asked the OS for less motion. */
function scrollBehavior(): ScrollBehavior {
  return typeof matchMedia === "function" &&
    matchMedia("(prefers-reduced-motion: reduce)").matches
    ? "auto"
    : "smooth";
}

export function DemoSplit({
  docPath,
  source,
  onSourceChange,
  nodeHeading,
  nodeLabel,
  outputCaption,
  sourceCaption,
  editableOutput,
  onOutputMappedBack,
  onOutputRejected,
  collab,
  compact,
  revealSpan,
  testId,
}: DemoSplitProps) {
  const [activePath, setActivePath] = useState<string | null>(null);
  const [leftView, setLeftView] = useState<EditorView | null>(null);
  const [rightView, setRightView] = useState<EditorView | null>(null);
  const [shapes, setShapes] = useState<RibbonShape[]>([]);
  const [hovered, setHovered] = useState<string | null>(null);
  const [revert, setRevert] = useState(0);

  const containerRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef(0);
  const rowsRef = useRef(new Map<string, HTMLElement>());

  const files = useMemo(() => weaveOutputs(source, docPath), [source, docPath]);
  const file = useMemo(
    () => files.find((f) => f.path === activePath) ?? files[0] ?? null,
    [files, activePath],
  );

  // Keep the open output valid as the document gains and loses files.
  useEffect(() => {
    setActivePath((p) => (p && files.some((f) => f.path === p) ? p : (files[0]?.path ?? null)));
  }, [files]);

  // One ribbon per pasted/literal fragment, coloured per distinct source
  // fragment across the whole weave: a fragment feeding two files keeps one
  // colour, which is the entire point of the middle column.
  const ribbons = useMemo(() => {
    const colors = new Map<string, number>();
    const out: FileRibbon[] = [];
    for (const f of files) {
      for (const r of deriveRibbons(f, docPath, source)) {
        const fragKey = `${r.sourceByteSpan[0]}:${r.sourceByteSpan[1]}`;
        let color = colors.get(fragKey);
        if (color === undefined) {
          color = colors.size % RIBBON_PALETTE_SIZE;
          colors.set(fragKey, color);
        }
        out.push({ ...r, color, filePath: f.path });
      }
    }
    return out;
  }, [files, docPath, source]);

  const colorPerFile = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of ribbons) if (!m.has(r.filePath)) m.set(r.filePath, r.color);
    return m;
  }, [ribbons]);

  // ----- geometry ----------------------------------------------------------

  const measureRef = useRef<() => void>(() => undefined);
  measureRef.current = () => {
    const left = leftView;
    const container = containerRef.current;
    if (!left || !container || ribbons.length === 0) {
      setShapes((s) => (s.length === 0 ? s : []));
      return;
    }
    const cRect = container.getBoundingClientRect();
    const lRect = left.scrollDOM.getBoundingClientRect();
    const rRect = rightView?.scrollDOM.getBoundingClientRect() ?? null;
    const leftLen = left.state.doc.length;
    const rightLen = rightView?.state.doc.length ?? 0;
    const x0 = lRect.right - cRect.left;
    const x1 = rRect ? rRect.left - cRect.left : x0;

    const byFile = new Map<string, FileRibbon[]>();
    for (const r of ribbons) {
      const list = byFile.get(r.filePath);
      if (list) list.push(r);
      else byFile.set(r.filePath, [r]);
    }

    const out: RibbonShape[] = [];
    let aboveCount = 0;
    let belowCount = 0;
    for (const [path, list] of byFile) {
      const row = rowsRef.current.get(path);
      if (!row) continue;
      const nRect = row.getBoundingClientRect();
      const nodeLeft = nRect.left - cRect.left;
      const nodeRight = nRect.right - cRect.left;
      const isActive = path === file?.path;

      const ordered = [...list].sort((a, b) => a.outputRange[0] - b.outputRange[0]);
      const totalBytes = ordered.reduce((s, r) => s + r.bytes, 0) || 1;
      // A node's height is its byte budget, subdivided among the fragments
      // feeding it — the middle column is a Sankey stage, not a legend.
      let cursor = nRect.top;
      for (const r of ordered) {
        const share = (r.bytes / totalBytes) * nRect.height;
        const nTop = cursor;
        const nBot = cursor + Math.max(share, 1);
        cursor = nBot;

        const sFrom = Math.min(r.sourceSpan[0], leftLen);
        const sTo = Math.min(r.sourceSpan[1], leftLen);
        const sTopY = left.lineBlockAt(sFrom).top + left.documentTop;
        const sBotY = left.lineBlockAt(sTo).bottom + left.documentTop;
        // Off-screen anchors all clamp to the same pane edge; without a
        // stagger they stack into one line and the picture reads as "nothing
        // here" rather than "more of the document is above you".
        const stagger =
          sBotY <= lRect.top ? aboveCount++ : sTopY >= lRect.bottom ? belowCount++ : 0;
        const sBand = clampBand(sTopY, sBotY, lRect.top, lRect.bottom, 2, stagger);
        const s = atLeast(sBand.yTop, sBand.yBot);

        if (isActive && rRect && rightView) {
          const oFrom = Math.min(r.outputRange[0], rightLen);
          const oTo = Math.min(Math.max(r.outputRange[1] - 1, oFrom), rightLen);
          const oBand = clampBand(
            rightView.lineBlockAt(oFrom).top + rightView.documentTop,
            rightView.lineBlockAt(oTo).bottom + rightView.documentTop,
            rRect.top,
            rRect.bottom,
          );
          const o = atLeast(oBand.yTop, oBand.yBot);
          out.push({
            key: r.key,
            color: r.color,
            clamped: sBand.clamped || oBand.clamped,
            filePath: path,
            path: ribbonPathVia(
              x0,
              s.yTop - cRect.top,
              s.yBot - cRect.top,
              nodeLeft,
              nodeRight,
              nTop - cRect.top,
              nBot - cRect.top,
              x1,
              o.yTop - cRect.top,
              o.yBot - cRect.top,
            ),
            sourceHl: [sFrom, sTo],
            outputHl: r.outputRange,
          });
        } else {
          out.push({
            key: r.key,
            color: r.color,
            clamped: sBand.clamped,
            filePath: path,
            path: ribbonStubPath(
              x0,
              s.yTop - cRect.top,
              s.yBot - cRect.top,
              nodeLeft,
              nTop - cRect.top,
              nBot - cRect.top,
            ),
            sourceHl: [sFrom, sTo],
            outputHl: null,
          });
        }
      }
    }
    setShapes((prev) => (sameShapes(prev, out) ? prev : out));
  };

  // rAF-throttled measurement; a hidden tab suspends rAF, so fall back to a
  // timer there and be correct the moment the tab is looked at again.
  const schedule = useCallback(() => {
    if (frameRef.current) return;
    const run = () => {
      frameRef.current = 0;
      measureRef.current();
    };
    frameRef.current =
      typeof document !== "undefined" && document.hidden
        ? -window.setTimeout(run, 50)
        : requestAnimationFrame(run);
  }, []);

  useEffect(
    () => () => {
      if (frameRef.current > 0) cancelAnimationFrame(frameRef.current);
      if (frameRef.current < 0) clearTimeout(-frameRef.current);
      frameRef.current = 0;
    },
    [],
  );

  const registerRow = useCallback(
    (path: string, el: HTMLElement | null) => {
      if (el) rowsRef.current.set(path, el);
      else rowsRef.current.delete(path);
      schedule();
    },
    [schedule],
  );

  const onEditorReady = useCallback(
    (set: (v: EditorView | null) => void) => (view: EditorView | null) => {
      set(view);
      schedule();
    },
    [schedule],
  );

  useEffect(() => {
    const container = containerRef.current;
    const targets = [leftView?.scrollDOM, rightView?.scrollDOM].filter(
      (el): el is HTMLElement => !!el,
    );
    for (const el of targets) el.addEventListener("scroll", schedule, { passive: true });
    window.addEventListener("resize", schedule);
    let observer: ResizeObserver | null = null;
    if (typeof ResizeObserver !== "undefined" && container) {
      observer = new ResizeObserver(schedule);
      observer.observe(container);
    }
    schedule();
    return () => {
      for (const el of targets) el.removeEventListener("scroll", schedule);
      window.removeEventListener("resize", schedule);
      observer?.disconnect();
    };
  }, [leftView, rightView, schedule]);

  useEffect(schedule, [ribbons, activePath, source, schedule]);

  // ----- editing from the generated side ------------------------------------

  const onOutputChange = (next: string) => {
    if (!file || next === file.content) return;
    const edits = toByteEdits(file.content, computeEdits(file.content, next));
    if (edits.length === 0) return;
    try {
      const sourceEdits = mapEditsToSource(file, edits);
      onSourceChange(applySourceEdits(source, sourceEdits));
      onOutputMappedBack?.({ path: file.path, edits: sourceEdits });
    } catch (e) {
      if (!(e instanceof SyntheticRangeViolation)) throw e;
      onOutputRejected?.(
        "That line was written by the weaver, so there is no source text behind it to change. Edit the document, or one of the fragments it pastes.",
      );
      // Put the buffer back: the document never accepted this text.
      setRevert((n) => n + 1);
    }
  };

  // ----- hover, reveal -------------------------------------------------------

  const flashRef = useRef(0);
  /** Guards the flash timer against firing into a destroyed view. */
  const viewAliveRef = useRef(true);
  useEffect(() => {
    viewAliveRef.current = true;
    return () => {
      viewAliveRef.current = false;
      clearTimeout(flashRef.current);
    };
  }, []);

  /** Highlight ranges in the document, optionally scrolling the first into
   * view. A `flash` highlight clears itself; a hover highlight does not. */
  const markSource = useCallback(
    (ranges: [number, number][], opts?: { scroll?: boolean; flash?: boolean }) => {
      const view = leftView;
      if (!view) return;
      clearTimeout(flashRef.current);
      const max = view.state.doc.length;
      const clamped = ranges
        .map(([from, to]): [number, number] => [Math.min(from, max), Math.min(to, max)])
        .filter(([from, to]) => to > from);
      view.dispatch({
        effects: setRangeHighlights.of(clamped.map(([from, to]) => ({ from, to }))),
      });
      if (opts?.scroll && clamped.length > 0) {
        // Scroll the EDITOR, never the page. CodeMirror's own scrollIntoView
        // walks up and scrolls every ancestor scroller too, so revealing a
        // line inside the demo would yank the whole page out from under
        // someone who was reading the paragraph above it.
        //
        // A frame late on purpose: the document just changed size, and
        // line geometry measured before CodeMirror re-measures points at
        // where the line used to be.
        requestAnimationFrame(() => {
          if (!viewAliveRef.current) return;
          const top = view.lineBlockAt(clamped[0][0]).top;
          view.scrollDOM.scrollTo({ top: Math.max(0, top - 16), behavior: scrollBehavior() });
        });
      }
      if (opts?.flash) {
        flashRef.current = window.setTimeout(() => {
          if (viewAliveRef.current) view.dispatch({ effects: setRangeHighlights.of([]) });
        }, 2200);
      }
    },
    [leftView],
  );

  // Show what a step just added, once the new text is actually in the buffer.
  //
  // The highlight is capped even when the change is not. Jumping several steps
  // at once changes most of the document, and painting all of it says "look
  // everywhere", which is the same as saying nothing — the scroll target is
  // what carries the message, so the flash only has to mark where it landed.
  useEffect(() => {
    if (!revealSpan || !leftView) return;
    const [from, to] = revealSpan;
    markSource([[from, Math.min(to, from + FLASH_CAP)]], { scroll: true, flash: true });
  }, [revealSpan, leftView, markSource]);

  /** Where in the document a generated file's content comes from. */
  const sourceSpansOf = useCallback(
    (path: string): [number, number][] => {
      const forFile = ribbons.filter((r) => r.filePath === path);
      // Prefer the pasted fragments: for a Jira ticket those are the note it
      // was written from, which is the answer to "where did this come from?".
      // The literal scaffolding around them is not.
      const pasted = forFile.filter((r) => r.kind === "paste");
      return (pasted.length > 0 ? pasted : forFile).map((r) => r.sourceSpan);
    },
    [ribbons],
  );

  const openFile = (path: string) => {
    setActivePath(path);
    markSource(sourceSpansOf(path), { scroll: true, flash: true });
  };

  const highlight = (shape: RibbonShape | null) => {
    setHovered(shape?.key ?? null);
    markSource(shape ? [shape.sourceHl] : []);
    rightView?.dispatch({
      effects: setRangeHighlights.of(
        shape?.outputHl ? [{ from: shape.outputHl[0], to: shape.outputHl[1] }] : [],
      ),
    });
  };

  const selectBoth = (shape: RibbonShape) => {
    if (shape.filePath !== file?.path) {
      openFile(shape.filePath);
      return;
    }
    markSource([shape.sourceHl], { scroll: true });
  };

  // Before the document produces anything there is no flow to draw, and two
  // empty columns saying so is worse than one document with room to read: the
  // split collapses to the document alone and opens up when there is
  // something to open up for.
  const flowing = files.length > 0;

  return (
    <div
      className={`demo-split${compact ? " demo-split-compact" : ""}${
        flowing ? "" : " demo-split-solo"
      }`}
      ref={containerRef}
      data-testid={testId}
    >
      <div className="demo-pane demo-pane-source">
        {sourceCaption && <p className="demo-pane-caption mono">{sourceCaption}</p>}
        <DemoEditor
          value={source}
          onChange={onSourceChange}
          hick
          // The generated file's scaffolding is visible rendered on the right;
          // its source body only costs the fragments above it their place on
          // screen, and with it their ribbons.
          foldBlocks={FOLD_ON_ARRIVAL}
          collab={collab}
          onViewReady={onEditorReady(setLeftView)}
          ariaLabel="Document source"
          testId={testId ? `${testId}-source` : undefined}
        />
      </div>

      {flowing && (
        <nav className="demo-nodes" aria-label={nodeHeading}>
          <h4 className="demo-nodes-heading">{nodeHeading}</h4>
          <ul className="demo-nodes-list">
            {files.map((f) => {
              const color = colorPerFile.get(f.path);
              const active = f.path === file?.path;
              return (
                <li key={f.path}>
                  <button
                    type="button"
                    ref={(el) => registerRow(f.path, el)}
                    className={`demo-node${active ? " on" : ""}${
                      color === undefined ? "" : ` tree-c${color}`
                    }`}
                    aria-current={active}
                    onClick={() => openFile(f.path)}
                    // Hovering a node lights up the text it was written from.
                    // The ribbons already say this, but they are thin targets
                    // and a node is a big one.
                    onMouseEnter={() => markSource(sourceSpansOf(f.path))}
                    onMouseLeave={() => markSource([])}
                    onFocus={() => markSource(sourceSpansOf(f.path))}
                    onBlur={() => markSource([])}
                    data-tip={f.path}
                  >
                    <span className="demo-node-name mono">
                      {nodeLabel ? nodeLabel(f.path) : basename(f.path)}
                    </span>
                    <span className="demo-node-bytes">
                      {humanBytes(new Blob([f.content]).size)}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>
      )}

      {flowing && file && (
        <div className="demo-pane demo-pane-output">
          <p className="demo-pane-caption mono">
            {outputCaption ? outputCaption(file.path) : file.path}
            {editableOutput && <span className="demo-editable-chip">editable</span>}
          </p>
          <DemoEditor
            key={file.path}
            value={file.content}
            onChange={editableOutput ? onOutputChange : undefined}
            readOnly={!editableOutput}
            language={file.language}
            syncToken={revert}
            className={editableOutput ? "demo-editor-editable" : undefined}
            onViewReady={onEditorReady(setRightView)}
            ariaLabel="Generated output"
            testId={testId ? `${testId}-output` : undefined}
          />
        </div>
      )}

      <svg
        className={`ribbon-layer${hovered ? " has-hover" : ""}`}
        data-testid={testId ? `${testId}-ribbons` : undefined}
        aria-hidden="true"
      >
        {shapes.map((s) => (
          <path
            key={s.key}
            d={s.path}
            className={`ribbon ribbon-c${s.color}${s.clamped ? " ribbon-faded" : ""}${
              hovered === s.key ? " ribbon-hovered" : ""
            }${s.outputHl ? "" : " ribbon-stub"}`}
            onMouseEnter={() => highlight(s)}
            onMouseLeave={() => highlight(null)}
            onClick={() => selectBoth(s)}
          />
        ))}
      </svg>
    </div>
  );
}
