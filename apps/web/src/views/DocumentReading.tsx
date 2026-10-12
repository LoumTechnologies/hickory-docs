import { SurfaceInfo } from "../components/SurfaceInfo";
import { useEffect, useState } from "react";
import { ComparisonEditor } from "../components/ComparisonEditor";
import type { ComparisonModel } from "../lib/projectComparison";
import { useReading } from "../lib/readingViews";
import { CommitReading } from "./CommitReading";
import "./DocumentReading.css";

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
  const [draft, setDraft] = useState<ComparisonModel | null>(null);
  useEffect(() => { setDraft(null); },[id,reading?.before,reading?.after]);
  if (id.startsWith("commit:")) return <CommitReading sha={id.slice(7)} />;
  if (!reading) return <p className="muted">This proposal is no longer connected. Reopen its conversation to review changes.</p>;
  const decide = async (accepted: boolean) => {
    setBusy(true); setError("");
    try { await reading.decide?.(accepted, draft?.sources.find(s => s.id === draft.new)?.text); } catch (e) { setError(e instanceof Error ? e.message : String(e)); }
    finally { setBusy(false); }
  };
  return <section className="document-reading" aria-label="Document change review">
    <SurfaceInfo name={`Change review · ${reading.path}`} editable={!!reading.editable} destination={reading.status === "pending" ? "Accept writes the proposed change to its target. Reject leaves the target unchanged." : "This review does not save or execute a document."} />
    <header className="doc-tab-toolbar"><span>{reading.title} · {reading.path} · {reading.status}</span>
      {reading.status === "pending" && <><button className="btn" disabled={busy} onClick={() => void decide(true)}>Accept change</button>
        <button className="btn" disabled={busy} onClick={() => void decide(false)}>Reject change</button></>}
    </header>
    {error && <p role="alert">{error}</p>}
    <ComparisonEditor path={reading.path} model={draft ?? { sources: [
      {id:"old",label:"Old · captured destination",kind:"directory",text:reading.before,project:reading.project},
      {id:"new",label:"New · proposed change",kind:"authored",target:reading.path,text:reading.after,project:reading.project},
    ],old:"old",new:"new",policy:reading.editable && !busy ? "new" : "neither",evidence:"unverified"}} onChange={reading.editable && !busy ? setDraft : undefined} />
  </section>;
}
