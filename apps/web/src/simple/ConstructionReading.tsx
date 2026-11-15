import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import type {SourceTestCandidate} from '../api/testSources';
import {request} from '../api/client';
export interface ConstructionInput {entry:string;sources:Record<string,string>;focus:string|null}
export interface ConstructedSnapshot {hash:string;kind:string;executable:boolean;size:number;preview:string|null;preview_truncated:boolean}
export interface ConstructionOutput {path:string;change:string;before:ConstructedSnapshot|null;after:ConstructedSnapshot|null;contributors:string[];origins_truncated:boolean;origins:{phase:string;path:string;source_bytes:[number,number];output_bytes:[number,number];exact:boolean;source_hash:string|null}[]}
export interface ConstructionReading {version:number;entry:string;focus:string|null;materialized:boolean;output_count:number;outputs_truncated:boolean;outputs:ConstructionOutput[];tests:SourceTestCandidate[];test_count:number;tests_truncated:boolean;test_relationship:string;evidence_status:string;notices:string[];unused_sources:string[]}
export class ConstructionReadingModel {
 value:ConstructionReading|null=null;notice='';loading=false;private ticket=0;
 constructor(private load:(input:ConstructionInput)=>Promise<ConstructionReading>,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(provided:ConstructionInput){const input={...provided,sources:{...provided.sources}};const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading construction…';this.changed();
 try{const value=await this.load(input);if(id!==this.ticket)return;if(value.version!==1||value.entry!==input.entry||value.focus!==input.focus||value.materialized!==false)throw new Error('Construction identity changed or unsupported response. Refresh the reading.');this.value=value;this.notice='Captured construction and draft consequences. Editor changes after capture are not included.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
export function ConstructionReadingView({sources,close}:{sources:()=>Record<string,string>;close:()=>void}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new ConstructionReadingModel(input=>request('POST','/api/construction/reading',input),paint),[]);
 const [entry,setEntry]=useState('project.md'),[focus,setFocus]=useState(''),[selected,setSelected]=useState(''),[phase,setPhase]=useState<'before'|'after'>('after');const panel=useRef<HTMLElement>(null);
 useEffect(()=>{panel.current?.focus();return()=>model.close();},[model]);
 const value=model.value,output=value?.outputs.find(item=>item.path===selected)??value?.outputs[0],snapshot=output?.[phase];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Construction reading" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Construction reading</strong><button onClick={close}>Close</button></header>
 <label>Construction entry <input value={entry} onChange={event=>setEntry(event.target.value)}/></label><label>Contributor or output (optional) <input value={focus} onChange={event=>setFocus(event.target.value)}/></label>
 <button disabled={model.loading||!entry.trim()} onClick={()=>{setSelected('');void model.read({entry:entry.trim(),focus:focus.trim()||null,sources:sources()});}}>Read captured drafts</button><p role="status">{model.notice}</p>
 {value&&<><p>Captured entry: {value.entry}{value.focus?` · focus: ${value.focus}`:""}</p><p>{value.output_count} related outputs{value.outputs_truncated?' · first 300 shown':''}. {value.test_count} detected tests{value.tests_truncated?' · test list truncated':''}. Evidence: {value.evidence_status}. Outputs have not been written.</p><p>{value.test_relationship}</p><details><summary>Detected test declarations</summary>{value.tests.map(test=><p key={test.id}>{test.name} · {test.source?.path??test.target.path}:{test.source?.line??test.target.line}</p>)}</details>{value.notices.map((notice,index)=><p key={index}>{notice}</p>)}{value.unused_sources.length>0&&<p>{value.unused_sources.length} supplied drafts did not contribute to this entry.</p>}
 <label>Constructed output <select value={output?.path??''} onChange={event=>setSelected(event.target.value)}>{value.outputs.map(item=><option key={item.path} value={item.path}>{item.change} · {item.path}</option>)}</select></label>
 {output&&<article><strong>{output.change} · {output.path}</strong><p>Contributors: {output.contributors.join(', ')||'No compiler source origins reported.'}</p><button onClick={()=>setPhase('before')}>Saved construction</button><button onClick={()=>setPhase('after')}>Draft construction</button>
 {snapshot?<><p>{snapshot.kind} · {snapshot.size} bytes · executable {String(snapshot.executable)}</p><pre style={{whiteSpace:'pre-wrap'}}>{snapshot.preview??'Binary output; text preview unavailable.'}</pre>{snapshot.preview_truncated&&<p>Preview abbreviated.</p>}</>:<p>Output absent from this construction.</p>}
 <details><summary>Compiler contribution ranges</summary>{output.origins.map((origin,index)=><p key={index}>{origin.phase} · {origin.path} bytes {origin.source_bytes.join('–')} → output bytes {origin.output_bytes.join('–')} · {origin.exact?'equal byte slices':'reported origin; exact bytes unconfirmed'}</p>)}{output.origins_truncated&&<p>First 100 origins shown.</p>}</details></article>}</>}
 </section>,document.body);
}
