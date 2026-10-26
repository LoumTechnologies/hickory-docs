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

</storycode:copy>
<a id="f-436917f654b3-focusConversationResponse"></a>
### `focusConversationResponse`

<storycode:copy id="f-436917f654b3-focusConversationResponse">
export function focusConversationResponse(host: Element | null) {
  const editor = host?.querySelector<HTMLElement>(".cm-editor");
  const view = editor && EditorView.findFromDOM(editor);
  if (!view) return;
  view.dispatch({ selection: { anchor: view.state.doc.length }, scrollIntoView: true });
  view.focus();
}

</storycode:copy>
<a id="f-436917f654b3-ConversationEditor"></a>
### `ConversationEditor`

<storycode:copy id="f-436917f654b3-ConversationEditor">
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
</storycode:copy>

### Assemble `apps/web/src/views/ConversationEditor.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/ConversationEditor.tsx"><storycode:paste select="#f-436917f654b3-noCells"/><storycode:paste select="#f-436917f654b3-focusConversationResponse"/><storycode:paste select="#f-436917f654b3-ConversationEditor"/></storycode:file>


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
import { afterEach, expect, it, vi } from "vitest";
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

<storycode:copy id="f-edc92cd336d7-expression_statement-4">
it("protects recorded and live content while preserving a response across streaming updates", async () => {
  const rendered = render(<Harness />);
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(view.state.doc.toString()).toContain("Answer a 📝"));
  const before = view.state.doc.toString();
  act(() => view.dispatch({ changes: { from: 0, to: 5, insert: "erase" }, userEvent: "input.type" }));
  expect(view.state.doc.toString()).toBe(before);
  // A widget's programmatic edit is protected too, not just keyboard events.
  act(() => view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: "replace everything" } }));
  expect(view.state.doc.toString()).toBe(before);
  act(() => view.dispatch({ changes: { from: view.state.field(responseStart), insert: "My response\nsecond line" }, userEvent: "input.type" }));
  expect(view.state.doc.toString()).toContain("My response\nsecond line");
  rendered.rerender(<Harness stream="Writing more now" />);
  await waitFor(() => expect(view.state.doc.toString()).toContain("Writing more now"));
  expect(view.state.doc.sliceString(view.state.field(responseStart))).toBe("My response\nsecond line");
  expect(view.state.doc.toString()).toContain("Thinking");
  expect(document.querySelector(".cm-turn-assistant")).toBeTruthy();
  expect(document.querySelector(".cm-turn-reasoning")).toBeTruthy();
  expect(lensSources().find(lens => lens.path === data.path)?.links.some(link => link.to.path === "note.md")).toBe(true);
  const protectedText = view.state.doc.sliceString(0, view.state.field(responseStart));
  act(() => { undo(view); });
  expect(view.state.doc.sliceString(0, view.state.field(responseStart))).toBe(protectedText);
});
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-5"></a>
### `expression_statement`

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
</storycode:copy>
<a id="f-edc92cd336d7-expression_statement-6"></a>
### `expression_statement`

<storycode:copy id="f-edc92cd336d7-expression_statement-6">
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
  const host = await screen.findByRole("textbox", { name: "Agent conversation and response" });
  const view = EditorView.findFromDOM(host)!;
  await waitFor(() => expect(foldedRanges(view.state).size).toBeGreaterThan(0));
  expect(host.querySelector("details")).toBeNull();
  expect(view.state.doc.toString()).toContain("Snapshot 📝");
  expect(host.textContent).not.toContain("Snapshot 📝");
  await waitFor(() => expect(lensSources().find(lens => lens.path === data.path)?.links).toHaveLength(0));
  rendered.rerender(<ConversationEditor {...props} showCollapsedLineage />);
  await waitFor(() => expect(lensSources().find(lens => lens.path === data.path)?.links[0]?.from.lines).toEqual([3,3]));
  const folds: { from: number; to: number }[] = [];
  foldedRanges(view.state).between(0, view.state.doc.length, (from, to) => { folds.push({ from, to }); });
  act(() => view.dispatch({ effects: folds.map(f => unfoldEffect.of(f)) }));
  await waitFor(() => expect(host.textContent).toContain("Snapshot 📝"));
  rendered.rerender(<ConversationEditor {...props} draft="new response" />);
  await waitFor(() => expect(view.state.doc.toString()).toContain("new response"));
  expect(foldedRanges(view.state).size).toBe(0);
  expect(lensSources().find(lens => lens.path === data.path)?.links[0]?.from.lines).toEqual([3,5]);
  installMockHandler(async (_method, path) => path.startsWith("/api/sessions/view") ? data : {});
});
</storycode:copy>

### Assemble `apps/web/src/views/ConversationEditor.test.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/ConversationEditor.test.tsx"><storycode:paste select="#f-edc92cd336d7-expression_statement"/><storycode:paste select="#f-edc92cd336d7-turn"/><storycode:paste select="#f-edc92cd336d7-first"/><storycode:paste select="#f-edc92cd336d7-source"/><storycode:paste select="#f-edc92cd336d7-start"/><storycode:paste select="#f-edc92cd336d7-data"/><storycode:paste select="#f-edc92cd336d7-expression_statement-2"/><storycode:paste select="#f-edc92cd336d7-Harness"/><storycode:paste select="#f-edc92cd336d7-expression_statement-3"/><storycode:paste select="#f-edc92cd336d7-expression_statement-4"/><storycode:paste select="#f-edc92cd336d7-expression_statement-5"/><storycode:paste select="#f-edc92cd336d7-expression_statement-6"/><storycode:paste select="#f-edc92cd336d7-expression_statement-7"/><storycode:paste select="#f-edc92cd336d7-expression_statement-8"/><storycode:paste select="#f-edc92cd336d7-expression_statement-9"/></storycode:file>


</storycode:doc>
