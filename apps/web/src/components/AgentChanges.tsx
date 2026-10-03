import "./AgentChanges.css";
import { useEffect, useRef, useState } from "react";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
import { DiffView } from "./DiffView";
import { AgentEditConflict } from "../lib/agentEdit";
import { unifiedDiff } from "../lib/diff";

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
      <DiffView path={change.path ?? change.name} diff={unifiedDiff(change.path ?? change.name, change.oldText, change.newText)}
        binary={false} staged={false} label={change.status === "pending" ? "proposed" : change.status} />
      {change.status === "pending" && <div>
        <button className="btn" onClick={() => void decide(change, true)}>Accept change</button>
        <button className="btn" onClick={() => void decide(change, false)}>Reject change</button>
      </div>}
    </details>)}
  </section>;
}
