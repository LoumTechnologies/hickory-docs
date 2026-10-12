import { useEffect, useState } from "react";
import { request } from "../api/client";
import { DocumentComparison } from "../views/DocumentReading";
import { ComparisonEditor } from "./ComparisonEditor";
import type { ProjectEntry } from "../editor/hickLang";
import type { ComparisonModel } from "../lib/projectComparison";

interface Snapshot { entries: Record<string,{ bytes: number[]; kind: string; executable: boolean }> }
interface Capture { entry: string; ambiguous: string[]; error: string | null; response: string | null; starting: Snapshot; completed: Snapshot | null }
const entries = (snapshot: Snapshot): ProjectEntry[] => Object.entries(snapshot.entries).map(([path,e]) => ({...e,path,kind:e.kind === "File" ? "file" : "symlink"}));
export function AgentProject({ doc, turn }: { doc: string; turn: string }) {
  const [capture,setCapture] = useState<Capture | null>(null);
  const [chosen,setChosen] = useState<ReadonlySet<string>>(new Set());
  const [error,setError] = useState("");
  const [busy,setBusy] = useState(false);
  const [job,setJob] = useState<{status: string; error?: string} | null>(null);
  const path = `/api/docs/${encodeURIComponent(doc)}/agent/projects/${turn}`;
  useEffect(() => { setCapture(null); setChosen(new Set()); setError(""); setJob(null); },[doc,turn]);
  useEffect(() => {
    if (!job || !["generating","reviewing","applying"].includes(job.status)) return;
    const timer = window.setInterval(() => { void request<typeof job>("GET",`${path}/incorporate`).then(setJob,e => setError(String(e))); },1000);
    return () => window.clearInterval(timer);
  },[path,job?.status]);
  const run = async (action: () => Promise<void>) => { setBusy(true); setError(""); try { await action(); } catch(e) { setError(e instanceof Error ? e.message : String(e)); } finally { setBusy(false); } };
  const finalize = () => run(async () => {
    const result = await request<{response: string}>("POST",path,{resolve:[...chosen]});
    setCapture(previous => previous && {...previous,response:result.response,ambiguous:[]});
  });
  const incorporate = () => run(async () => {
    setJob(await request<{status:string}>("POST",`${path}/incorporate`,{}));
  });
  return <section className="agent-project" aria-label="Literate project changes">
    <button className="btn" disabled={busy} onClick={() => void run(async () => { setCapture(await request<Capture>("GET",path)); })}>Review completed project changes</button>
    {error && <p role="alert">{error}</p>}
    {capture && <>
      <p>{capture.entry} · Old is the captured starting project, including pre-existing work. New is the actual completed source tree.</p>
      {capture.error && <p role="alert">Incomplete capture: {capture.error}</p>}
      {capture.ambiguous.length > 0 && <fieldset><legend>Resolve attribution before finalizing</legend>
        <p className="muted">These writes do not have an exclusive agent identity. Confirm which belong in this transition; unconfirmed paths prevent finalization.</p>
        {capture.ambiguous.map(path => <label key={path}><input type="checkbox" checked={chosen.has(path)} onChange={() => { const next = new Set(chosen); if (!next.delete(path)) next.add(path); setChosen(next); }} />{path}</label>)}
      </fieldset>}
      {capture.completed && [...new Set([...Object.keys(capture.starting.entries),...Object.keys(capture.completed.entries)])].map(file => {
        const old = capture.starting.entries[file], next = capture.completed!.entries[file];
        if (JSON.stringify(old) === JSON.stringify(next)) return null;
        let before: string, after: string;
        try { const decode = (e: typeof old) => e ? new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(e.bytes)) : ""; before = decode(old); after = decode(next); }
        catch { return <p key={file}>{file} · byte-exact binary entry changed</p>; }
        const model: ComparisonModel = { sources: [
          {id:"old",label:"Old · captured input",kind:"directory",text:before,project:entries(capture.starting)},
          {id:"new",label:"New · completed project",kind:"directory",text:after,project:entries(capture.completed!)},
        ],old:"old",new:"new",policy:"neither",evidence:"unverified"};
        return <ComparisonEditor key={file} path={file} model={model} />;
      })}
      {!capture.response && <button className="btn" disabled={busy || !!capture.error || chosen.size !== capture.ambiguous.length} onClick={() => void finalize()}>Verify A-to-B explanation</button>}
      {capture.response && <>
        <p role="status">A-to-B source equality verified. Runtime behavior and explanatory accuracy are separate checks.</p>
        <DocumentComparison id={`final:${turn}`} path="response.md" before="" after={capture.response} />
        <button className="btn" disabled={busy || !!job && ["generating","reviewing","applying"].includes(job.status)} onClick={() => void incorporate()}>Apply to literate source</button>
      </>}
      {job && <p role={job.error ? "alert" : "status"}>{job.error ?? (job.status === "reviewing" ? "Verified candidate ready in Document edits. Review every source change before applying." : `Incorporation · ${job.status}`)}</p>}
    </>}
  </section>;
}
