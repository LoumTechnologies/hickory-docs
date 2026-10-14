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
  const decide = async (change: AgentChange, accepted: boolean, replacement?: string) => {
    const revised = replacement === undefined ? change : { ...change, newText: replacement };
    if (inflight.current.has(change.id)) return;
    inflight.current.add(change.id); setError(null);
    let failure: string | undefined;
    let currentText: string | undefined;
    try {
      if (accepted && change.editor) {
        if (!apply) throw new Error("Open this document's workspace editor before accepting its change.");
        await apply(revised);
      }
    } catch (e) { failure = e instanceof Error ? e.message : String(e); currentText = e instanceof AgentEditConflict ? e.currentText : undefined; setError(failure); }
    try { update(await acpApi.edits(doc, { id: change.id, accepted: accepted && !failure, error: failure, ...(replacement !== undefined ? { new_text: replacement } : {}), ...(currentText !== undefined ? { current_text: currentText } : {}) }, state?.backend)); }
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
        editable: change.status === "pending", decide: (accepted, text) => latest.current(change, accepted, text) });
      if (change.status === "pending" && !opened.current.has(id)) {
        opened.current.add(id); openReading(id, `${change.path ?? change.name} · review`, true);
      }
    });
  }, [doc, state?.edits?.changes]);
  useEffect(() => () => { opened.current.forEach(forgetReading); opened.current.clear(); }, [doc]);
  useEffect(() => {
    if (state?.backend === "python") return;
    state?.edits?.changes.filter(c => c.status === "applying").forEach(c => { void decide(c, true); });
  }, [state?.edits?.changes]);
  if (!state?.edits) return null;
  return <section className="agent-changes" aria-label="Conversation document edits">
    <label>Document edits <select aria-label="Document edits" value={state.edits.mode} disabled={running || state.backend === "python"}
      onChange={async e => {
        try { update(await acpApi.edits(doc, { mode: e.target.value as "review" | "auto-accept" }, state.backend)); }
        catch (e) { setError(e instanceof Error ? e.message : String(e)); }
      }}><option value="review">Review</option>{state.backend !== "python" && <option value="auto-accept">Auto-accept</option>}</select></label>
    <p className="muted">Reviews cover Hickory document tools. {state.backend === "python" ? "Every Python agent edit requires approval. Acceptance does not verify the edit or change its authorship. Checks remain unrun." : state.backend === "builtin" ? "Agent scripts follow the chosen executor." : "Native agent commands follow the adapter’s permissions and can bypass this review."}</p>
    {error && <p role="alert">{error}</p>}
    {state.edits.changes.map(change => <div key={change.id} className="agent-change-link">
      <span>{change.status === "pending" ? "Proposed change" : change.status === "applying" ? "Applying change" : `${change.status.charAt(0).toUpperCase()}${change.status.slice(1)} change`} · {change.path ?? change.name}</span>
      <button className="btn" onClick={() => openReading(`proposal:${doc}:${change.id}`, `${change.path ?? change.name} · review`)}>Review in document</button>
    </div>)}
  </section>;
}
