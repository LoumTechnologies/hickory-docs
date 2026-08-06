import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import * as Y from "yjs";
import { Awareness } from "y-protocols/awareness";
import { yCollab } from "y-codemirror.next";
import { CellRegistry, EnvRegistry, setVerifiedExpects, structureOf, wysiwyg } from "./wysiwyg";
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

    // Seed only when the shared doc is empty (fresh doc / mock mode); when
    // collaborating, the server's sync step supplies the content.
    if (ytext.length === 0 && initialSource.length > 0) {
      ytext.insert(0, initialSource);
    }

    const view = new EditorView({
      parent: host,
      state: EditorState.create({
        doc: ytext.toString(),
        extensions: [
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
          wysiwyg(registry, envRegistry),
          yCollab(ytext, awareness),
          EditorView.lineWrapping,
          EditorView.updateListener.of((u) => {
            if (u.docChanged) onChange?.(u.state.doc.toString());
          }),
        ],
      }),
    });
    viewRef.current = view;
    setSlots(registry.list());
    setEnvSlots(envRegistry.list());

    return () => {
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
