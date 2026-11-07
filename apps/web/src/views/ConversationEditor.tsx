import type { ReactNode } from "react";
import { parseStructure } from "../editor/hickDoc";
import { registerReviewEditor } from "../lib/reviews";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { isolateHistory } from "@codemirror/commands";
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

/** Browser recognition produces draft text only. No audio enters the agent API. */
interface DictationRecognition {
  continuous: boolean; interimResults: boolean; lang: string;
  onstart: (() => void) | null;
  onresult: ((event: { results: ArrayLike<{ isFinal: boolean; [index: number]: { transcript: string } }> }) => void) | null;
  onerror: ((event: { error: string }) => void) | null;
  onend: (() => void) | null;
  start(): void; stop(): void; abort(): void;
}
function DictationButton({ disabled, onText, onActive, cancelRef }: { cancelRef?: { current: (() => void) | null }; disabled: boolean; onText: (text: string) => void; onActive: (active: boolean) => void }) {
  const recognizer = useRef<DictationRecognition | null>(null);
  const callbacks = useRef({ onText, onActive }); callbacks.current = { onText, onActive };
  const finishTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const [active, setActive] = useState(false);
  const [finishing, setFinishing] = useState(false);
  const [status, setStatus] = useState("");
  const release = (abort: boolean) => {
    clearTimeout(finishTimer.current);
    const current = recognizer.current; recognizer.current = null;
    if (current) {
      current.onstart = current.onresult = current.onerror = current.onend = null;
      if (abort) { try { current.abort(); } catch { /* Already ended. */ } }
    }
    setActive(false); setFinishing(false); callbacks.current.onActive(false);
  };
  useEffect(() => {
    const cancel = () => { release(true); setStatus(""); };
    if (cancelRef) cancelRef.current = cancel;
    return () => { if (cancelRef?.current === cancel) cancelRef.current = null; release(true); };
  }, [cancelRef]);
  useEffect(() => { if (disabled) { release(true); setStatus(""); } }, [disabled]);
  const start = () => {
    const host = window as unknown as { SpeechRecognition?: new () => DictationRecognition; webkitSpeechRecognition?: new () => DictationRecognition };
    const Recognition = host.SpeechRecognition ?? host.webkitSpeechRecognition;
    if (!Recognition) { setStatus("Dictation is unavailable in this browser. Use your system's dictation or type your message."); return; }
    let current: DictationRecognition;
    try { current = new Recognition(); }
    catch { setStatus("Could not start dictation. Try again or use your system's dictation."); return; }
    recognizer.current = current;
    const seen = new Set<number>();
    const live = () => recognizer.current === current;
    current.continuous = true; current.interimResults = true; current.lang = navigator.language || "en-US";
    current.onstart = () => { if (live()) setStatus("Listening…"); };
    current.onresult = event => {
      if (!live()) return;
      const final: string[] = [], interim: string[] = [];
      for (let index = 0; index < event.results.length; index++) {
        const result = event.results[index], text = result[0]?.transcript?.trim();
        if (!text) continue;
        if (result.isFinal) { if (!seen.has(index)) { seen.add(index); final.push(text); } }
        else interim.push(text);
      }
      if (final.length) callbacks.current.onText(final.join(" "));
      setStatus(interim.length ? interim.join(" ").slice(0, 500) : "Listening…");
    };
    current.onerror = event => {
      if (!live()) return;
      const messages: Record<string, string> = {
        "not-allowed": "Microphone or speech recognition access was denied. Allow access in your browser settings, then retry.",
        "service-not-allowed": "Speech recognition is blocked. Check your browser settings or use your system's dictation.",
        "audio-capture": "No microphone is available. Connect a microphone and retry.",
        "no-speech": "No speech was detected. Check your microphone and try dictating again.",
        "network": "Speech recognition could not connect. Check your connection and retry, or type your message.",
        "language-not-supported": "Your browser cannot dictate in this language. Change your browser language or use your system's dictation.",
      };
      release(true);
      setStatus(event.error === "aborted" ? "" : messages[event.error] ?? "Dictation failed. Try again or type your message; completed text is kept.");
    };
    current.onend = () => { if (live()) { release(false); setStatus(""); } };
    setActive(true); callbacks.current.onActive(true); setStatus("Starting dictation…");
    try { current.start(); }
    catch { release(true); setStatus("Could not start dictation. Allow microphone access and retry, or type your message."); }
  };
  const stop = () => {
    const current = recognizer.current;
    if (!current) return;
    setFinishing(true); setStatus("Finishing dictation…");
    // Keep the final-result callback until onend, but bound a stalled service.
    finishTimer.current = setTimeout(() => {
      if (recognizer.current === current) { release(true); setStatus("Dictation took too long to finish. Completed text is kept; try again."); }
    }, 5000);
    try { current.stop(); } catch { release(true); setStatus("Dictation stopped. Completed text is kept; you can retry."); }
  };
  return <span style={{ display: "inline-flex", position: "relative" }}><button type="button" aria-label={active ? "Stop dictation" : "Dictate"} disabled={disabled || finishing} aria-pressed={active}
    title={active ? "Stop dictation" : "Dictate"}
    onMouseDown={event => event.preventDefault()} onClick={() => active ? stop() : start()}><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" aria-hidden="true"><rect x="9" y="2" width="6" height="12" rx="3"/><path d="M5 10v2a7 7 0 0 0 14 0v-2M12 19v3M8 22h8"/></svg></button>
    <span role="status" aria-label="Dictation status" title={status} style={{ position: "absolute", bottom: "calc(100% + 4px)", left: 0, width: "min(360px, 70vw)", maxHeight: 96, overflow: "auto", zIndex: 10, padding: status ? "4px 6px" : 0, background: "var(--bg-raised)", color: "var(--fg)", boxShadow: status ? "var(--shadow)" : undefined }}>{status}</span></span>;
}

export function ConversationEditor({ dictationScope = "response", onDictatingChange, cancelDictation, hasAttachments = false, onFiles, composerTools, branch, path, running, stream, reasoning, draft, onDraft, onSend, canSend, sending = false, stamp, showCollapsedLineage = false, renderFinal }: {
  cancelDictation?: { current: (() => void) | null }; dictationScope?: string; onDictatingChange?: (active: boolean) => void;
  renderFinal?: (turn: AgentTurn) => ReactNode;
  hasAttachments?: boolean; onFiles?: (files: File[]) => void; composerTools?: ReactNode;
  branch: AgentTurn[]; path: string | null; running: string | null; stream: string; reasoning: string;
  draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; sending?: boolean; stamp: string; showCollapsedLineage?: boolean;
}) {
  const composer = useRef<EditorView | null>(null);
  const scope = useRef(dictationScope); scope.current = dictationScope;
  const [dictating, setDictating] = useState(false);
  const dictate = (text: string, expectedScope: string) => {
    const view = composer.current;
    if (!view || scope.current !== expectedScope || sending) return;
    const selection = view.state.selection.main;
    const from = Math.max(view.state.field(responseStart), selection.from);
    const to = Math.max(from, selection.to);
    const before = view.state.doc.sliceString(Math.max(0, from - 1), from);
    const after = view.state.doc.sliceString(to, to + (to < view.state.doc.length ? 1 : 0));
    const insert = (before && !/[\s([{]/.test(before) ? " " : "") + text + (after && !/[\s,.;:!?)}\]]/.test(after) ? " " : "");
    view.dispatch({ changes: { from, to, insert }, selection: { anchor: from + insert.length },
      annotations: isolateHistory.of("full"), userEvent: "input.dictation", scrollIntoView: true });
  };
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
    <section className="conversation-card conversation-card--composer" aria-label="Your response"
      onPasteCapture={event => { const files = Array.from(event.clipboardData.files); if (files.length && onFiles) { event.preventDefault(); event.stopPropagation(); onFiles(files); } }}
      onDragOver={event => { if (onFiles && event.dataTransfer.types.includes("Files")) event.preventDefault(); }}
      onDropCapture={event => { const files = Array.from(event.dataTransfer.files); if (files.length && onFiles) { event.preventDefault(); event.stopPropagation(); onFiles(files); } }}>
      <div aria-label="Chat input tools" style={{ display: "flex", alignItems: "center", gap: 8, flexWrap: "wrap" }}><DictationButton key={dictationScope} disabled={sending} cancelRef={cancelDictation}
        onText={text => dictate(text, dictationScope)} onActive={active => { setDictating(active); onDictatingChange?.(active); }} />{composerTools}</div>
      <ConversationBuffer reading={{ source: "", links: [] }} path={null} source={null} id="response" readOnly={false} hasAttachments={hasAttachments} onView={view => { composer.current = view; }}
        running={running} sending={sending} draft={draft} onDraft={onDraft} onSend={onSend} canSend={canSend && !dictating} showCollapsedLineage={false} />
    </section>
  </div>;
}

/** Each card owns its editor and undo history; only the composer reports edits. */
function ConversationBuffer({ onView, hasAttachments = false, reading, path, source, id, readOnly, running, sending, draft, onDraft, onSend, canSend, showCollapsedLineage }: {
  onView?: (view: EditorView | null) => void; hasAttachments?: boolean; reading: ReturnType<typeof conversationReading>; path: string | null; source: string | null; id: string; readOnly: boolean;
  running: string | null; sending: boolean; draft: string; onDraft: (draft: string) => void; onSend: () => void; canSend: boolean; showCollapsedLineage: boolean;
}) {
  const [view, setView] = useState<EditorView | null>(null);
  const realtime = useMemo(() => new LocalRealtime(), []);
  const following = useRef(true);
  const initialFocus = useRef(false);
  const syncLinks = useRef(() => {});
  const latest = useRef({ onDraft, onSend, canSend, onView }); latest.current = { onDraft, onSend, canSend, onView };
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
  const ready = useCallback((editor: EditorView | null) => { setView(editor); latest.current.onView?.(editor); }, []);
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
    if (view) view.dispatch({ effects: setSendState.of({ enabled: canSend, label: sending ? "Starting…" : "Send", visible: !readOnly && !running, allowEmpty: hasAttachments }) });
  }, [view, canSend, sending, running, hasAttachments]);
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
