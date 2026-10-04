import "./AgentChanges.css";
import { useEffect, useRef, useState } from "react";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
import { publishReading, forgetReading, openReading } from "../lib/readingViews";
import { AgentEditConflict } from "../lib/agentEdit";

export function AgentChanges({ doc, state, running, apply, update }: {
  doc: string; state: AcpState | null; running: boolean;
  apply?: (change: AgentChange) => void | Promise<void>;
  update: (state: AcpState) => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const inflight = useRef(new Set<string>());
  const decide = async (change: AgentChange, accepted: boolean) => {
    if (inflight.current.has(change.id)) return;
    inflight.current.add(change.id); setError(null);
    let failure: string | undefined;
    let currentText: string | undefined;
    try {
      if (accepted && change.editor) {
        if (!apply) throw new Error("Open this document's workspace editor before accepting its change.");
        await apply(change);
      }
    } catch (e) { failure = e instanceof Error ? e.message : String(e); currentText = e instanceof AgentEditConflict ? e.currentText : undefined; setError(failure); }
    try { update(await acpApi.edits(doc, { id: change.id, accepted: accepted && !failure, error: failure, ...(currentText !== undefined ? { current_text: currentText } : {}) })); }
    catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { inflight.current.delete(change.id); }
  };
  const latest = useRef(decide); latest.current = decide;
  const opened = useRef(new Set<string>());
  useEffect(() => {
    state?.edits?.changes.forEach(change => {
      const id = `proposal:${doc}:${change.id}`;
      publishReading(id, { title: "Proposed change", path: change.path ?? change.name,
        before: change.oldText, after: change.newText, status: change.status,
        decide: accepted => latest.current(change, accepted) });
      if (change.status === "pending" && !opened.current.has(id)) {
        opened.current.add(id); openReading(id, `${change.path ?? change.name} · review`);
      }
    });
  }, [doc, state?.edits?.changes]);
  useEffect(() => () => { opened.current.forEach(forgetReading); opened.current.clear(); }, [doc]);
  useEffect(() => {
    state?.edits?.changes.filter(c => c.status === "applying").forEach(c => { void decide(c, true); });
  }, [state?.edits?.changes]);
  if (!state?.edits) return null;
  return <section className="agent-changes" aria-label="Conversation document edits">
    <label>Document edits <select aria-label="Document edits" value={state.edits.mode} disabled={running}
      onChange={async e => {
        try { update(await acpApi.edits(doc, { mode: e.target.value as "review" | "auto-accept" })); }
        catch (e) { setError(e instanceof Error ? e.message : String(e)); }
      }}><option value="review">Review</option><option value="auto-accept">Auto-accept</option></select></label>
    {error && <p role="alert">{error}</p>}
    {state.edits.changes.map(change => <details key={change.id} open={change.status === "pending" || change.status === "applying"}>
      <summary>{change.status === "pending" ? "Proposed change" : change.status === "applying" ? "Applying change" : `${change.status.charAt(0).toUpperCase()}${change.status.slice(1)} change`} · {change.path ?? change.name}</summary>
      <button className="btn" onClick={() => openReading(`proposal:${doc}:${change.id}`, `${change.path ?? change.name} · review`)}>Review in document</button>
    </details>)}
  </section>;
}
