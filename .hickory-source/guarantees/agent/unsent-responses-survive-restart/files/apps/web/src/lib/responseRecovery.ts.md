<book:doc xmlns:book="http://www.hickorydocs.com/1.0">
# apps/web/src/lib/responseRecovery.ts

For: maintainers changing this file and reviewers tracing the related behavioral requirements.

Edit this entry to change the generated file. It has one constructor. References below are navigation candidates; they do not establish semantic test coverage.

- [Unsent responses survive restart](../../../../../contract.md)

Captured SHA-256: `b461327b0c310e9b412c2caa00dc64c97898964e668dbef019d06fc76264aa72`.

<book:entry path="apps/web/src/lib/responseRecovery.ts">
// Personal response recovery, separate from immutable conversation evidence.
import { useEffect, useMemo, useState } from "react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/types";

interface SavedResponses { version: 1; tip: string | null; drafts: Record<string, string> }
const queues = new Map<string, Promise<unknown>>();
const active = new Set<ResponseRecovery>();

/** The native close gate waits for response writes, including closed panes. */
export async function flushResponseRecovery() {
  await Promise.all([...active].map(recovery => recovery.flush()));
  while (queues.size) await Promise.allSettled([...queues.values()]);
}
function ordered<T>(key: string, operation: () => Promise<T>): Promise<T> {
  const next = (queues.get(key) ?? Promise.resolve()).catch(() => {}).then(operation);
  queues.set(key, next);
  void next.finally(() => { if (queues.get(key) === next) queues.delete(key); }).catch(() => {});
  return next;
}
export function responseThread(turns: readonly AgentTurn[], tip: string | null): string {
  const seen = new Set<string>();
  let root = tip;
  while (root && !seen.has(root)) {
    seen.add(root);
    const parent = turns.find(turn => turn.id === root)?.parent_id;
    if (!parent) break;
    root = parent;
  }
  return root ?? "new";
}
function readSaved(raw: unknown): SavedResponses {
  if (raw === null) return { version: 1, tip: null, drafts: {} };
  const value = raw as Partial<SavedResponses>;
  if (!value || value.version !== 1 || !(value.tip === null || typeof value.tip === "string") ||
    !value.drafts || typeof value.drafts !== "object" || Array.isArray(value.drafts) ||
    Object.values(value.drafts).some(text => typeof text !== "string")) throw new Error("Damaged response recovery");
  return { version: 1, tip: value.tip, drafts: { ...value.drafts } };
}

/** A controller outlives a pane while queued writes or a send finish. */
export class ResponseRecovery {
  state: SavedResponses = { version: 1, tip: null, drafts: {} };
  text: string;
  error: string | null = null;
  scope: string | null = null;
  turns: readonly AgentTurn[] = [];
  private changed = false;
  private followLatest = true;
  private loaded = false;
  private revision = 0;
  private textRevision = 0;
  private savedRevision = -1;
  private listeners = new Set<() => void>();
  constructor(private doc: string, private key: string, private initial: string) { this.text = initial; }
  subscribe = (listener: () => void) => { this.listeners.add(listener); return () => { this.listeners.delete(listener); }; };
  private notify() { this.listeners.forEach(listener => listener()); }
  private failed() { this.error = "Response recovery could not be saved. Keep this pane open or copy your response before closing."; this.notify(); }
  async load(turns: readonly AgentTurn[]) {
    this.turns = turns;
    if (!this.changed) this.state.tip = turns.at(-1)?.id ?? null;
    const early = this.text;
    try {
      await ordered(this.key, async () => {
        const result = await api.responseDraft(this.doc);
        this.scope = result.scope;
        const stored = readSaved(result.state);
        if (!this.changed) {
          this.state = stored;
          // Preserve an unknown/deleted thread identity rather than binding its
          // draft to whichever conversation happens to be newest.
          this.followLatest = result.state === null;
          if (this.followLatest) this.state.tip = this.turns.at(-1)?.id ?? null;
          this.text = this.state.drafts[responseThread(this.turns, this.state.tip)] ?? early;
        } else {
          // Typing/selection beats late restoration, but retain other threads.
          const thread = responseThread(this.turns, this.state.tip);
          this.state.drafts = { ...stored.drafts, ...this.state.drafts, [thread]: this.text };
        }
        // Complete pre-hydration edits before a remounted pane can read.
        if (this.changed) {
          const revision = this.revision;
          await api.saveResponseDraft(this.doc, this.scope, structuredClone(this.state));
          this.savedRevision = revision;
        }
        this.error = null;
      });
    } catch { this.failed(); }
    this.loaded = true;
    this.notify();
    if (this.changed && this.scope) void this.flush();
  }
  history(turns: readonly AgentTurn[]) {
    const previous = responseThread(this.turns, this.state.tip);
    this.turns = turns;
    if (this.followLatest && !this.changed) this.state.tip = turns.at(-1)?.id ?? null;
    const thread = responseThread(turns, this.state.tip);
    if (!this.changed) this.text = this.state.drafts[thread] ?? this.initial;
    else if (previous !== thread) {
      this.state.drafts[thread] = this.text;
      delete this.state.drafts[previous];
      this.revision++; void this.flush();
    }
    this.notify();
  }
  edit = (text: string) => {
    this.text = text; this.changed = true; this.revision++; this.textRevision++;
    this.state.drafts[responseThread(this.turns, this.state.tip)] = text;
    this.notify();
    // Save during editing, even if shutdown never delivers a lifecycle event.
    void this.flush();
  };
  select = (tip: string | null) => {
    this.state.tip = tip;
    this.changed = true; this.revision++;
    this.text = this.state.drafts[responseThread(this.turns, tip)] ?? "";
    this.notify(); void this.flush();
  };
  snapshot() { return { thread: responseThread(this.turns, this.state.tip), revision: this.textRevision, text: this.text }; }
  accepted(snapshot: ReturnType<ResponseRecovery["snapshot"]>, id: string) {
    // A send can finish after its pane closed and reopened. Let the live
    // owner preserve any edits/discard made since that restoration.
    const owner = [...active].reverse().find(recovery => recovery.key === this.key);
    if (owner && owner !== this) {
      owner.accepted({ ...snapshot, revision: owner.changed ? -1 : owner.textRevision }, id);
      return;
    }
    // Clear only the submitted revision. New typing (even identical text) wins.
    const currentThread = responseThread(this.turns, this.state.tip);
    if (currentThread !== snapshot.thread) {
      if (this.state.drafts[snapshot.thread] === snapshot.text) this.state.drafts[snapshot.thread] = "";
      this.changed = true; this.revision++; this.notify(); void this.flush();
      return;
    }
    if (this.textRevision === snapshot.revision) this.text = "";
    this.state.drafts[snapshot.thread] = this.text;
    if (snapshot.thread === "new" && currentThread === "new") {
      this.state.drafts[id] = this.text;
      this.state.drafts.new = "";
    }
    this.state.tip = id;
    this.changed = true; this.revision++;
    this.notify(); void this.flush();
  }
  discard = () => { this.edit(""); };
  async flush(keepalive = false) {
    if (!this.loaded) return;
    if (!this.scope) { this.loaded = false; await this.load(this.turns); return; }
    if (!this.changed) return;
    if (this.savedRevision === this.revision) { await queues.get(this.key)?.catch(() => {}); return; }
    const revision = this.revision;
    const state = structuredClone(this.state);
    const scope = this.scope;
    // Capture bytes now, and serialize across remounts as well as this pane.
    this.savedRevision = revision;
    try {
      await ordered(this.key, () => revision < this.revision ? Promise.resolve({ ok: true }) : api.saveResponseDraft(this.doc, scope, state, keepalive));
      this.error = null; this.notify();
    } catch { this.savedRevision = -1; this.failed(); }
  }
}

export function useResponseRecovery(doc: string, workspace: string, initial: string, turns: AgentTurn[], ready: boolean) {
  const recovery = useMemo(() => new ResponseRecovery(doc, JSON.stringify([workspace, doc]), initial), [doc, workspace]);
  const [, render] = useState(0);
  useEffect(() => recovery.subscribe(() => render(value => value + 1)), [recovery]);
  useEffect(() => { void recovery.load(turns); }, [recovery]);
  useEffect(() => { if (ready) recovery.history(turns); }, [recovery, ready, turns]);
  useEffect(() => {
    active.add(recovery);
    const flush = () => { void recovery.flush(true); };
    const hide = () => { if (document.visibilityState === "hidden") flush(); };
    const timer = window.setInterval(() => void recovery.flush(), 2000);
    window.addEventListener("pagehide", flush);
    document.addEventListener("visibilitychange", hide);
    return () => { active.delete(recovery); clearInterval(timer); window.removeEventListener("pagehide", flush); document.removeEventListener("visibilitychange", hide); flush(); };
  }, [recovery]);
  return recovery;
}
</book:entry>

</book:doc>
