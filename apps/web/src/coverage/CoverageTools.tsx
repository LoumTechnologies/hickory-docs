import { PerformanceTools } from "../performance/PerformanceTools";
import { createPortal } from "react-dom";
import { useCallback, useEffect, useId, useRef, useState } from "react";
import { coverageCall } from "./client";
import type { CoverageComparison, CoverageDiscovery, CoverageRun } from "./types";
import {coverageSourceNavigation,coverageLanguage,type CoverageSurfaceClient} from "./CoverageSurface";
import { selectCoverage } from "./selection";
import { openLocation } from "../lib/revealLine";
import { useUiCommands, useUiRevision } from "../lib/uiActionBridge";
import "./coverage.css";

export function CoverageTools({ path }: { path?: string | null }) {
  const [open, setOpen] = useState(false);
  const id = `coverage:${useId()}`, revision = useUiRevision(() => [open, path]);
  const invoke = useUiCommands([{ key: id, command: { subject: id, parent: id, label: open ? "Close test and code coverage tools" : "Open test and code coverage tools", parameters: {}, prerequisites: [], read: () => ({ revision: revision(), enabled: true, value: { open, path: path ?? null } }), execute: async () => { setOpen(value => !value); return { status: "completed", value: { panel_open: !open } }; } } }]);
  return <><PerformanceTools path={path} /><button className="btn" onClick={() => { void invoke(id); }}>Tests &amp; coverage</button>{open && createPortal(<CoveragePanel initialPath={path ?? "."} close={() => setOpen(false)} />, document.body)}</>;
}
const browserSurface:CoverageSurfaceClient={call:coverageCall,select:selectCoverage,navigate:location=>openLocation(location.path,location.line??1)};
export function CoveragePanel({ initialPath = ".", initialLanguage="python", initialSelectedId, close, client=browserSurface, embedded=false }: { initialPath?: string;initialLanguage?:string;initialSelectedId?:string|null; close: (navigated?: boolean) => void;client?:CoverageSurfaceClient;embedded?:boolean }) {
  const coverageCall=client.call,selectCoverage=client.select,retireNavigation=client.retireNavigation;
  const live=useRef(true),ticket=useRef(0),selectedId=useRef<string|null>(null),listPending=useRef<Promise<void>|null>(null),listTicket=useRef(0);
  const current=useCallback((id:number)=>live.current&&ticket.current===id,[]);
  useEffect(()=>{live.current=true;return()=>{live.current=false;ticket.current++;listTicket.current++;listPending.current=null;};},[]);
  const closeSurface=(navigated=false)=>{ticket.current++;close(navigated);};
  const [path, setPath] = useState(initialPath), [language, setLanguage] = useState(()=>coverageLanguage(initialLanguage));
  const [runs, setRuns] = useState<CoverageRun[]>([]), [selected, setSelected] = useState<CoverageRun | null>(null);
  const [group, setGroup] = useState(""), [label, setLabel] = useState(""), [filter, setFilter] = useState("");
  const [url, setUrl] = useState("http://127.0.0.1:9222"), [page, setPage] = useState("");
  const [profile, setProfile] = useState(""), [error, setError] = useState(""), [pending, setPending] = useState(false);
  const [discovered, setDiscovered] = useState<CoverageRun["tests"]>([]), [config, setConfig] = useState<CoverageRun["metadata"] | null>(null);
  const [discovery,setDiscovery]=useState<CoverageDiscovery|null>(null);
  const [other, setOther] = useState(""), [metric, setMetric] = useState("line"), [comparison, setComparison] = useState<CoverageComparison | null>(null);
  const [format, setFormat] = useState("lcov"), [report, setReport] = useState(""), [identity, setIdentity] = useState("");
  const [verdict, setVerdict] = useState("partial"), [notes, setNotes] = useState("");
  const [limit, setLimit] = useState(100);
  const refresh = useCallback(():Promise<void>=>{
    if(listPending.current)return listPending.current;
    const id=++listTicket.current;
    const request=coverageCall<{runs:CoverageRun[]}>({action:"list"}).then(value=>{if(live.current&&listTicket.current===id)setRuns(value.runs);}).catch(error=>{if(live.current&&listTicket.current===id)setError(String(error));}).finally(()=>{if(listTicket.current===id)listPending.current=null;});
    listPending.current=request;return request;
  },[coverageCall]);
  useEffect(()=>{void refresh().catch(error=>{if(live.current)setError(String(error));});},[refresh]);
  // Watch only the product's own collector records; never CI or post-commit checks.
  useEffect(()=>{if(!runs.some(run=>run.status==="running"))return;const timer=setInterval(()=>{void refresh().catch(error=>{if(live.current)setError(String(error));});},2000);return()=>clearInterval(timer);},[runs,refresh]);
  const inputs=useRef("");inputs.current=path+"\0"+language;
  const install=(run:CoverageRun)=>{setSelected(run);selectedId.current=run.id;selectCoverage(run);setNotes(run.notes);setConfig(run.metadata);};
  const act=async(action:string,extra:Record<string,unknown>={})=>{
    retireNavigation?.();
    const id=++ticket.current;setPending(true);setError("");setComparison(null);
    try {
      const value=await coverageCall<CoverageRun>({action,path,language,...(group?{group}:{}),labels:label.split(",").map(value=>value.trim()).filter(Boolean),...(selected?{id:["stop","checkpoint","browser-checkpoint"].includes(action)?selected.parent??selected.id:selected.id}:{}),...extra});
      if(value.id&&value.status!=="stopping") {const run=await coverageCall<CoverageRun>({action:"get",id:value.id});if(current(id))install(run);}
      if(live.current)await refresh();
    }catch(error){if(current(id))setError(String(error));}finally{if(current(id))setPending(false);}
  };
  const choose=useCallback(async(id:string)=>{
    retireNavigation?.();
    const request=++ticket.current;setPending(true);setError("");
    try {const run=await coverageCall<CoverageRun>({action:"get",id});if(current(request)){setSelected(run);selectedId.current=run.id;setPath(run.project||".");setLanguage(coverageLanguage(run.language));setGroup(run.group??"");setConfig(run.metadata);setNotes(run.notes);selectCoverage(run);setComparison(null);}}
    catch(error){if(current(request))setError(String(error));}finally{if(current(request))setPending(false);}
  },[coverageCall,selectCoverage,retireNavigation,current]);
  useEffect(()=>{if(initialSelectedId&&initialSelectedId!==selectedId.current)void choose(initialSelectedId);},[initialSelectedId,choose]);
  // List rows omit report bodies. Refresh the selected body only when its observed
  // status/source state transitions, and never borrow a newer user operation.
  useEffect(()=>{
    const id=selected?.id,summary=runs.find(run=>run.id===id);
    if(!id||!summary||summary.status===selected?.status&&summary.source_state===selected?.source_state)return;
    const request=ticket.current;let active=true;
    void coverageCall<CoverageRun>({action:"get",id}).then(run=>{
      if(active&&current(request)&&selectedId.current===id){setSelected(run);selectCoverage(run);}
    }).catch(error=>{if(active&&current(request)&&selectedId.current===id)setError(String(error));});
    return()=>{active=false;};
  },[runs,selected?.id,selected?.status,selected?.source_state,coverageCall,current,selectCoverage]);
  const inspect=async()=>{
    const id=++ticket.current,basis=inputs.current;setPending(true);setError("");
    try {const value=await coverageCall<{metadata:CoverageRun["metadata"]}>({action:"inspect",path,language});if(current(id)&&inputs.current===basis)setConfig(value.metadata);}
    catch(error){if(current(id))setError(String(error));}finally{if(current(id))setPending(false);}
  };
  const discover=async()=>{
    retireNavigation?.();
    const id=++ticket.current,basis=inputs.current;setPending(true);setError("");setComparison(null);
    try {const value=await coverageCall<CoverageDiscovery>({action:"discover",path,language});if(current(id)&&inputs.current===basis){setDiscovered(value.tests);setConfig(value.metadata);setDiscovery(value.version===1?value:null);setSelected(null);selectedId.current=null;selectCoverage(null);}}
    catch(error){if(current(id))setError(String(error));}finally{if(current(id))setPending(false);}
  };
  const comparisonBasis=useRef("");comparisonBasis.current=JSON.stringify([selected?.id,other,metric]);
  const compare=async()=>{
    const id=++ticket.current,basis=comparisonBasis.current;setPending(true);setError("");setComparison(null);
    try {const value=await coverageCall<CoverageComparison>({action:"compare",id:selected?.id,other,metric});if(current(id)&&basis===comparisonBasis.current)setComparison(value);}
    catch(error){if(current(id))setError(String(error));}finally{if(current(id))setPending(false);}
  };
  const supported=["python","csharp","typescript","rust"].includes(language);
  const active = selected && ["running", "collecting"].includes(runs.find(r => r.id === (selected.parent ?? selected.id))?.status ?? selected.status);
  const files = Object.entries(selected?.evidence?.report.files ?? {});
  const rows = files.flatMap(([file, data]) => Object.entries(data.lines).map(([n, count]) => ({ file, line: +n, count, countsAvailable: data.line_counts })));
  const filtered = rows.filter(row => !filter || `${row.file}:${row.line} ${row.count ? "covered" : "uncovered"}`.includes(filter));
  const visibleRuns = runs.filter(r => !label || label.split(",").some(l => [...r.labels, ...(r.evidence?.labels ?? [])].includes(l.trim())));
  const results = selected?.tests ?? discovered;
  const navigate=async(file:string,line:number|null)=>{
    const id=++ticket.current;setPending(true);setError("");
    try {
      const revealed=await client.navigate(coverageSourceNavigation(selected,file,line,discovery),()=>current(id));
      if(revealed!==false&&current(id))closeSurface(true);
    }catch(error){if(current(id))setError(String(error));}finally{if(current(id))setPending(false);}
  };
  return <div className={`coverage-backdrop${embedded?" coverage-embedded":""}`}><section className="coverage-panel" role="dialog" aria-label="Tests and code coverage" aria-modal={!embedded}>
    <header><h2>Tests &amp; code coverage</h2><button className="btn" onClick={()=>closeSurface()} aria-label="Close coverage">Close</button></header>
    <p>Coverage measures executed code. Test results and manual workflow verdicts are separate. Uninstrumented or missing data is unknown. Runner discovery may build or import the saved project; it does not inspect unsaved drafts.</p>
    <div className="coverage-controls">
      <label>Project or source file<input value={path} onChange={e => {setPath(e.target.value);setConfig(null);setGroup("");setDiscovery(null);setDiscovered([]);}} /></label>
      <label>Language<select value={language} onChange={e => {setLanguage(e.target.value);setConfig(null);setGroup("");setDiscovery(null);setDiscovered([]);}}>{!supported&&<option value={language}>{language} (collector unavailable)</option>}{["python", "csharp", "typescript", "rust"].map(s => <option key={s}>{s}</option>)}</select></label>
      <label>Group<select value={group} onChange={e => setGroup(e.target.value)}><option value="">All project tests</option>{Object.keys(config?.groups ?? {}).map(g => <option key={g}>{g}</option>)}</select></label>
      <label>Labels / history filter (comma separated)<input value={label} onChange={e => setLabel(e.target.value)} placeholder="happy-path, boundary" /></label>
      <button className="btn" disabled={pending} onClick={() => { void inspect(); }}>Read project groups and profiles</button>
      <button className="btn" disabled={pending||!supported} onClick={() => { void discover(); }}>Discover runner cases</button>
      <button className="btn" disabled={pending||!supported} onClick={() => { void act("run"); }}>Run saved project with coverage</button>
      <button className="btn" disabled={pending || !selected?.tests.some(t => t.status === "failed")} onClick={() => { void act("rerun-failed"); }}>Rerun failed tests from selected run</button>
    </div>
    <details><summary>Human or AI manual testing</summary>
      <p>Start an instrumented app, exercise it yourself or with an AI, then collect checkpoints. Browser counters use the start-time baseline; reloads or counter resets require a new session.</p>
      <div className="coverage-controls">
        <label>Chrome CDP URL<input value={url} onChange={e => setUrl(e.target.value)} /></label>
        <label>Page URL contains<input value={page} onChange={e => setPage(e.target.value)} placeholder="localhost:3000" /></label>
        <button className="btn" disabled={pending} onClick={() => { void act("manual-start", { url, selector: page || undefined }); }}>Start browser coverage</button>
        <button className="btn" disabled={pending || !active || selected?.argv.length !== 0} onClick={() => { void act("browser-checkpoint", { url, selector: page || undefined }); }}>Capture browser checkpoint</button>
        <label>Native app profile<input value={profile} list="coverage-profiles" onChange={e => setProfile(e.target.value)} placeholder=".hick-tests.json profile" /><datalist id="coverage-profiles">{Object.keys(config?.manual ?? {}).map(p => <option key={p} value={p} />)}</datalist></label>
        <button className="btn" disabled={pending || !profile} onClick={() => { void act("manual-start", { profile }); }}>Start instrumented app</button>
        <button className="btn" disabled={pending || !active || !selected?.argv.length} onClick={() => { void act("checkpoint"); }}>Capture app checkpoint</button>
        <button className="btn" disabled={pending || !active} onClick={() => { void act("stop"); }}>Stop collection</button>
      </div>
      <p>Native profiles supply collector launch, checkpoint and finish commands. Missing collector output is reported as unavailable. AI agents can use the <code>code_coverage</code> tool for the same operations.</p>
    </details>
    <details><summary>Import a standard report</summary><div className="coverage-controls">
      <label>Format<select value={format} onChange={e => setFormat(e.target.value)}>{["lcov", "coverage-py", "cobertura", "istanbul"].map(s => <option key={s}>{s}</option>)}</select></label>
      <label>Report file<input type="file" accept=".json,.xml,.info,.lcov" onChange={e => { const file = e.target.files?.[0]; if (file) void file.text().then(setReport); }} /></label>
      <label>Report contents<textarea value={report} onChange={event=>setReport(event.target.value)} /></label>
      <label>Original identity JSON (optional)<textarea value={identity} onChange={e => setIdentity(e.target.value)} placeholder="Without original source hashes this report remains unknown." /></label>
      <button className="btn" disabled={pending || !report} onClick={() => { try { void act("import", { format, raw: report, ...(identity ? { identity: JSON.parse(identity) as unknown } : {}) }); } catch (e) { setError(String(e)); } }}>Import report</button>
    </div></details>
    {error && <p className="error" role="alert">{error}</p>}
    <div className="coverage-columns"><nav aria-label="Coverage run history"><h3>Runs and checkpoints</h3><button className="btn" disabled={pending} onClick={() => { void refresh().catch(e => setError(String(e))); }}>Refresh runs</button>
      {visibleRuns.length === 0 && <p>No recorded evidence.</p>}{visibleRuns.map(r => <button key={r.id} className="coverage-run" aria-pressed={selected?.id === r.id} onClick={() => { void choose(r.id); }}>{new Date(r.started_ms).toLocaleString()} · {r.mode} · {r.status}<br />{r.source_state} · {r.evidence?.labels.join(", ") || r.labels.join(", ") || "unlabeled"}<br /><small>{r.id}</small></button>)}
    </nav><main>
      {selected && <><h3>{selected.mode} · {selected.status} · {selected.source_state} source</h3><p>Run <code>{selected.id}</code> · {selected.identity.collector} · scope: {selected.project || "."} · attribution: {selected.evidence?.attribution ?? "unavailable"}</p>
        {selected.error && <p role="alert" className="error">{selected.error}</p>}
        <button className="btn" onClick={() => {ticket.current++;setPending(false);selectedId.current=null;selectCoverage(null);}}>Clear editor coverage</button>
        <button className="btn" disabled={pending} onClick={() => { void choose(selected.id); }}>Reload selected run</button>
        {selected.evidence ? <p><strong>{rows.filter(r => r.count > 0).length} / {rows.length} executable lines covered</strong>. {files.some(([, f]) => f.branches !== null) ? `${files.reduce((n, [, f]) => n + (f.branches?.filter(b => b.covered).length ?? 0), 0)} / ${files.reduce((n, [, f]) => n + (f.branches?.length ?? 0), 0)} identified branches covered in files with branch data.` : "Identified branch coverage unavailable; provider aggregates, if any, remain in the raw report."}</p> : <p>Coverage unavailable. This is not zero coverage.</p>}
        <div className="coverage-controls"><label>Compare with<select value={other} onChange={e => {setOther(e.target.value);setComparison(null);}}><option value="">Choose another run</option>{runs.filter(r => r.id !== selected.id).map(r => <option key={r.id} value={r.id}>{r.id.slice(0, 8)} · {r.source_state} · {r.evidence?.labels.join(", ") || r.labels.join(", ")}</option>)}</select></label><label>Metric<select value={metric} onChange={e => {setMetric(e.target.value);setComparison(null);}}><option value="line">Line</option><option value="branch">Identified branch</option></select></label><button className="btn" disabled={pending || !other} onClick={() => { void compare(); }}>Compare compatible groups</button></div>
        {comparison && <div role="status"><p>{selected.source_state !== "current" || runs.find(r => r.id === other)?.source_state !== "current" ? "Historical comparison of compatible recorded sources. " : ""}{comparison.denominator} measured {metric} locations: {comparison.shared.length} shared, {comparison.left_only.length} only selected, {comparison.right_only.length} only other, {comparison.missed.length} missed by both.</p>{(["shared", "left_only", "right_only", "missed"] as const).map(key => <details key={key}><summary>{key.replaceAll("_", " ")} ({comparison[key].length})</summary><pre>{comparison[key].join("\n")}</pre></details>)}</div>}
        <label>Filter executable locations<input value={filter} onChange={e => { setFilter(e.target.value); setLimit(100); }} placeholder="file, covered, uncovered" /></label>
        <table><thead><tr><th>Source / document</th><th>Line</th><th>Count</th><th>State</th></tr></thead><tbody>{filtered.slice(0, limit).map(row => <tr key={`${row.file}:${row.line}`}><td><button className="coverage-link" onClick={() => navigate(row.file, row.line)}>{row.file}{selected.mappings[row.file]?.[row.line] ? " → document" : ""}</button></td><td>{row.line}</td><td>{row.countsAvailable ? row.count : "unavailable"}</td><td>{row.count ? "covered" : "uncovered executable"}</td></tr>)}</tbody></table>
        {filtered.length > limit && <button className="btn" onClick={() => setLimit(n => n + 100)}>Show more ({filtered.length - limit})</button>}
        <details><summary>Branches and excluded lines</summary>{files.map(([file, data]) => <div key={file}><strong>{file}</strong><p>{Object.keys(data.branch_totals).length ? <>Aggregate branches: {Object.values(data.branch_totals).reduce((n, v) => n + v[0], 0)} / {Object.values(data.branch_totals).reduce((n, v) => n + v[1], 0)} reported; individual outcome attribution unavailable.</> : "Aggregate branch coverage unavailable."}</p><p>Excluded: {data.excluded.join(", ") || "none reported"}</p>{data.branches === null ? <p>Identified branches unavailable</p> : data.branches.map(b => <div key={b.id}><button className="coverage-link" onClick={() => navigate(file, b.line)}>Line {b.line} · branch {b.id}</button>: {b.covered ? "covered" : "uncovered"} · count {b.count ?? "unavailable"}</div>)}</div>)}</details>
        <details><summary>Collector output and original report</summary><pre>{selected.output || "No runner output"}</pre><pre>{selected.evidence?.report.raw}</pre><pre>{JSON.stringify(selected.identity, null, 2)}</pre></details>
        {selected.mode === "manual" && <div className="coverage-controls"><label>Manual workflow verdict<select value={verdict} onChange={e => setVerdict(e.target.value)}>{["passed", "failed", "partial", "blocked", "untested"].map(v => <option key={v}>{v}</option>)}</select></label><label>Observation / prerequisites<textarea value={notes} onChange={e => setNotes(e.target.value)} /></label><button className="btn" disabled={pending || !!active} onClick={() => { void act("annotate", { verdict, notes }); }}>Record manual verdict</button><p>Recorded verdict: {selected.manual_verdict ?? "none"}</p></div>}
      </>}
      <details open><summary>Test discovery and results ({results.length})</summary>
        {!selected&&discovered.length>0&&<p>{discovery?`Runner listing from ${discovery.project||"."} · ${discovery.source_state} saved source. Files were not frozen.`:"Unversioned runner listing. Exact source positions unavailable."}</p>}
        {results.length===0&&<p>No individual test results available.</p>}{results.map((test,index)=>{const location=test.path?coverageSourceNavigation(selected,test.path,test.line,discovery):null;return <details key={`${test.id}:${index}`}><summary>{test.status} · {test.runner_name==null?test.id:test.runner_name.trim()?test.runner_name:JSON.stringify(test.runner_name)}{test.runner_name!=null&&test.path?` · ${test.path}`:""} · {test.labels.join(", ")}</summary>{test.path&&<button className="coverage-link" onClick={()=>{void navigate(test.path!,test.line);}}>{location?.line?`Open source line ${location.line}`:"Open file (position unavailable)"}</button>}<pre>{test.output||"No failure output"}</pre></details>;})}</details>
    </main></div>
  </section></div>;
}
