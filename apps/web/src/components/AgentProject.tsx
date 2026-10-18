import { useEffect, useMemo, useState } from "react";
import { request } from "../api/client";
import { ChangesetDocument } from "./ChangesetDocument";
import { ComparisonEditor } from "./ComparisonEditor";
import type { AgentEditorContext } from "../api/agentTypes";
import type { AgentChange } from "../api/acp";
import type { ProjectEntry } from "../editor/hickLang";
import type { ComparisonModel } from "../lib/projectComparison";

interface Snapshot { entries: Record<string,{ bytes: number[]; kind: string; executable: boolean }> }
interface Capture { workspace?: { root: string | null; identities: Record<string,string | null> }; artifact?: { entry: string; sources: Record<string,string>; expected: string }; entry: string; ambiguous: string[]; error: string | null; response: string | null; starting: Snapshot; completed: Snapshot | null }
const entries = (snapshot: Snapshot): ProjectEntry[] => Object.entries(snapshot.entries).map(([path,e]) => ({...e,path,kind:e.kind === "File" ? "file" : "symlink"}));
export function AgentProject({ doc, turn, getContext, applyChange }: { doc: string; turn: string; getContext?: () => Promise<AgentEditorContext>; applyChange?: (change: AgentChange) => void | Promise<void> }) {
  const [draft,setDraft] = useState<Record<string,string> | null>(null);
  const [pending,setPending] = useState(false);
  const [receipt,setReceipt] = useState<{applied: string[]; error: string | null; undo: unknown[]; editors: AgentChange[]; recordError?: string; editorResults?: {buffer:string;ok:boolean;error?:string}[]} | null>(null);
  const [capture,setCapture] = useState<Capture | null>(null);
  const [chosen,setChosen] = useState<ReadonlySet<string>>(new Set());
  const [error,setError] = useState("");
  const [busy,setBusy] = useState(false);
  const [job,setJob] = useState<{status: string; error?: string} | null>(null);
  const starting = useMemo(() => capture ? entries(capture.starting) : [], [capture?.starting]);
  const path = `/api/docs/${encodeURIComponent(doc)}/agent/projects/${turn}`;
  useEffect(() => { setCapture(null); setDraft(null); setPending(false); setReceipt(null); setChosen(new Set()); setError(""); setJob(null); },[doc,turn]);
  useEffect(() => {
    let live = true;
    void request<Capture>("GET",path).then(next => { if (live && next.starting?.entries) setCapture(next); },() => {});
    return () => { live = false; };
  },[path]);
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
  const applyDraft = () => run(async () => {
    if (!draft || pending) throw new Error("Wait for successful downstream construction before applying.");
    if (capture?.workspace && !getContext) throw new Error("Open the captured workspace window before applying.");
    const context = getContext ? await getContext() : {buffers:[]};
    const result = await request<NonNullable<typeof receipt>>("POST",`${path}/draft/apply`,{sources:draft,context});
    const editorResults: NonNullable<typeof receipt>["editorResults"] = [];
    // Disk and editor publication are independent transactions. Preserve every result.
    let stopped = false;
    for (const change of result.editors) {
      if (stopped) { editorResults.push({buffer:change.buffer ?? change.name,ok:false,error:"Not applied after an earlier editor failed."}); continue; }
      try {
        if (!applyChange) throw new Error("The captured editor is unavailable. Keep the revision and reopen its editor.");
        await applyChange(change);
        editorResults.push({buffer:change.buffer ?? change.name,ok:true});
      } catch (e) {
        editorResults.push({buffer:change.buffer ?? change.name,ok:false,error:String(e)});
        stopped = true;
      }
    }
    const report = {...result,editorResults};
    setReceipt(report);
    try { await request("POST",`${path}/draft/result`,report); }
    catch (e) { setReceipt({...report,recordError:`Could not record editor results: ${String(e)}. Keep this receipt.`}); }
  });
  const retryable = !!receipt && (!!receipt.error || !!receipt.editorResults?.some(r => !r.ok)) && receipt.applied.length === 0 && !receipt.editorResults?.some(r => r.ok);
  if (!capture && !error) return null;
  return <section className="agent-project" aria-label="Literate project changes">
    <button className="btn" disabled={busy || !!draft} onClick={() => void run(async () => { setCapture(await request<Capture>("GET",path)); })}>Review completed project changes</button>
    {error && <p role="alert">{error}</p>}
    {capture && <>
      <p>{capture.workspace ? capture.workspace.root ? `Workspace ${capture.workspace.root}` : "Open editors · no folder" : capture.entry} · Old is the captured starting state, including pre-existing work. New is the actual completed state.</p>
      {capture.workspace && <p className="muted">Editor addresses identify captured buffers and have no filesystem destination. Disk patches use the complete captured filesystem for native tools; editor-only patches have no executable project. Incorporation requires maintained literate source.</p>}
      {capture.error && <p role="alert">Incomplete capture: {capture.error}</p>}
      {capture.ambiguous.length > 0 && <fieldset><legend>Resolve attribution before finalizing</legend>
        <p className="muted">These writes do not have an exclusive agent identity. Confirm which belong in this transition; unconfirmed paths prevent finalization.</p>
        {capture.ambiguous.map(path => <label key={path}><input type="checkbox" checked={chosen.has(path)} onChange={() => { const next = new Set(chosen); if (!next.delete(path)) next.add(path); setChosen(next); }} />{path}</label>)}
      </fieldset>}
      {!capture.response && capture.completed && [...new Set([...Object.keys(capture.starting.entries),...Object.keys(capture.completed.entries)])].map(file => {
        const old = capture.starting.entries[file], next = capture.completed!.entries[file];
        if (JSON.stringify(old) === JSON.stringify(next)) return null;
        let before: string, after: string;
        try { const decode = (e: typeof old) => e ? new TextDecoder("utf-8",{fatal:true}).decode(new Uint8Array(e.bytes)) : ""; before = decode(old); after = decode(next); }
        catch { return <p key={file}>{file} · byte-exact binary entry changed</p>; }
        const model: ComparisonModel = { sources: [
          {id:"old",label:"Old · captured input",kind:"directory",text:before},
          {id:"new",label:"New · completed project",kind:"directory",text:after},
        ],old:"old",new:"new",policy:"neither",evidence:"unverified"};
        return <ComparisonEditor key={file} path={file} model={model} />;
      })}
      {(!capture.response || capture.ambiguous.length > 0) && <button className="btn" disabled={busy || !!capture.error || chosen.size !== capture.ambiguous.length} onClick={() => void finalize()}>Verify A-to-B explanation</button>}
      {capture.response && <>
        <p role="status">A-to-B source equality verified. Runtime behavior and explanatory accuracy are separate checks.</p>
        <ChangesetDocument identity={`final:${doc}:${turn}`} entry={capture.artifact?.entry ?? "response.md"}
          tools={!capture.workspace || capture.workspace.root !== null} filesystemOnly={!!capture.workspace} sources={capture.artifact?.sources ?? {"response.md":capture.response}} input={starting} />
        {!draft && <button className="btn" disabled={busy || !!capture.error || capture.ambiguous.length > 0} onClick={() => { setDraft({...capture.artifact?.sources ?? {"response.md":capture.response!}}); setPending(false); setReceipt(null); }}>Revise New</button>}
        {draft && <section aria-label="Authored final changeset draft">
          <p>Authored revision of the agent result · Old remains captured A. This draft constructs C; Apply replaces accepted B with C. Inherited verification claims do not apply. Runtime checks are unrun.</p>
          <p role="status">{pending ? "Draft has unchecked edits; application is disabled." : "C construction succeeded. Destinations have not been changed by editing."}</p>
          <ChangesetDocument identity={`revision:${doc}:${turn}`} entry={capture.artifact?.entry ?? "response.md"} sources={draft} input={starting}
            tools={!capture.workspace || capture.workspace.root !== null} filesystemOnly={!!capture.workspace}
            onEdit={receipt || busy ? undefined : setDraft} onPending={setPending} />
          <button className="btn" disabled={busy || pending || !!receipt && !retryable} onClick={() => void applyDraft()}>Apply revision</button>
          <button className="btn" disabled={busy} onClick={() => { setDraft(null); setPending(false); }}>Discard draft</button>
          <p className="muted">Application is guarded per destination, not atomic across files and editors. Editor undo is retained where supported. Discarding an applied draft does not undo its writes. On partial failure, preserve this receipt and restore landed disk paths individually against their after bytes.</p>
        </section>}
        {receipt && <div>
          <p role={receipt.error || receipt.editorResults?.some(r => !r.ok) ? "alert" : "status"}>{receipt.error || receipt.editorResults?.find(r => !r.ok)?.error || "Revision application succeeded. Runtime checks remain unrun."} Applied disk destinations: {receipt.applied.join(", ") || "none"}. Applied editor destinations: {receipt.editorResults?.filter(r => r.ok).map(r => r.buffer).join(", ") || "none"}.</p>
          {retryable && <p>Nothing applied. Restore expected B or open the captured editor, then Apply revision again.</p>}
          {receipt.recordError && <p role="alert">{receipt.recordError}</p>}
          <details><summary>Application and recovery receipt</summary><pre>{JSON.stringify(receipt,null,2)}</pre></details>
        </div>}
        <button className="btn" disabled={!!capture.workspace || !!draft || busy || capture.ambiguous.length > 0 || !!job && ["generating","reviewing","applying"].includes(job.status)} onClick={() => void incorporate()}>Apply to literate source</button>
      </>}
      {job && <p role={job.error ? "alert" : "status"}>{job.error ?? (job.status === "reviewing" ? "Verified candidate ready in Document edits. Review every source change before applying." : `Incorporation · ${job.status}`)}</p>}
    </>}
  </section>;
}
