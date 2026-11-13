<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep one explicit workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The workspace owns the session so moving or hiding its tab retains the literate draft and in-flight send. Explicit subscription choices always use the Hickory Python backend; the backend refuses missing runtime or sign-in rather than selecting an API provider. Durable history refreshes independently and stops when unmounted.

Render prompts and answers through the installed Markdown parser. Walk its syntax tree into a small allowlist of React elements so nested emphasis and headings follow Markdown boundaries, while code stays literal. Unknown constructs remain text; raw HTML never executes. Rendering only observes the retained messages.

<ui:copy id="simple-981c695688-0">
import { createElement, useEffect, useRef, useSyncExternalStore, type ReactNode } from "react";
import { markdownLanguage } from "@codemirror/lang-markdown";
import type { SyntaxNode } from "@lezer/common";
import { Buffer, Buffers } from "./buffers";
import { Document, editorForBuffer } from "./Document";
import { RichOwners } from "./RichInline";
import { api } from "../api/client";
import type { AgentTurn } from "../api/agentTypes";
export function MarkdownMessage({ text }: { text: string }) {
  const render = (node: SyntaxNode): ReactNode => {
    const literal = text.slice(node.from, node.to);
    if (node.name === "FencedCode" || node.name === "CodeBlock") {
      const body = node.getChildren("CodeText").map(child => text.slice(child.from, child.to)).join("\n");
      return <pre key={node.from}><code>{body}</code></pre>;
    }
    if (node.name === "InlineCode") {
      const marks = node.getChildren("CodeMark");
      let body = text.slice(marks[0].to, marks.at(-1)!.from).replace(/\n/g, " ");
      if (/^ .* $/.test(body) && /[^ ]/.test(body)) body = body.slice(1, -1);
      return <code key={node.from}>{body}</code>;
    }
    if (node.name === "Escape") return literal.slice(1);
    const heading = /^(?:ATX|Setext)Heading([1-6])$/.exec(node.name);
    const tags: Record<string, string> = { Paragraph: "p", StrongEmphasis: "strong", Emphasis: "em",
      BulletList: "ul", OrderedList: "ol", ListItem: "li", Blockquote: "blockquote" };
    const tag = heading ? `h${heading[1]}` : tags[node.name];
    // Unhandled syntax, including HTML and links, remains readable literal text.
    // Only known tags enter the DOM; message text never becomes HTML or a URL.
    if (!tag && node.name !== "Document") return literal;
    const children: ReactNode[] = [];
    let offset = node.from;
    for (let child = node.firstChild; child; child = child.nextSibling) {
      if (child.from > offset) children.push(text.slice(offset, child.from));
      if (!["HeaderMark", "EmphasisMark", "ListMark", "QuoteMark"].includes(child.name)) children.push(render(child));
      offset = child.to;
    }
    if (offset < node.to) children.push(text.slice(offset, node.to));
    return tag ? createElement(tag, { key: node.from }, ...children) : children;
  };
  return <div className="message markdown-message">{render(markdownLanguage.parser.parse(text).topNode)}</div>;
}
</ui:copy>

## Retain documents across pane moves and sends

Use transient Buffer objects with the same literate document editor as workspace
files. Message paths are editor identities only: transient buffers never save or
open language/debug sessions. History uses the read-only buffer path, including
inline grids. The draft retains selection, undo and unsaved text through provider
changes. Subscribe to every draft edit so Send availability follows blankness.
Capture a send once, keep it busy until acknowledged, and clear only that exact
draft; typing during a request belongs to the next message.

<ui:copy id="simple-agent-document-session">
export class AgentSession {
  constructor() { this.draft.subscribe(() => this.publish()); }
  turns: AgentTurn[] = [];
  provider = "codex-subscription";
  model = "";
  effort = "";
  readonly documents = new Buffers(() => this.publish());
  readonly draft = this.document("draft", "", false);
  get prompt() { return this.draft.text; }
  set prompt(value: string) {
    this.draft.edit(value);
    const view = editorForBuffer(this.draft)?.view;
    if (view && view.state.doc.toString() !== value) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: value } });
  }
  document(id: string, text: string, readonly = true) {
    const path = `conversation/${id}.md`;
    let buffer = this.documents.values.get(path);
    if (!buffer) {
      buffer = new Buffer({ name: id, path, dir: false, ...(readonly ? { generated_by: "conversation" } : {}) }, text, "", () => this.publish(), true);
      this.documents.values.set(path, buffer);
    } else if (buffer.text !== text) {
      buffer.text = buffer.base = text; buffer.publish();
      const view = editorForBuffer(buffer)?.view;
      if (view) view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } });
    }
    return buffer;
  }
  setTurns(turns: AgentTurn[]) {
    // Rebase observable document buffers before publishing the conversation.
    // Rendering an existing message must never dispatch editor transactions.
    for (const turn of turns) {
      this.document(`${turn.id}-user`, turn.prompt);
      this.document(`${turn.id}-agent`, turn.answer ?? turn.error ?? (turn.status === "running" ? "Working…" : ""));
    }
    this.turns = turns;
  }
  busy = false;
  error = "";
  mutation = 0;
  private version = 0;
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  get running() { return this.busy || this.turns.some(turn => turn.status === "running"); }
  async send() {
    if (!this.prompt.trim() || this.running) return;
    const text = this.prompt; this.busy = true; this.error = ""; this.mutation++; this.publish();
    try {
      const started = await api.agent("workspace", text, this.turns.at(-1)?.id, this.provider, this.model, "python", undefined, undefined, this.effort || undefined);
      this.setTurns([...this.turns.filter(turn => turn.id !== started.session_id), { id: started.session_id, prompt: text, answer: null, status: "running", parent_id: null, error: null, created_at: "", provider: this.provider, model: this.model, usage: null }]);
      if (this.prompt === text) this.prompt = "";
    } catch (reason) { this.error = `${String(reason)} The request was not retried. Refresh history before sending again.`; }
    finally { this.busy = false; this.mutation++; this.publish(); }
  }
}
</ui:copy>

## Refresh history independently of the composer

History polling never recreates the draft editor. Reject replies from before a
send mutation, and stop polling when the pane detaches. Provider failures remain
visible beside the retained draft; no request is automatically retried.

<ui:copy id="simple-agent-history-refresh">
export function Agent({ session }: { session: AgentSession }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      const mutation = session.mutation;
      try {
        const response = await api.agentTurns("workspace");
        if (!cancelled && mutation === session.mutation && !session.busy) { session.setTurns(response.turns); session.publish(); }
      } catch (reason) { if (!cancelled) { session.error = `${String(reason)} Check subscription sign-in on the workspace server, then refresh.`; session.publish(); } }
      if (!cancelled) timer = setTimeout(refresh, 1000);
    };
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); };
  }, [session]);
  const conversation = useRef<HTMLDivElement>(null);
  const following = useRef(true);
  const { turns, prompt, error } = session;
  const tail = turns.at(-1);
  useEffect(() => {
    if (following.current && conversation.current) conversation.current.scrollTop = conversation.current.scrollHeight;
  }, [tail?.id, tail?.answer, tail?.error]);
  const running = session.running;
</ui:copy>

Conversation controls belong inside the agent pane. Stop requests are explicit; failures remain visible. A missing provider never prevents document editing.

<ui:copy id="simple-981c695688-1">
  return <div className="agent-body">
    <RichOwners buffers={session.documents} />
    <header className="agent-heading"><strong>Workspace agent</strong><span>Hickory · Python execution only</span></header>
    <div ref={conversation} className="conversation" role="log" aria-label="Conversation" onScroll={event => { const node = event.currentTarget; following.current = node.scrollHeight - node.clientHeight - node.scrollTop < 48; }}>
      {!turns.length && <div className="agent-empty"><h2>What shall we work on?</h2><p>Write a literate document below. Code, tables and explanations belong together.</p></div>}
      {turns.map(turn => <article key={turn.id}>
        <section className="agent-message user-message"><p className="message-label">You</p><Document buffer={session.document(`${turn.id}-user`, turn.prompt)} /></section>
        <section className="agent-message"><p className="message-label">Agent · {turn.status}{turn.model && ` · ${turn.model}`}</p><Document buffer={session.document(`${turn.id}-agent`, turn.answer ?? turn.error ?? (turn.status === "running" ? "Working…" : ""))} /></section>
      </article>)}
    </div>
    {error && <div role="alert" className="notice">{error}</div>}
    <form className="agent-composer" onSubmit={event => { event.preventDefault(); void session.send(); }}>
      <label className="message-label">Your message · literate document</label>
      <Document buffer={session.draft} />
      <div className="agent-controls">
        <label>Provider<select aria-label="Provider" disabled={running} value={session.provider} onChange={event => { session.provider = event.target.value; session.model = ""; session.effort = ""; session.publish(); }}>
          <option value="codex-subscription">ChatGPT subscription</option><option value="claude-subscription">Claude Code subscription</option><option value="grok-subscription">Grok subscription</option>
        </select></label>
        <label>Model<input aria-label="Model" disabled={running} value={session.model} placeholder="Provider default" onChange={event => { session.model = event.target.value; session.publish(); }} /></label>
        <label>Effort<select aria-label="Effort" disabled={running || session.provider === "grok-subscription"} value={session.effort} onChange={event => { session.effort = event.target.value; session.publish(); }}>
          <option value="">Default</option>{["low", "medium", "high", "xhigh", "max"].map(value => <option key={value}>{value}</option>)}
        </select></label>
        <div className="agent-send"><button type="button" disabled={!running} onClick={() => { void api.agentStop("workspace").catch(reason => { session.error = `${String(reason)} Refresh history and try Stop again.`; session.publish(); }); }}>Stop</button><button disabled={running || !prompt.trim()}>Send ↑</button></div>
      </div>
      <small>Uses the selected subscription on the workspace server. Sign in there with {session.provider === "claude-subscription" ? "claude auth login" : session.provider === "grok-subscription" ? "grok login" : "codex login"}. {session.provider === "grok-subscription" && "Grok manages effort."}</small>
    </form>
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Agent.tsx"><ui:paste select="#simple-981c695688-0"/><ui:paste select="#simple-agent-document-session"/><ui:paste select="#simple-agent-history-refresh"/><ui:paste select="#simple-981c695688-1"/></ui:file>

</ui:doc>
