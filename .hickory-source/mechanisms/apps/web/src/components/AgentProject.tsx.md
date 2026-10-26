<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/components/AgentProject.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-33368606905c"></a>
## `apps/web/src/components/AgentProject.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

<a id="f-33368606905c-Snapshot"></a>
### `Snapshot`

<storycode:copy id="f-33368606905c-Snapshot">
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import { useUiReading } from "../lib/uiReading";
import type { UiCommand, UiOutcome } from "../lib/uiActions";
import { EvidenceDisclosure } from "./EvidenceDisclosure";
import { useEffect, useId, useMemo, useRef, useState } from "react";
import { request } from "../api/client";
import { ChangesetDocument } from "./ChangesetDocument";
import { ComparisonEditor } from "./ComparisonEditor";
import type { AgentEditorContext } from "../api/agentTypes";
import type { AgentChange } from "../api/acp";
import type { ProjectEntry } from "../editor/hickLang";
import type { ComparisonModel } from "../lib/projectComparison";

interface Snapshot { entries: Record<string,{ bytes: number[]; kind: string; executable: boolean }> }
</storycode:copy>
<a id="f-33368606905c-Capture"></a>
### `Capture`

<storycode:copy id="f-33368606905c-Capture">
interface Capture { workspace?: { root: string | null; identities: Record<string,string | null> }; artifact?: { entry: string; sources: Record<string,string>; expected: string }; entry: string; ambiguous: string[]; error: string | null; response: string | null; starting: Snapshot; completed: Snapshot | null }
</storycode:copy>
<a id="f-33368606905c-entries"></a>
### `entries`

<storycode:copy id="f-33368606905c-entries">
const entries = (snapshot: Snapshot): ProjectEntry[] => Object.entries(snapshot.entries).map(([path,e]) => ({...e,path,kind:e.kind === "File" ? "file" : "symlink"}));
</storycode:copy>
<a id="f-33368606905c-AgentProject"></a>
### `AgentProject`

<storycode:copy id="f-33368606905c-AgentProject">
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
  const [loadedFor, setLoadedFor] = useState("");
  const operation = useMemo(() => ({ pending: false }), [path]);
  const owner = useRef(path); owner.current = path;
  const mounted = useRef(true);
  useEffect(() => { mounted.current = true; return () => { mounted.current = false; }; }, []);
  const current = useRef({ draft, capture, pending }); current.current = { draft, capture, pending };
  const owns = () => mounted.current && owner.current === path;
  useEffect(() => { setCapture(null); setDraft(null); setPending(false); setReceipt(null); setChosen(new Set()); setError(""); setJob(null); setBusy(false); },[doc,turn]);
  useEffect(() => {
    let live = true;
    void request<Capture>("GET",path).then(next => { if (live && next.starting?.entries) { setCapture(next); setLoadedFor(path); } },e => { if (live) setError(String(e)); });
    return () => { live = false; };
  },[path]);
  useEffect(() => {
    if (!job || !["generating","reviewing","applying"].includes(job.status)) return;
    let live = true;
    const timer = window.setInterval(() => { void request<typeof job>("GET",`${path}/incorporate`).then(next => { if (live) setJob(next); }, e => { if (live) setError(String(e)); }); },1000);
    return () => { live = false; window.clearInterval(timer); };
  },[path,job?.status]);
  const run = async (action: () => Promise<UiOutcome>): Promise<UiOutcome> => {
    if (operation.pending || !owns()) return { status: "unavailable", reason: "Project request pending or owning conversation changed" };
    operation.pending = true; setBusy(true); setError("");
    try { return await action(); }
    catch (e) { const reason = e instanceof Error ? e.message : String(e); if (owns()) setError(reason); return { status: "unavailable", reason }; }
    finally { operation.pending = false; if (owns()) setBusy(false); }
  };
  const review = () => run(async () => {
    const next = await request<Capture>("GET", path);
    if (owns()) { setCapture(next); setLoadedFor(path); }
    return { status: "completed", value: { entry: next.entry, response_available: !!next.response, error: next.error, ambiguous: next.ambiguous } };
  });
  const finalize = () => run(async () => {
    const result = await request<{response: string}>("POST",path,{resolve:[...chosen]});
    if (owns()) setCapture(previous => previous && {...previous,response:result.response,ambiguous:[]});
    return { status: "completed", value: { source_equality_verified: true, runtime_verified: false, scope: "Actual finalization reply; human attribution selections are separate" } };
  });
  const incorporate = () => run(async () => {
    const result = await request<{status:string}>("POST",`${path}/incorporate`,{});
    if (owns()) setJob(result);
    return { status: "queued", job: `${path}/incorporate`, value: { status: result.status, scope: "Host incorporation requested; review and application are separate" } };
  });
  const applyDraft = () => run(async () => {
    if (!draft || pending) throw new Error("Wait for successful downstream construction before applying.");
    if (capture?.workspace && !getContext) throw new Error("Open the captured workspace window before applying.");
    const sources = structuredClone(draft), expected = JSON.stringify(sources);
    const context = getContext ? await getContext() : {buffers:[]};
    if (!owns() || current.current.pending || JSON.stringify(current.current.draft) !== expected || current.current.capture !== capture) throw new Error("Draft or owning conversation changed during context capture; apply the current revision again.");
    const result = await request<NonNullable<typeof receipt>>("POST",`${path}/draft/apply`,{sources,context});
    const editorResults: NonNullable<typeof receipt>["editorResults"] = [];
    // Disk and editor publication are independent transactions. Preserve every result.
    let stopped = false;
    for (const change of result.editors) {
      if (stopped) { editorResults.push({buffer:change.buffer ?? change.name,ok:false,error:"Not applied after an earlier editor failed."}); continue; }
      try {
        if (!owns()) throw new Error("The owning conversation closed or changed; editor publication was refused.");
        if (!applyChange) throw new Error("The captured editor is unavailable. Keep the revision and reopen its editor.");
        await applyChange(change);
        editorResults.push({buffer:change.buffer ?? change.name,ok:true});
      } catch (e) {
        editorResults.push({buffer:change.buffer ?? change.name,ok:false,error:String(e)});
        stopped = true;
      }
    }
    const report = {...result,editorResults};
    if (owns()) setReceipt(report);
    let recordError: string | null = result.recordError ?? null;
    try { await request("POST",`${path}/draft/result`,report); }
    catch (e) { recordError = [recordError, `Could not record editor results: ${String(e)}. Keep this receipt.`].filter(Boolean).join(" "); if (owns()) setReceipt({...report,recordError}); }
    return { status: "completed", value: { applied_disk: result.applied, editor_results: editorResults, application_succeeded: !result.error && editorResults.every(r => r.ok), error: result.error, record_error: recordError, runtime_verified: false, scope: "Actual per-destination results; disk and editor writes are not atomic together" } };
  });
  const retryable = !!receipt && (!!receipt.error || !!receipt.editorResults?.some(r => !r.ok)) && receipt.applied.length === 0 && !receipt.editorResults?.some(r => r.ok);
  const parent = `agent-project:${doc}:${turn}:${useId()}`;
  const revision = useUiRevision(() => [path, loadedFor, current.current, [...chosen], receipt, busy, operation.pending, job, error]);
  const commands: { key: string; command: UiCommand }[] = [];
  const add = (key: string, label: string, enabled: boolean, execute: UiCommand["execute"], parameters: UiCommand["parameters"] = {}) => commands.push({ key: `${parent}:${key}`, command: { parent, subject: parent, label, parameters, prerequisites: [], read: () => ({ revision: revision(), enabled: enabled && !operation.pending && owns(), reason: enabled ? undefined : "Wait for capture/construction, resolve attribution, or discard the current draft" }), execute } });
  add("review", "Review completed project changes", !busy && !draft, review);
  if (capture && loadedFor === path) {
    if (capture.ambiguous.length) add("attribution", "Set human project attribution choice", !busy, async (args, actor) => {
      if (actor.kind !== "human") return { status: "requires_input", request: `${parent}:attribution`, reason: "A human must confirm exclusive attribution of ambiguous writes using the displayed checkboxes" };
      const target = args.path as string;
      if (!capture.ambiguous.includes(target)) return { status: "unavailable", reason: "Path is not an ambiguous captured write" };
      setChosen(previous => { const next = new Set(previous); if (args.selected) next.add(target); else next.delete(target); return next; });
      return { status: "completed", value: { path: target, selected: args.selected, actor: "human" } };
    }, { path: { type: "string", maxLength: 4096 }, selected: { type: "boolean" } });
    if (!capture.response || capture.ambiguous.length) add("finalize", "Verify project A-to-B explanation", !busy && !capture.error && chosen.size === capture.ambiguous.length, finalize);
    if (capture.response) {
      if (!draft) add("revise", "Revise completed project New", !busy && !capture.error && !capture.ambiguous.length, async () => { setDraft({...capture.artifact?.sources ?? {"response.md":capture.response!}}); setPending(false); setReceipt(null); return { status: "completed", value: { draft_created: true, destinations_changed: false, runtime_verified: false } }; });
      if (draft) {
        add("apply", "Apply authored project revision", !busy && !pending && (!receipt || retryable), applyDraft);
        add("discard", "Discard authored project draft", !busy, async () => { setDraft(null); setPending(false); return { status: "completed", value: { draft_discarded: true, prior_writes_undone: false } }; });
      }
      add("incorporate", "Apply project to literate source", !capture.workspace && !draft && !busy && !capture.ambiguous.length && (!job || !["generating","reviewing","applying"].includes(job.status)), incorporate);
    }
    if (job) add("job", "Refresh project incorporation status", !busy, () => run(async () => { const result = await request<NonNullable<typeof job>>("GET", `${path}/incorporate`); if (owns()) setJob(result); return { status: "completed", value: result }; }));
  }
  const invoke = useUiCommands(!capture && !error ? [] : commands);
  const human = (key: string, args = {}) => void invoke(`${parent}:${key}`, args).catch(e => setError(String(e)));
  const statusReading = { document: doc, turn, loaded: loadedFor === path, pending: busy, error, draft: !!draft, construction_pending: pending, chosen_attribution: [...chosen], capture: capture && loadedFor === path ? { entry: capture.entry, workspace: capture.workspace, ambiguous: capture.ambiguous, error: capture.error, response_available: !!capture.response, starting_files: Object.keys(capture.starting.entries), completed_files: capture.completed && Object.keys(capture.completed.entries) } : null, job, receipt: receipt && { applied: receipt.applied, error: receipt.error, editor_results: receipt.editorResults, record_error: receipt.recordError }, scope: "Transition metadata; source and recovery bytes require their explicit disclosed reading surfaces" };
  if (!capture && !error) return null;
  return <section className="agent-project" aria-label="Literate project changes">
    <ProjectStatusReading value={statusReading} />
    <button className="btn" disabled={busy || !!draft} onClick={() => human("review")}>Review completed project changes</button>
    {error && <p role="alert">{error}</p>}
    {capture && <>
      <p>{capture.workspace ? capture.workspace.root ? `Workspace ${capture.workspace.root}` : "Open editors · no folder" : capture.entry} · Old is the captured starting state, including pre-existing work. New is the actual completed state.</p>
      {capture.workspace && <p className="muted">Editor addresses identify captured buffers and have no filesystem destination. Disk patches use the complete captured filesystem for native tools; editor-only patches have no executable project. Incorporation requires maintained literate source.</p>}
      {capture.error && <p role="alert">Incomplete capture: {capture.error}</p>}
      {capture.ambiguous.length > 0 && <fieldset><legend>Resolve attribution before finalizing</legend>
        <p className="muted">These writes do not have an exclusive agent identity. Confirm which belong in this transition; unconfirmed paths prevent finalization.</p>
        {capture.ambiguous.map(path => <label key={path}><input type="checkbox" checked={chosen.has(path)} disabled={busy} onChange={event => human("attribution", { path, selected: event.target.checked })} />{path}</label>)}
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
      {(!capture.response || capture.ambiguous.length > 0) && <button className="btn" disabled={busy || !!capture.error || chosen.size !== capture.ambiguous.length} onClick={() => human("finalize")}>Verify A-to-B explanation</button>}
      {capture.response && <>
        <p role="status">A-to-B source equality verified. Runtime behavior and explanatory accuracy are separate checks.</p>
        <ChangesetDocument identity={`final:${doc}:${turn}`} entry={capture.artifact?.entry ?? "response.md"}
          tools={!capture.workspace || capture.workspace.root !== null} filesystemOnly={!!capture.workspace} sources={capture.artifact?.sources ?? {"response.md":capture.response}} input={starting} />
        {!draft && <button className="btn" disabled={busy || !!capture.error || capture.ambiguous.length > 0} onClick={() => human("revise")}>Revise New</button>}
        {draft && <section aria-label="Authored final changeset draft">
          <p>Authored revision of the agent result · Old remains captured A. This draft constructs C; Apply replaces accepted B with C. Inherited verification claims do not apply. Runtime checks are unrun.</p>
          <p role="status">{pending ? "Draft has unchecked edits; application is disabled." : "C construction succeeded. Destinations have not been changed by editing."}</p>
          <ChangesetDocument identity={`revision:${doc}:${turn}`} entry={capture.artifact?.entry ?? "response.md"} sources={draft} input={starting}
            tools={!capture.workspace || capture.workspace.root !== null} filesystemOnly={!!capture.workspace}
            onEdit={receipt || busy ? undefined : next => { current.current.draft = next; setDraft(next); }} onPending={next => { current.current.pending = next; setPending(next); }} />
          <button className="btn" disabled={busy || pending || !!receipt && !retryable} onClick={() => human("apply")}>Apply revision</button>
          <button className="btn" disabled={busy} onClick={() => human("discard")}>Discard draft</button>
          <p className="muted">Application is guarded per destination, not atomic across files and editors. Editor undo is retained where supported. Discarding an applied draft does not undo its writes. On partial failure, preserve this receipt and restore landed disk paths individually against their after bytes.</p>
        </section>}
        {receipt && <div>
          <p role={receipt.error || receipt.editorResults?.some(r => !r.ok) ? "alert" : "status"}>{receipt.error || receipt.editorResults?.find(r => !r.ok)?.error || "Revision application succeeded. Runtime checks remain unrun."} Applied disk destinations: {receipt.applied.join(", ") || "none"}. Applied editor destinations: {receipt.editorResults?.filter(r => r.ok).map(r => r.buffer).join(", ") || "none"}.</p>
          {retryable && <p>Nothing applied. Restore expected B or open the captured editor, then Apply revision again.</p>}
          {receipt.recordError && <p role="alert">{receipt.recordError}</p>}
          <EvidenceDisclosure label="Application and recovery receipt" value={receipt}><pre>{JSON.stringify(receipt,null,2)}</pre></EvidenceDisclosure>
        </div>}
        <button className="btn" disabled={!!capture.workspace || !!draft || busy || capture.ambiguous.length > 0 || !!job && ["generating","reviewing","applying"].includes(job.status)} onClick={() => human("incorporate")}>Apply to literate source</button>
      </>}
      {job && <p role={job.error ? "alert" : "status"}>{job.error ?? (job.status === "reviewing" ? "Verified candidate ready in Document edits. Review every source change before applying." : `Incorporation · ${job.status}`)}</p>}
    </>}
  </section>;
}

</storycode:copy>
<a id="f-33368606905c-ProjectStatusReading"></a>
### `ProjectStatusReading`

<storycode:copy id="f-33368606905c-ProjectStatusReading">
function ProjectStatusReading({ value }: { value: unknown }) { useUiReading("agent project transition status", value); return null; }
</storycode:copy>

### Assemble `apps/web/src/components/AgentProject.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/components/AgentProject.tsx"><storycode:paste select="#f-33368606905c-Snapshot"/><storycode:paste select="#f-33368606905c-Capture"/><storycode:paste select="#f-33368606905c-entries"/><storycode:paste select="#f-33368606905c-AgentProject"/><storycode:paste select="#f-33368606905c-ProjectStatusReading"/></storycode:file>

</storycode:doc>
