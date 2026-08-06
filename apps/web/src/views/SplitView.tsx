// Split (lineage) view: Document editor left, generated output right, and an
// SVG ribbon layer in between connecting each source fragment to the output
// ranges it produced — a Sankey-style picture of the weave.
//
// Layering rules honoured here:
//  - RIBBONS ARE OVERLAY ONLY. Nothing in this file adds decorations that
//    could affect editor vertical layout; the only decorations dispatched are
//    background marks (rangeHighlightField, a StateField).
//  - Anchors track LIVE geometry: EditorView.lineBlockAt + documentTop give
//    document-space rects that stay valid through folding (a folded range's
//    lineBlockAt returns the merged visible line, so ribbons collapse onto
//    the folded line), and off-screen anchors clamp to the pane edge with a
//    faded tail. Measurement is rAF-throttled on scroll/resize/fold/change.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorState, StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { api } from "../api/client";
import type { ExecBlock, OutputFile, OutputFileMeta } from "../api/types";
import type { Realtime } from "../api/realtime";
import { DocumentEditor } from "../editor/DocumentEditor";
import { languageExtensions } from "../editor/languages";
import { structureOf } from "../editor/wysiwyg";
import { rangeHighlightField, setRangeHighlights } from "../editor/rangeHighlight";
import { deriveRibbons } from "../lib/ribbons";
import { bandAround, clampBand, ribbonPath, thicknessFor } from "../lib/ribbonGeometry";

export interface SplitViewProps {
  docId: string;
  docPath: string;
  /** Server copy of the source — provenance spans index into this. */
  docSource: string;
  editorKey: string;
  realtime: Realtime;
  onChange: (source: string) => void;
  selectSpan: [number, number] | null;
  execBlocks: ExecBlock[];
  runningCells: Set<string>;
  onRunCell: (execId: string) => void;
}

interface RibbonShape {
  key: string;
  color: number;
  path: string;
  clamped: boolean;
  /** Source block span to highlight/select (char offsets, left editor). */
  sourceHl: [number, number];
  /** Output range to highlight/select (char offsets, right editor). */
  outputHl: [number, number];
}

/** Views that already received the appended highlight/measure config. */
const configured = new WeakSet<EditorView>();

export function SplitView({
  docId,
  docPath,
  docSource,
  editorKey,
  realtime,
  onChange,
  selectSpan,
  execBlocks,
  runningCells,
  onRunCell,
}: SplitViewProps) {
  const [files, setFiles] = useState<OutputFileMeta[] | null>(null);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [file, setFile] = useState<OutputFile | null>(null);
  const [leftView, setLeftView] = useState<EditorView | null>(null);
  const [rightView, setRightView] = useState<EditorView | null>(null);
  const [shapes, setShapes] = useState<RibbonShape[]>([]);
  const [hovered, setHovered] = useState<string | null>(null);

  const containerRef = useRef<HTMLDivElement>(null);
  const rightHostRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef(0);

  const ribbons = useMemo(
    () => (file ? deriveRibbons(file, docPath, docSource) : []),
    [file, docPath, docSource],
  );

  // ----- output files (right pane), ribbons for the ACTIVE file only -------

  useEffect(() => {
    let stale = false;
    api.outputs(docId).then(
      (r) => {
        if (stale) return;
        setFiles(r.files);
        setActivePath((p) =>
          p && r.files.some((f) => f.path === p) ? p : (r.files[0]?.path ?? null),
        );
      },
      () => !stale && setFiles([]),
    );
    return () => {
      stale = true;
    };
  }, [docId, docSource]);

  useEffect(() => {
    if (!activePath) {
      setFile(null);
      return;
    }
    let stale = false;
    api.outputFile(docId, activePath).then(
      (f) => !stale && setFile(f),
      () => !stale && setFile(null),
    );
    return () => {
      stale = true;
    };
  }, [docId, activePath, docSource]);

  // ----- geometry: rAF-throttled measurement of live anchors ---------------

  const measureRef = useRef<() => void>(() => undefined);
  measureRef.current = () => {
    const left = leftView;
    const right = rightView;
    const container = containerRef.current;
    if (!left || !right || !container || ribbons.length === 0) {
      setShapes((s) => (s.length === 0 ? s : []));
      return;
    }
    const cRect = container.getBoundingClientRect();
    const lRect = left.scrollDOM.getBoundingClientRect();
    const rRect = right.scrollDOM.getBoundingClientRect();
    const structure = structureOf(left.state);
    const totalBytes = ribbons.reduce((sum, r) => sum + r.bytes, 0);
    const leftLen = left.state.doc.length;
    const rightLen = right.state.doc.length;
    const x0 = lRect.right - cRect.left;
    const x1 = rRect.left - cRect.left;

    const out: RibbonShape[] = [];
    for (const r of ribbons) {
      // Source anchor: first line of the enclosing copy/cut/file block (its
      // chip line), falling back to the span's own first line. When that line
      // is inside a fold, lineBlockAt returns the merged folded line.
      let anchorPos = Math.min(r.sourceSpan[0], leftLen);
      let sourceHl: [number, number] = [
        Math.min(r.sourceSpan[0], leftLen),
        Math.min(r.sourceSpan[1], leftLen),
      ];
      for (const b of structure.blocks) {
        if (
          (b.name === "copy" || b.name === "cut" || b.name === "file") &&
          r.sourceSpan[0] >= b.from &&
          r.sourceSpan[1] <= b.to
        ) {
          anchorPos = Math.min(b.from, leftLen);
          sourceHl = [Math.min(b.from, leftLen), Math.min(b.to, leftLen)];
          break;
        }
      }
      const sBlock = left.lineBlockAt(anchorPos);
      const sBand = clampBand(
        sBlock.top + left.documentTop,
        sBlock.bottom + left.documentTop,
        lRect.top,
        lRect.bottom,
      );
      const t = thicknessFor(r.bytes, totalBytes);
      const s = bandAround(sBand.yTop, sBand.yBot, Math.min(t, sBand.yBot - sBand.yTop || t));

      // Output anchor: the provenance range's first→last line rects.
      const oFrom = Math.min(r.outputRange[0], rightLen);
      const oTo = Math.min(Math.max(r.outputRange[1] - 1, oFrom), rightLen);
      const oTop = right.lineBlockAt(oFrom);
      const oBot = right.lineBlockAt(oTo);
      const oBand = clampBand(
        oTop.top + right.documentTop,
        oBot.bottom + right.documentTop,
        rRect.top,
        rRect.bottom,
      );

      out.push({
        key: r.key,
        color: r.color,
        clamped: sBand.clamped || oBand.clamped,
        path: ribbonPath(
          x0,
          s.yTop - cRect.top,
          s.yBot - cRect.top,
          x1,
          oBand.yTop - cRect.top,
          oBand.yBot - cRect.top,
        ),
        sourceHl,
        outputHl: r.outputRange,
      });
    }
    setShapes(out);
  };

  // rAF-throttled measurement. In a hidden tab rAF is suspended, so fall
  // back to a short timer there — geometry is then already correct the
  // moment the tab becomes visible.
  const schedule = useCallback(() => {
    if (frameRef.current) return;
    const run = () => {
      frameRef.current = 0;
      measureRef.current();
    };
    frameRef.current = document.hidden
      ? -window.setTimeout(run, 50)
      : requestAnimationFrame(run);
  }, []);

  useEffect(
    () => () => {
      // Cancel AND reset the id: under StrictMode's double-mount a stale id
      // would make schedule() think a frame is pending forever.
      if (frameRef.current > 0) cancelAnimationFrame(frameRef.current);
      if (frameRef.current < 0) clearTimeout(-frameRef.current);
      frameRef.current = 0;
    },
    [],
  );

  // Left editor: append the highlight field + a measure trigger once per view.
  const onLeftViewReady = useCallback(
    (view: EditorView | null) => {
      setLeftView(view);
      if (view && !configured.has(view)) {
        configured.add(view);
        view.dispatch({
          effects: StateEffect.appendConfig.of([
            rangeHighlightField,
            EditorView.updateListener.of((u) => {
              if (u.docChanged || u.geometryChanged || u.viewportChanged) schedule();
            }),
          ]),
        });
      }
    },
    [schedule],
  );

  // Right editor: read-only CodeMirror over the active output file.
  useEffect(() => {
    const host = rightHostRef.current;
    if (!host || !file) {
      setRightView(null);
      return;
    }
    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: file.content,
        extensions: [
          rangeHighlightField,
          ...languageExtensions(file.language),
          EditorView.lineWrapping,
          EditorState.readOnly.of(true),
          EditorView.editable.of(false),
          EditorView.updateListener.of((u) => {
            if (u.geometryChanged || u.viewportChanged) schedule();
          }),
        ],
      }),
    });
    setRightView(view);
    return () => {
      view.destroy();
      setRightView(null);
    };
  }, [file, schedule]);

  // Scroll / resize wiring.
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

  useEffect(schedule, [ribbons, schedule]);

  // ----- hover / click lineage -----

  const highlight = (shape: RibbonShape | null) => {
    setHovered(shape?.key ?? null);
    leftView?.dispatch({
      effects: setRangeHighlights.of(shape ? [{ from: shape.sourceHl[0], to: shape.sourceHl[1] }] : []),
    });
    rightView?.dispatch({
      effects: setRangeHighlights.of(shape ? [{ from: shape.outputHl[0], to: shape.outputHl[1] }] : []),
    });
  };

  const selectBoth = (shape: RibbonShape) => {
    if (leftView) {
      const max = leftView.state.doc.length;
      leftView.dispatch({
        selection: {
          anchor: Math.min(shape.sourceHl[0], max),
          head: Math.min(shape.sourceHl[1], max),
        },
        scrollIntoView: true,
      });
    }
    if (rightView) {
      const max = rightView.state.doc.length;
      rightView.dispatch({
        selection: {
          anchor: Math.min(shape.outputHl[0], max),
          head: Math.min(shape.outputHl[1], max),
        },
        scrollIntoView: true,
      });
    }
  };

  return (
    <div className="split-view" ref={containerRef}>
      <div className="split-pane split-left">
        <DocumentEditor
          key={editorKey}
          docId={docId}
          initialSource={docSource}
          realtime={realtime}
          onChange={onChange}
          selectSpan={selectSpan}
          execBlocks={execBlocks}
          runningCells={runningCells}
          onRunCell={onRunCell}
          onViewReady={onLeftViewReady}
        />
      </div>
      <div className="split-pane split-right">
        {files && files.length > 1 && (
          <div className="output-tabs" role="tablist">
            {files.map((f) => (
              <button
                key={f.path}
                role="tab"
                aria-selected={f.path === activePath}
                className={`output-tab mono${f.path === activePath ? " on" : ""}`}
                onClick={() => setActivePath(f.path)}
              >
                {f.path}
              </button>
            ))}
          </div>
        )}
        {files !== null && files.length === 0 ? (
          <p className="muted">
            No generated outputs yet — run the document to weave its files.
          </p>
        ) : (
          <>
            {files && files.length === 1 && (
              <div className="split-file-label mono">{files[0].path}</div>
            )}
            <div ref={rightHostRef} className="split-output-editor" data-testid="split-output" />
          </>
        )}
      </div>
      <svg
        className={`ribbon-layer${hovered ? " has-hover" : ""}`}
        data-testid="ribbon-layer"
        aria-hidden="true"
      >
        {shapes.map((s) => (
          <path
            key={s.key}
            d={s.path}
            className={`ribbon ribbon-c${s.color}${s.clamped ? " ribbon-faded" : ""}${
              hovered === s.key ? " ribbon-hovered" : ""
            }`}
            onMouseEnter={() => highlight(s)}
            onMouseLeave={() => highlight(null)}
            onClick={() => selectBoth(s)}
          />
        ))}
      </svg>
    </div>
  );
}
