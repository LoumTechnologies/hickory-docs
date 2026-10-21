import "./AgentChanges.css";
import { useEffect, useId, useRef, useState } from "react";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { acpApi, type AcpState, type AgentChange } from "../api/acp";
import { publishReading, forgetReading, openReading } from "../lib/readingViews";
import { DocumentReading } from "../views/DocumentReading";
import { AgentEditConflict } from "../lib/agentEdit";

export function AgentChanges({ doc, state, running, apply, update }: {
  doc: string; state: AcpState | null; running: boolean;
  apply?: (change: AgentChange) => void | Promise<void>;
  update: (state: AcpState) => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const inflight = useRef(new Set<string>());
  const decide = async (change: AgentChange, accepted: boolean, replacement?: string): Promise<UiOutcome> => {
    const revised = replacement === undefined ? change : { ...change, newText: replacement };
    if (inflight.current.has(change.id)) return { status: "unavailable", reason: "Decision already in flight" };
    inflight.current.add(change.id); setError(null);
    let failure: string | undefined;
    let currentText: string | undefined;
    try {
      if (accepted && change.editor) {
        if (!apply) throw new Error("Open this document's workspace editor before accepting its change.");
        await apply(revised);
      }
    } catch (e) { failure = e instanceof Error ? e.message : String(e); currentText = e instanceof AgentEditConflict ? e.currentText : undefined; setError(failure); }
    try {
      const actual = await acpApi.edits(doc, { id: change.id, accepted: accepted && !failure, error: failure, ...(replacement !== undefined ? { new_text: replacement } : {}), ...(currentText !== undefined ? { current_text: currentText } : {}) }, state?.backend);
      update(actual);
      return failure ? { status: "unavailable", reason: failure } : { status: "completed", value: { id: change.id, status: actual.edits?.changes.find(c => c.id === change.id)?.status ?? "not present in returned host state", verification: "not established" } };
    }
    catch (e) { const reason = e instanceof Error ? e.message : String(e); setError(reason); return { status: "unavailable", reason: `${reason}; inspect the editor and host state before retrying: a local edit may already have occurred` }; }
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
        opened.current.add(id);
      }
    });
  }, [doc, state?.edits?.changes]);
  useEffect(() => () => { opened.current.forEach(forgetReading); opened.current.clear(); }, [doc]);
  useEffect(() => {
    if (state?.backend === "python") return;
    state?.edits?.changes.filter(c => c.status === "applying").forEach(c => { void decide(c, true); });
  }, [state?.edits?.changes]);
  const parent = `agent-changes:${useId()}`;
  const revision = useUiRevision(() => [doc, state?.backend, state?.edits, running, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, parameters: UiCommand["parameters"], enabled: boolean, execute: UiCommand["execute"]) => commands.push({ key: `${parent}:${key}`, command: { label, parent, subject: parent, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled }), execute } });
  if (state?.edits) {
    add("read", "Read conversation edit review bytes", { offset: { type: "number", minimum: 0, maximum: 1000000000, integer: true }, limit: { type: "number", minimum: 1, maximum: 8192, integer: true } }, true, async args => {
      const bytes = new TextEncoder().encode(JSON.stringify({ doc, backend: state.backend, edits: state.edits, error, scope: "Proposals and actual recorded statuses; acceptance does not verify a change" }));
      const offset = args.offset as number, end = Math.min(offset + (args.limit as number), bytes.length);
      if (offset > bytes.length) return { status: "unavailable", reason: "Offset outside edit review" };
      return { status: "completed", value: { bytes: Array.from(bytes.slice(offset, end)), encoding: "JSON UTF-8", offset, total_bytes: bytes.length, next_offset: end < bytes.length ? end : null } };
    });
    add("open", "Open conversation change review", { id: { type: "string", maxLength: 4096 } }, true, async args => {
      const change = state.edits?.changes.find(c => c.id === args.id);
      if (!change) return { status: "unavailable", reason: "Observe a current proposal identity" };
      openReading(`proposal:${doc}:${change.id}`, `${change.path ?? change.name} · review`);
      return { status: "completed", value: { review_open_requested: change.id } };
    });
    add("decision", "Decide conversation change", { id: { type: "string", maxLength: 4096 }, accepted: { type: "boolean" } }, true, async (args, actor) => {
      const change = state.edits?.changes.find(c => c.id === args.id);
      if (!change || change.status !== "pending") return { status: "unavailable", reason: "Proposal is no longer pending; inspect current host status" };
      if (actor.kind === "agent") return { status: "requires_input", request: `edit-review:${doc}:${state.backend}:${change.id}`, reason: "The human reviews and decides this proposed edit in the actual change review; inspect its returned status afterward" };
      return decide(change, args.accepted as boolean);
    });
    add("mode", "Set conversation edit review mode", { mode: { type: "string", maxLength: 11, choices: ["review", "auto-accept"] } }, !running && state.backend !== "python", async (args, actor) => {
      if (actor.kind === "agent" && args.mode === "auto-accept") return { status: "requires_input", request: `edit-mode:${doc}:${state.backend}`, reason: "A human must authorize automatic acceptance of future edits" };
      try {
        const actual = await acpApi.edits(doc, { mode: args.mode as "review" | "auto-accept" }, state.backend); update(actual);
        return { status: "completed", value: { mode: actual.edits?.mode ?? null } };
      } catch (reason) { setError(String(reason)); return { status: "unavailable", reason: String(reason) }; }
    });
  }
  const invoke = useUiCommands(commands);
  if (!state?.edits) return null;
  return <section className="agent-changes" aria-label="Conversation document edits">
    <label>Document edits <select aria-label="Document edits" value={state.edits.mode} disabled={running || state.backend === "python"}
      onChange={e => void invoke(`${parent}:mode`, { mode: e.target.value })}><option value="review">Review</option>{state.backend !== "python" && <option value="auto-accept">Auto-accept</option>}</select></label>
    <p className="muted">Reviews cover Hickory document tools. {state.backend === "python" ? "Python document-edit previews require approval. Shared UI operations retain their separate host permissions. Acceptance does not verify the edit or change its authorship. Checks remain unrun." : "Native agent commands follow the adapter’s permissions and can bypass this review."}</p>
    {error && <p role="alert">{error}</p>}
    {state.edits.changes.map(change => <div key={change.id} className="agent-change-link">
      <span>{change.status === "pending" ? "Proposed change" : change.status === "applying" ? "Applying change" : `${change.status.charAt(0).toUpperCase()}${change.status.slice(1)} change`} · {change.path ?? change.name}</span>
      <DocumentReading id={`proposal:${doc}:${change.id}`} />
      <button className="btn" onClick={() => void invoke(`${parent}:open`, { id: change.id })}>Review in document</button>
    </div>)}
  </section>;
}
