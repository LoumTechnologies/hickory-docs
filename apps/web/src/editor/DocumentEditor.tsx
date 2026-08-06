import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import { CellRegistry, EnvRegistry, setVerifiedExpects, structureOf, wysiwyg } from "./wysiwyg";
import { hickoryFolding } from "./folding";
import type { CellSlot, EnvSlot } from "./wysiwyg";
import { execBlocksOf, expectRangeOf } from "./hickDoc";
import { CellPanel } from "../components/CellPanel";
import { EnvCard } from "../components/EnvCard";
import { api } from "../api/client";
import type { Realtime } from "../api/realtime";
import type { ExecBlock, ExecutorInfo } from "../api/types";

export interface DocumentEditorProps {
  docId: string;
  /** Initial .hick source, used to seed the Y.Doc when it is empty. */
  initialSource: string;
  realtime: Realtime;
  onChange?: (source: string) => void;
  /** Byte span to select (provenance click-through from the Output view). */
  selectSpan?: [number, number] | null;
  /** Rendered exec blocks (statuses/transcripts) to attach to cells. */
  execBlocks: ExecBlock[];
  runningCells: Set<string>;
  onRunCell: (execId: string) => void;
  /** Fired with the live EditorView on mount and null on teardown (the Split
   * view uses it to measure ribbon anchors against real geometry). */
  onViewReady?: (view: EditorView | null) => void;
}

/** Overlap length of two [from, to) spans. */
function overlap(a: [number, number], b: [number, number]): number {
  return Math.max(0, Math.min(a[1], b[1]) - Math.max(a[0], b[0]));
}

/**
 * Match a widget slot (source span of the exec block as currently typed) to
 * the server-rendered exec block: best span overlap first, ordinal fallback
 * (spans drift while the user edits above a cell).
 */
export function matchExecBlock(
  slot: { span: [number, number]; index: number },
  blocks: ExecBlock[],
): ExecBlock | undefined {
  let best: ExecBlock | undefined;
  let bestOverlap = 0;
  for (const b of blocks) {
    const o = overlap(slot.span, b.span);
    if (o > bestOverlap) {
      bestOverlap = o;
      best = b;
    }
  }
  return best ?? blocks[slot.index];
}

/**
 * The Document view: ONE CodeMirror instance over the raw .hick source with
 * Typora-style decorations (see editor/wysiwyg.ts) and collaborative editing
 * via Yjs — the user always edits real source; styling never replaces text.
 * Exec-cell panels are React components rendered into CM block widgets
 * through portals.
 */
export function DocumentEditor({
  docId,
  initialSource,
  realtime,
  onChange,
  selectSpan,
  execBlocks,
  runningCells,
  onRunCell,
  onViewReady,
}: DocumentEditorProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const registry = useMemo(() => new CellRegistry(), []);
  const envRegistry = useMemo(() => new EnvRegistry(), []);
  const [slots, setSlots] = useState<CellSlot[]>([]);
  const [envSlots, setEnvSlots] = useState<EnvSlot[]>([]);
  const [executorInfo, setExecutorInfo] = useState<ExecutorInfo | null>(null);

  useEffect(() => registry.subscribe(() => setSlots(registry.list())), [registry]);
  useEffect(
    () => envRegistry.subscribe(() => setEnvSlots(envRegistry.list())),
    [envRegistry],
  );

  // Widget DOM is filled by React portals AFTER CodeMirror measures the
  // (initially empty) slot elements, and panel content keeps changing size
  // (transcripts stream in, replay toggles). CM caches per-line heights, so
  // without a re-measure every vertical cursor motion works from stale
  // geometry — the classic "ArrowUp jumps half a screen" bug. Observe every
  // slot and ask CM to re-measure whenever one resizes.
  useEffect(() => {
    if (typeof ResizeObserver === "undefined") return;
    let last = new Map<Element, number>();
    let scheduled = false;
    const observer = new ResizeObserver((entries) => {
      // Only re-measure when a slot's height actually changed — Chrome fires
      // an initial callback per observe(), and an unconditional
      // requestMeasure can feed back into layout and loop.
      let changed = false;
      for (const e of entries) {
        const h = Math.round(e.contentRect.height);
        if (last.get(e.target) !== h) {
          last.set(e.target, h);
          changed = true;
        }
      }
      if (!changed || scheduled) return;
      scheduled = true;
      requestAnimationFrame(() => {
        scheduled = false;
        viewRef.current?.requestMeasure();
      });
    });
    for (const slot of slots) observer.observe(slot.el);
    for (const slot of envSlots) observer.observe(slot.el);
    return () => {
      observer.disconnect();
      last = new Map();
    };
  }, [slots, envSlots]);
  useEffect(() => {
    api.executor().then(setExecutorInfo, () => setExecutorInfo(null));
  }, []);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const ydoc = new Y.Doc();
    const ytext = ydoc.getText("source");
    const awareness = new Awareness(ydoc);
    awareness.setLocalStateField("user", {
      name: localStorage.getItem("hickory.name") ?? "anonymous",
      color: "#8f6f3f",
    });
    realtime.bindDoc(ydoc, awareness);

    // Seeding is ONLY for realtimes with no server behind them (mock mode).
    // With a server, the room's Y.Doc is built from `docs.source`; seeding
    // here mints an independent copy of the same text under a different
    // client id, and the CRDT merges the two by concatenation — the document
    // doubles on every single connect. (Measured: a 15KB doc reached 49MB.)
    let cancelled = false;
    if (!realtime.serverAuthoritative) {
      void realtime.whenSynced().then(() => {
        if (!cancelled && ytext.length === 0 && initialSource.length > 0) {
          ytext.insert(0, initialSource);
        }
      });
    }

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          wysiwyg(registry, envRegistry),
          hickoryFolding(),
          yCollab(ytext, awareness),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onChange?.(u.state.doc.toString());
          }),
        ],
      }),
    });
    viewRef.current = view;
    // Debug handle for driving the editor from automation (kept out of the
    // normal path; enable with localStorage "hickory.debug" = "1").
    if (localStorage.getItem("hickory.debug") === "1") {
      (window as unknown as { __hickoryView?: EditorView }).__hickoryView = view;
    }
    setSlots(registry.list());
    setEnvSlots(envRegistry.list());
    onViewReady?.(view);

    return () => {
      cancelled = true;
      onViewReady?.(null);
      view.destroy();
      viewRef.current = null;
      awareness.destroy();
      ydoc.destroy();
    };
    // Recreate the editor per document.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [docId, realtime, registry, envRegistry]);

  // Push verified-expect ranges into the editor: for each exec cell whose
  // last run is ok AND that has an expect block, style the expect body in the
  // source as the verified output (the single visible copy — see CellPanel).
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const structure = structureOf(view.state);
    const spans: [number, number][] = [];
    execBlocksOf(structure).forEach((exec, index) => {
      const block = matchExecBlock({ span: [exec.from, exec.to], index }, execBlocks);
      if (block?.status === "ok" && block.expect) {
        const range = expectRangeOf(structure, exec);
        if (range) spans.push(range);
      }
    });
    view.dispatch({ effects: setVerifiedExpects.of(spans) });
  }, [execBlocks]);

  useEffect(() => {
    const view = viewRef.current;
    if (!view || !selectSpan) return;
    const max = view.state.doc.length;
    const from = Math.min(selectSpan[0], max);
    const to = Math.min(selectSpan[1], max);
    view.dispatch({
      selection: { anchor: from, head: to },
      scrollIntoView: true,
    });
    view.focus();
  }, [selectSpan]);

  return (
    <>
      <div ref={hostRef} className="document-editor" />
      {slots.map((slot) => {
        const block = matchExecBlock(slot, execBlocks);
        return createPortal(
          <CellPanel
            block={block}
            running={block ? runningCells.has(block.id) : false}
            onRun={onRunCell}
          />,
          slot.el,
          slot.key,
        );
      })}
      {envSlots.map((slot) =>
        createPortal(
          <EnvCard
            name={slot.name}
            image={slot.image}
            rules={slot.rules}
            executor={executorInfo}
          />,
          slot.el,
          slot.key,
        ),
      )}
    </>
  );
}
