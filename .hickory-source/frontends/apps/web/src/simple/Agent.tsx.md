<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep one explicit workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The workspace owns the session so moving or hiding its tab retains the literate draft and in-flight send. Explicit subscription choices always use the Hickory Python backend; the backend refuses missing runtime or sign-in rather than selecting an API provider. Durable history refreshes independently and stops when unmounted. Tabs and history rows read the same optional title. A short excerpt is available on the first refresh. Each tab reads late titles through the shared workspace history, so an inactive tab updates without restarting its transcript watcher.

Prompts, provisional answers and accepted answers use the same Document surface as
files and the response draft. Mount readable text immediately; parser readiness
only adds capabilities to that editor. Keep the answer buffer and editor identity
when a running turn becomes recorded, so selection and scroll survive. Apply
minimal authored edits when its text advances; replacing the whole buffer would
collapse a selection in the unchanged prefix. A running history refresh keeps
the current provisional answer until an accepted answer replaces it; briefly
rebasing it to empty would lose the reading selection. Rebase completion events
before publishing so the render only observes already-current editors. Provisional
answers remain immutable and gain recorded fence actions only after the host has
accepted their exact source. Plain Markdown follows this same path, so adding a
code fence does not replace the reading surface.

A composer’s source replacement uses the shared authored edit builder, preserving explicit CRLF and mixed separators along with the existing editor history. Read checks compare exact authored content rather than normalized display text.

Run and Debug share one gutter disclosure on the recorded opening line.
Run sends the recorded address on an explicit click and reads the returned
terminal only while the mounted card remains current. The output stays in
the floating disclosure so it cannot shift the message or the next gutter.

A declared patch looks up the exact operation in a retained project capture.
Its opening-line disclosure shows the compiler's Before and After readings
and lets the engine explain when the capture cannot establish the operation.

History opens a fresh conversation tab without changing any existing tab's
selection, composer, settings or editor buffers. Each session pins its selected
tip; durable history can grow without moving that tip. Workspace run exclusion
remains authoritative, while Stop and usage belong only to the selected ancestry.
Usage sums recorded turns and provisional running usage once; unknown usage is
left unknown rather than borrowing the workspace total. Branch controls follow
the message they retain. Shift+Enter sends; Enter remains the editor's newline.

Format tool payloads only for reading: preserve raw input/result and JSON lexemes, including duplicate keys and large integers. Invalid or over-budget text stays literal. A branch draft lives only in the composer; its recorded source never becomes editable. Assistant drafts use an explicit idempotent save with the original source hash, then return to a user-message composer. Empty conversations have known zero usage; missing usage for an actual model turn stays unknown.

<ui:copy id="simple-981c695688-0">
import {authoredSource,authoredChanges,normalizedSource} from './SourceText';
import { useEffect, useMemo, useRef, useState, useSyncExternalStore, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { Compartment, StateEffect, Text } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { Buffer, Buffers } from "./buffers";
import type {AgentEditorContext} from "../api/agentTypes";
import { projectionEstimate } from "../lib/contextProjection";
import { Document, editorForBuffer } from "./Document";
import { RichOwners } from "./RichInline";
import { api, request } from "../api/client";
import { getWorkspaceRealtime } from "../api/realtime";
import type { RunWsMessage } from "../api/types";
import { acpApi, type AgentChange } from "../api/acp";
import { parseHickDoc, proseCodeFences } from "../editor/hickDoc";
import { useHickLangStatus } from "../editor/hickLang";
import { normalizeLanguage } from "../editor/languages";
import { isDebuggable } from "../debug/languages";
import { debugBreakpointLine, debugEditor, debugStateEffects } from "../debug/cmDebug";
import { DebugRailGroup, debugRailElement } from "../debug/DebugRail";
import { DebugStrip } from "../debug/DebugStrip";
import { useWorkspaceDebugger, type DebugSession } from "../debug/useDebugger";
import { debugUTF8, type DebugSourceReading } from "../debug/SourceGuards";
import { agentTurnLabel, type AgentTotals, type AgentTurn, type AgentUsage } from "../api/agentTypes";
import type { TerminalSession } from "../api/types";
import { computeEdits, simplePatchHunks } from "../lib/diff";
import {sha256,type ReviewEvent,type ReviewTarget} from "../lib/reviews";
import type { ReviewRecordedPatch } from "./AgentProposalAdapter";
</ui:copy>

## Retain documents across pane moves and sends

Use transient Buffer objects with the same literate document editor as workspace
files. Message paths are editor identities only: transient buffers never save or
open language/debug sessions. History uses the read-only buffer path, including
inline grids. The draft retains selection, undo and unsaved text through provider
changes. Subscribe to every draft edit so Send availability follows blankness.
Capture a send once, keep it busy until acknowledged, and clear only that exact
draft; typing during a request belongs to the next message.
Send and edit refresh keep exact buffers for the host. The explicit empty reading
projection keeps those buffers out of the model prompt. Add excerpts to the message
when the model should read them.
The live status and collapsed tool receipt show the operation and a short path
for recognized source and file actions. The event name still pairs each receipt
with its start. Other arguments stay out of the status and receipt label.
An exact disk edit waits for a human decision in the originating window. A
review pins both source versions from that turn and sends their digests to the
host. If a refreshed proposal differs, the previous review cannot authorize it.

<ui:copy id="simple-agent-document-session">
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

export function formattedToolJSON(source: string): string {
  if (source.length > 262144) return source;
  try { JSON.parse(source); } catch { return source; }
  let result = '', depth = 0, quoted = false, escaped = false;
  const newline = () => { result += '\n' + '  '.repeat(depth); };
  for (let index = 0; index < source.length; index++) {
    const char = source[index];
    if (quoted) { result += char; if (escaped) escaped = false; else if (char === '\\') escaped = true; else if (char === '"') quoted = false; continue; }
    if (/\s/.test(char)) continue;
    if (char === '"') { quoted = true; result += char; }
    else if (char === '{' || char === '[') {
      result += char;
      const next = source.slice(index + 1).trimStart()[0];
      if (next !== (char === '{' ? '}' : ']')) { depth++; if (depth > 64) return source; newline(); }
    } else if (char === '}' || char === ']') {
      const previous = source.slice(0, index).trimEnd().at(-1);
      if (previous !== (char === '}' ? '{' : '[')) { depth--; newline(); }
      result += char;
    } else if (char === ',') { result += char; newline(); }
    else result += char === ':' ? ': ' : char;
  }
  return result;
}
export function AgentActivityIcon({working}: {working: boolean}) {
  const label = working ? 'Working' : 'Waiting for input';
  return <span className="agent-activity-icon" aria-hidden="true" title={label}>{working ? '●' : '○'}</span>;
}

export function AgentTabTitle({session, history}: {session: AgentSession; history: AgentSession}) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const transcript = session.transcript;
  const selected = transcript.at(-1)?.turn;
  const opening = transcript.find(({turn}) => turn.session === selected?.session)?.turn;
  const title = useSyncExternalStore(history.subscribe, () => history.turns.find(turn => turn.id === opening?.id)?.title);
  return <><span aria-hidden="true">✦ </span><AgentActivityIcon working={session.working} /> {title ?? (opening ? agentTurnLabel(opening) : "New conversation")}</>;
}

export class AgentSession {
  openedTip: string | null = null;
  conversationKey?: string;
  branchEdit?: {turn: string; side: 'user' | 'assistant'; source: string; request: string};
  get working() {
    if (this.branchEdit) return this.busy;
    const selected = this.turns.find(turn => turn.id === this.tip);
    const active = this.turns.find(turn => turn.status === 'running' && (turn.id === this.tip || !!selected?.session && turn.session === selected.session));
    return this.busy || !!active && !(this.pendingReviews?.turn === active.id && this.pendingReviews.changes.length > 0);
  }
  sameConversation(id: string) {
    if (this.branchEdit) return false;
    const selected = this.turns.find(turn => turn.id === this.tip);
    const requested = this.turns.find(turn => turn.id === id);
    const key = selected?.session ?? this.conversationKey;
    return this.openedTip === id || this.tip === id || !!(key && key === requested?.session);
  }
  conversationScroll = 0;
  following = true;
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
  reviewReadError = "";
  historyReload = 0;
  retryHistory() { this.historyReload++; this.publish(); }
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
  activity = new Map<string, { phase?: string; answer?: string; usage?: AgentUsage; usd?: number | null; tools: { name: string; label: string; input?: string; result?: string; ok?: boolean }[] }>();
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
      case "response_complete":
        activity.answer = event.text;
        this.document(`${message.run_id}-agent`, event.text);
        break;
      case "turn_usage": activity.usage = event.total_usage; activity.usd = event.total_cost_usd; break;
      case "tool_started": {
        const label = toolLabel(event.name, event.data);
        activity.phase = label;
        activity.tools.push({ name: event.name, label, input: event.data });
        break;
      }
      case "tool_finished": {
        const tool = [...activity.tools].reverse().find(tool => tool.name === event.name && tool.ok === undefined);
        if (tool) { tool.ok = event.ok; tool.result = event.text.slice(0, 262144); }
        activity.phase = "Preparing the next step…";
        break;
      }
      default: return;
    }
    this.activity.set(message.run_id, activity);
    this.publish();
  }

  get canStop() { return this.transcript.some(({turn, userOnly}) => !userOnly && turn.status === "running"); }
  async stop() {
    if (!this.workspaceCurrent() || !this.canStop || this.stopPending) return;
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
      if (view) view.dispatch(authoredChanges(view.state, computeEdits(authoredSource(view.state), text).map(edit => ({from:edit.start,to:edit.end,insert:edit.text}))));
    }
    return buffer;
  }
  setTurns(turns: AgentTurn[]) {
    if (!turns.some(turn => turn.status === "running")) this.stopPending = false;
    // Rebase observable document buffers before publishing the conversation.
    // Rendering an existing message must never dispatch editor transactions.
    for (const turn of turns) {
      this.document(`${turn.id}-user`, turn.prompt);
      this.document(`${turn.id}-agent`, turn.answer ?? turn.error ?? (turn.status === "running" ? this.activity.get(turn.id)?.answer : undefined) ?? "");
    }
    const key = turns.find(turn => turn.id === this.tip)?.session;
    if (!this.branchEdit && key) this.conversationKey = key;
    this.turns = turns;
  }
  busy = false;
  error = "";
  mutation = 0;
  liveTurn: string | null = null;
  pendingReviews: { turn: string; changes: AgentChange[] } | null = null;
  reviewBusy = false;
  reviewError = "";
  reviewCurrent(turn: string, preview: AgentChange) {
    return this.workspaceCurrent() && !this.stopPending && this.tip === turn
      && this.liveTurn === turn && this.pendingReviews?.turn === turn
      && this.turns.some(row => row.id === turn && row.status === "running")
      && this.pendingReviews.changes.some(change => change.status === "pending" && change.id === preview.id
        && change.path === preview.path && change.oldText === preview.oldText && change.newText === preview.newText);
  }
  async decideReview(turn: string, preview: AgentChange, accepted: boolean) {
    if (this.reviewBusy || !this.reviewCurrent(turn, preview)) return false;
    this.reviewBusy = true; this.reviewError = ""; this.mutation++; this.publish();
    try {
      const [before, after] = await Promise.all([sha256(preview.oldText), sha256(preview.newText)]);
      if (!this.reviewCurrent(turn, preview)) return false;
      const result = await request<{ received: boolean }>("POST", "/api/docs/workspace/agent/python/review-exact", {
        turn, id: preview.id, accepted, before, after,
      });
      if (!this.workspaceCurrent() || !result.received) return false;
      if (this.pendingReviews?.turn === turn) {
        this.pendingReviews = { turn, changes: this.pendingReviews.changes.filter(change => change.id !== preview.id) };
      }
      return true;
    } catch (reason) {
      if (this.workspaceCurrent()) this.reviewError = `${String(reason)} Refresh the proposal before making another decision.`;
      return false;
    } finally {
      this.reviewBusy = false; this.mutation++; this.publish();
    }
  }
  private version = 0;
  private listeners = new Set<() => void>();
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  snapshot = () => this.version;
  publish() { this.version++; this.listeners.forEach(listener => listener()); }
  get running() { return this.busy || this.turns.some(turn => turn.status === "running"); }
  async saveAssistantBranch() {
    const edit = this.branchEdit;
    if (!edit || edit.side !== 'assistant' || !this.workspaceCurrent() || this.busy) return;
    const text = this.prompt;
    this.busy = true; this.mutation++; this.error = ''; this.publish();
    try {
      const turn = await request<AgentTurn>('POST', '/api/docs/workspace/agent/branch', {turn: edit.turn, source_hash: await sha256(edit.source), answer: text, request_id: edit.request});
      if (!this.workspaceCurrent()) return;
      this.setTurns([...this.turns.filter(row => row.id !== turn.id), turn]); this.selection = {id: turn.id}; this.openedTip = turn.id; this.conversationKey = turn.session;
      if (this.prompt === text) { this.prompt = ''; this.branchEdit = undefined; }
      else this.branchEdit = {turn: turn.id, side: 'assistant', source: text, request: crypto.randomUUID().replaceAll('-', '')};
    } catch (reason) { if (this.workspaceCurrent()) this.error = `${String(reason)} Your edited answer is retained. Refresh history before retrying.`; }
    finally { this.busy = false; this.mutation++; this.publish(); }
  }
  async send() {
    if (this.branchEdit?.side === 'assistant') return this.saveAssistantBranch();
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
      this.conversationKey = fork ? undefined : this.conversationKey ?? this.turns.find(turn => turn.id === parent)?.session;
      this.selection = { id: started.session_id }; this.openedTip = started.session_id; this.branchEdit = undefined;
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
</ui:copy>

## Refresh history independently of the composer

The completion supervisor asks for a fresh editor snapshot before it accepts
an artifact. Only the pane that started the turn answers that request. Capture
the current authored buffers after observing the request and keep the engine's
request ID, so an expired request cannot approve a later turn.

Send captures the retained authored editor roster and its original loaded guards.
It marks the request live so the originating turn can ask for another capture.
A later capture serves only the matching host request and leaves the initial
snapshot intact. Applying a sealed proposal still requires its own original
source authority. Conversation claims carry no implementation or test evidence.

History polling never recreates the draft editor. Reject replies from before a
send mutation. Poll while the pane is visible or its own live turn can still
request an editor capture. Provider failures remain
visible beside the retained draft; no request is automatically retried.

Render only the selected ancestry in the composer. The incoming edge of each
turn decides whether its parent answer belongs to that ancestry. The sidebar
uses the same IDs but retains siblings so a fork never hides its origin.

Load organization alongside the turn tree. When a filter hides a parent,
keep its visible branch at the top level and use the full tree for the
branch's origin label. A failed write leaves the last confirmed flags visible
and asks the reader to reload them before making another change.

Sort the top-level entries using the newest recorded message in each session.
A continuation can move its conversation up without moving the reply inside
that conversation. A fork keeps its original parent for replay and its branch
label, but gets a top-level slot so its own favorite and date decide its position.
The status strip counts the same filtered session identities. Rows already available
remain readable while a listing or organization request fails. Retry restarts
the failed reading through the observer or organization loader.

<ui:copy id="simple-agent-ancestry">
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

type ConversationFlags = { favorite: boolean; archived: boolean };
type ConversationHistoryView = "History" | "Favorites" | "Archived";
const conversationHistoryPath = "/api/docs/workspace/agent/history";

export function AgentHistoryPane({ session, open }: { session: AgentSession; open: (id: string | null) => void }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const [collapsed, setCollapsed] = useState<Set<string>>(() => new Set());
  const [view, setView] = useState<ConversationHistoryView>("History");
  const [flags, setFlags] = useState<Record<string, ConversationFlags>>();
  const [organizationError, setOrganizationError] = useState("");
  const [busy, setBusy] = useState(false);
  const [reload, setReload] = useState(0);
  useEffect(() => {
    let active = true;
    setOrganizationError("");
    setBusy(true);
    const readFlags = async () => {
      try {
        const saved = await request<Record<string, ConversationFlags>>("GET", conversationHistoryPath);
        if (active && session.workspaceCurrent()) setFlags(saved);
      } catch (error) {
        if (active && session.workspaceCurrent()) setOrganizationError(`Cannot read conversation organization: ${String(error)}`);
      } finally {
        if (active) setBusy(false);
      }
    };
    void readFlags();
    return () => { active = false; };
  }, [session, reload]);

  const update = async (turn: AgentTurn, change: Partial<ConversationFlags>) => {
    if (busy || organizationError || !session.workspaceCurrent()) return;
    setBusy(true);
    try {
      const saved = await request<Record<string, ConversationFlags>>("POST", conversationHistoryPath, { turn: turn.id, ...change });
      if (session.workspaceCurrent()) setFlags(saved);
    } catch (error) {
      if (session.workspaceCurrent()) setOrganizationError(`Cannot confirm the change: ${String(error)}`);
    } finally { setBusy(false); }
  };

  const { turns } = session;
  const byId = new Map(turns.map(turn => [turn.id, turn]));
  const stateOf = (turn: AgentTurn) => turn.session ? flags?.[turn.session] : undefined;
  const visible = turns.filter(turn => {
    const state = stateOf(turn);
    if (view === "Archived") return state?.archived === true;
    return !state?.archived && (view === "History" || state?.favorite);
  });
  const identity = (turn: AgentTurn) => turn.session || turn.id;
  const visibleById = new Map(visible.map(turn => [turn.id, turn]));
  const newest = new Map<string, number>();
  const children = new Map<string | null, AgentTurn[]>();
  for (const turn of visible) {
    const key = identity(turn);
    const at = turn.last_message_at_ms ?? Date.parse(turn.created_at);
    if (Number.isFinite(at) && at > (newest.get(key) ?? -Infinity)) newest.set(key, at);

    const source = turn.parent_id ? visibleById.get(turn.parent_id) : undefined;
    const fork = source?.session && turn.session && source.session !== turn.session;
    const parent = source && !fork ? source.id : null;
    const siblings = children.get(parent) ?? [];
    siblings.push(turn);
    children.set(parent, siblings);
  }
  children.get(null)?.sort((a, b) => {
    const favorites = view === "History" ? Number(!!stateOf(b)?.favorite) - Number(!!stateOf(a)?.favorite) : 0;
    return favorites || (newest.get(identity(b)) ?? Number.MIN_SAFE_INTEGER) - (newest.get(identity(a)) ?? Number.MIN_SAFE_INTEGER);
  });

  const rows = (parent: string | null, seen: Set<string>): ReactNode => {
    const group = (children.get(parent) ?? []).filter(turn => !seen.has(turn.id));
    return group.length ? <ul>{group.map(turn => {
      const title = agentTurnLabel(turn);
      const source = turn.parent_id && byId.get(turn.parent_id);
      const fork = !!(source && turn.session && source.session && turn.session !== source.session);
      const hasChildren = children.has(turn.id);
      const expanded = hasChildren && !collapsed.has(turn.id);
      const state = stateOf(turn);
      return <li key={turn.id}>
        <div className="conversation-history-row">
          {hasChildren && <button type="button" aria-label={`Replies to ${title}`} aria-expanded={expanded} onClick={() =>
            setCollapsed(current => { const next = new Set(current); if (next.has(turn.id)) next.delete(turn.id); else next.add(turn.id); return next; })
          }>{expanded ? "▾" : "▸"}</button>}
          <button className="conversation-history-title" title={title} aria-current={session.tip === turn.id ? "true" : undefined}
            onClick={() => open(turn.id)}><AgentActivityIcon working={turns.some(row => row.status === 'running' && (row.id === turn.id || !!turn.session && row.session === turn.session)) && !session.pendingReviews?.changes.length} /> {fork ? `Branch from ${turn.parent_is_user ? "user" : "agent"} · ` : ""}{title}</button>
          {state?.favorite && <span role="img" aria-label="Favorite">★</span>}
          {flags && turn.session && <details className="conversation-history-actions" onKeyDown={event => {
            if (event.key === "Escape") {
              event.currentTarget.open = false;
              event.currentTarget.querySelector("summary")?.focus();
              event.stopPropagation();
            }
          }}>
            <summary aria-label={`More actions for ${title}`}>⋯</summary>
            <div onClick={event => {
              const menu = event.currentTarget.closest("details");
              if (menu) { menu.open = false; menu.querySelector("summary")?.focus(); }
            }}>
              <button type="button" disabled={busy || !!organizationError} onClick={() => void update(turn, { favorite: !state?.favorite })}>{state?.favorite ? "Remove favorite" : "Favorite"}</button>
              <button type="button" disabled={busy || !!organizationError} onClick={() => void update(turn, { archived: !state?.archived })}>{state?.archived ? "Restore" : "Archive"}</button>
            </div>
          </details>}
        </div>
        {expanded && rows(turn.id, new Set([...seen, turn.id]))}
      </li>;
    })}</ul> : null;
  };
  const count = new Set(visible.map(identity)).size;
  const noun = view === "Favorites" ? "favorite" : view === "Archived" ? "archived conversation" : "conversation";
  const failure = session.historyError || organizationError;
  const reading = failure ? "History unavailable" : !session.historyLoaded || !flags ? "Loading history…" : `${count} ${noun}${count === 1 ? "" : "s"}`;
  return <section className="pane-with-status" aria-label="Agent History">
    <div className="file-tree">
      <button className="new-conversation-button" aria-label="New conversation" title="New conversation" onClick={() => open(null)}>✎</button>
      {flags && <select aria-label="Conversation history view" value={view} onChange={event => setView(event.target.value as ConversationHistoryView)}>
        <option>History</option><option>Favorites</option><option>Archived</option>
      </select>}
      {session.historyError && <p role="alert">{session.historyError}</p>}
      {organizationError && <p role="alert">{organizationError}</p>}
      {rows(null, new Set())}
      {!visible.length && !session.historyError && <p role="status">{!session.historyLoaded ? "Loading conversations…" :
        view === "Favorites" ? "No favorite conversations." : view === "Archived" ? "No archived conversations." : "No conversations yet."}</p>}
    </div>
    <footer className="pane-status" aria-label="Agent History status">
      <span className="pane-status-label" role="status" title={failure || undefined}>{reading}</span>
      {failure && <button type="button" aria-label={session.historyError ? "Retry history" : "Refresh conversation organization"} onClick={() => {
        if (session.historyError) session.retryHistory();
        if (organizationError) setReload(value => value + 1);
      }}>Retry</button>}
    </footer>
  </section>;
}
</ui:copy>

<ui:copy id="simple-agent-history-refresh">
export function AgentHistory({ session, visible }: { session: AgentSession; visible: boolean }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const observing = visible || session.running;
  const reload = session.historyReload;
  useEffect(() => {
    if (!observing) return;
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const current = () => !cancelled && session.workspaceCurrent();
    const refresh = async () => {
      const mutation = session.mutation;
      let listed = false;
      try {
        const response = await api.agentTurns("workspace");
        listed = true;
        if (current() && mutation === session.mutation && !session.busy) {
          const own = session.liveTurn;
          const active = response.turns.find(turn => turn.status === "running");
          session.historyError = "";
          session.historyLoaded = true;
          session.messageForks = response.message_forks === true;
          session.totals = response.totals;
          session.setTurns(response.turns);
          session.publish();
          if (active) {
            const state = await request<{ turn: string | null; edits: { captureRequest?: string; changes?: AgentChange[] } }>("GET", "/api/docs/workspace/agent/python?view=pending");
            if (current() && mutation === session.mutation && own === session.liveTurn) {
              session.pendingReviews = state.turn === active.id ? {
                turn: active.id,
                changes: (state.edits?.changes ?? []).filter(change => !change.editor && change.status === "pending"),
              } : null;
              session.publish();
              const capture = state.edits?.captureRequest;
              if (capture && own && state.turn === own) {
                const context = session.liveAgentContext();
                if (context) await acpApi.edits("workspace", { capture_request: capture, context }, "python");
              }
            }
          } else if (own) {
            session.liveTurn = null;
            session.pendingReviews = null;
            session.publish();
          }
          if (current() && mutation === session.mutation && session.reviewReadError) {
            session.reviewReadError = "";
            session.publish();
          }
        }
      } catch (reason) {
        if (current() && mutation === session.mutation) {
          if (listed) session.reviewReadError = `Cannot refresh agent review: ${String(reason)}`;
          else session.historyError = `Cannot read agent history: ${String(reason)}`;
          session.publish();
        }
      }
      if (current()) timer = setTimeout(refresh, 1000);
    };
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); };
  }, [session, observing, reload]);
  useEffect(() => {
    if (!observing) return;
    return getWorkspaceRealtime()?.onRunEvent(message => {
      if (session.workspaceCurrent()) session.receive(message);
    });
  }, [session, observing]);
  return null;
}

export function Agent({ session, reviewProposals, reviewRecordedPatch, newConversation }: {
  newConversation?: (id?: string | null, fork?: 'user' | 'assistant') => void;
  session: AgentSession; reviewProposals?: () => void; reviewRecordedPatch?: ReviewRecordedPatch;
}) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  const conversation = useRef<HTMLDivElement>(null);
  const following = useRef(session.following);
  const { transcript, prompt, error } = session;
  const [review, setReview] = useState<{ turn: string; preview: AgentChange; anchor: HTMLButtonElement } | null>(null);
  const tail = transcript.at(-1)?.turn;
  const liveAnswer = tail && session.activity.get(tail.id)?.answer;
  useEffect(() => { if (conversation.current) conversation.current.scrollTop = session.conversationScroll; }, [session]);
  useEffect(() => {
    if (following.current && conversation.current) conversation.current.scrollTop = conversation.current.scrollHeight;
  }, [tail?.id, tail?.answer, tail?.error, liveAnswer]);
  const running = session.branchEdit?.side === "assistant" ? session.busy : session.running;
  const inlineRenderNotice = [...session.documents.values.values()].find(buffer => buffer.inlineRenderNotice)?.inlineRenderNotice;
</ui:copy>

Keep Send and Stop beside the draft. Offer Stop once the workspace identifies a
running turn. Tool disclosures stay closed as events arrive. An accepted answer
can appear before the history read confirms the recording and enables its fence
actions.

<ui:copy id="simple-981c695688-1">
  return <div className="agent-body">
    <RichOwners buffers={session.documents} recorded />

    <div ref={conversation} className="conversation" role="log" aria-label="Conversation" onScroll={event => { const node = event.currentTarget; session.conversationScroll = node.scrollTop; session.following = following.current = node.scrollHeight - node.clientHeight - node.scrollTop < 48; }}>
      {!transcript.length && <div className="agent-empty"><h2>What shall we work on?</h2><p>Write a literate document below. Code, tables and explanations belong together.</p></div>}
      {transcript.map(({ turn, userOnly }) => {
        const activity = session.activity.get(turn.id);
        const streaming = turn.status === "running";
        const pending = session.pendingReviews?.turn === turn.id ? session.pendingReviews.changes : [];
        const answer = turn.answer ?? turn.error ?? activity?.answer;
        return <article key={turn.id}>
          <section className="agent-message user-message"><p className="message-label">You</p><RecordedMessage turn={turn.id} side="user" reviewRecordedPatch={reviewRecordedPatch} buffer={session.document(`${turn.id}-user`, turn.prompt)} />
            {session.messageForks && (!turn.backend || turn.backend === "python") && <button onClick={() => newConversation ? newConversation(turn.id, "user") : session.select(turn.id, "user")}>Branch from user message</button>}</section>
          {!userOnly && <section className="agent-message">
            <p className="message-label">Agent{turn.model && ` · ${turn.model}`}</p>
            {!!activity?.tools.length && <details className="agent-tool-calls"><summary>Tool calls ({activity.tools.length})</summary>
              {activity.tools.map((tool, index) => <details key={index}><summary>{tool.label} · {tool.ok === undefined ? streaming ? "Running" : "Outcome unavailable" : tool.ok ? "Completed" : "Failed"}</summary>{tool.input && <><strong>Input</strong><pre>{formattedToolJSON(tool.input)}</pre></>}{tool.result && <><strong>Result</strong><pre>{formattedToolJSON(tool.result)}</pre></>}</details>)}
            </details>}
            {streaming && <div className="agent-progress" role="status"><span aria-hidden="true" />{session.stopPending ? "Stopping…" : pending.length ? "Waiting for your review" : activity?.phase ?? "Starting…"}
              {pending.map(preview => <button key={preview.id} disabled={session.stopPending} title={preview.path ?? preview.name}
                style={{ maxWidth: "60%", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                onClick={event => setReview({ turn: turn.id, preview: { ...preview }, anchor: event.currentTarget })}>Review {preview.path ?? preview.name}</button>)}
            </div>}
            {answer && <RecordedMessage turn={turn.id} side="assistant" reviewRecordedPatch={reviewRecordedPatch} recorded={!streaming && turn.answer != null} buffer={session.document(`${turn.id}-agent`, answer)} />}
            {session.messageForks && turn.answer != null && (!turn.backend || turn.backend === "python") && <button onClick={() => newConversation ? newConversation(turn.id, "assistant") : session.select(turn.id, "assistant")}>Branch from assistant message</button>}
          </section>}
        </article>;
      })}
    </div>
    {review && <AgentExactReview session={session} review={review} close={() => setReview(value => value === review ? null : value)} />}
    {session.selection?.fork && <p className="message-label" role="status">New branch from {session.selection.fork === "user" ? "user" : "assistant"} message. {session.branchEdit?.side === "assistant" ? "Save the edited answer to create it." : "Send to start it."}</p>}
    {(error || session.reviewReadError) && <div role="alert" className="notice">{error || session.reviewReadError}</div>}
    <form className="agent-composer" onKeyDownCapture={event => {
      if (event.key === 'Enter' && event.shiftKey && !event.ctrlKey && !event.metaKey && !event.altKey && !event.nativeEvent.isComposing) {
        event.preventDefault(); event.stopPropagation(); void session.send();
      }
    }} onSubmit={event => { event.preventDefault(); void session.send(); }}>
      <label className="message-label">{session.branchEdit ? `Edit ${session.branchEdit.side} message · The original stays unchanged.` : 'Your message · Paste any source excerpts you want the model to read.'}</label>
      <Document buffer={session.draft} />
      <div className="agent-send">
        {session.canStop
          ? <button type="button" aria-label="Stop" title={session.stopPending ? "A stop request is in progress." : "Stop"} disabled={session.stopPending} onClick={() => void session.stop()}>■</button>
          : <button aria-label={session.branchEdit?.side === "assistant" ? "Save assistant branch" : "Send"} title={session.branchEdit?.side === "assistant" ? "Save this edited answer on the new branch without calling a model" : running ? "Wait for the current operation to finish." : !prompt.trim() ? "Enter a message first." : "Send"} disabled={running || session.branchEdit?.side !== "assistant" && !prompt.trim()}>{session.branchEdit?.side === "assistant" ? "✓" : "↑"}</button>}
      </div>
    </form>
    <footer className="pane-status agent-pane-status" aria-label="Agent status"><AgentStatus session={session} />
      {inlineRenderNotice && <span className="pane-status-label" role="status" title={inlineRenderNotice}>{inlineRenderNotice}</span>}
      <button type="button" aria-label="New conversation" title="New conversation" onClick={() => newConversation ? newConversation() : session.select(null)}>✎</button>
      {reviewProposals && <button type="button" aria-label="Review pending proposals" title="Review pending proposals" onClick={reviewProposals}>Review</button>}
    </footer>
  </div>;
}

function AgentExactReview({ session, review, close }: {
  session: AgentSession;
  review: { turn: string; preview: AgentChange; anchor: HTMLButtonElement };
  close: () => void;
}) {
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    const dialog = panel.current;
    dialog?.focus({ preventScroll: true });
    return () => {
      if (review.anchor.isConnected && (document.activeElement === document.body || dialog?.contains(document.activeElement))) {
        review.anchor.focus({ preventScroll: true });
      }
    };
  }, [review]);
  const current = session.reviewCurrent(review.turn, review.preview);
  const decide = async (accepted: boolean) => {
    if (await session.decideReview(review.turn, review.preview, accepted)) close();
  };
  return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Agent edit review" onKeyDown={event => {
    if (event.key === "Escape") { event.preventDefault(); close(); }
  }}>
    <header><strong>{review.preview.path ?? review.preview.name}</strong><button onClick={close}>Close</button></header>
    {!current && <p role="status">This proposal changed or ended. Open the current proposal to make a decision.</p>}
    {session.reviewError && <p role="alert">{session.reviewError}</p>}
    <h3>Before</h3><pre>{review.preview.oldText}</pre>
    <h3>After</h3><pre>{review.preview.newText}</pre>
    <button disabled={!current || session.reviewBusy} onClick={() => void decide(false)}>Decline edit</button>
    <button disabled={!current || session.reviewBusy} onClick={() => void decide(true)}>Approve edit</button>
  </section>, document.body);
}
</ui:copy>

## Put conversation settings in the Agent pane status bar

The focused Agent tab mounts these controls in the footer. The popover floats
above the footer and the draft stays mounted. Codex choices come from the
workspace server's catalog, with a provider default and a manual model fallback.

<ui:copy id="simple-agent-status-controls">
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
  const readings = session.transcript.map(({turn, userOnly}) => userOnly ? undefined : turn.status === 'running' ? session.activity.get(turn.id)?.usage : turn.usage);
  const known = readings.filter((usage): usage is AgentUsage => usage != null);
  const tokens = known.length ? known.reduce((sum, usage) => sum + usage.input_tokens + usage.output_tokens + usage.cache_read_input_tokens + usage.cache_creation_input_tokens, 0) : session.transcript.length === 0 && !session.busy ? 0 : undefined;
  const cost = "Conversation cost unavailable";

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
    <span className="pane-status-label" title={`Reported usage for this conversation; unavailable turn usage is excluded · ${cost}`}>{tokens === undefined ? "Tokens unavailable" : `${tokens.toLocaleString()} tokens`}{tokens !== undefined && ` · ${cost}`}</span>
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

</ui:copy>

## Debug a recorded code fence in its message editor

Use the normal gutter for breakpoint intentions and anchor launch to the fence
opening. The engine returns a pinned address for the exact turn, side and
revision after the click. Verified frames decorate the existing message editor
in place. If the reading disappears during preparation, release the capture
before launch. The editor continues to own an immutable recording.

<ui:copy id="recorded-message-fence-controls">
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
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Agent.tsx"><ui:paste select="#simple-981c695688-0"/><ui:paste select="#simple-agent-document-session"/><ui:paste select="#simple-agent-ancestry"/><ui:paste select="#simple-agent-history-refresh"/><ui:paste select="#simple-981c695688-1"/><ui:paste select="#simple-agent-status-controls"/><ui:paste select="#recorded-message-fence-controls"/></ui:file>

</ui:doc>
