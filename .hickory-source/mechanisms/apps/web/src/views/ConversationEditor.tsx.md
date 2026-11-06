<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/ConversationEditor.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-436917f654b3"></a>
## `apps/web/src/views/ConversationEditor.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-436917f654b3-noCells"></a>
### `noCells`

<storycode:copy id="f-436917f654b3-noCells">
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

</storycode:copy>
<a id="f-436917f654b3-focusConversationResponse"></a>
### `focusConversationResponse`

The response action must find the composer rather than the first recorded
message editor. Selecting its end preserves the user’s existing Markdown.

<storycode:copy id="f-436917f654b3-focusConversationResponse">
export function focusConversationResponse(host: Element | null) {
  const editor = host?.querySelector<HTMLElement>(".conversation-card--composer .cm-editor");
  const view = editor && EditorView.findFromDOM(editor);
  if (!view) return;
  view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
  view.focus();
}

</storycode:copy>
<a id="f-436917f654b3-ConversationEditor"></a>
### `ConversationEditor`

Split only session-level messages so nested file, patch and executable blocks
stay intact. Each card keeps exact source text and remaps receipt coordinates to
its own editor; streaming text never gains recorded evidence. The final captured
construction follows its turn, with native execution/debugging owned by the
existing captured-project surface. The composer has an independent undo history
and never receives a replacement of the recorded prefix.

The default reading emphasizes the conversation. Reasoning and completed project
construction remain available in disclosures; opening them does not remount the
message buffers or discard draft history. An empty thread gives a short invitation
rather than exposing implementation concepts before the first request.

Message slicing is memoized by the captured history and stream, so typing a draft
does not reparse all prior messages.

Recorded buffers replace only their session/message envelope tags in the visual
reading. The bytes and receipt offsets stay intact; nested file/patch syntax and
all draft syntax keep their normal editor behavior. This avoids stripping tags
from the source or accidentally concealing authored code with broad CSS.

The buffer protects every mutation except an owner-authorized reading update,
including widget writes at the end of a message. It retains native folding and
ribbon anchors in each editor. Focus and Send target only the composer.

Browser dictation is an input method for this same response buffer. Interim
recognition stays in a compact status field; final results enter the current
selection through an ordinary undoable editor transaction. Recognition does not
send a turn, synthesize speech or choose agent actions. It works independently of
provider setup, through the browser's speech service where available.

Stop waits for a final result before re-enabling Send. A bounded finish timer,
identity check, detached callbacks and abort on unmount prevent late results from
changing a closed or different response. The outer conversation scope also rejects
old inserts during navigation or explicit discard. Permission, network and missing
browser support failures keep completed draft text and explain how to continue.

<storycode:copy id="f-436917f654b3-ConversationEditor">
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
  return <span style={{ display: "inline-flex", position: "relative" }}><button type="button" style={{ minWidth: 120 }} disabled={disabled || finishing} aria-pressed={active}
    title="Use browser speech recognition to dictate draft text. Review your message before sending."
    onMouseDown={event => event.preventDefault()} onClick={() => active ? stop() : start()}>{active ? "Stop dictation" : "Dictate"}</button>
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
      <header>Message agent</header>
      <ConversationBuffer reading={{ source: "", links: [] }} path={null} source={null} id="response" readOnly={false} hasAttachments={hasAttachments} onView={view => { composer.current = view; }}
        running={running} sending={sending} draft={draft} onDraft={onDraft} onSend={onSend} canSend={canSend && !dictating} showCollapsedLineage={false} />
    </section>
  </div>;
}

</storycode:copy>

### Keep message buffers immutable and the response independently editable

A card keeps native Markdown rendering, folding and receipt anchors. Its owner
may replace streamed text, but keyboard, undo and widget edits cannot change the
record. The composer alone reports draft changes and owns focus, Send and scroll
following. Resize observation follows the outer conversation so a growing message
does not move a reader who deliberately scrolled upward.

<storycode:copy id="conversation-message-buffer">
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
</storycode:copy>

### Assemble `apps/web/src/views/ConversationEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/ConversationEditor.tsx"><storycode:paste select="#f-436917f654b3-noCells"/><storycode:paste select="#f-436917f654b3-focusConversationResponse"/><storycode:paste select="#f-436917f654b3-ConversationEditor"/><storycode:paste select="#conversation-message-buffer"/></storycode:file>


## Exercise this mechanism

The test module below imports this implementation directly. Keep the assertions with the mechanism they exercise; passing reconstruction alone does not show those assertions ran.

<a id="output-edc92cd336d7"></a>
## `apps/web/src/views/ConversationEditor.test.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Read the retained branch of an agent conversation](../../../../../guarantees/agent/a-session-is-the-conversation/contract.md), [Trace conversation passages without confusing context with support](../../../../../guarantees/agent/an-answer-in-the-agent-pane-has-ribbons/contract.md), [The Agent pane is a live document](../../../../../guarantees/agent/the-agent-pane-is-a-live-document/contract.md).

<a id="f-edc92cd336d7-expression_statement"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement">
// @vitest-environment jsdom
// Guarantees: agent/the-agent-pane-is-a-live-document.md; agent/an-answer-in-the-agent-pane-has-ribbons.md.
import { useState } from "react";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditorView } from "@codemirror/view";
import { undo } from "@codemirror/commands";
import { ConversationEditor, focusConversationResponse } from "./ConversationEditor";
import { responseStart } from "../editor/protectedPrefix";
import { installMockHandler } from "../api/client";
import { lensSources } from "../lib/lensSources";
import type { AgentTurn, SessionViewResponse } from "../api/types";
import { conversationReading } from "./conversationReading";
import { charToByte } from "../lib/offsets";
import { foldedRanges, unfoldEffect } from "@codemirror/language";
afterEach(cleanup);
</storycode:copy>
<a id="f-edc92cd336d7-turn"></a>
### `turn`

<storycode:copy id="f-edc92cd336d7-turn">
const turn = (id: string, parent: string | null = null): AgentTurn => ({ id, parent_id: parent, prompt: `Request ${id}`, answer: `Answer ${id}`, status: "ok", error: null, provider: "acp:codex", model: "", created_at: "", usage: null });
</storycode:copy>
<a id="f-edc92cd336d7-first"></a>
### `first`

<storycode:copy id="f-edc92cd336d7-first">
const first = turn("a"), live = { ...turn("b", "a"), status: "running" as const, answer: null };
</storycode:copy>
<a id="f-edc92cd336d7-source"></a>
### `source`

<storycode:copy id="f-edc92cd336d7-source">
const source = `<hick:session>\n<hick:user turn="a">Request a</hick:user>\n<hick:read file="note.md" lines="1-2"/>\n<hick:assistant>Answer a 📝</hick:assistant>\n</hick:session>\n`;
</storycode:copy>
<a id="f-edc92cd336d7-start"></a>
### `start`

<storycode:copy id="f-edc92cd336d7-start">
const start = source.indexOf("<hick:read"), end = source.indexOf("/>", start) + 2;
</storycode:copy>
<a id="f-edc92cd336d7-data"></a>
### `data`

<storycode:copy id="f-edc92cd336d7-data">
const data: SessionViewResponse = { path: "sessions/s.md", source, blocks: [], view: { start: null, doc: null, turns: [{ id: "a", parent: null, prompt: first.prompt, answer: first.answer, provider: first.provider, model: "", steps: [], usage: null, session_line: 2 }] },
  links: [{ family: "context", span: [charToByte(source, start), charToByte(source, end)], lines: [3, 3], to: { path: "note.md", lines: [1, 2] }, title: "read" }] };
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-2"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-2">
installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? data : {});
</storycode:copy>
<a id="f-edc92cd336d7-Harness"></a>
### `Harness`

<storycode:copy id="f-edc92cd336d7-Harness">
function Harness({ stream = "Writing", reasoning = "Thinking" }: { stream?: string; reasoning?: string }) {
  const [draft, setDraft] = useState("");
  return <ConversationEditor branch={[first, live]} path="sessions/s.md" stamp="a,b" running="b" stream={stream} reasoning={reasoning} draft={draft} onDraft={setDraft} onSend={() => {}} canSend={false} />;
}
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-3"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-3">
it("opens at the editable response and can return there without rewriting the conversation", async () => {
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "My draft", onDraft: () => {}, onSend: () => {}, canSend: true, stamp: "" };
  const rendered = render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(view.state.selection.main.head).toBe(view.state.doc.length));
  expect(view.hasFocus).toBe(true);
  const before = view.state.doc.toString();
  act(() => view.dispatch({ selection: { anchor: 0 } }));
  act(() => focusConversationResponse(rendered.container));
  expect(view.state.selection.main.head).toBe(view.state.doc.length);
  expect(view.state.doc.toString()).toBe(before);
  rendered.rerender(<ConversationEditor {...props} stream="Update" />);
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("My draft");
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-4"></a>
### `expression_statement`

Recorded messages need different editor identities, not just separate visual
wrappers. Try keyboard and programmatic writes at both boundaries, stream an
update and undo the response; only the editable draft may change.

<storycode:copy id="f-edc92cd336d7-expression_statement-4">
it("gives every message a protected buffer and keeps the draft independent during streaming", async () => {
  const rendered = render(<Harness />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const response = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(rendered.container.textContent).toContain("Answer a 📝"));
  const cards = screen.getAllByRole("textbox", { name: "Recorded message", hidden: true });
  expect(cards.length).toBeGreaterThanOrEqual(4);
  const views = cards.map(card => EditorView.findFromDOM(card)!);
  expect(new Set(views).size).toBe(cards.length);
  for (const view of views) {
    expect(view.state.readOnly).toBe(true);
    const before = view.state.doc.toString();
    act(() => view.dispatch({ changes: { from: 0, to: 5, insert: "erase" }, userEvent: "input.type" }));
    act(() => view.dispatch({ changes: { from: view.state.doc.length, insert: "widget write" } }));
    act(() => { undo(view); });
    expect(view.state.doc.toString()).toBe(before);
  }
  act(() => response.dispatch({ changes: { from: 0, insert: "My response\nsecond line" }, userEvent: "input.type" }));
  const answer = views.find(view => view.state.doc.toString().includes("Answer a 📝"))!;
  rendered.rerender(<Harness stream="Writing more now" />);
  await waitFor(() => expect(rendered.container.textContent).toContain("Writing more now"));
  expect(response.state.doc.toString()).toBe("My response\nsecond line");
  expect(EditorView.findFromDOM(answer.dom)).toBe(answer);
  expect(rendered.container.querySelector(".conversation-card--reasoning")).toBeTruthy();
  expect(lensSources().some(lens => lens.path === data.path && lens.links.some(link => link.to.path === "note.md"))).toBe(true);
  // Separate histories: undoing the draft cannot touch recorded speech.
  const before = answer.state.doc.toString();
  act(() => { undo(response); });
  expect(answer.state.doc.toString()).toBe(before);
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-5"></a>
### `expression_statement`

Dictation tests use a controllable recognition service to make permission,
interim/final and obsolete-callback timing deterministic. They drive the real
composer, assert editable text and undo, and require an explicit Send after the
recognizer ends. A browser transport witness covers the same user controls; these
checks do not claim microphone accuracy or a working remote speech service.

<storycode:copy id="f-edc92cd336d7-expression_statement-5">
it("submits only with the send shortcut; ordinary Enter remains multiline editing", async () => {
  const send = vi.fn();
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "Hello", onDraft: () => {}, onSend: send, canSend: true, stamp: "" };
  render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  act(() => view.dispatch({ selection: { anchor: view.state.doc.length } }));
  view.focus();
  fireEvent.keyDown(host, { key: "Enter", code: "Enter" });
  expect(send).not.toHaveBeenCalled();
  fireEvent.keyDown(host, { key: "Enter", code: "Enter", ctrlKey: true });
  expect(send).toHaveBeenCalledTimes(1);
});


describe("composer dictation", () => {
  class Recognition {
    static instances: Recognition[] = [];
    continuous = false; interimResults = false; lang = "";
    onstart: (() => void) | null = null;
    onend: (() => void) | null = null;
    onerror: ((event: { error: string }) => void) | null = null;
    onresult: ((event: { results: { isFinal: boolean; 0: { transcript: string } }[] }) => void) | null = null;
    start = vi.fn(); stop = vi.fn(); abort = vi.fn();
    constructor() { Recognition.instances.push(this); }
    result(text: string, isFinal: boolean) { this.onresult?.({ results: [{ isFinal, 0: { transcript: text } }] }); }
  }
  afterEach(() => { cleanup(); vi.unstubAllGlobals(); vi.useRealTimers(); Recognition.instances = []; });
  function Draft({ scope = "one", cancel }: { scope?: string; cancel?: { current: (() => void) | null } }) {
    const [draft, setDraft] = useState("Before after");
    return <ConversationEditor branch={[]} path={null} stamp="" running={null} stream="" reasoning="" draft={draft}
      onDraft={setDraft} onSend={sent} canSend dictationScope={scope} cancelDictation={cancel} />;
  }
  const sent = vi.fn();
  async function setup(cancel?: { current: (() => void) | null }) {
    sent.mockClear(); vi.stubGlobal("SpeechRecognition", Recognition);
    const rendered = render(<Draft cancel={cancel} />);
    const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
    return { ...rendered, host, view: EditorView.findFromDOM(host)! };
  }
  it("inserts finalized speech at the live caret, leaves interim speech out, and preserves undo", async () => {
    const { view, host } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!;
    expect(speech.start).toHaveBeenCalledOnce();
    expect(speech.continuous && speech.interimResults).toBe(true);
    act(() => speech.result("uncertain words", false));
    expect(view.state.doc.toString()).toBe("Before after");
    expect(screen.getByLabelText("Dictation status").textContent).toBe("uncertain words");
    // Typing and moving the caret while recognition is pending remain authoritative.
    act(() => view.dispatch({ changes: { from: 0, insert: "Typed " }, selection: { anchor: 12 }, userEvent: "input.type" }));
    act(() => speech.result("spoken", true));
    expect(view.state.doc.toString()).toBe("Typed Before spoken after");
    act(() => speech.result("spoken", true));
    expect(view.state.doc.toString()).toBe("Typed Before spoken after");
    fireEvent.keyDown(host, { key: "Enter", code: "Enter", ctrlKey: true });
    expect(sent).not.toHaveBeenCalled();
    act(() => { undo(view); });
    expect(view.state.doc.toString()).toBe("Typed Before after");
  });
  it("waits for the last result on Stop and never sends speech automatically", async () => {
    const { view } = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!;
    expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(true);
    fireEvent.click(screen.getByRole("button", { name: "Stop dictation" }));
    expect(speech.stop).toHaveBeenCalledOnce();
    act(() => speech.result("last words", true));
    expect(view.state.doc.toString()).toBe("Before after last words");
    expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(true);
    act(() => speech.onend?.());
    expect(speech.abort).not.toHaveBeenCalled();
    expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(false);
    expect(sent).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Send" }));
    expect(sent).toHaveBeenCalledOnce();
  });
  it("aborts navigation and unmount, rejecting even a previously queued callback", async () => {
    const rendered = await setup();
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!, queued = speech.onresult!;
    rendered.rerender(<Draft scope="two" />);
    expect(speech.abort).toHaveBeenCalledOnce();
    expect(speech.onresult).toBeNull();
    act(() => queued({ results: [{ isFinal: true, 0: { transcript: "obsolete" } }] }));
    expect(rendered.view.state.doc.toString()).toBe("Before after");
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const second = Recognition.instances.at(-1)!;
    rendered.unmount();
    expect(second.abort).toHaveBeenCalledOnce();
  });
  it("degrades without recognition and handles permission denial without losing the draft", async () => {
    vi.stubGlobal("SpeechRecognition", undefined); vi.stubGlobal("webkitSpeechRecognition", undefined);
    render(<Draft />);
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    expect(screen.getByLabelText("Dictation status").textContent).toContain("Use your system's dictation");
    vi.stubGlobal("webkitSpeechRecognition", Recognition);
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!;
    act(() => speech.onerror?.({ error: "not-allowed" }));
    expect(speech.abort).toHaveBeenCalledOnce();
    expect(screen.getByLabelText("Dictation status").textContent).toContain("Allow access");
    expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("Before after");
    expect(screen.getByRole("button", { name: "Dictate" })).toBeTruthy();
  });
  it("cancels synchronously on discard before a queued final can restore discarded text", async () => {
    const cancel: { current: (() => void) | null } = { current: null };
    const { view } = await setup(cancel);
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!, queued = speech.onresult!;
    act(() => { cancel.current?.(); view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "" } }); });
    act(() => queued({ results: [{ isFinal: true, 0: { transcript: "discarded speech" } }] }));
    expect(speech.abort).toHaveBeenCalledOnce();
    expect(view.state.doc.toString()).toBe("");
  });
  it("bounds a stalled Stop and rejects its late final result", async () => {
    const { view } = await setup();
    vi.useFakeTimers();
    fireEvent.click(screen.getByRole("button", { name: "Dictate" }));
    const speech = Recognition.instances.at(-1)!, queued = speech.onresult!;
    fireEvent.click(screen.getByRole("button", { name: "Stop dictation" }));
    act(() => { vi.advanceTimersByTime(5000); });
    expect(speech.abort).toHaveBeenCalledOnce();
    expect(screen.getByLabelText("Dictation status").textContent).toContain("too long");
    act(() => queued({ results: [{ isFinal: true, 0: { transcript: "too late" } }] }));
    expect(view.state.doc.toString()).toBe("Before after");
  });
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-6"></a>
### `expression_statement`

Keep each completed construction with its owning turn. The composer comes last,
with Send after its Markdown rather than inside a recorded message.

<storycode:copy id="f-edc92cd336d7-expression_statement-6">
it("keeps the final construction beside its own turn before the independent composer", async () => {
  const second = turn("b", "a");
  const props = { branch: [first, second], path: null, running: null, stream: "", reasoning: "", draft: "## My response", onDraft: () => {}, onSend: () => {}, canSend: true, stamp: "" };
  const rendered = render(<ConversationEditor {...props} renderFinal={item => <p>Construction {item.id}</p>} />);
  const cards = screen.getAllByRole("region", { name: "Final construction" });
  expect(cards).toHaveLength(2);
  const reviews = rendered.container.querySelectorAll<HTMLDetailsElement>(".conversation-review");
  expect(reviews).toHaveLength(2);
  expect([...reviews].every(review => !review.open)).toBe(true);
  fireEvent.click(reviews[0].querySelector("summary")!);
  expect(screen.getByText("Construction a")).toBeTruthy();
  expect(cards[0].closest("[data-turn]")?.getAttribute("data-turn")).toBe("a");
  expect(cards[1].closest("[data-turn]")?.getAttribute("data-turn")).toBe("b");
  expect(rendered.container.lastElementChild?.lastElementChild?.getAttribute("aria-label")).toBe("Your response");
  expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("My response");
});

it("places Send after the multiline draft without adding bytes, and disables an empty draft", async () => {
  const send = vi.fn();
  const props = { branch: [], path: null, running: null, stream: "", reasoning: "", draft: "Hello\nthere", onDraft: () => {}, onSend: send, canSend: true, stamp: "" };
  const rendered = render(<ConversationEditor {...props} />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  const button = await screen.findByRole("button", { name: "Send" });
  expect(button.closest(".cm-line")).toBeNull();
  expect(view.contentDOM.textContent).toContain("there");
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("Hello\nthere");
  fireEvent.click(button);
  expect(send).toHaveBeenCalledTimes(1);
  rendered.rerender(<ConversationEditor {...props} draft="" />);
  await waitFor(() => expect((screen.getByRole("button", { name: "Send" }) as HTMLButtonElement).disabled).toBe(true));
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-7"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-7">
it("follows growing responses, pauses when scrolling upward, and resumes on sending", async () => {
  const props = { branch: [live], path: null, running: "b", stream: "Writing", reasoning: "", draft: "", onDraft: () => {}, onSend: () => {}, canSend: false, stamp: "" };
  const rendered = render(<div className="chat-log"><ConversationEditor {...props} /></div>);
  const log = rendered.container.firstElementChild as HTMLElement;
  Object.defineProperties(log, { scrollHeight: { configurable: true, value: 1000 }, clientHeight: { value: 200 } });
  await waitFor(() => expect(log.scrollTop).toBe(1000));
  log.scrollTop = 800; fireEvent.scroll(log);
  log.scrollTop = 400; fireEvent.scroll(log);
  Object.defineProperty(log, "scrollHeight", { value: 1200 });
  rendered.rerender(<div className="chat-log"><ConversationEditor {...props} stream="Writing more" /></div>);
  await act(async () => { await new Promise(resolve => requestAnimationFrame(resolve)); });
  expect(log.scrollTop).toBe(400);
  rendered.rerender(<div className="chat-log"><ConversationEditor {...props} sending /></div>);
  await waitFor(() => expect(log.scrollTop).toBe(1200));
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-8"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-8">
it("reads only the selected branch and remaps recorded Unicode links without claiming live evidence", () => {
  const other = turn("alternative", "a");
  const fixture = { ...data, source: data.source.replace("</hick:session>", '<hick:user turn="alternative">OTHER BRANCH</hick:user>\n<hick:assistant>other</hick:assistant>\n</hick:session>'),
    view: { ...data.view, turns: [...data.view.turns, { ...data.view.turns[0], id: other.id, session_line: 5 }] } };
  const reading = conversationReading(fixture, [first, live], "b", "Live words", "");
  expect(reading.source).not.toContain("OTHER BRANCH");
  expect(reading.source).toContain("Live words");
  expect(reading.links).toHaveLength(1);
  expect(reading.source.slice(...reading.links[0].span)).toBe(source.slice(start, end));
});

</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-9"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-9">
it("folds newly arriving context with native editor folds and refreshes its receipts on unfold", async () => {
  const contextSource = '<hick:session>\n<hick:user turn="a">Request a</hick:user>\n<hick:context kind="editor-buffers">\nSnapshot 📝\n</hick:context>\n<hick:assistant>Answer a</hick:assistant>\n</hick:session>\n';
  const from = contextSource.indexOf("<hick:context"), to = contextSource.indexOf("</hick:context>") + 15;
  const contextData = { ...data, source: contextSource, links: [{ ...data.links[0], span: [charToByte(contextSource, from), charToByte(contextSource, to)] as [number, number], lines: [3,5] as [number,number] }] };
  installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? contextData : {});
  const props = { branch: [first], path: data.path, stamp: "a", running: null, stream: "", reasoning: "", draft: "", onDraft: () => {}, onSend: () => {}, canSend: false };
  const rendered = render(<ConversationEditor {...props} />);
  const hosts = await screen.findAllByRole("textbox", { name: "Recorded message" });
  const host = hosts[0];
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(foldedRanges(view.state).size).toBeGreaterThan(0));
  expect(host.querySelector("details")).toBeNull();
  expect(view.state.doc.toString()).toContain("Snapshot 📝");
  expect(host.textContent).not.toContain("Snapshot 📝");
  await waitFor(() => expect(lensSources().find(lens => lens.view === view)?.links).toHaveLength(0));
  rendered.rerender(<ConversationEditor {...props} showCollapsedLineage />);
  await waitFor(() => expect(lensSources().find(lens => lens.view === view)?.links[0]?.from.lines).toEqual([3,3]));
  const folds: { from: number; to: number }[] = [];
  foldedRanges(view.state).between(0, view.state.doc.length, (from, to) => { folds.push({ from, to }); });
  act(() => view.dispatch({ effects: folds.map(f => unfoldEffect.of(f)) }));
  await waitFor(() => expect(host.textContent).toContain("Snapshot 📝"));
  rendered.rerender(<ConversationEditor {...props} draft="new response" />);
  await waitFor(() => expect(screen.getByRole("textbox", { name: "Agent conversation and response" }).textContent).toContain("new response"));
  expect(foldedRanges(view.state).size).toBe(0);
  expect(lensSources().find(lens => lens.view === view)?.links[0]?.from.lines).toEqual([3,5]);
  installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? data : {});
});
</storycode:copy>

### Assemble `apps/web/src/views/ConversationEditor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/ConversationEditor.test.tsx"><storycode:paste select="#f-edc92cd336d7-expression_statement"/><storycode:paste select="#f-edc92cd336d7-turn"/><storycode:paste select="#f-edc92cd336d7-first"/><storycode:paste select="#f-edc92cd336d7-source"/><storycode:paste select="#f-edc92cd336d7-start"/><storycode:paste select="#f-edc92cd336d7-data"/><storycode:paste select="#f-edc92cd336d7-expression_statement-2"/><storycode:paste select="#f-edc92cd336d7-Harness"/><storycode:paste select="#f-edc92cd336d7-expression_statement-3"/><storycode:paste select="#f-edc92cd336d7-expression_statement-4"/><storycode:paste select="#f-edc92cd336d7-expression_statement-5"/><storycode:paste select="#f-edc92cd336d7-expression_statement-6"/><storycode:paste select="#f-edc92cd336d7-expression_statement-7"/><storycode:paste select="#f-edc92cd336d7-expression_statement-8"/><storycode:paste select="#f-edc92cd336d7-expression_statement-9"/></storycode:file>


</storycode:doc>
