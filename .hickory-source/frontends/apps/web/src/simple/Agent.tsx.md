<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Keep one explicit workspace conversation

For maintainers iterating on the equivalent Apple, Android and browser editor frontends.

The workspace owns the session so moving or hiding its tab retains the unsent prompt and in-flight send. The first agent surface uses the configured Python backend and the existing durable turn history. It does not connect providers, retry sends, or fabricate replies. Refreshing is local to this pane and stops when unmounted.

<ui:copy id="simple-981c695688-0">
import { useEffect, useSyncExternalStore } from "react";
import { api } from "../api/client";
import type { AgentTurn } from "../api/agentTypes";
export class AgentSession {
  turns: AgentTurn[] = [];
  prompt = "";
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
      const started = await api.agent("workspace", text, this.turns.at(-1)?.id, undefined, undefined, "python");
      this.turns = [...this.turns.filter(turn => turn.id !== started.session_id), { id: started.session_id, prompt: text, answer: null, status: "running", parent_id: null, error: null, created_at: "", provider: "", model: "", usage: null }];
      if (this.prompt === text) this.prompt = "";
    } catch (reason) { this.error = `${String(reason)} The request was not retried. Refresh history before sending again.`; }
    finally { this.busy = false; this.mutation++; this.publish(); }
  }
}
export function Agent({ session }: { session: AgentSession }) {
  useSyncExternalStore(session.subscribe, session.snapshot);
  useEffect(() => {
    let cancelled = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      const mutation = session.mutation;
      try {
        const response = await api.agentTurns("workspace");
        if (!cancelled && mutation === session.mutation && !session.busy) { session.turns = response.turns; session.publish(); }
      } catch (reason) { if (!cancelled) { session.error = `${String(reason)} Check agent configuration in the existing frontend, then refresh.`; session.publish(); } }
      if (!cancelled) timer = setTimeout(refresh, 1000);
    };
    void refresh();
    return () => { cancelled = true; clearTimeout(timer); };
  }, [session]);
  const { turns, prompt, error } = session;
  const running = session.running;
</ui:copy>

Conversation controls belong inside the agent pane. Stop requests are explicit; failures remain visible. A missing provider never prevents document editing.

<ui:copy id="simple-981c695688-1">
  return <div className="agent-body">
    <div className="conversation" aria-label="Conversation">{turns.map(turn => <article key={turn.id}>
      <p><strong>You</strong></p><p className="message">{turn.prompt}</p>
      <p><strong>Agent · {turn.status}</strong></p><p className="message">{turn.answer ?? turn.error ?? (turn.status === "running" ? "Working…" : "")}</p>
    </article>)}</div>
    {error && <div role="alert" className="notice">{error}</div>}
    <form onSubmit={event => { event.preventDefault(); void session.send(); }}>
      <textarea aria-label="Message" value={prompt} onChange={event => { session.prompt = event.target.value; session.publish(); }} placeholder="Message the workspace agent" />
      <button disabled={running || !prompt.trim()}>Send</button>
      <button type="button" disabled={!running} onClick={() => { void api.agentStop("workspace").catch(reason => { session.error = `${String(reason)} Refresh history and try Stop again.`; session.publish(); }); }}>Stop</button>
    </form>
  </div>;
}
</ui:copy>

## Assemble the maintained output

Keep construction order explicit; the explanations above describe the decisions at each boundary.

<ui:file path="apps/web/src/simple/Agent.tsx"><ui:paste select="#simple-981c695688-0"/><ui:paste select="#simple-981c695688-1"/></ui:file>

</ui:doc>
