import { useCallback, useEffect, useState } from "react";
import { bisects, type BisectSession } from "../api/representations";
import { layout, graphWidth, laneColor } from "../lib/gitGraph";
import { FILES_CHANGED_EVENT } from "../shell/FolderTreePane";

export function BisectPane() {
  const [sessions, setSessions] = useState<BisectSession[]>([]);
  const [good, setGood] = useState("");
  const [bad, setBad] = useState("HEAD");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const refresh = useCallback(() => { void bisects.list().then(v => setSessions(v.sessions), e => setNotice(String(e))); }, []);
  useEffect(() => { refresh(); const timer = setInterval(refresh, 3000); window.addEventListener("focus", refresh); return () => { clearInterval(timer); window.removeEventListener("focus", refresh); }; }, [refresh]);
  const run = async (action: () => Promise<unknown>) => {
    setBusy(true); setNotice("");
    try { const result = await action(); if (result && typeof result === "object" && "retained_inspections" in result) setNotice("Bisect ended. Inspection worktrees are retained so open windows and experiments remain intact. Remove them with git worktree remove when finished."); refresh(); window.dispatchEvent(new Event(FILES_CHANGED_EVENT)); }
    catch (e) { setNotice(e instanceof Error ? e.message : String(e)); }
    finally { setBusy(false); }
  };
  return <details className="bisect-pane"><summary>Visual bisect</summary>
    <p>Find the first bad commit. Each candidate runs in a separate worktree; your checkout stays in place.</p>
    <form onSubmit={e => { e.preventDefault(); void run(() => bisects.start(good, bad)); }}>
      <label>Known good <input aria-label="Known good commit" value={good} onChange={e => setGood(e.target.value)} /></label>
      <label>Known bad <input aria-label="Known bad commit" value={bad} onChange={e => setBad(e.target.value)} /></label>
      <button className="btn" disabled={busy || !good.trim() || !bad.trim()}>Start bisect</button>
    </form>
    {notice && <p role="alert">{notice}</p>}
    {sessions.map(session => <section key={session.id} aria-label="Bisect search">
      <p><strong>{session.outcome === "first_bad" ? "First bad commit" : session.outcome === "ambiguous" ? "Skipped commits leave an ambiguous result" : "Candidate"}</strong> <span className="mono">{session.candidate.slice(0, 12)}</span></p>
      <p className="mono">{session.worktree}</p>
      <p>Open the candidate window to use its literate views, comparison editor, ACP agent, tests, and debugger. Earlier candidate windows stay on their own commits when the search advances.</p>
      <button className="btn" disabled={busy} onClick={() => void run(() => bisects.open(session.id))}>Open candidate window</button>
      {!session.outcome && ["good", "bad", "skip"].map(verdict => <button key={verdict} className="btn" disabled={busy || session.modified} onClick={() => void run(() => bisects.mark(session, verdict))}>{verdict[0].toUpperCase() + verdict.slice(1)}</button>)}
      {session.modified && <p role="status">Candidate modified. Preserve and restore the experiment before classifying it.</p>}
      {session.modified && <button className="btn" disabled={busy} onClick={() => void run(async () => { const saved = await bisects.restore(session.id); setNotice(`Experiment saved at ${saved.patch}; candidate restored.`); })}>Keep experiment as patch and restore candidate</button>}
      <button className="btn" disabled={busy || session.modified} onClick={() => void run(() => bisects.finish(session.id))}>End bisect</button>
      <pre className="bisect-evidence">{session.said}</pre>
      <BisectGraph session={session} />
      <details><summary>Remaining commits</summary><pre>{session.commits}</pre></details>
      <ol>{session.history.map((entry, i) => <li key={i}><span className="mono">{entry.candidate.slice(0, 12)}</span> · {entry.verdict}</li>)}</ol>
    </section>)}
  </details>;
}

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
