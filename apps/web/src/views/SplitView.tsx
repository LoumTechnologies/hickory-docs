// Split (lineage) view: the document on the left, the tree of generated files
// in the middle, the generated text on the right, and an SVG ribbon layer
// threading all three — a Sankey picture of the weave that you can edit from
// either end.
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
//  - A ribbon covers its WHOLE block on the source side, not a token at its
//    start: the band runs from the first line's top to the last line's bottom,
//    so the ribbon visibly carries the entire fragment into the output.
//  - Each file node's height is that file's byte budget, subdivided among the
//    fragments feeding it in output order — the middle column is a real Sankey
//    stage, not a legend.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { StateEffect } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { api } from "../api/client";
import type { ExecBlock, OutputFile, OutputFileMeta, SourceEdit } from "../api/types";
import type { Realtime } from "../api/realtime";
import { DocumentEditor } from "../editor/DocumentEditor";
import { OutputEditorPane, type ProvChar } from "../components/OutputEditorPane";
import { OutputTree } from "../components/OutputTree";
import { structureOf } from "../editor/wysiwyg";
import { rangeHighlightField, setRangeHighlights } from "../editor/rangeHighlight";
import { deriveRibbons, RIBBON_PALETTE_SIZE, type Ribbon } from "../lib/ribbons";
import { atLeast, clampBand, ribbonPathVia, ribbonStubPath } from "../lib/ribbonGeometry";

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
  /** An output edit resolved back into the document. */
  onSourceEdited: (edits: SourceEdit[]) => void;
}

/** A ribbon plus the file it flows into. */
interface FileRibbon extends Ribbon {
  filePath: string;
}

interface RibbonShape {
  key: string;
  color: number;
  path: string;
  clamped: boolean;
  /** Source block span to highlight/select (char offsets, left editor). */
  sourceHl: [number, number];
  /** Output range to highlight/select (char offsets, right editor). */
  outputHl: [number, number] | null;
  filePath: string;
}

/** Views that already received the appended highlight/measure config. */
const configured = new WeakSet<EditorView>();

/** Cap on files fetched for the tree — a document that weaves hundreds of
 * files should not turn opening Split into a hundred requests. */
const MAX_TREE_FILES = 24;

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
  onSourceEdited,
}: SplitViewProps) {
  const [files, setFiles] = useState<OutputFileMeta[] | null>(null);
  const [activePath, setActivePath] = useState<string | null>(null);
  const [loaded, setLoaded] = useState<Map<string, OutputFile>>(new Map());
  const [leftView, setLeftView] = useState<EditorView | null>(null);
  const [rightView, setRightView] = useState<EditorView | null>(null);
  const [shapes, setShapes] = useState<RibbonShape[]>([]);
  const [hovered, setHovered] = useState<string | null>(null);
  const [lineage, setLineage] = useState<ProvChar[]>([]);

  const containerRef = useRef<HTMLDivElement>(null);
  const frameRef = useRef(0);
  const rowsRef = useRef(new Map<string, HTMLElement>());

  const file = activePath ? (loaded.get(activePath) ?? null) : null;

  // ----- output files: the tree, and provenance for every file -------------

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
    if (!files) return;
    let stale = false;
    const wanted = files.slice(0, MAX_TREE_FILES).map((f) => f.path);
    Promise.all(
      wanted.map((p) => api.outputFile(docId, p).then((f) => [p, f] as const, () => null)),
    ).then((entries) => {
      if (stale) return;
      const next = new Map<string, OutputFile>();
      for (const e of entries) if (e) next.set(e[0], e[1]);
      setLoaded(next);
    });
    return () => {
      stale = true;
    };
  }, [docId, files, docSource]);

  // Ribbons for EVERY file, colored per distinct source fragment across the
  // whole weave: one copy block feeding two files keeps one color.
  const ribbons = useMemo(() => {
    const colors = new Map<string, number>();
    const out: FileRibbon[] = [];
    for (const [path, f] of loaded) {
      for (const r of deriveRibbons(f, docPath, docSource)) {
        const fragKey = `${r.sourceByteSpan[0]}:${r.sourceByteSpan[1]}`;
        let color = colors.get(fragKey);
        if (color === undefined) {
          color = colors.size % RIBBON_PALETTE_SIZE;
          colors.set(fragKey, color);
        }
        out.push({ ...r, color, filePath: path });
      }
    }
    return out;
  }, [loaded, docPath, docSource]);

  const bytesPerFile = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of ribbons) m.set(r.filePath, (m.get(r.filePath) ?? 0) + r.bytes);
    return m;
  }, [ribbons]);

  const colorPerFile = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of ribbons) if (!m.has(r.filePath)) m.set(r.filePath, r.color);
    return m;
  }, [ribbons]);

  // ----- geometry: rAF-throttled measurement of live anchors ---------------

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
    const structure = structureOf(left.state);
    const leftLen = left.state.doc.length;
    const rightLen = rightView?.state.doc.length ?? 0;
    const x0 = lRect.right - cRect.left;
    const x1 = rRect ? rRect.left - cRect.left : x0;

    // The whole enclosing block on the source side, resolved once per ribbon.
    const blockOf = (r: FileRibbon): [number, number] => {
      for (const b of structure.blocks) {
        if (
          (b.name === "copy" || b.name === "cut" || b.name === "file") &&
          r.sourceSpan[0] >= b.from &&
          r.sourceSpan[1] <= b.to
        ) {
          return [Math.min(b.from, leftLen), Math.min(b.to, leftLen)];
        }
      }
      return [Math.min(r.sourceSpan[0], leftLen), Math.min(r.sourceSpan[1], leftLen)];
    };

    const out: RibbonShape[] = [];
    // Group by file so each node's band can be subdivided by byte weight.
    const byFile = new Map<string, FileRibbon[]>();
    for (const r of ribbons) {
      const list = byFile.get(r.filePath);
      if (list) list.push(r);
      else byFile.set(r.filePath, [r]);
    }

    for (const [path, list] of byFile) {
      const row = rowsRef.current.get(path);
      if (!row) continue;
      const nRect = row.getBoundingClientRect();
      const nodeLeft = nRect.left - cRect.left;
      const nodeRight = nRect.right - cRect.left;
      const isActive = path === activePath;

      const ordered = [...list].sort((a, b) => a.outputRange[0] - b.outputRange[0]);
      const totalBytes = ordered.reduce((s, r) => s + r.bytes, 0) || 1;
      // Inactive files collapse their fragments into one stub per source block
      // — one file that a block feeds is one statement, not fifty.
      const groups: { ribbons: FileRibbon[]; bytes: number }[] = [];
      if (isActive) {
        for (const r of ordered) groups.push({ ribbons: [r], bytes: r.bytes });
      } else {
        const byBlock = new Map<string, { ribbons: FileRibbon[]; bytes: number }>();
        for (const r of ordered) {
          const [bf, bt] = blockOf(r);
          const k = `${bf}:${bt}`;
          const g = byBlock.get(k);
          if (g) {
            g.ribbons.push(r);
            g.bytes += r.bytes;
          } else byBlock.set(k, { ribbons: [r], bytes: r.bytes });
        }
        groups.push(...byBlock.values());
      }

      let cursor = nRect.top;
      for (const g of groups) {
        const r = g.ribbons[0];
        const share = (g.bytes / totalBytes) * nRect.height;
        const nTop = cursor;
        const nBot = cursor + Math.max(share, 1);
        cursor = nBot;

        const [bf, bt] = blockOf(r);
        // Source band spans the ENTIRE block: first line top → last line bottom.
        const sTopBlock = left.lineBlockAt(bf);
        const sBotBlock = left.lineBlockAt(bt);
        const sBand = clampBand(
          sTopBlock.top + left.documentTop,
          sBotBlock.bottom + left.documentTop,
          lRect.top,
          lRect.bottom,
        );
        const s = atLeast(sBand.yTop, sBand.yBot);

        if (isActive && rRect && rightView) {
          const oFrom = Math.min(r.outputRange[0], rightLen);
          const oTo = Math.min(Math.max(r.outputRange[1] - 1, oFrom), rightLen);
          const oTopBlock = rightView.lineBlockAt(oFrom);
          const oBotBlock = rightView.lineBlockAt(oTo);
          const oBand = clampBand(
            oTopBlock.top + rightView.documentTop,
            oBotBlock.bottom + rightView.documentTop,
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
            sourceHl: [bf, bt],
            outputHl: r.outputRange,
          });
        } else {
          out.push({
            key: `${path}:${bf}:${bt}`,
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
            sourceHl: [bf, bt],
            outputHl: null,
          });
        }
      }
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

  const registerRow = useCallback(
    (path: string, el: HTMLElement | null) => {
      if (el) rowsRef.current.set(path, el);
      else rowsRef.current.delete(path);
      schedule();
    },
    [schedule],
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

  const rightExtensions = useMemo(
    () => [
      rangeHighlightField,
      EditorView.updateListener.of((u) => {
        if (u.docChanged || u.geometryChanged || u.viewportChanged) schedule();
      }),
    ],
    [schedule],
  );

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

  useEffect(schedule, [ribbons, activePath, schedule]);

  // ----- hover / click lineage -----

  const highlight = (shape: RibbonShape | null) => {
    setHovered(shape?.key ?? null);
    leftView?.dispatch({
      effects: setRangeHighlights.of(
        shape ? [{ from: shape.sourceHl[0], to: shape.sourceHl[1] }] : [],
      ),
    });
    rightView?.dispatch({
      effects: setRangeHighlights.of(
        shape?.outputHl ? [{ from: shape.outputHl[0], to: shape.outputHl[1] }] : [],
      ),
    });
  };

  const selectBoth = (shape: RibbonShape) => {
    if (shape.filePath !== activePath) {
      setActivePath(shape.filePath);
      return;
    }
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
    if (rightView && shape.outputHl) {
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

      <div className="split-middle">
        <OutputTree
          files={files ?? []}
          activePath={activePath}
          onSelect={setActivePath}
          registerRow={registerRow}
          colorOf={(p) => colorPerFile.get(p)}
          bytesOf={(p) => bytesPerFile.get(p)}
        />
        {lineage.length > 0 && (
          <div className="tree-lineage" data-testid="tree-lineage">
            {lineage.slice(0, 3).map((p, i) => (
              <p key={i} className={p.origin.kind === "synthetic" ? "prov-synthetic" : ""}>
                {p.origin.kind === "synthetic"
                  ? "weaver-generated — not editable"
                  : `${p.origin.kind} from ${p.origin.doc_path}`}
              </p>
            ))}
          </div>
        )}
      </div>

      <div className="split-pane split-right">
        {files !== null && files.length === 0 ? (
          <p className="muted">No generated outputs yet — run the document to weave its files.</p>
        ) : file ? (
          <OutputEditorPane
            key={file.path}
            docId={docId}
            file={file}
            className="split-output-editor"
            testId="split-output"
            extensions={rightExtensions}
            onViewReady={setRightView}
            onLineage={setLineage}
            onSaved={onSourceEdited}
          />
        ) : (
          <p className="muted">Loading output…</p>
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
