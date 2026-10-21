import { registerReviewEditor } from "../lib/reviews";
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
import { responseSend, setSendState } from "../editor/responseSend";
import { conversationFold, sessionBookkeeping } from "../editor/sessionChrome";
import { foldEffect, unfoldEffect } from "@codemirror/language";
import { computeEdits } from "../lib/diff";
import "./DocumentReading.css";
import { AssumptionAudit, VerificationStanding } from "../components/AssumptionAudit";
const noCells = new Set<string>();

export function focusConversationResponse(host: Element | null) {
  const editor = host?.querySelector<HTMLElement>(".cm-editor");
  const view = editor && EditorView.findFromDOM(editor);
  if (!view) return;
  view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
  view.focus();
}

export function ConversationEditor({ branch, path, running, stream, reasoning, draft, onDraft, onSend, canSend, sending = false, stamp, showCollapsedLineage = false }: {
  branch: AgentTurn[]; path: string | null; running: string | null; stream: string; reasoning: string;
  draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; sending?: boolean; stamp: string; showCollapsedLineage?: boolean;
}) {
  const [data, setData] = useState<SessionViewResponse | null>(null);
  const [view, setView] = useState<EditorView | null>(null);
  const realtime = useMemo(() => new LocalRealtime(), []);
  const following = useRef(true);
  const initialFocus = useRef(false);
  const syncLinks = useRef(() => {});
  const latest = useRef({ onDraft, onSend, canSend }); latest.current = { onDraft, onSend, canSend };
  useEffect(() => { setData(null); }, [path]);
  useEffect(() => {
    if (!path) return;
    let live = true, pending = false;
    const refresh = () => {
      if (pending) return;
      pending = true;
      api.sessionView(path).then(next => { if (live) setData(next); }, () => {}).finally(() => { pending = false; });
    };
    refresh();
    // Retain one recorded capture during a running turn; live output is
    // streamed below. Repeatedly parsing/transferring its growing context
    // journal can obstruct the editor-capture replies needed for completion.
    // The separate session-opening path still exposes current recorded work.
    const timer = running ? undefined : window.setInterval(refresh, 5000);
    return () => { live = false; window.clearInterval(timer); };
  }, [path, running, stamp]);
  const reading = useMemo(() => conversationReading(data, branch, running, stream, reasoning), [data, branch, running, stream, reasoning]);
  useEffect(() => {
    if (!view || !path) return;
    return registerReviewEditor(view, path, () => data?.source ?? null);
  }, [view, path, data?.source]);
  const extensions = useMemo(() => [...protectedPrefix, sessionBookkeeping, ...responseSend(() => { if (latest.current.canSend) { following.current = true; latest.current.onSend(); } }),
    EditorView.domEventHandlers({
      mousedown(event) {
        if (event.target instanceof Element && event.target.closest(".cm-hick-fold-placeholder, .cm-foldGutter")) following.current = false;
        return false;
      },
    }),
    EditorView.contentAttributes.of({ "aria-label": "Agent conversation and response", role: "textbox", "aria-multiline": "true" }),
    EditorView.updateListener.of(update => {
      if (update.docChanged || update.transactions.some(tr => tr.effects.some(effect => effect.is(foldEffect) || effect.is(unfoldEffect)))) queueMicrotask(() => syncLinks.current());
      if (update.docChanged && !update.transactions.some(tr => tr.annotation(replaceReading)))
        latest.current.onDraft(update.state.doc.sliceString(update.state.field(responseStart)));
    }),
    Prec.high(keymap.of(["Ctrl-Enter", "Meta-Enter"].map(key => ({ key, run: () => { if (latest.current.canSend) { following.current = true; latest.current.onSend(); } return true; } })))),
  ], []);
  const ready = useCallback((editor: EditorView | null) => { setView(editor); }, []);
  useEffect(() => {
    if (!view) return;
    const next = reading.source + draft;
    const before = view.state.doc.toString();
    view.dispatch({ changes: computeEdits(before, next).map(e => ({ from: e.start, to: e.end, insert: e.text })),
      effects: setResponseStart.of(reading.source.length), annotations: [replaceReading.of(true), Transaction.addToHistory.of(false)] });
    if (!initialFocus.current && !view.dom.closest("[hidden]")) {
      initialFocus.current = true;
      view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
      view.focus();
    }
  }, [view, reading.source, draft]);
  useEffect(() => {
    if (view) view.dispatch({ effects: setSendState.of({ enabled: canSend, label: sending ? "Starting…" : "Send", visible: !running }) });
  }, [view, canSend, sending, running]);
  useEffect(() => {
    if (running || sending) following.current = true;
  }, [running, sending]);
  useEffect(() => {
    if (!view) return;
    const log = view.dom.closest<HTMLElement>(".chat-log");
    if (!log) return;
    let previousTop = log.scrollTop;
    const onScroll = () => {
      if (log.scrollTop < previousTop) following.current = false;
      if (log.scrollHeight - log.clientHeight - log.scrollTop < 32) following.current = true;
      previousTop = log.scrollTop;
    };
    let frame = 0;
    const follow = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(() => {
        if (following.current) log.scrollTop = log.scrollHeight;
      });
    };
    log.addEventListener("scroll", onScroll);
    const observer = new ResizeObserver(follow);
    observer.observe(view.contentDOM);
    follow();
    return () => { cancelAnimationFrame(frame); observer.disconnect(); log.removeEventListener("scroll", onScroll); };
  }, [view, reading.source, sending]);
  useEffect(() => {
    if (!view || !path) return;
    // Links refer only to recorded session bytes. The editable draft has no receipts.
    let unregister = () => {};
    const sync = () => {
      const visible = reading.links.flatMap(link => {
        const anchor = conversationAnchor(view.dom, link.span[0], link.span[1], showCollapsedLineage);
        const folded = conversationFold(view, link.span[0], link.span[1]);
        if (folded !== null && !showCollapsedLineage) return [];
        return ribbonLinksOf(path, [link]).map(ribbon => ({ ...ribbon,
          key: `conversation:${path}:${link.span.join(":")}:${link.family}:${link.to.path}:${link.to.lines?.join(":")}`,
          ...(folded !== null ? { from: { path, lines: [folded, folded] as [number, number] } } : anchor ? { anchor } : {}), alwaysVisible: true }));
      });
      unregister(); unregister = registerLens({ path, view, source: view.state.doc.toString(), links: visible });
    };
    syncLinks.current = sync;
    sync();
    view.dom.addEventListener("toggle", sync, true);
    view.dom.addEventListener("click", sync);
    return () => { syncLinks.current = () => {}; unregister(); view.dom.removeEventListener("toggle", sync, true); view.dom.removeEventListener("click", sync); };
  }, [view, path, reading, draft, showCollapsedLineage]);
  useEffect(() => () => realtime.close(), [realtime]);
  return <div className="conversation-document">
    {running && path && <p className="muted">Recorded history is held at its last loaded capture during this turn. Current text streams below; open the session to inspect recorded work.</p>}
    {path && data && <VerificationStanding records={data.audit ?? []} turn={branch.at(-1)?.id} running={!!running} />}
    {path && data && <AssumptionAudit records={data.audit ?? []} path={path} source={data.source} />}
    <DocumentEditor docId="conversation-reading" initialSource="" realtime={realtime}
      execBlocks={[]} runningCells={noCells} onRunCell={() => {}} onViewReady={ready}
      preserveBytes editorExtensions={extensions} />
  </div>;
}
