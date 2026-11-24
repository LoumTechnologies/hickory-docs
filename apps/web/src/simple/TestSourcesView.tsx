import {useEffect,useRef,useState,useSyncExternalStore} from "react";
import {createPortal} from "react-dom";
import type {SourceTestInspection} from "./SourceTestGutters";
import {editorForBuffer} from "./Document";
import type {Buffer} from "./buffers";
import {readPlainDebugBuffer,readPlainDebugOwner,subscribePlainDebugSessions,plainDebugSessionsSnapshot} from "../debug/plainDebugHosts";
import type { Buffers } from "./buffers";
import {request as engineRequest} from "../api/client";
import {api} from "../api/client";
import type {SourceTestScan,SourceTestCandidate,TestSourcePoint,SourceTestRunResult} from "../api/testSources";
import {candidatePoint,sourceTestRun,testSourceCurrent,sourceTestDebugChoice,prepareSourceTestDebug,parseSourceTestDebugJson,type PreparedSourceTestDebug,type SourceTestDebugProcess,type TestSourceSnapshot} from "./TestSources";
type Run={id:string;name:string;scope:string;state:string;exit:number|null;output:string;sourceHash:string;fingerprint:string;contributors:Record<string,string>};
export function TestSourcesView({visible,close,current,navigate,terminal,buffers,inspection,inspectDebugger}:{inspection:SourceTestInspection|null;buffers:Buffers;visible:boolean;close:()=>void;current:(entry:string)=>TestSourceSnapshot|null;navigate:(point:TestSourcePoint)=>Promise<void>;terminal:(id:string)=>void;inspectDebugger:(owner:Buffer)=>void}) {
  const [entry,setEntry]=useState(""),[query,setQuery]=useState(""),[page,setPage]=useState(0);
  const [observation,setObservation]=useState<{snapshot:TestSourceSnapshot;facts:SourceTestScan}|null>(null);
  const [reading,setReading]=useState(false),[running,setRunning]=useState(false),[notice,setNotice]=useState("");
  const [runs,setRuns]=useState<Run[]>([]),[selected,setSelected]=useState<string|null>(null);
  type DebugRow={key:string;owner:Buffer;host:string;record:SourceTestDebugProcess};
  const [debugs,setDebugs]=useState<DebugRow[]>([]),[selectedDebug,setSelectedDebug]=useState<string|null>(null);
  const debugTicket=useRef(0),debugOperation=useRef<number|null>(null),debugAbort=useRef<(()=>void)|null>(null),shown=useRef(visible);shown.current=visible;
  const ownerObservation=useRef(new Map<string,Buffer>()),observed=useRef(observation);observed.current=observation;
  const hostRevision=useSyncExternalStore(subscribePlainDebugSessions,plainDebugSessionsSnapshot,plainDebugSessionsSnapshot);
  function invalidateDebug(){debugTicket.current++;debugAbort.current?.();debugAbort.current=null;if(debugOperation.current!==null){debugOperation.current=null;busy.current=false;setRunning(false);}}
  function dismiss(){invalidateDebug();close();}
  useEffect(()=>{if(!visible)invalidateDebug();},[visible]);
  useEffect(()=>()=>{debugAbort.current?.();debugTicket.current++;},[]);
  useEffect(()=>{setDebugs(items=>{let changed=false;const next=items.map(item=>{const current=readPlainDebugOwner(item.host)?.sourceTestProcesses?.().find(record=>record.session===item.record.session);if(current&&(current.phase!==item.record.phase||current.exitCode!==item.record.exitCode||current.message!==item.record.message)){changed=true;return {...item,record:current};}return item;});return changed?next:items;});},[hostRevision,debugs.length]);
  const outputTickets=useRef(new Map<string,number>());
  const ticket=useRef(0),busy=useRef(false),latest=useRef(current);latest.current=current;const panel=useRef<HTMLDivElement>(null);
  useEffect(()=>{if(visible)panel.current?.focus();},[visible]);
  useEffect(()=>()=>{ticket.current++;},[]);
  const now=current(entry.trim()),valid=!!observation&&!!now&&testSourceCurrent(observation.snapshot,observation.facts,now);
  const observedPaths=[...new Set([...(now?[now.request.path]:[]),...Object.keys(observation?.facts.source_hashes??{})])];
  useSyncExternalStore(listener=>{const off=observedPaths.flatMap(path=>{const buffer=buffers.values.get(path);return buffer?[buffer.subscribe(listener)]:[];});return()=>off.forEach(stop=>stop());},()=>observedPaths.map(path=>buffers.values.get(path)?.snapshot()??-1).join(":"));
  async function read() {
    invalidateDebug();const owners=new Map(buffers.values);const snapshot=latest.current(entry.trim());if(!snapshot){setNotice("Open a source file first.");return;}
    const id=++ticket.current;setReading(true);setObservation(null);setNotice("");
    try {const facts=await api.sourceTestCandidates(snapshot.request);if(id!==ticket.current)return;
      const next=latest.current(entry.trim());if(!next||!testSourceCurrent(snapshot,facts,next)){setNotice("Source changed during inspection. Refresh candidates.");return;}
      ownerObservation.current=owners;setObservation({snapshot,facts});setPage(0);
    }catch(error){if(id===ticket.current)setNotice(String(error));}finally{if(id===ticket.current)setReading(false);}
  }
  const inspected=useRef<SourceTestInspection|null>(null);
  useEffect(()=>{if(!visible||!inspection||inspection===inspected.current||now?.request.path!==inspection.path||!!inspection.owner&&(buffers.values.get(inspection.path)!==inspection.owner||inspection.owner.sourceRevision!==inspection.revision))return;inspected.current=inspection;setQuery(inspection.name);void read();},[visible,inspection,now?.request.path]);
  async function run(candidate:SourceTestCandidate,selection:"name_filter"|"containing_scope") {
    if(busy.current||!observation)return;const snapshot=latest.current(entry.trim());if(!snapshot)return;
    busy.current=true;setRunning(true);setNotice("");
    try {const request=sourceTestRun(observation.snapshot,observation.facts,snapshot,candidate.id,selection);
      const reply:SourceTestRunResult=await api.runSourceTest(request);
      setRuns(items=>[{id:reply.id,name:candidate.name,scope:selection==="name_filter"?"name filter":candidate.run_scope,state:reply.state,exit:reply.exit_code??null,output:"",sourceHash:reply.source_hash,fingerprint:reply.analysis_fingerprint,contributors:reply.checked_sources},...items]);setSelected(reply.id);
      setNotice("Run admitted against saved bytes. Files remain mutable while it executes.");
    }catch(error){setNotice(String(error));}finally{busy.current=false;setRunning(false);}
  }
  async function debug(candidate:SourceTestCandidate,selection:'name_filter'|'containing_scope'){
    if(busy.current||!observation)return;const snapshot=latest.current(entry.trim()),owner=snapshot&&((snapshot.owner as Buffer|undefined)??buffers.values.get(snapshot.request.path));if(!snapshot||!owner){setNotice('Retained source editor unavailable. Reopen it and refresh candidates.');return;}
    const held=observation,owners=ownerObservation.current,namespace=buffers.workspace,source=owner.text,sourceRevision=owner.sourceRevision,file=owner.file,sourceEntry=entry.trim(),editor=editorForBuffer(owner),document=editor?.view.state.doc,selectionState=editor?.view.state.selection,interaction=editor?.interaction;
    invalidateDebug();const id=debugTicket.current;debugOperation.current=id;busy.current=true;setRunning(true);setNotice('Preparing saved-source Test Debug…');
    const currentChoice=()=>{const now=latest.current(sourceEntry);return shown.current&&id===debugTicket.current&&observed.current===held&&!!now&&testSourceCurrent(held.snapshot,held.facts,now)&&buffers.workspace===namespace&&!!namespace&&buffers.values.get(file.path)===owner&&owner.file===file&&owner.sourceRevision===sourceRevision&&owner.text===source&&!owner.dirty&&!owner.saving&&!owner.resourceDeleted&&Object.keys(held.facts.source_hashes).every(path=>{const before=owners.get(path),after=buffers.values.get(path);return before===after&&(!after||!after.dirty&&!after.saving&&!after.resourceDeleted);})&&(!editor||editorForBuffer(owner)===editor&&editor.view.state.doc===document&&editor.view.state.selection.eq(selectionState!)&&editor.interaction===interaction);};
    try{
      if(!currentChoice())throw Error('Saved test source, contributors, editor interaction or namespace changed. Refresh candidates before Debug.');
      const choice=sourceTestDebugChoice(held.snapshot,held.facts,snapshot,candidate.id,selection),host=readPlainDebugBuffer(owner);if(!host?.debug.startSourceTest)throw Error('The source debugger is still attaching or does not implement Test Debug. No launch was sent.');
      if(host.debug.isActive?.())throw Error('This source editor already owns an active debugger. Stop it before another Test Debug choice.');
      const plan=await engineRequest<unknown>('POST','/api/tests/debug-plan',choice.request,false,parseSourceTestDebugJson);if(!currentChoice())throw Error('Test Debug choice retired while reading its provider plan. No Start was sent.');
      const prepared:PreparedSourceTestDebug=prepareSourceTestDebug(choice,plan);debugAbort.current=()=>host.debug.cancelSourceTestStart?.(prepared);
      const record=await host.debug.startSourceTest(prepared,currentChoice);
      // Adoption already checked visibility and ownership. Closing immediately
      // afterward must retain the positively bound session and its selection.
      const key=crypto.randomUUID();setDebugs(items=>[{key,owner,host:host.owner,record},...items].slice(0,64));setSelectedDebug(key);
      if(id===debugTicket.current)setNotice([...prepared.plan.notices,'Test Debug admitted. This is a process session; collected test verdicts are unavailable.'].join(' · '));
    }catch(error){if(id===debugTicket.current)setNotice(String(error));}
    finally{if(debugOperation.current===id){debugOperation.current=null;debugAbort.current=null;busy.current=false;setRunning(false);}}
  }
  function openDebug(item:DebugRow){const host=readPlainDebugOwner(item.host);if(!host?.sourceTestInspectable?.(item.record.session)){setNotice('The original debugger view was released or replaced. Its retained process record is not a link to another session.');return;}try{inspectDebugger(item.owner);dismiss();}catch(error){setNotice(String(error));}}
  async function refreshRun(id:string) {
    const read=(outputTickets.current.get(id)??0)+1;outputTickets.current.set(id,read);
    try {const [sessions,screen]=await Promise.all([api.terminals(),api.terminalScreen(id,0,"")]);const session=sessions.sessions.find(item=>item.id===id);if(outputTickets.current.get(id)!==read)return;
      setRuns(items=>items.map(item=>item.id!==id?item:{...item,state:session?.state??"session unavailable",exit:session?.exit_code??null,output:new TextDecoder().decode(new Uint8Array(screen.bytes))+(screen.next===null?"":"\n[Screen page truncated. Open terminal for retained output.]")}));
    }catch(error){if(outputTickets.current.get(id)===read)setNotice(String(error));}
  }
  const candidates=(observation?.facts.candidates??[]).filter(item=>`${item.name} ${item.target.path}`.toLowerCase().includes(query.toLowerCase()));
  const debugInput=now&&buffers.values.get(now.request.path),debugSourceReady=now?.executionCurrent!==false&&(!now?.owner||now.owner===debugInput)&&!!debugInput&&!!buffers.workspace&&debugInput.namespace===buffers.workspace&&!debugInput.dirty&&!debugInput.saving&&!debugInput.resourceDeleted&&ownerObservation.current.get(debugInput.file.path)===debugInput;
  const debugActive=!!debugInput&&!!readPlainDebugBuffer(debugInput)?.debug.isActive?.();
  const last=Math.max(0,Math.ceil(candidates.length/100)-1),index=Math.min(page,last),record=runs.find(item=>item.id===selected);
  if(!visible)return null;
  return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Source tests" onKeyDown={event=>{if(event.key==="Escape"){event.preventDefault();dismiss();}}}>
    <button aria-label="Close source tests" onClick={dismiss}>×</button><strong>Source tests</strong>
    <p>Source declarations are candidates. Name filters may match several cases; parameterized cases need runner discovery. Results below are terminal runs, not per-case verification.</p>
    <p>{now?.request.path??"No source selected"}</p>
    <label>Construction entry (optional)<input value={entry} onChange={event=>{invalidateDebug();ticket.current++;setReading(false);setEntry(event.target.value);}} placeholder="project.md"/></label>
    <button disabled={reading||!now} title={reading ? "Wait for the current reading to load." : !now ? "Open a source file to discover test candidates." : undefined} onClick={()=>{void read();}}>{reading?"Reading source…":"Refresh candidates"}</button>
    {observation&&<p>{valid?"Source observation":"Stale observation — refresh before running"} · {observation.facts.scope} · {observation.facts.syntax_complete?"parsed":"partial syntax"}</p>}
    {now?.executionCurrent===false&&<p role="status">Save this current owned source before Run or Debug. Deleted, saving and unsaved editors remain readable but cannot authorize execution.</p>}
    {notice&&<p role="status">{notice}</p>}
    {observation&&<p>{[...observation.facts.notices,...observation.facts.unsupported_languages.map(item=>`No source detector: ${item}`),...observation.facts.unclassified_outputs.map(item=>`Unclassified output: ${item}`)].join(" · ")}</p>}
    <input aria-label="Filter source tests" value={query} onChange={event=>{invalidateDebug();setQuery(event.target.value);setPage(0);}}/>
    {candidates.slice(index*100,(index+1)*100).map(item=><div key={item.id}><strong>{item.name}</strong> · {item.confidence}<p>{item.target.path}:{item.target.line}</p><button disabled={!valid} title={!valid ? "This candidate is stale. Refresh candidates from current source." : undefined} onClick={()=>{void navigate(candidatePoint(item)).catch(error=>setNotice(String(error)));}}>{item.source?"Open authored source":"Open runtime source"} {candidatePoint(item).line}</button><button disabled={!valid||now?.executionCurrent===false||running||item.name_filter===null||item.confidence==='dynamic'} title={!valid ? "This candidate is stale. Refresh candidates from current source." : now?.executionCurrent===false ? "Save the current source before running tests." : running ? "A test run is in progress." : item.name_filter===null ? "This candidate has no exact name filter. Run its containing scope." : item.confidence==='dynamic' ? "This candidate has a dynamic name. Run its containing scope." : undefined} onClick={()=>{void run(item,"name_filter");}}>Run name filter</button><button disabled={!valid||now?.executionCurrent===false||running} title={!valid ? "This candidate is stale. Refresh candidates from current source." : now?.executionCurrent===false ? "Save the current source before running tests." : running ? "A test run is in progress." : undefined} onClick={()=>{void run(item,"containing_scope");}}>Run {item.run_scope.replaceAll("_"," ")} scope</button><button disabled={!valid||running||!debugSourceReady||debugActive||item.name_filter===null||item.confidence==='dynamic'} title={!valid ? "This candidate is stale. Refresh candidates from current source." : running ? "A test run is in progress." : !debugSourceReady ? "Save the source so the debugger can bind to it." : debugActive ? "Stop the current debugger session first." : item.name_filter===null ? "This candidate has no exact name filter. Run its containing scope." : item.confidence==='dynamic' ? "This candidate has a dynamic name. Run its containing scope." : undefined} onClick={()=>{void debug(item,'name_filter');}}>Debug name filter</button><button disabled={!valid||running||!debugSourceReady||debugActive} title={!valid ? "This candidate is stale. Refresh candidates from current source." : running ? "A test run is in progress." : !debugSourceReady ? "Save the source so the debugger can bind to it." : debugActive ? "Stop the current debugger session first." : undefined} onClick={()=>{void debug(item,'containing_scope');}}>Debug containing {item.run_scope==='file_filter'?'file':item.run_scope.replaceAll('_',' ')} scope</button></div>)}
    {observation&&candidates.length===0&&<p>No source candidates returned. This does not prove the project has no tests.</p>}
    {last>0&&<p><button disabled={!index} title={!index ? "You’re on the first page." : undefined} onClick={()=>setPage(index-1)}>Previous</button>{index+1}/{last+1}<button disabled={index===last} title={index===last ? "You’re on the last page." : undefined} onClick={()=>setPage(index+1)}>Next</button></p>}
    <details open><summary>Retained test debug sessions ({debugs.length})</summary><p>Process state and exit code are not collected test verdicts. No entry breakpoint is inserted; execution continues until your breakpoint or completion.</p>{debugs.map(item=><button key={item.key} aria-pressed={selectedDebug===item.key} onClick={()=>setSelectedDebug(item.key)}>{item.record.choice.candidateName} · {item.record.choice.selection.kind} · {item.record.phase}{item.record.exitCode===null?'':` · process exit ${item.record.exitCode}`}</button>)}{debugs.filter(item=>item.key===selectedDebug).map(item=><div key={item.key}><button onClick={()=>openDebug(item)}>Open this debugger/source</button><p>{item.record.message??'No collected case result is available for this debug process.'}</p><details><summary>Saved selection and revisions checked at admission</summary><pre>{JSON.stringify(item.record.binding,null,2)}</pre></details></div>)}</details>
    <details open><summary>Retained test runs ({runs.length})</summary>{runs.map(item=><button key={item.id} aria-pressed={selected===item.id} onClick={()=>{setSelected(item.id);void refreshRun(item.id);}}>{item.name} · {item.scope} · {item.state}{item.exit===null?"":` · exit ${item.exit}`}</button>)}{record&&<div><button onClick={()=>{void refreshRun(record.id);}}>Refresh this run</button><button onClick={()=>terminal(record.id)}>Open terminal output</button><details><summary>Saved revisions checked at admission</summary><p>Files were not frozen during execution. These revisions identify this run, not current verification.</p><code>{record.fingerprint}</code><pre>{JSON.stringify({target:record.sourceHash,contributors:record.contributors},null,2)}</pre></details><pre>{record.output||"Select or refresh this run to read its output."}</pre></div>}</details>
  </section>,document.body);
}
