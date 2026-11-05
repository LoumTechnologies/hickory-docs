import type { ReactNode } from "react";
import { parseStructure } from "../editor/hickDoc";
import { registerReviewEditor } from "../lib/reviews";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { EditorState, Prec, Transaction } from "@codemirror/state";
import { Decoration, EditorView, keymap } from "@codemirror/view";
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
  const editor = host?.querySelector<HTMLElement>(".conversation-card--composer .cm-editor");
  const view = editor && EditorView.findFromDOM(editor);
  if (!view) return;
  view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
  view.focus();
}

export function ConversationEditor({ branch, path, running, stream, reasoning, draft, onDraft, onSend, canSend, sending = false, stamp, showCollapsedLineage = false, renderFinal }: {
  renderFinal?: (turn: AgentTurn) => ReactNode;
  branch: AgentTurn[]; path: string | null; running: string | null; stream: string; reasoning: string;
  draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; sending?: boolean; stamp: string; showCollapsedLineage?: boolean;
}) {
  const [data, setData] = useState<SessionViewResponse | null>(null);
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
  const messages = useMemo(() => branch.map(turn => {
    const reading = conversationReading(data, [turn], running, stream, reasoning);
    const structure = parseStructure(reading.source);
    // Only session-level messages split cards. Tags inside a message remain
    // native literate content, including executable/project constructions.
    const messages = structure.blocks.filter(block => ["user", "assistant", "reasoning"].includes(block.name)
      && !structure.blocks.some(parent => parent !== block && parent.name !== "session" && parent.from < block.from && parent.to >= block.to))
      .sort((a, b) => a.from - b.from);
    const end = reading.source.lastIndexOf("</hick:session>");
    const ranges = messages.map((block, index) => ({
      from: block.from, to: messages[index + 1]?.from ?? end, role: block.name,
    }));
    const cards = ranges.map(range => {
      const prefix = '<hick:session xmlns:hick="http://www.hickorydocs.com/1.0">\n';
      const lineOffset = 1 - reading.source.slice(0, range.from).split("\n").length + 1;
      const card = { source: prefix + reading.source.slice(range.from, range.to) + "</hick:session>\n", links: reading.links
        .filter(link => link.span[0] >= range.from && link.span[1] <= range.to)
        .map(link => ({ ...link, span: [link.span[0] - range.from + prefix.length, link.span[1] - range.from + prefix.length] as [number, number],
          lines: [link.lines[0] + lineOffset, link.lines[1] + lineOffset] as [number, number] })) };
      return { role: range.role, reading: card };
    });
    return { turn, cards };
  }), [data, branch, running, stream, reasoning]);
  return <div className="conversation-document">
    {branch.length === 0 && <div className="conversation-welcome">
      <span className="conversation-welcome__mark" aria-hidden="true">✳</span>
      <h2>What would you like to work on?</h2>
      <p>Ask a question, explore an idea, or describe a change.</p>
    </div>}
    {path && data && <VerificationStanding records={data.audit ?? []} turn={branch.at(-1)?.id} running={!!running} />}
    {path && data && <AssumptionAudit records={data.audit ?? []} path={path} source={data.source} />}
    {messages.map(({ turn, cards }) => {
      return <div className="conversation-turn" key={turn.id} data-turn={turn.id}>
        {cards.map(({ role, reading: card }, index) => {
          const message = <section className={`conversation-card conversation-card--${role}`} key={index} aria-label={`${role === "user" ? "You" : role === "reasoning" ? "Reasoning" : "Agent"} message`}>
            <header>{role === "user" ? "You" : role === "reasoning" ? "Reasoning" : "Agent"}</header>
            <ConversationBuffer reading={card} path={path} source={data?.source ?? null} id={`${turn.id}:${index}`} readOnly
              running={running} sending={sending} draft="" onDraft={() => {}} onSend={() => {}} canSend={false} showCollapsedLineage={showCollapsedLineage} />
          </section>;
          return role === "reasoning" ? <details className="conversation-reasoning" key={index}>
            <summary>{running === turn.id ? "Thinking…" : "Reasoning"}</summary>{message}
          </details> : message;
        })}
        {turn.status === "ok" && renderFinal && <section className="conversation-card conversation-card--final" aria-label="Final construction">
          <details className="conversation-review"><summary>Review project changes</summary>{renderFinal(turn)}</details>
        </section>}
      </div>;
    })}
    <section className="conversation-card conversation-card--composer" aria-label="Your response">
      <header>Message agent</header>
      <ConversationBuffer reading={{ source: "", links: [] }} path={null} source={null} id="response" readOnly={false}
        running={running} sending={sending} draft={draft} onDraft={onDraft} onSend={onSend} canSend={canSend} showCollapsedLineage={false} />
    </section>
  </div>;
}

/** Each card owns its editor and undo history; only the composer reports edits. */
function ConversationBuffer({ reading, path, source, id, readOnly, running, sending, draft, onDraft, onSend, canSend, showCollapsedLineage }: {
  reading: ReturnType<typeof conversationReading>; path: string | null; source: string | null; id: string; readOnly: boolean;
  running: string | null; sending: boolean; draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; showCollapsedLineage: boolean;
}) {
  const [view, setView] = useState<EditorView | null>(null);
  const realtime = useMemo(() => new LocalRealtime(), []);
  const following = useRef(true);
  const initialFocus = useRef(false);
  const syncLinks = useRef(() => {});
  const latest = useRef({ onDraft, onSend, canSend }); latest.current = { onDraft, onSend, canSend };
  useEffect(() => {
    if (!view || !path) return;
    return registerReviewEditor(view, path, () => source);
  }, [view, path, source]);
  const extensions = useMemo(() => [...protectedPrefix,
    // Omit only conversation envelopes from the reading, retaining exact bytes
    // and native syntax for nested constructions and the editable draft.
    ...(readOnly ? [EditorView.decorations.compute(["doc"], state => {
      const structure = parseStructure(state.doc.toString());
      const wrappers = structure.blocks.filter(block => ["user", "assistant", "reasoning"].includes(block.name)
        && !structure.blocks.some(parent => parent !== block && parent.name !== "session" && parent.from < block.from && parent.to >= block.to));
      return Decoration.set(structure.tags.filter(tag => tag.name === "session" || wrappers.some(block => tag.from === block.from || tag.to === block.to))
        .map(tag => Decoration.replace({}).range(tag.from, tag.to)), true);
    })] : []),
    EditorState.transactionFilter.of(tr => readOnly && tr.docChanged && !tr.annotation(replaceReading) ? [] : tr), sessionBookkeeping, ...responseSend(() => { if (latest.current.canSend) { following.current = true; latest.current.onSend(); } }),
    EditorView.domEventHandlers({
      mousedown(event) {
        if (event.target instanceof Element && event.target.closest(".cm-hick-fold-placeholder, .cm-foldGutter")) following.current = false;
        return false;
      },
    }),
    EditorView.contentAttributes.of({ "aria-label": readOnly ? "Recorded message" : "Agent conversation and response", "aria-readonly": String(readOnly), role: "textbox", "aria-multiline": "true" }),
    EditorView.updateListener.of(update => {
      if (update.docChanged || update.transactions.some(tr => tr.effects.some(effect => effect.is(foldEffect) || effect.is(unfoldEffect)))) queueMicrotask(() => syncLinks.current());
      if (!readOnly && update.docChanged && !update.transactions.some(tr => tr.annotation(replaceReading)))
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
    if (!readOnly && !initialFocus.current && !view.dom.closest("[hidden]")) {
      initialFocus.current = true;
      view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
      view.focus();
    }
  }, [view, reading.source, draft]);
  useEffect(() => {
    if (view) view.dispatch({ effects: setSendState.of({ enabled: canSend, label: sending ? "Starting…" : "Send", visible: !readOnly && !running }) });
  }, [view, canSend, sending, running]);
  useEffect(() => {
    if (running || sending) following.current = true;
  }, [running, sending]);
  useEffect(() => {
    if (!view || readOnly) return;
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
    observer.observe(view.dom.closest(".conversation-document") ?? view.contentDOM);
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
          key: `conversation:${path}:${id}:${link.span.join(":")}:${link.family}:${link.to.path}:${link.to.lines?.join(":")}`,
          from: { ...ribbon.from, view, ...(folded !== null ? { lines: [folded, folded] as [number, number] } : {}) },
          ...(folded === null && anchor ? { anchor } : {}), alwaysVisible: true }));
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
  return <DocumentEditor docId={`conversation:${id}`} initialSource="" realtime={realtime} readOnly={readOnly}
    execBlocks={[]} runningCells={noCells} onRunCell={() => {}} onViewReady={ready}
    preserveBytes editorExtensions={extensions} />;
}
