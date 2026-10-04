import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Prec, Transaction } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { DocumentEditor } from "../editor/DocumentEditor";
import { LocalRealtime } from "../api/realtime";
import { api } from "../api/client";
import type { AgentTurn, SessionViewResponse } from "../api/types";
import { protectedPrefix, replaceReading, responseStart, setResponseStart } from "../editor/protectedPrefix";
import { conversationReading } from "./conversationReading";
import { registerLens, ribbonLinksOf } from "../lib/lensSources";
import { conversationAnchor } from "../lib/conversationLineage";
import { sessionBookkeeping } from "../editor/sessionChrome";
import { foldedRanges } from "@codemirror/language";
import { computeEdits } from "../lib/diff";
import "./DocumentReading.css";
const noCells = new Set<string>();

export function ConversationEditor({ branch, path, running, stream, reasoning, draft, onDraft, onSend, canSend, stamp, showCollapsedLineage = false }: {
  branch: AgentTurn[]; path: string | null; running: string | null; stream: string; reasoning: string;
  draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; stamp: string; showCollapsedLineage?: boolean;
}) {
  const [data, setData] = useState<SessionViewResponse | null>(null);
  const [view, setView] = useState<EditorView | null>(null);
  const realtime = useMemo(() => new LocalRealtime(), []);
  const latest = useRef({ onDraft, onSend, canSend }); latest.current = { onDraft, onSend, canSend };
  useEffect(() => {
    setData(null);
    if (!path) return;
    let live = true, pending = false;
    const refresh = () => {
      if (pending) return;
      pending = true;
      api.sessionView(path).then(next => { if (live) setData(next); }, () => {}).finally(() => { pending = false; });
    };
    refresh();
    const timer = window.setInterval(refresh, running ? 1000 : 5000);
    return () => { live = false; window.clearInterval(timer); };
  }, [path, running, stamp]);
  const reading = useMemo(() => conversationReading(data, branch, running, stream, reasoning), [data, branch, running, stream, reasoning]);
  const extensions = useMemo(() => [...protectedPrefix, sessionBookkeeping,
    EditorView.contentAttributes.of({ "aria-label": "Agent conversation and response", role: "textbox", "aria-multiline": "true" }),
    EditorView.updateListener.of(update => {
      if (update.docChanged && !update.transactions.some(tr => tr.annotation(replaceReading)))
        latest.current.onDraft(update.state.doc.sliceString(update.state.field(responseStart)));
    }),
    Prec.high(keymap.of(["Ctrl-Enter", "Meta-Enter"].map(key => ({ key, run: () => { if (latest.current.canSend) latest.current.onSend(); return true; } })))),
  ], []);
  const ready = useCallback((editor: EditorView | null) => { setView(editor); }, []);
  useEffect(() => {
    if (!view) return;
    const next = reading.source + draft;
    const before = view.state.doc.toString();
    view.dispatch({ changes: computeEdits(before, next).map(e => ({ from: e.start, to: e.end, insert: e.text })),
      effects: setResponseStart.of(reading.source.length), annotations: [replaceReading.of(true), Transaction.addToHistory.of(false)] });
  }, [view, reading.source, draft]);
  useEffect(() => {
    if (!view || !path) return;
    // Links refer only to recorded session bytes. The editable draft has no receipts.
    let unregister = () => {};
    const sync = () => {
      const visible = reading.links.flatMap(link => {
        const anchor = conversationAnchor(view.dom, link.span[0], link.span[1], showCollapsedLineage);
        const isBookkeeping = view.dom.querySelector(`[data-session-from="${link.span[0]}"]`);
        if (isBookkeeping && !anchor) return [];
        let hidden = false;
        foldedRanges(view.state).between(0, view.state.doc.length, (from, to) => { if (from <= link.span[0] && to >= link.span[1]) hidden = true; });
        if (hidden && !showCollapsedLineage) return [];
        return ribbonLinksOf(path, [link]).map(ribbon => ({ ...ribbon,
          key: `conversation:${path}:${link.span.join(":")}:${link.family}:${link.to.path}:${link.to.lines?.join(":")}`,
          ...(anchor ? { anchor, alwaysVisible: true } : {}) }));
      });
      unregister(); unregister = registerLens({ path, view, source: view.state.doc.toString(), links: visible });
    };
    sync();
    view.dom.addEventListener("toggle", sync, true);
    view.dom.addEventListener("click", sync);
    return () => { unregister(); view.dom.removeEventListener("toggle", sync, true); view.dom.removeEventListener("click", sync); };
  }, [view, path, reading, draft, showCollapsedLineage]);
  useEffect(() => () => realtime.close(), [realtime]);
  return <div className="conversation-document">
    <DocumentEditor docId="conversation-reading" initialSource="" realtime={realtime}
      execBlocks={[]} runningCells={noCells} onRunCell={() => {}} onViewReady={ready}
      preserveBytes editorExtensions={extensions} />
  </div>;
}
