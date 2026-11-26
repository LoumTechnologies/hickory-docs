import {authoredSource,authoredChanges,normalizedSource} from './SourceText';
import { createElement, useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Compartment, StateEffect, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { markdownLanguage } from "@codemirror/lang-markdown";
import type { SyntaxNode } from "@lezer/common";
import { Buffer, Buffers } from "./buffers";
import type {AgentEditorContext} from "../api/agentTypes";
import { projectionEstimate } from "../lib/contextProjection";
import { Document, editorForBuffer } from "./Document";
import { RichOwners } from "./RichInline";
import { api, request } from "../api/client";
import { getWorkspaceRealtime } from "../api/realtime";
import type { RunWsMessage } from "../api/types";
import { acpApi } from "../api/acp";
import { parseHickDoc, proseCodeFences } from "../editor/hickDoc";
import { useHickLangStatus } from "../editor/hickLang";
import { normalizeLanguage } from "../editor/languages";
import { isDebuggable } from "../debug/languages";
import { debugBreakpointLine, debugEditor, debugStateEffects } from "../debug/cmDebug";
import { DebugRailGroup, debugRailElement } from "../debug/DebugRail";
import { DebugStrip } from "../debug/DebugStrip";
import { useWorkspaceDebugger, type DebugSession } from "../debug/useDebugger";
import { debugUTF8, type DebugSourceReading } from "../debug/SourceGuards";
import type { AgentTotals, AgentTurn, AgentUsage } from "../api/agentTypes";
import type { TerminalSession } from "../api/types";
import { simplePatchHunks } from "../lib/diff";
import {sha256,type ReviewEvent,type ReviewTarget} from "../lib/reviews";
import type { ReviewRecordedPatch } from "./AgentProposalAdapter";
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
const toolLabels = new Map<string, [operation: string, fields?: string[]]>([
  ["read_file", ["Read file", ["path"]]],
  ["read_doc", ["Read source", ["upstream", "doc"]]],
  ["read_output", ["Read output", ["path"]]],
  ["edit_doc", ["Propose source edit", ["upstream", "doc"]]],
  ["edit_output", ["Propose output edit", ["path"]]],
  ["create_doc", ["Propose new document", ["path"]]],
  ["search", ["Search workspace"]],
  ["changeset_status", ["Inspect changes"]],
  ["submit_changeset", ["Submit changeset"]],
]);

function toolLabel(name: string, data: unknown): string {
  const [operation, fields] = toolLabels.get(name) ?? [name];
  if (!fields || typeof data !== "string" || data.length > 65_536 || new TextEncoder().encode(data).length > 65_536) return operation;
  try {
    const args: Record<string, unknown> | undefined = JSON.parse(data)?.arguments;
    for (const field of fields) {
      const value = args?.[field];
      if (typeof value !== "string") continue;
      const target = value.replace(/[\r\n\f\u000B\u0085\u2028\u2029]+/g, " ").trim();
      if (target) return `${operation} · ${Array.from(target).slice(0, 140).join("")}`;
    }
  } catch {
    // Older or malformed event metadata still has an operation name.
  }
  return operation;
}

export class AgentSession {
  captureContext?:()=>AgentEditorContext;
  workspaceCurrent = () => true;
  liveAgentContext(): AgentEditorContext | undefined {
    const context = this.captureContext?.();
    if (!context) return undefined;
    return {
      ...context, live: true,
      projection: { version: 1, documents: [], observations: [], estimate: projectionEstimate({ documents: [], observations: [] }) },
    };
  }
  constructor() { this.draft.subscribe(() => this.publish()); }
  turns: AgentTurn[] = [];
  historyLoaded = false;
  messageForks = false;
  historyError = "";
  selection: { id: string | null; fork?: "user" | "assistant" } | undefined;
  get tip() { return this.selection ? this.selection.id : this.turns.at(-1)?.id ?? null; }
  get transcript() { return agentTranscript(this.turns, this.tip, this.selection?.fork === "user"); }
  select(id: string | null, fork?: "user" | "assistant") {
    if (!this.workspaceCurrent() || this.running) return false;
    if (fork && !this.messageForks) return false;
    // Preserve the in-progress draft instead of moving it to another thread.
    if (this.prompt.length) {
      this.error = "Send or clear your current draft before switching conversations.";
      this.publish();
      return false;
    }
    this.selection = { id, fork };
    this.error = "";
    this.publish();
    return true;
  }
  totals?: AgentTotals;
  // Activity is provisional and keyed to the server's run. The history
  // response owns the final answer and the recorded message revision.
  activity = new Map<string, { phase?: string; answer?: string; usage?: AgentUsage; usd?: number | null; tools: { name: string; label: string; result?: string; ok?: boolean }[] }>();
  private early: RunWsMessage[] = [];
  stopPending = false;

  receive(message: RunWsMessage) {
    if (!("run_id" in message && "event" in message) || message.exec_id !== "agent") return;
    if (!this.turns.some(turn => turn.id === message.run_id && turn.status === "running")) {
      if (this.busy && this.early.length < 128) this.early.push(message);
      return;
    }
    const event = message.event;
    const activity = this.activity.get(message.run_id) ?? { tools: [] };
    switch (event.kind) {
      case "thinking": activity.phase = "Waiting for the model…"; break;
      case "token":
        if (activity.phase === "Receiving model output…") return;
        activity.phase = "Receiving model output…"; break;
      case "response_complete": activity.answer = event.text; break;
      case "turn_usage": activity.usage = event.total_usage; activity.usd = event.total_cost_usd; break;
      case "tool_started": {
        const label = toolLabel(event.name, event.data);
        activity.phase = label;
        activity.tools.push({ name: event.name, label });
        break;
      }
      case "tool_finished": {
        const tool = [...activity.tools].reverse().find(tool => tool.name === event.name && tool.ok === undefined);
        if (tool) { tool.ok = event.ok; tool.result = event.text.slice(0, 4000); }
        activity.phase = "Preparing the next step…";
        break;
      }
      default: return;
    }
    this.activity.set(message.run_id, activity);
    this.publish();
  }

  get canStop() { return this.turns.some(turn => turn.status === "running"); }
  async stop() {
    if (!this.canStop || this.stopPending) return;
    this.stopPending = true; this.publish();
    try { await api.agentStop("workspace"); }
    catch (reason) {
      this.error = `${String(reason)} Refresh history and try Stop again.`;
      this.stopPending = false; this.publish();
    }
  }
  provider = "codex-subscription";
  model = "";
  effort = "";
  readonly documents = new Buffers(() => this.publish());
  readonly draft = this.document("draft", "", false);
  get prompt() { return this.draft.text; }
  set prompt(value: string) {
    this.draft.edit(value);
    const view = editorForBuffer(this.draft)?.view;
    if (view && authoredSource(view.state) !== value) view.dispatch(authoredChanges(view.state,[{from:0,to:authoredSource(view.state).length,insert:value}]));
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
      if (view) view.dispatch(authoredChanges(view.state,[{from:0,to:authoredSource(view.state).length,insert:text}]));
    }
    return buffer;
  }
  setTurns(turns: AgentTurn[]) {
    if (!turns.some(turn => turn.status === "running")) this.stopPending = false;
    // Rebase observable document buffers before publishing the conversation.
    // Rendering an existing message must never dispatch editor transactions.
    for (const turn of turns) {
      this.document(`${turn.id}-user`, turn.prompt);
      this.document(`${turn.id}-agent`, turn.answer ?? turn.error ?? "");
    }
    this.turns = turns;
  }
  busy = false;
  error = "";
  mutation = 0;
  liveTurn: string | null = null;
  private version = 0;
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  get running() { return this.busy || this.turns.some(turn => turn.status === "running"); }
  async send() {
    if (!this.workspaceCurrent() || !this.prompt.trim() || this.running) return;
    const text = this.prompt, parent = this.tip, fork = this.selection?.fork;
    const backend = this.turns.find(turn => turn.id === parent)?.backend;
    if (backend && backend !== "python") {
      this.error = "This conversation uses a different agent. Start a new conversation to send with Hickory Python.";
      this.publish();
      return;
    }
    if (fork && !this.messageForks) {
      this.error = "This workspace server does not support message branches. Update the server before sending.";
      this.publish();
      return;
    }
    this.busy = true; this.error = ""; this.mutation++; this.publish();
    try {
      const context = this.liveAgentContext();
      const args = ["workspace", text, this.selection ? parent : parent ?? undefined, this.provider, this.model, "python", context, undefined, this.effort || undefined] as const;
      const started = await (fork ? api.agent(...args, fork) : api.agent(...args));
      if (!this.workspaceCurrent()) return;
      this.liveTurn = context ? started.session_id : null;
      this.selection = { id: started.session_id };
      this.setTurns([...this.turns.filter(turn => turn.id !== started.session_id), { id: started.session_id, prompt: text, answer: null, status: "running", parent_id: parent, parent_is_user: fork === "user", error: null, created_at: "", provider: this.provider, model: this.model, usage: null }]);
      const early = this.early;
      this.early = [];
      for (const message of early) if ("run_id" in message && message.run_id === started.session_id) this.receive(message);
      if (this.prompt === text) this.prompt = "";
    } catch (reason) {
      if (this.workspaceCurrent()) this.error = `${String(reason)} The request was not retried. Refresh history before sending again.`;
    }
    finally { this.early = []; this.busy = false; this.mutation++; this.publish(); }
  }
}
export function agentTranscript(turns: AgentTurn[], tip: string | null, userOnly = false) {
  const byId = new Map(turns.map(turn => [turn.id, turn]));
  const seen = new Set<string>();
  const result: { turn: AgentTurn; userOnly: boolean }[] = [];
  while (tip && !seen.has(tip)) {
    seen.add(tip);
    const turn = byId.get(tip);
    if (!turn) break;
    result.push({ turn, userOnly });
    tip = turn.parent_id;
    userOnly = turn.parent_is_user ?? false;
  }
  return result.reverse();
}

export function AgentHistoryPane({ session, open }: { session: AgentSession; open: () => void }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const { turns } = session;
  const byId = new Map(turns.map(turn => [turn.id, turn]));
  const children = new Map<string | null, AgentTurn[]>();
  for (const turn of turns) {
    const parent = turn.parent_id && byId.has(turn.parent_id) ? turn.parent_id : null;
    const siblings = children.get(parent) ?? [];
    siblings.push(turn);
    children.set(parent, siblings);
  }
  const select = (id: string | null) => { session.select(id); open(); };
  const rows = (parent: string | null, seen: Set<string>): ReactNode => {
    const group = (children.get(parent) ?? []).filter(turn => !seen.has(turn.id));
    return group.length ? <ul>{group.map(turn => {
      const title = turn.prompt.trim().split(/\r?\n/)[0] || "Conversation";
      const source = turn.parent_id && byId.get(turn.parent_id);
      const fork = !!(source && turn.session && source.session && turn.session !== source.session);
      const hasChildren = children.has(turn.id);
      const expanded = hasChildren && !collapsed.has(turn.id);
      const item = <button title={title} disabled={session.running} aria-current={session.tip === turn.id ? "true" : undefined}
        onClick={() => select(turn.id)}>{fork ? `Branch from ${turn.parent_is_user ? "user" : "agent"} · ` : ""}{title}</button>;
      return <li key={turn.id}>
        {hasChildren && <button type="button" aria-label={`Replies to ${title}`} aria-expanded={expanded} onClick={() =>
          setCollapsed(current => { const next = new Set(current); if (next.has(turn.id)) next.delete(turn.id); else next.add(turn.id); return next; })
        }>{expanded ? "▾" : "▸"}</button>}
        {item}{expanded && rows(turn.id, new Set([...seen, turn.id]))}
      </li>;
    })}</ul> : null;
  };
  return <section className="file-tree" aria-label="Agent History">
    <button disabled={session.running} onClick={() => select(null)}>New conversation</button>
    {session.historyError && <p role="alert">{session.historyError}</p>}
    {rows(null, new Set())}
    {!turns.length && !session.historyError && <p role="status">{session.historyLoaded ? "No conversations yet." : "Loading conversations…"}</p>}
  </section>;
}
export function AgentHistory({ session, visible }: { session: AgentSession; visible: boolean }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const observing = visible || session.running;
  useEffect(() => {
    if (!observing) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const current = () => !cancelled && session.workspaceCurrent();
    const unsubscribe = getWorkspaceRealtime()?.onRunEvent(message => { if (current()) session.receive(message); });
    const refresh = async () => {
      const mutation = session.mutation;
      try {
        const response = await api.agentTurns("workspace");
        if (current() && mutation === session.mutation && !session.busy) {
          const own = session.liveTurn;
          session.historyError = "";
          session.historyLoaded = true;
          session.messageForks = response.message_forks === true;
          session.totals = response.totals;
          session.setTurns(response.turns);
          session.publish();
          if (own && response.turns.some(turn => turn.id === own && turn.status === "running")) {
            const state = await request<{ turn: string | null; edits: { captureRequest?: string } }>("GET", "/api/docs/workspace/agent/python");
            const capture = state.edits?.captureRequest;
            if (capture && state.turn === own && current() && mutation === session.mutation && own === session.liveTurn) {
              const context = session.liveAgentContext();
              if (context) await acpApi.edits("workspace", { capture_request: capture, context }, "python");
            }
          } else if (own) {
            session.liveTurn = null;
            session.publish();
          }
        }
      } catch (reason) { if (current() && mutation === session.mutation) { session.historyError = `Cannot read agent history: ${String(reason)}`; session.publish(); } }
      if (current()) timer = setTimeout(refresh, 1000);
    };
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); unsubscribe?.(); };
  }, [session, observing]);
  return null;
}

export function Agent({ session, reviewProposals, reviewRecordedPatch }: {
  session: AgentSession; reviewProposals?: () => void; reviewRecordedPatch?: ReviewRecordedPatch;
}) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const conversation = useRef<HTMLDivElement>(null);
  const following = useRef(true);
  const { transcript, prompt, error } = session;
  const tail = transcript.at(-1)?.turn;
  const liveAnswer = tail && session.activity.get(tail.id)?.answer;
  useEffect(() => {
    if (following.current && conversation.current) conversation.current.scrollTop = conversation.current.scrollHeight;
  }, [tail?.id, tail?.answer, tail?.error, liveAnswer]);
  const running = session.running;
  const inlineRenderNotice = [...session.documents.values.values()].find(buffer => buffer.inlineRenderNotice)?.inlineRenderNotice;
  return <div className="agent-body">
    <RichOwners buffers={session.documents} recorded />
    <header className="agent-heading"><strong>Workspace agent</strong><span role="status" title={inlineRenderNotice}>{inlineRenderNotice ? "Inline rendering unavailable" : "Hickory · Python execution only"}</span><button disabled={running} onClick={() => session.select(null)}>New conversation</button>{reviewProposals&&<button onClick={reviewProposals}>Review pending proposals</button>}</header>
    <div ref={conversation} className="conversation" role="log" aria-label="Conversation" onScroll={event => { const node = event.currentTarget; following.current = node.scrollHeight - node.clientHeight - node.scrollTop < 48; }}>
      {!transcript.length && <div className="agent-empty"><h2>What shall we work on?</h2><p>Write a literate document below. Code, tables and explanations belong together.</p></div>}
      {transcript.map(({ turn, userOnly }) => {
        const activity = session.activity.get(turn.id);
        const streaming = turn.status === "running";
        const answer = turn.answer ?? turn.error ?? activity?.answer;
        return <article key={turn.id}>
          <section className="agent-message user-message"><p className="message-label">You {session.messageForks && (!turn.backend || turn.backend === "python") && <button disabled={running} onClick={() => session.select(turn.id, "user")}>Branch from user message</button>}</p><RecordedMessage turn={turn.id} side="user" reviewRecordedPatch={reviewRecordedPatch} buffer={session.document(`${turn.id}-user`, turn.prompt)} /></section>
          {!userOnly && <section className="agent-message">
            <p className="message-label">Agent{turn.model && ` · ${turn.model}`} {session.messageForks && turn.answer != null && (!turn.backend || turn.backend === "python") && <button disabled={running} onClick={() => session.select(turn.id, "assistant")}>Branch from assistant message</button>}</p>
            {!!activity?.tools.length && <details className="agent-tool-calls"><summary>Tool calls ({activity.tools.length})</summary>
              {activity.tools.map((tool, index) => <details key={index}><summary>{tool.label} · {tool.ok === undefined ? streaming ? "Running" : "Outcome unavailable" : tool.ok ? "Completed" : "Failed"}</summary>{tool.result && <pre>{tool.result}</pre>}</details>)}
            </details>}
            {streaming && <div className="agent-progress" role="status"><span aria-hidden="true" />{session.stopPending ? "Stopping…" : activity?.phase ?? "Starting…"}</div>}
            {answer && (streaming || !answer.includes('xmlns:') && !proseCodeFences(parseHickDoc(answer), answer).length
              ? <MarkdownMessage text={answer} />
              : <RecordedMessage turn={turn.id} side="assistant" reviewRecordedPatch={reviewRecordedPatch} recorded={turn.answer != null} buffer={session.document(`${turn.id}-agent`, answer)} />)}
          </section>}
        </article>;
      })}
    </div>
    {session.selection?.fork && <p className="message-label" role="status">New branch from {session.selection.fork === "user" ? "user" : "assistant"} message. Send to start it.</p>}
    {error && <div role="alert" className="notice">{error}</div>}
    <form className="agent-composer" onSubmit={event => { event.preventDefault(); void session.send(); }}>
      <label className="message-label">Your message · Paste any source excerpts you want the model to read.</label>
      <Document buffer={session.draft} />
      <div className="agent-send">
        {session.canStop
          ? <button type="button" aria-label="Stop" title={session.stopPending ? "A stop request is in progress." : "Stop"} disabled={session.stopPending} onClick={() => void session.stop()}>■</button>
          : <button aria-label="Send" title={running ? "Wait for the current operation to finish." : !prompt.trim() ? "Enter a message first." : "Send"} disabled={running || !prompt.trim()}>↑</button>}
      </div>
    </form>
  </div>;
}
const subscriptionChoices = {
  "codex-subscription": { name: "Codex", icon: "⌘", models: ["gpt-6.1-sol"], efforts: ["low", "medium", "high", "xhigh", "max"], login: "codex login" },
  "claude-subscription": { name: "Claude", icon: "✳", models: ["sonnet", "opus", "haiku"], efforts: ["low", "medium", "high", "xhigh", "max"], login: "claude auth login" },
  "grok-subscription": { name: "Grok", icon: "◈", models: ["grok-build"], efforts: [], login: "grok login" },
};

export function AgentStatus({ session }: { session: AgentSession }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const [open, setOpen] = useState(false);
  const [custom, setCustom] = useState(false);
  const anchor = useRef<HTMLButtonElement>(null);
  const panel = useRef<HTMLDivElement>(null);
  const provider = session.provider as keyof typeof subscriptionChoices;
  const choice = subscriptionChoices[provider];
  const [codexModels, setCodexModels] = useState<string[]>([]);
  const models = provider === "codex-subscription" && codexModels.length ? codexModels : choice.models;
  useEffect(() => {
    if (!open || provider !== "codex-subscription") return;
    let current = true;
    void request<{ models: string[] }>("GET", "/api/agents/codex-subscription/models").then(value => {
      if (!current || !value?.models?.length) return;
      setCodexModels(value.models);
      if (value.models.includes(session.model)) setCustom(false);
    }, () => undefined);
    return () => { current = false; };
  }, [open, provider, session]);
  const change = (key: "model" | "effort", value: string) => { session[key] = value; session.publish(); };
  const totals = session.totals;
  const active = session.turns.find(turn => turn.status === "running");
  const reported = active && session.activity.get(active.id);
  const usage = reported?.usage;
  const runningTokens = usage && usage.input_tokens + usage.output_tokens + usage.cache_read_input_tokens + usage.cache_creation_input_tokens;
  const recordedTokens = totals && session.turns.some(turn => turn.usage) ? totals.input + totals.output + totals.cache_read + totals.cache_write : undefined;
  const tokens = runningTokens ? (recordedTokens ?? 0) + runningTokens : recordedTokens;
  const usd = !totals?.subscription && (!active || !active.provider.endsWith("-subscription"))
    && totals?.usd != null && (!active || reported?.usd != null)
    ? totals.usd + (reported?.usd ?? 0) : null;
  const cost = usd == null ? "Cost unavailable" : `$${usd.toFixed(4)}`;
  useEffect(() => {
    if (!open) return;
    panel.current?.querySelector("select")?.focus();
    const dismiss = (event: PointerEvent) => {
      if (event.target instanceof Node && !panel.current?.contains(event.target) && !anchor.current?.contains(event.target)) setOpen(false);
    };
    const escape = (event: KeyboardEvent) => {
      if (event.key === "Escape") { setOpen(false); anchor.current?.focus(); }
    };
    document.addEventListener("pointerdown", dismiss);
    document.addEventListener("keydown", escape);
    return () => { document.removeEventListener("pointerdown", dismiss); document.removeEventListener("keydown", escape); };
  }, [open]);
  return <>
    <button ref={anchor} className="agent-status-trigger" aria-label="Agent settings" aria-expanded={open} aria-haspopup="dialog"
      title={`${choice.name} · ${session.model || choice.models[0]} · Effort ${session.effort || "default"}`}
      onClick={() => { setCustom(!!session.model && !models.includes(session.model)); setOpen(!open); }}>
      <span aria-hidden="true">{choice.icon}</span> {session.model || choice.models[0]} <span aria-hidden="true">◴</span> {session.effort || "Default"}
    </button>
    <span title={`Workspace agent usage · ${cost}`}>{tokens === undefined ? "Tokens unavailable" : `${tokens.toLocaleString()} tokens`}{tokens !== undefined && ` · ${cost}`}</span>
    {open && createPortal(<div ref={panel} role="dialog" aria-label="Agent settings" className="agent-settings"
      style={{ left: Math.max(8, Math.min(anchor.current?.getBoundingClientRect().left ?? 8, window.innerWidth - 298)), bottom: window.innerHeight - (anchor.current?.getBoundingClientRect().top ?? window.innerHeight) + 4 }}>
      <label>Provider<select aria-label="Provider" disabled={session.running} title={session.running ? "Wait for the current operation to finish." : undefined} value={provider} onChange={event => {
        session.provider = event.target.value; session.model = ""; session.effort = ""; setCustom(false); session.publish();
      }}>{Object.entries(subscriptionChoices).map(([id, option]) => <option key={id} value={id}>{option.name} subscription</option>)}</select></label>
      <label>Model<select aria-label="Model" disabled={session.running} title={session.running ? "Wait for the current operation to finish." : undefined} value={custom ? "custom" : session.model} onChange={event => {
        setCustom(event.target.value === "custom");
        if (event.target.value !== "custom") change("model", event.target.value);
      }}><option value="">Provider default ({choice.models[0]})</option>
        {models.map(model => <option key={model}>{model}</option>)}<option value="custom">Custom model…</option>
      </select></label>
      {custom && <label>Custom model<input aria-label="Custom model" disabled={session.running} title={session.running ? "Wait for the current operation to finish." : undefined} value={session.model} onChange={event => change("model", event.target.value)} /></label>}
      <label>Effort<select aria-label="Effort" disabled={session.running || !choice.efforts.length} title={session.running ? "Wait for the current operation to finish." : !choice.efforts.length ? "The selected model does not expose effort settings." : undefined} value={session.effort} onChange={event => change("effort", event.target.value)}>
        <option value="">{choice.efforts.length ? "Default" : "Managed by provider"}</option>
        {choice.efforts.map(effort => <option key={effort} value={effort}>{effort}</option>)}
      </select></label>
      <small>Model presets depend on subscription access. Sign in on the workspace server with <code>{choice.login}</code>.</small>
    </div>, document.body)}
  </>;
}

type MessageSide = "user" | "assistant";
type FenceCapture = {
  version: number; id: string; uri: string; program: string; workspace: string;
  opening_line: number; source_hash: string; started: boolean; turn: string; side: MessageSide;
};
type MessageFence = { first: number; last: number; language: string; kind: "code" | "patch" | "construction" };
type FenceRun = Pick<FenceCapture, "version" | "turn" | "side" | "source_hash" | "opening_line" | "workspace"> & {
  terminal: TerminalSession;
};
type ConstructionValue = { kind: string; executable: boolean; bytes: number; sha256: string; text?: string | null;
  text_status: "exact" | "over_limit" | "non_utf8" };
type FenceConstruction = Omit<FenceRun, "terminal"> & {
  read_only: true; entry: string; document: string; operation_id: string; opening_byte: number;
  changes: { path: string; before: ConstructionValue | null; after: ConstructionValue | null }[];
};
// The durable thread carries the exact recorded message hash and row. Refreshing
// or replying uses the workspace review store without changing the message.
function RecordedPatchComment({source,turn,row,current,close}:{
  source:string;turn:string;row:{line:number;opening:number;quote:string};current:()=>boolean;close:()=>void;
}) {
  const [history,setHistory]=useState<{scope:string;events:ReviewEvent[]}>();
  const [target,setTarget]=useState<ReviewTarget>(),[thread,setThread]=useState("");
  const [body,setBody]=useState(""),[notice,setNotice]=useState(""),[busy,setBusy]=useState(false);
  const alive=useRef(true);
  const load=()=>request<{scope:string;events:ReviewEvent[]}>("GET","/api/reviews");
  useEffect(()=>{
    alive.current=true;
    void Promise.all([load(),sha256(source)]).then(([value,hash])=>{
      if(!alive.current||!current())return;
      setHistory(value);
      setTarget({kind:"patch-line",path:`hick-agent-message:///${turn}-agent.md`,quote:row.quote,source_hash:hash,
        lines:[row.line+1,row.line+1],relationship:{turn,side:"assistant",patch_opening_line:row.opening+1}});
    }).catch(error=>{if(alive.current)setNotice(String(error));});
    return ()=>{alive.current=false;};
  },[source,turn,row]);
  const threads=history?.events.filter(event=>event.target?.kind==="patch-line"&&event.target.path===target?.path
    &&event.target.source_hash===target?.source_hash&&event.target.lines[0]===target?.lines[0]&&event.target.quote===target?.quote)??[];
  const save=async()=>{
    if(!current()||!history||!target||busy||!body.trim())return;
    setBusy(true);setNotice("");
    try {
      const saved=await request<ReviewEvent>("POST","/api/reviews",{scope:history.scope,thread:thread||null,target:thread?null:target,action:"comment",body});
      if(!alive.current||!current())return;
      setBody("");setThread(saved.thread);
      setHistory(await load());
    } catch(error) {if(alive.current){setHistory(undefined);setNotice(String(error)+" Refresh comments before retrying.");}}
    finally {if(alive.current)setBusy(false);}
  };
  return createPortal(<section role="dialog" aria-label={`Patch line ${row.line+1} comments`} className="simple-workspace-problems simple-source-tests"
      onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();close();}}}>
    <header><strong>Patch line {row.line+1}</strong><button type="button" onClick={close}>Close</button></header>
    <pre>{row.quote}</pre>
    {threads.length>0 && <select aria-label="Review thread" value={thread} onChange={event=>setThread(event.target.value)}>
      <option value="">New thread</option>{threads.map(root=><option key={root.id} value={root.thread}>{root.body.slice(0,70)}</option>)}
    </select>}
    {thread && history?.events.filter(event=>event.thread===thread).map(event=><p key={event.id}>{event.body}</p>)}
    <textarea autoFocus aria-label={thread?"Write a reply":"Write a comment"} value={body} onChange={event=>setBody(event.target.value)}/>
    <button type="button" disabled={!history||!target||busy||!body.trim()} title={!history ? "Wait for the comment history to load." : !target ? "Select a comment target." : busy ? "An operation is already in progress." : !body.trim() ? "Enter a comment first." : undefined} onClick={()=>void save()}>{thread?"Save reply":"Save comment"}</button>
    <button type="button" disabled={busy} title={busy ? "An operation is already in progress." : undefined} onClick={()=>void load().then(value=>{if(alive.current&&current()){setHistory(value);setNotice("");}}).catch(error=>setNotice(String(error)))}>Refresh comments</button>
    {notice&&<p role="status">{notice}</p>}
  </section>,document.body);
}

function constructionValue(value: ConstructionValue | null): boolean {
  return value === null || !!value && typeof value.kind === "string" && typeof value.executable === "boolean"
    && Number.isSafeInteger(value.bytes) && value.bytes >= 0 && /^[a-f0-9]{64}$/.test(value.sha256)
    && (value.text == null || typeof value.text === "string");
}

function RecordedMessage({ turn, side, buffer, recorded = true, reviewRecordedPatch }: {
  turn: string; side: MessageSide; buffer: Buffer; recorded?: boolean; reviewRecordedPatch?: ReviewRecordedPatch;
}) {
  const source = buffer.text;
  return <Document buffer={buffer} renderTools={recorded ? view =>
    <RecordedFenceControls key={source} source={source} turn={turn} side={side} view={view} reviewRecordedPatch={reviewRecordedPatch}
      current={() => buffer.text === source && authoredSource(view.state) === source} /> : undefined} />;
}

function releaseFence(id: string) {
  if (/^[a-f0-9]{16}$/.test(id)) void request("DELETE", `/api/project-contexts/${id}`).catch(() => {});
}

export function RecordedFenceControls({ source, turn, side, view, current: sourceCurrent, conversation = "workspace", lineOffset = 0, reviewRecordedPatch }: {
  source: string; turn: string; side: MessageSide; view: EditorView; current: () => boolean;
  conversation?: string; lineOffset?: number; reviewRecordedPatch?: ReviewRecordedPatch;
}) {
  const parser = useHickLangStatus();
  const fences = useMemo<MessageFence[]>(() => {
    if (parser !== "ready" || source.length > 1024 * 1024) return [];
    const display = normalizedSource(source);
    const lines = Text.of(display.split("\n"));
    const structure = parseHickDoc(display);
    // The shared parser represents a bare Markdown patch fence as a patch
    // block. Let the fence scan see that opening while keeping its enclosing
    // verbatim blocks masked. Declared project patches keep their own tools.
    const ordinaryPatch = (block: typeof structure.blocks[number]) => block.name === "patch"
      && Object.keys(block.attrs).length === 0
      && /^ {0,3}(```|~~~)[ \t]*patch[ \t]*$/.test(lines.lineAt(block.from).text);
    const scan = { ...structure, blocks: structure.blocks.filter(block => !ordinaryPatch(block)) };
    const ordinary = proseCodeFences(scan, display).flatMap<MessageFence>(({ closed, info, from, to, body }) => {
      const token = info.split(/\s/, 1)[0];
      const patch = token === "diff" || token === "patch";
      const language = patch ? token : normalizeLanguage(token);
      if (!closed || !language || (!patch && !isDebuggable(language)) || info !== token || !body.trim()) return [];
      const declaration = structure.blocks.find(block => block.from === from && ["file", "exec", "output", "patch"].includes(block.name));
      if (declaration && !(patch && ordinaryPatch(declaration))) return [];
      return [{ first: lines.lineAt(from).number - 1, last: lines.lineAt(to).number - 1, language, kind: patch ? "patch" : "code" }];
    });
    const declared = scan.blocks.flatMap<MessageFence>(block => {
      if (block.name !== "patch" || !block.close || block.contentTo <= block.contentFrom) return [];
      return [{ first: lines.lineAt(block.from).number - 1, last: lines.lineAt(block.to).number - 1, language: "patch", kind: "construction" }];
    });
    // A line with multiple declaration openings cannot identify one operation.
    const count = new Map<number, number>();
    for (const fence of [...ordinary, ...declared]) count.set(fence.first, (count.get(fence.first) ?? 0) + 1);
    return [...ordinary, ...declared].filter(fence => count.get(fence.first) === 1);
  }, [source, parser]);
  const commentRows = useMemo(() => {
    if (side !== "assistant") return [];
    const rows = source.split("\n");
    return fences.filter(fence => fence.kind !== "code").flatMap(fence =>
      simplePatchHunks(rows,fence.first+1,fence.last).flatMap(hunk => hunk.changed.map(line=>({line,opening:fence.first,quote:rows[line]}))));
  },[side,source,fences]);
  const [commentLine,setCommentLine]=useState<number>();
  const [marks, setMarks] = useState<number[]>([]);
  const [capture, setCapture] = useState<{ receipt: FenceCapture; fence: MessageFence } | null>(null);
  const [run, setRun] = useState<{ receipt: FenceRun; first: number } | null>(null);
  const [construction, setConstruction] = useState<{ first: number; receipt?: FenceConstruction; error?: string } | null>(null);
  const [preparing, setPreparing] = useState(false);
  const [message, setMessage] = useState("");
  const generation = useRef(0);
  const inFlight = useRef(false);
  const session = useRef<DebugSession | null>(null);
  const [sessionState, setSessionState] = useState<DebugSession | null>(null);
  const current = () => view.dom.isConnected && sourceCurrent();
  useEffect(() => () => { generation.current++; }, [source, view]);
  useEffect(() => () => { if (capture) releaseFence(capture.receipt.id); }, [capture]);
  const localState = useRef({ capture, marks });
  localState.current = { capture, marks };
  useEffect(() => {
    if (!fences.length) return;
    const compartment = new Compartment();
    const ownsLine = (fence: MessageFence, line: number) => fence.kind === "code" && line > fence.first && line < fence.last;
    view.dispatch({ effects: [StateEffect.appendConfig.of(compartment.of([
      debugBreakpointLine.of(line => {
        if (line < 0 || line >= view.state.doc.lines || !view.state.doc.line(line + 1).text.trim()) return null;
        return fences.some(fence => ownsLine(fence, line - lineOffset)) ? line : null;
      }),
      ...debugEditor({ lineNumbers: false,
        onToggleBreakpoint: position => {
          const line = position - lineOffset;
          const held = localState.current.capture;
          if (held && ownsLine(held.fence, line) && session.current) session.current.toggleBreakpoint(line);
          else setMarks(previous => previous.includes(line) ? previous.filter(mark => mark !== line) : [...previous, line]);
        },
        onSelectFrame: id => session.current?.selectFrame(id),
        onEvaluate: label => session.current?.query(label) ?? Promise.resolve(null),
        evaluationOwner: () => session.current?.reading,
        onAddWatch: label => session.current?.addWatch(label),
      }),
    ])), view.scrollSnapshot()] });
    return () => { view.dispatch({ effects: compartment.reconfigure([]) }); };
  }, [fences, view, lineOffset]);
  useEffect(() => {
    if (!fences.length) return;
    const paused = sessionState?.positionSourceBound === false ? null : sessionState?.pausedLine ?? null;
    const pausedLine = paused === null ? null : paused + lineOffset;
    const localMarks = marks.filter(line => !capture || line <= capture.fence.first || line >= capture.fence.last);
    view.dispatch({ effects: debugStateEffects({
      breakpoints: [...(sessionState?.breakpoints ?? []), ...localMarks.map(line => ({ line, state: "pending" as const }))]
        .map(mark => ({ ...mark, line: mark.line + lineOffset })),
      pausedLine, variables: pausedLine === null ? [] : sessionState?.variables ?? [],
      frames: pausedLine === null ? [] : (sessionState?.frames ?? []).map(frame => ({ ...frame, line: frame.line == null ? frame.line : frame.line + lineOffset })),
      watches: sessionState?.watches ?? [],
    }) });
  }, [view, fences, capture, sessionState, marks, lineOffset]);

  const launch = async (fence: MessageFence, anchor: HTMLElement, action: "run" | "debug" | "patch" | "construction") => {
    if (inFlight.current || session.current?.isActive?.() || !current()) return;
    inFlight.current = true;
    const ticket = ++generation.current;
    setPreparing(true);
    setMessage("");
    try {
      const digest = await crypto.subtle.digest("SHA-256", debugUTF8(source));
      const hash = Array.from(new Uint8Array(digest), byte => byte.toString(16).padStart(2, "0")).join("");
      if (generation.current !== ticket || !current()) return;
      const address = { side, source_hash: hash, opening_line: fence.first };
      if (action === "patch") {
        await reviewRecordedPatch?.(turn, address, () => generation.current === ticket && current(), anchor);
        return;
      }
      const endpoint = `/api/docs/${encodeURIComponent(conversation)}/agent/turns/${encodeURIComponent(turn)}/fence-${action}`;
      const matches = (receipt: FenceRun | FenceCapture | FenceConstruction) => receipt.version === 1 && receipt.turn === turn && receipt.side === side
        && receipt.source_hash === hash && receipt.opening_line === fence.first && !!receipt.workspace;
      if (action === "construction") {
        setConstruction({ first: fence.first });
        const receipt = await request<FenceConstruction>("POST", endpoint, address);
        if (generation.current !== ticket || !current()) return;
        if (!matches(receipt) || receipt.read_only !== true || !receipt.document || !receipt.entry || !receipt.operation_id
          || !Number.isSafeInteger(receipt.opening_byte) || receipt.opening_byte < 0
          || !Array.isArray(receipt.changes) || !receipt.changes.every(change => typeof change?.path === "string"
            && constructionValue(change.before) && constructionValue(change.after))) {
          throw Error("The engine could not confirm this recorded construction reading.");
        }
        setConstruction({ first: fence.first, receipt });
        return;
      }
      if (action === "run") {
        setRun(null);
        const receipt = await request<FenceRun>("POST", endpoint, address);
        if (generation.current !== ticket || !current()) return;
        if (!matches(receipt) || !/^[a-zA-Z0-9-]+$/.test(receipt.terminal?.id ?? "")) {
          throw Error("The engine could not confirm this fence run. Check Terminal for the dispatched process.");
        }
        setRun({ receipt, first: fence.first });
        return;
      }
      const receipt = await request<FenceCapture>("POST", endpoint, address);
      if (generation.current !== ticket || !current()) { releaseFence(receipt.id); return; }
      if (!matches(receipt) || receipt.started !== false || !/^[a-f0-9]{16}$/.test(receipt.id)
        || receipt.uri !== `hick-project:///${receipt.id}/message.md`) {
        releaseFence(receipt.id);
        throw Error("The engine could not bind the selected fence. Refresh the conversation and choose it again.");
      }
      session.current = null;
      setSessionState(null);
      setCapture({ receipt, fence });
    } catch (error) {
      if (generation.current !== ticket) return;
      if (action === "construction") setConstruction({ first: fence.first, error: String(error) });
      else setMessage(String(error));
    }
    finally {
      inFlight.current = false;
      if (generation.current === ticket) setPreparing(false);
    }
  };
  const rail = debugRailElement(view);
  const activeComment=commentRows.find(row=>row.line===commentLine);
  return <>
    {rail && current() && createPortal(commentRows.filter(row=>row.line+lineOffset<view.state.doc.lines).map(row=>
      <DebugRailGroup key={`comment:${row.line}`} view={view} at={view.state.doc.line(row.line+lineOffset+1).from}>
        <button type="button" className="debug-icon" aria-label={`Comment on patch line ${row.line+1}`} onClick={()=>setCommentLine(row.line)}>◫</button>
      </DebugRailGroup>),rail)}
    {activeComment&&current()&&<RecordedPatchComment key={activeComment.line} source={source} turn={turn} row={activeComment} current={current} close={()=>{setCommentLine(undefined);view.focus();}}/>}
    {rail && createPortal(fences.filter(fence => fence.first + lineOffset < view.state.doc.lines && (fence.kind !== "patch" || reviewRecordedPatch))
      .map(fence => <DebugRailGroup key={fence.first} view={view} at={view.state.doc.line(fence.first + lineOffset + 1).from}>
      {fence.kind === "patch" ? <button type="button" className="debug-icon" disabled={preparing || !!sessionState?.isActive?.()} title={preparing ? "An agent fence action is being prepared." : !!sessionState?.isActive?.() ? "Stop the current execution or wait for it to finish." : undefined}
        aria-label={`Review patch fence on line ${fence.first + 1}`}
        onClick={event => { void launch(fence, event.currentTarget, "patch"); }}>≋</button>
      : <details className="more-actions" onKeyDown={event => {
        if (event.key !== "Escape") return;
        event.currentTarget.open = false;
        event.currentTarget.querySelector("summary")?.focus();
        event.stopPropagation();
      }}>
        <summary aria-label={fence.kind === "construction" ? `Review construction patch on line ${fence.first + 1}` : `Fence actions on line ${fence.first + 1}`}
          onClick={event => {
            if (fence.kind !== "construction") return;
            const disclosure = event.currentTarget.parentElement as HTMLDetailsElement;
            if (disclosure.open) return;
            if (inFlight.current || session.current?.isActive?.() || !current()) event.preventDefault();
            else void launch(fence, event.currentTarget, "construction");
          }}>{fence.kind === "construction" ? "≋" : "⋯"}</summary>
        <div className="more-actions__items">
          {fence.kind === "construction" ? <section aria-label="Recorded construction patch">
            {construction?.first !== fence.first || !construction.receipt && !construction.error ? <p role="status">Reading the captured construction…</p>
              : construction.error ? <p role="alert">{construction.error}</p>
              : construction.receipt && <RecordedConstructionReading receipt={construction.receipt} />}
          </section> : <>
          <button type="button" disabled={preparing || !!sessionState?.isActive?.()} title={preparing ? "An agent fence action is being prepared." : !!sessionState?.isActive?.() ? "Stop the current execution or wait for it to finish." : undefined}
            aria-label={`Run ${fence.language} fence on line ${fence.first + 1}`}
            onClick={event => { void launch(fence, event.currentTarget, "run"); }}>Run</button>
          <button type="button" disabled={preparing || !!sessionState?.isActive?.()} title={preparing ? "An agent fence action is being prepared." : !!sessionState?.isActive?.() ? "Stop the current execution or wait for it to finish." : undefined}
            aria-label={`Debug ${fence.language} fence on line ${fence.first + 1}`}
            onClick={event => {
              event.currentTarget.closest("details")?.removeAttribute("open");
              void launch(fence, event.currentTarget, "debug");
            }}>Debug</button>
          {run?.first === fence.first && <RecordedRunOutput key={run.receipt.terminal.id} terminal={run.receipt.terminal} />}
          </>}
        </div>
      </details>}
    </DebugRailGroup>), rail)}
    {message && <p role="alert" className="notice">{message}</p>}
    {capture && <ActiveFenceDebugger key={capture.receipt.id} capture={capture.receipt} source={source}
      view={view} current={current} lineOffset={lineOffset} marks={marks.filter(line => line > capture.fence.first && line < capture.fence.last)}
      publish={debug => { session.current = debug; setSessionState(debug); }} />}
  </>;
}

function RecordedConstructionReading({ receipt }: { receipt: FenceConstruction }) {
  return <>
    <p>{receipt.document} · {receipt.operation_id}</p>
    {receipt.changes.length === 0 ? <p>This patch produced no changed outputs.</p> : receipt.changes.map(change => <details key={change.path}>
      <summary style={{ display: "block", width: "auto", height: "auto", overflowWrap: "anywhere" }}>{change.path}</summary>
      {([["Before", change.before], ["After", change.after]] as const).map(([label, reading]) => <div key={label}>
        <strong>{label}</strong>
        {reading === null ? <p>Absent</p> : reading.text == null ? <p title={reading.sha256}>{reading.kind} · {reading.bytes} bytes · {reading.text_status === "non_utf8" ? "Non-UTF-8" : "Preview limit"}</p>
          : <pre style={{ maxHeight: "10rem", overflow: "auto" }}>{reading.text}</pre>}
      </div>)}
    </details>)}
  </>;
}

function RecordedRunOutput({ terminal }: { terminal: TerminalSession }) {
  const [reading, setReading] = useState({ state: terminal.state as string, exit: terminal.exit_code, output: "" });
  useEffect(() => {
    let retired = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const refresh = async () => {
      try {
        const { sessions } = await api.terminals();
        const live = sessions.find(session => session.id === terminal.id);
        if (!live) throw Error("This terminal session is unavailable.");
        const screen = await api.terminalScreen(terminal.id, 0, "");
        if (retired) return;
        const output = new TextDecoder().decode(new Uint8Array(screen.bytes));
        setReading({ state: live.state, exit: live.exit_code, output });
        if (live.exit_code === null) timer = setTimeout(refresh, 300);
      } catch (error) {
        if (!retired) setReading(previous => ({ ...previous, state: String(error) }));
      }
    };
    void refresh();
    return () => { retired = true; clearTimeout(timer); };
  }, [terminal.id]);
  return <section aria-label="Recorded fence run" style={{ gridColumn: "1 / -1", minWidth: "12rem" }}>
    <span role="status">{reading.exit === null ? reading.state : `Process exit ${reading.exit}`}</span>
    <pre style={{ maxHeight: "12rem", maxWidth: "min(28rem, 65vw)", overflow: "auto" }}>{reading.output}</pre>
  </section>;
}

function ActiveFenceDebugger({ capture, source, view, current, marks, publish, lineOffset }: {
  capture: FenceCapture; source: string; view: EditorView; current: () => boolean; marks: number[];
  publish: (session: DebugSession) => void; lineOffset: number;
}) {
  const reading: DebugSourceReading = { uri: capture.uri, workspace: capture.workspace, namespace: capture.uri,
    provenance: capture.source_hash, identity: capture.id, text: source, saved: true, current };
  const debug = useWorkspaceDebugger(capture.uri, view, current, { read: uri => {
    if (uri !== capture.uri) throw Error("This debugger owns only the recorded message source.");
    return reading;
  }});
  const start = () => debug.start(capture.program, {}, "new", undefined, {
    [capture.uri]: { reading, marks: marks.map(line => ({ line })) },
  });
  useEffect(() => { start(); }, [capture]);
  useEffect(() => { publish(debug); }, [debug]);
  const pausedLine = debug.positionSourceBound === false || debug.pausedLine === null ? null : debug.pausedLine + lineOffset;
  useEffect(() => {
    if (pausedLine === null || pausedLine < 0 || pausedLine >= view.state.doc.lines || !current()) return;
    view.dispatch({ effects: EditorView.scrollIntoView(view.state.doc.line(pausedLine + 1).from, { y: "nearest" }) });
  }, [pausedLine, view]);
  return <DebugStrip editorView={view} pausedLine={pausedLine} status={debug.status} program={debug.program}
    message={debug.message} capabilities={debug.capabilities} frames={debug.frames} selectedFrame={debug.selectedFrame}
    variables={debug.variables} watches={debug.watches} exitCode={debug.exitCode} buildOutput={debug.buildOutput}
    onSelectFrame={debug.selectFrame} onStep={debug.step} onStop={debug.stop} onStart={start}
    onJumpHere={() => debug.jumpTo(view.state.doc.lineAt(view.state.selection.main.head).number - 1 - lineOffset)}
    onAddWatch={() => { const expression = window.prompt("Expression to watch"); if (expression) debug.addWatch(expression); }}
    exceptionFilters={debug.exceptionFilters} onToggleExceptionFilter={debug.toggleExceptionFilter} onRemoveWatch={debug.removeWatch} />;
}
