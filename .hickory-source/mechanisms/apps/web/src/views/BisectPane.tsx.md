<storycode:doc xmlns:storycode="http://www.hickorydocs.com/1.0">
# Maintain apps/web/src/views/BisectPane.tsx

For: maintainers changing this mechanism and reviewing the contracts that depend on it.

This page owns the output construction and the explained fragments below. Source-comment explanations retain their original evidentiary limits; the explanation review ledger lists gaps.

<a id="output-73f5ba1f30b7"></a>
## `apps/web/src/views/BisectPane.tsx`

Present and edit the shared engine state. Browser-side revisions, pending requests and local drafts affect which source and UI actions are still valid.

Related contracts: [Bisect keeps candidates isolated](../../../../../guarantees/git/bisect-keeps-candidates-isolated/contract.md).

<a id="f-73f5ba1f30b7-BisectPane"></a>
### `BisectPane`

<storycode:copy id="f-73f5ba1f30b7-BisectPane">
import { useBisectActions } from "./uiBisectActions";
import { useUiReading } from "../lib/uiReading";
import type { UiOutcome } from "../lib/uiActions";
import { EvidenceDisclosure } from "../components/EvidenceDisclosure";
import { useCallback, useEffect, useRef, useState } from "react";
import { bisects, type BisectSession } from "../api/representations";
import { layout, graphWidth, laneColor } from "../lib/gitGraph";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";

export function BisectPane() {
  const [open, setOpen] = useState(false);
  const inFlight = useRef(false);
  const generation = useRef(0);
  const [sessions, setSessions] = useState<BisectSession[]>([]);
  const [good, setGood] = useState("");
  const [bad, setBad] = useState("HEAD");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const refresh = useCallback(() => { const request = ++generation.current; void bisects.list().then(v => { if (request === generation.current) setSessions(v.sessions); }, e => { if (request === generation.current) setNotice(String(e)); }); }, []);
  useEffect(() => { refresh(); const timer = setInterval(refresh, 3000); window.addEventListener("focus", refresh); return () => { generation.current++; clearInterval(timer); window.removeEventListener("focus", refresh); }; }, [refresh]);
  const run = async (action: () => Promise<unknown>): Promise<UiOutcome> => {
    if (inFlight.current) return { status: "unavailable", reason: "Another bisect operation is pending" };
    inFlight.current = true; setBusy(true); setNotice("");
    try {
      const result = await action();
      if (result && typeof result === "object" && "retained_inspections" in result) setNotice("Bisect ended. Inspection worktrees are retained so open windows and experiments remain intact. Remove them with git worktree remove when finished.");
      if (result && typeof result === "object" && "patch" in result) setNotice(`Experiment saved at ${result.patch}; candidate restored.`);
      refresh(); window.dispatchEvent(new Event(FILES_CHANGED_EVENT));
      return { status: "completed", value: { reply: JSON.stringify(result ?? null), scope: "Actual host reply; not independent test evidence or classification" } };
    } catch (e) { const reason = e instanceof Error ? e.message : String(e); setNotice(reason); return { status: "unavailable", reason }; }
    finally { inFlight.current = false; setBusy(false); }
  };
  const action = useBisectActions({ open, setOpen, sessions, good, bad, setGood, setBad, busy, notice, run, refresh });
  return <details className="bisect-pane" open={open}><summary onClick={event => { event.preventDefault(); void action("visibility", { open: !open }); }}>Visual bisect</summary>
    {open && <>
    <BisectReading value={{ good, bad, busy, notice, sessions: sessions.map(({ commits: _commits, ...session }) => session), scope: "Host-reported sessions and graph; remaining commit text is separately disclosed. No independent verification." }} />
    <p>Find the first bad commit. Each candidate runs in a separate worktree; your checkout stays in place.</p>
    <form onSubmit={e => { e.preventDefault(); void action("start"); }}>
      <label>Known good <input aria-label="Known good commit" value={good} onChange={e => setGood(e.target.value)} /></label>
      <label>Known bad <input aria-label="Known bad commit" value={bad} onChange={e => setBad(e.target.value)} /></label>
      <button className="btn" disabled={busy || !good.trim() || !bad.trim()}>Start bisect</button>
    </form>
    {notice && <p role="alert">{notice}</p>}
    {sessions.map(session => <section key={session.id} aria-label="Bisect search">
      <p><strong>{session.outcome === "first_bad" ? "First bad commit" : session.outcome === "ambiguous" ? "Skipped commits leave an ambiguous result" : "Candidate"}</strong> <span className="mono">{session.candidate.slice(0, 12)}</span></p>
      <p className="mono">{session.worktree}</p>
      <p>Open the candidate window to use its literate views, comparison editor, ACP agent, tests, and debugger. Earlier candidate windows stay on their own commits when the search advances.</p>
      <button className="btn" disabled={busy} onClick={() => void action(`open:${session.id}`)}>Open candidate window</button>
      {!session.outcome && ["good", "bad", "skip"].map(verdict => <button key={verdict} className="btn" disabled={busy || session.modified} onClick={() => void action(`${verdict}:${session.id}`)}>{verdict[0].toUpperCase() + verdict.slice(1)}</button>)}
      {session.modified && <p role="status">Candidate modified. Preserve and restore the experiment before classifying it.</p>}
      {session.modified && <button className="btn" disabled={busy} onClick={() => void action(`restore:${session.id}`)}>Keep experiment as patch and restore candidate</button>}
      <button className="btn" disabled={busy || session.modified} onClick={() => void action(`finish:${session.id}`)}>End bisect</button>
      <pre className="bisect-evidence">{session.said}</pre>
      <BisectGraph session={session} />
      <EvidenceDisclosure label={`Remaining commits for bisect ${session.id}`} value={{ session: session.id, candidate: session.candidate, commits: session.commits }}><pre>{session.commits}</pre></EvidenceDisclosure>
      <ol>{session.history.map((entry, i) => <li key={i}><span className="mono">{entry.candidate.slice(0, 12)}</span> · {entry.verdict}</li>)}</ol>
    </section>)}
    </>}
  </details>;
}

</storycode:copy>
<a id="f-73f5ba1f30b7-BisectReading"></a>
### `BisectReading`

<storycode:copy id="f-73f5ba1f30b7-BisectReading">
function BisectReading({ value }: { value: unknown }) { useUiReading("visual bisect state", value); return null; }

</storycode:copy>
<a id="f-73f5ba1f30b7-BisectGraph"></a>
### `BisectGraph`

<storycode:copy id="f-73f5ba1f30b7-BisectGraph">
function BisectGraph({session}:{session:BisectSession}) {
  const rows=layout(session.graph), width=(graphWidth(rows)+1)*18;
  const verdicts=new Map(session.history.map(v=>[v.candidate,v.verdict]));
  return <ol className="bisect-graph" aria-label="Bisect commit graph">{session.graph.map((commit,i)=> {
    const row=rows[i], status=commit.sha===session.candidate ? (session.outcome??"candidate") : verdicts.get(commit.sha)??(commit.sha===session.bad?"known bad":"untested");
    return <li key={commit.sha} aria-current={commit.sha===session.candidate?"step":undefined}>
      <svg width={width} height={28} aria-hidden="true">{row.through.map((edge,j)=><path key={j} className={`git-edge git-edge--c${laneColor(edge.from)}`} d={`M ${9+edge.from*18} 0 L ${9+edge.from*18} 10 L ${9+edge.to*18} 18 L ${9+edge.to*18} 28`} />)}<circle className={`git-node git-node--c${laneColor(row.lane)}`} cx={9+row.lane*18} cy={14} r={4}/></svg>
      <span className="mono">{commit.sha.slice(0,8)}</span> <span>{commit.subject}</span> <strong>{status.replaceAll("_"," ")}</strong>
    </li>;
  })}</ol>;
}
</storycode:copy>

### Assemble `apps/web/src/views/BisectPane.tsx`

The selections below preserve the original byte order. Narrative reading order does not change the output.

<storycode:file path="apps/web/src/views/BisectPane.tsx"><storycode:paste select="#f-73f5ba1f30b7-BisectPane"/><storycode:paste select="#f-73f5ba1f30b7-BisectReading"/><storycode:paste select="#f-73f5ba1f30b7-BisectGraph"/></storycode:file>

</storycode:doc>
