import { SurfaceInfo } from "../components/SurfaceInfo";
import { useId, useState } from "react";
import { ComparisonEditor, CodeEditor } from "../components/ComparisonEditor";
import type { ComparisonModel } from "../lib/projectComparison";
import { useReading } from "../lib/readingViews";
import { CommitReading } from "./CommitReading";
import "./DocumentReading.css";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";

/** A reading has no save path; an explicit fork edits only an authored draft. */
export function DocumentComparison({ id, path, oldPath = path, before, after, projects }: { id: string; path: string; oldPath?: string; before: string; after: string; projects?: {old: import("../editor/hickLang").ProjectEntry[];new: import("../editor/hickLang").ProjectEntry[]} }) {
  const [draft, setDraft] = useState<ComparisonModel | null>(null);
  const model: ComparisonModel = draft ?? { sources: [
    { id: `${id}:old`, label: "Old", kind: "git", text: before,path:oldPath,project:projects?.old },
    { id: `${id}:new`, label: "New", kind: "git", text: after,project:projects?.new },
  ], old: `${id}:old`, new: `${id}:new`, policy: "neither", evidence: "unverified" };
  return <ComparisonEditor path={path} model={model} onChange={setDraft} allowFork />;
}

export function DocumentReading({ id }: { id: string }) {
  const reading = useReading(id);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [draftState, setDraftState] = useState<{ id: string; before: string; after: string; model: ComparisonModel } | null>(null);
  const draft = draftState && draftState.id === id && draftState.before === reading?.before && draftState.after === reading?.after ? draftState.model : null;
  const setDraft = (model: ComparisonModel) => { if (reading) setDraftState({ id, before: reading.before, after: reading.after, model }); };
  const parent = `document-review:${useId()}`;
  const revision = useUiRevision(() => [id, reading, draft, busy, error]);
  useUiReading("document change review", reading && !reading.codeUri ? { path: reading.path, title: reading.title, status: reading.status,
    editable: !!reading.editable, busy, error, draft_modified: draft !== null } : { available: false, reason: !reading ? "This proposal is no longer connected. Reopen its conversation to review changes." : "Captured code has its own source reading" });
  const decide = async (accepted: boolean): Promise<UiOutcome> => {
    if (!reading?.decide || reading.status !== "pending" || busy) return { status: "unavailable", reason: "This captured proposal is not available for decision" };
    setBusy(true); setError("");
    try {
      const outcome = await reading.decide(accepted, draft?.sources.find(s => s.id === draft.new)?.text);
      if (outcome) { if (outcome.status === "unavailable" || outcome.status === "requires_input") setError(outcome.reason); return outcome; }
      return { status: "completed", value: { accepted, decision_handler_completed: true, execution_certified: false } };
    } catch (e) {
      const reason = e instanceof Error ? e.message : String(e); setError(reason);
      return { status: "unavailable", reason };
    } finally { setBusy(false); }
  };
  useUiCommands(reading && !reading.codeUri && !id.startsWith("commit:") && reading.status === "pending" ? [true, false].map(accepted => ({ key: `${parent}:${accepted}`, command: {
    label: accepted ? "Accept change" : "Reject change", parent, subject: parent, parameters: {}, prerequisites: [],
    read: () => ({ revision: revision(), enabled: !busy && !!reading.decide, reason: busy ? "A decision is pending" : !reading.decide ? "Proposal decision handler unavailable" : undefined }),
    execute: (_args, actor) => actor.kind === "agent"
      ? Promise.resolve({ status: "requires_input" as const, request: `edit-review:${id}`, reason: "A human must review and decide this proposal; inspect the retained review status afterward" })
      : decide(accepted),
  } })) : []);
  if (id.startsWith("commit:")) return <CommitReading sha={id.slice(7)} />;
  if (!reading) return <p className="muted">This proposal is no longer connected. Reopen its conversation to review changes.</p>;
  if (reading.codeUri) return <section className="captured-code-reading" aria-label={`Code · ${reading.path}`}>
    <SurfaceInfo name={reading.path} editable={false} destination="Captured project source. Navigation and debugging use this exact project." />
    <CodeEditor path={reading.path} item={{ id, label: reading.path, kind: "git", text: reading.after,
      project: reading.project, codeUri: reading.codeUri, debugOwner: reading.debugOwner, location: reading.location }} editable={false} onChange={() => {}} compact />
  </section>;
  return <section className="document-reading" aria-label="Document change review">
    <SurfaceInfo name={`Change review · ${reading.path}`} editable={!!reading.editable} destination={reading.status === "pending" ? "Accept writes the proposed change to its target. Reject leaves the target unchanged." : "This review does not save or execute a document."} />
    <header className="doc-tab-toolbar"><span>{reading.title} · {reading.path} · {reading.status}</span>
      {reading.status === "pending" && <><button className="btn" disabled={busy || !reading.decide} onClick={() => void decide(true)}>Accept change</button>
        <button className="btn" disabled={busy || !reading.decide} onClick={() => void decide(false)}>Reject change</button></>}
    </header>
    {error && <p role="alert">{error}</p>}
    <ComparisonEditor path={reading.path} model={draft ?? { sources: [
      {id:"old",label:"Old · captured destination",kind:"directory",text:reading.before,project:reading.project},
      {id:"new",label:"New · proposed change",kind:"authored",target:reading.path,text:reading.after,project:reading.project},
    ],old:"old",new:"new",policy:reading.editable && !busy ? "new" : "neither",evidence:"unverified"}} onChange={reading.editable && !busy ? setDraft : undefined} />
  </section>;
}
