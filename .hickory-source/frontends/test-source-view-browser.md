<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Open source tests without moving code on browser

For maintainers changing test interaction independently of parsing and execution.

Retain the observation and test-run identities independently of visibility. Refresh is read-only. A request ticket and current source snapshot reject obsolete responses; another read cannot install facts after source or workspace replacement.

<ui:copy id="source-test-view-browser-0">
import {useEffect,useRef,useState,useSyncExternalStore} from "react";
import {createPortal} from "react-dom";
import type { Buffers } from "./buffers";
import {api} from "../api/client";
import type {SourceTestScan,SourceTestCandidate,TestSourcePoint,SourceTestRunResult} from "../api/testSources";
import {candidatePoint,sourceTestRun,testSourceCurrent,type TestSourceSnapshot} from "./TestSources";
type Run={id:string;name:string;scope:string;state:string;exit:number|null;output:string;sourceHash:string;fingerprint:string;contributors:Record<string,string>};
export function TestSourcesView({visible,close,current,navigate,terminal,buffers}:{buffers:Buffers;visible:boolean;close:()=>void;current:(entry:string)=>TestSourceSnapshot|null;navigate:(point:TestSourcePoint)=>Promise<void>;terminal:(id:string)=>void}) {
  const [entry,setEntry]=useState(""),[query,setQuery]=useState(""),[page,setPage]=useState(0);
  const [observation,setObservation]=useState<{snapshot:TestSourceSnapshot;facts:SourceTestScan}|null>(null);
  const [reading,setReading]=useState(false),[running,setRunning]=useState(false),[notice,setNotice]=useState("");
  const [runs,setRuns]=useState<Run[]>([]),[selected,setSelected]=useState<string|null>(null);
  const outputTickets=useRef(new Map<string,number>());
  const ticket=useRef(0),busy=useRef(false),latest=useRef(current);latest.current=current;const panel=useRef<HTMLDivElement>(null);
  useEffect(()=>{if(visible)panel.current?.focus();},[visible]);
  useEffect(()=>()=>{ticket.current++;},[]);
  const now=current(entry.trim()),valid=!!observation&&!!now&&testSourceCurrent(observation.snapshot,observation.facts,now);
  const observedPaths=[...new Set([...(now?[now.request.path]:[]),...Object.keys(observation?.facts.source_hashes??{})])];
  useSyncExternalStore(listener=>{const off=observedPaths.flatMap(path=>{const buffer=buffers.values.get(path);return buffer?[buffer.subscribe(listener)]:[];});return()=>off.forEach(stop=>stop());},()=>observedPaths.map(path=>buffers.values.get(path)?.snapshot()??-1).join(":"));
  async function read() {
    const snapshot=latest.current(entry.trim());if(!snapshot){setNotice("Open a source file first.");return;}
    const id=++ticket.current;setReading(true);setObservation(null);setNotice("");
    try {const facts=await api.sourceTestCandidates(snapshot.request);if(id!==ticket.current)return;
      const next=latest.current(entry.trim());if(!next||!testSourceCurrent(snapshot,facts,next)){setNotice("Source changed during inspection. Refresh candidates.");return;}
      setObservation({snapshot,facts});setPage(0);
    }catch(error){if(id===ticket.current)setNotice(String(error));}finally{if(id===ticket.current)setReading(false);}
  }
</ui:copy>

Execution requires the model to construct an explicit saved-source request. Prevent overlapping run clicks, retain the returned terminal identity, and refresh only a user-selected test run. Exit state and terminal output remain distinct from structured per-case results.

<ui:copy id="source-test-view-browser-1">
  async function run(candidate:SourceTestCandidate,selection:"name_filter"|"containing_scope") {
    if(busy.current||!observation)return;const snapshot=latest.current(entry.trim());if(!snapshot)return;
    busy.current=true;setRunning(true);setNotice("");
    try {const request=sourceTestRun(observation.snapshot,observation.facts,snapshot,candidate.id,selection);
      const reply:SourceTestRunResult=await api.runSourceTest(request);
      setRuns(items=>[{id:reply.id,name:candidate.name,scope:selection==="name_filter"?"name filter":candidate.run_scope,state:reply.state,exit:reply.exit_code??null,output:"",sourceHash:reply.source_hash,fingerprint:reply.analysis_fingerprint,contributors:reply.checked_sources},...items]);setSelected(reply.id);
      setNotice("Run admitted against saved bytes. Files remain mutable while it executes.");
    }catch(error){setNotice(String(error));}finally{busy.current=false;setRunning(false);}
  }
  async function refreshRun(id:string) {
    const read=(outputTickets.current.get(id)??0)+1;outputTickets.current.set(id,read);
    try {const [sessions,screen]=await Promise.all([api.terminals(),api.terminalScreen(id,0,"")]);const session=sessions.sessions.find(item=>item.id===id);if(outputTickets.current.get(id)!==read)return;
      setRuns(items=>items.map(item=>item.id!==id?item:{...item,state:session?.state??"session unavailable",exit:session?.exit_code??null,output:new TextDecoder().decode(new Uint8Array(screen.bytes))+(screen.next===null?"":"\n[Screen page truncated. Open terminal for retained output.]")}));
    }catch(error){if(outputTickets.current.get(id)===read)setNotice(String(error));}
  }
</ui:copy>

Render paged candidates, explicit name-filter/containing-scope choices, partial/unsupported notices and retained output. The floating surface leaves editor geometry unchanged. Closing it does not stop a runner or discard the observed terminal identity. No automatic runner discovery or result-status inference is hidden in rendering.

<ui:copy id="source-test-view-browser-2">
  const candidates=(observation?.facts.candidates??[]).filter(item=>`${item.name} ${item.target.path}`.toLowerCase().includes(query.toLowerCase()));
  const last=Math.max(0,Math.ceil(candidates.length/100)-1),index=Math.min(page,last),record=runs.find(item=>item.id===selected);
  if(!visible)return null;
  return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Source tests" onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();close();}}}>
    <button aria-label="Close source tests" onClick={close}>×</button><strong>Source tests</strong>
    <p>Source declarations are candidates. Name filters may match several cases; parameterized cases need runner discovery. Results below are terminal runs, not per-case verification.</p>
    <p>{now?.request.path??"No source selected"}</p>
    <label>Construction entry (optional)<input value={entry} onChange={event=>{ticket.current++;setReading(false);setEntry(event.target.value);}} placeholder="project.md"/></label>
    <button disabled={reading||!now} onClick={()=>{void read();}}>{reading?"Reading source…":"Refresh candidates"}</button>
    {observation&&<p>{valid?"Source observation":"Stale observation — refresh before running"} · {observation.facts.scope} · {observation.facts.syntax_complete?"parsed":"partial syntax"}</p>}
    {notice&&<p role="status">{notice}</p>}
    {observation&&<p>{[...observation.facts.notices,...observation.facts.unsupported_languages.map(item=>`No source detector: ${item}`),...observation.facts.unclassified_outputs.map(item=>`Unclassified output: ${item}`)].join(" · ")}</p>}
    <input aria-label="Filter source tests" value={query} onChange={event=>{setQuery(event.target.value);setPage(0);}}/>
    {candidates.slice(index*100,(index+1)*100).map(item=><div key={item.id}><strong>{item.name}</strong> · {item.confidence}<p>{item.target.path}:{item.target.line}</p><button disabled={!valid} onClick={()=>{void navigate(candidatePoint(item)).catch(error=>setNotice(String(error)));}}>{item.source?"Open authored source":"Open runtime source"} {candidatePoint(item).line}</button><button disabled={!valid||running||item.name_filter===null} onClick={()=>{void run(item,"name_filter");}}>Run name filter</button><button disabled={!valid||running} onClick={()=>{void run(item,"containing_scope");}}>Run {item.run_scope.replaceAll("_"," ")} scope</button></div>)}
    {observation&&candidates.length===0&&<p>No source candidates returned. This does not prove the project has no tests.</p>}
    {last>0&&<p><button disabled={!index} onClick={()=>setPage(index-1)}>Previous</button>{index+1}/{last+1}<button disabled={index===last} onClick={()=>setPage(index+1)}>Next</button></p>}
    <details open><summary>Retained test runs ({runs.length})</summary>{runs.map(item=><button key={item.id} aria-pressed={selected===item.id} onClick={()=>{setSelected(item.id);void refreshRun(item.id);}}>{item.name} · {item.scope} · {item.state}{item.exit===null?"":` · exit ${item.exit}`}</button>)}{record&&<div><button onClick={()=>{void refreshRun(record.id);}}>Refresh this run</button><button onClick={()=>terminal(record.id)}>Open terminal output</button><details><summary>Saved revisions checked at admission</summary><p>Files were not frozen during execution. These revisions identify this run, not current verification.</p><code>{record.fingerprint}</code><pre>{JSON.stringify({target:record.sourceHash,contributors:record.contributors},null,2)}</pre></details><pre>{record.output||"Select or refresh this run to read its output."}</pre></div>}</details>
  </section>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/TestSourcesView.tsx"><ui:paste select="#source-test-view-browser-0"/><ui:paste select="#source-test-view-browser-1"/><ui:paste select="#source-test-view-browser-2"/></ui:file>
</ui:doc>
