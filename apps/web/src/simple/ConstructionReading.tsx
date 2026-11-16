import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import type {SourceTestCandidate,TestSourcePoint} from '../api/testSources';
import {request} from '../api/client';
import {retainedBehaviorReading} from './BehaviorReading';
import {BehaviorReadingPanel} from './BehaviorReadingView';
export interface ConstructionInput {entry:string;sources:Record<string,string>;focus:string|null}
export interface ConstructedSnapshot {hash:string;kind:string;executable:boolean;size:number;preview:string|null;preview_truncated:boolean}
export interface ConstructionOutput {path:string;change:string;before:ConstructedSnapshot|null;after:ConstructedSnapshot|null;contributors:string[];origins_truncated:boolean;origins:{phase:string;path:string;source_bytes:[number,number];output_bytes:[number,number];exact:boolean;source_hash:string|null;source_point?:TestSourcePoint|null;fragment?:{name:string;kind:string;body:[number,number]}|null}[]}
export interface ConstructionReading {version:number;entry:string;focus:string|null;materialized:boolean;output_count:number;outputs_truncated:boolean;outputs:ConstructionOutput[];tests:SourceTestCandidate[];test_count:number;tests_truncated:boolean;test_relationship:string;evidence_status:string;notices:string[];unused_sources:string[]}
export class ConstructionReadingModel {
 value:ConstructionReading|null=null;notice='';loading=false;private ticket=0;private linkTicket=0;
 constructor(private load:(input:ConstructionInput)=>Promise<ConstructionReading>,private changed:()=>void=()=>{}){}
 sourceLink(value:ConstructionReading){const id=++this.linkTicket;return ()=>this.value===value&&id===this.linkTicket;}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(provided:ConstructionInput){const input={...provided,sources:{...provided.sources}};const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading construction…';this.changed();
 try{const value=await this.load(input);if(id!==this.ticket)return;if(value.version!==1||value.entry!==input.entry||value.focus!==input.focus||value.materialized!==false)throw new Error('Construction identity changed or unsupported response. Refresh the reading.');this.value=value;this.notice='Captured construction and draft consequences. Editor changes after capture are not included.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
export function ConstructionReadingView({sources,retainedSources,capture,workspace,engineWorkspace,terminal,close,navigate}:{sources:()=>Record<string,string>;retainedSources?:()=>{sources:Record<string,string>;missing_sources:string[];unaddressed_sources:Record<string,string>;saved_guards:Record<string,string>};capture?:()=>()=>boolean;workspace?:string;engineWorkspace?:string;terminal?:(id:string)=>void;close:()=>void;navigate:(point:TestSourcePoint,current:()=>boolean)=>Promise<void>}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new ConstructionReadingModel(input=>request('POST','/api/construction/reading',input),paint),[]);
 const behavior=useMemo(()=>retainedBehaviorReading(workspace??'unknown',engineWorkspace??''),[workspace,engineWorkspace]);behavior.changed=paint;
 const [entry,setEntry]=useState('project.md'),[focus,setFocus]=useState(''),[selected,setSelected]=useState(''),[phase,setPhase]=useState<'before'|'after'>('after');const panel=useRef<HTMLElement>(null);
 useEffect(()=>{panel.current?.focus();return()=>{model.close();behavior.close();behavior.changed=()=>{};};},[model,behavior]);
 const value=model.value,output=value?.outputs.find(item=>item.path===selected)??value?.outputs[0],snapshot=output?.[phase];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Construction reading" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Construction reading</strong><button onClick={close}>Close</button></header>
 <label>Construction entry <input value={entry} onChange={event=>setEntry(event.target.value)}/></label><label>Contributor or output (optional) <input value={focus} onChange={event=>setFocus(event.target.value)}/></label>
 <button disabled={model.loading||!entry.trim()} onClick={()=>{setSelected('');const retained=retainedSources?.(),input={entry:entry.trim(),focus:focus.trim()||null,sources:retained?.sources??sources()},lease=capture?.()??(()=>JSON.stringify(sources())===JSON.stringify(input.sources));void model.read(input);void behavior.read({version:1,...input,missing_sources:retained?.missing_sources??[],unaddressed_sources:retained?.unaddressed_sources??{},saved_guards:retained?.saved_guards??{}},lease);}}>Read captured drafts</button><p role="status">{model.notice}</p>
 {value&&<><p>Captured entry: {value.entry}{value.focus?` · focus: ${value.focus}`:""}</p><p>{value.output_count} related outputs{value.outputs_truncated?' · first 300 shown':''}. {value.test_count} detected tests{value.tests_truncated?' · test list truncated':''}. Evidence: {value.evidence_status}. Outputs have not been written.</p><p>{value.test_relationship}</p><details><summary>Detected test declarations</summary>{value.tests.map(test=><p key={test.id}>{test.name} · {test.source?.path??test.target.path}:{test.source?.line??test.target.line}</p>)}</details>{value.notices.map((notice,index)=><p key={index}>{notice}</p>)}{value.unused_sources.length>0&&<p>{value.unused_sources.length} supplied drafts did not contribute to this entry.</p>}
 <label>Constructed output <select value={output?.path??''} onChange={event=>setSelected(event.target.value)}>{value.outputs.map(item=><option key={item.path} value={item.path}>{item.change} · {item.path}</option>)}</select></label>
 {output&&<article><strong>{output.change} · {output.path}</strong><p>Contributors: {output.contributors.join(', ')||'No compiler source origins reported.'}</p><button onClick={()=>setPhase('before')}>Saved construction</button><button onClick={()=>setPhase('after')}>Draft construction</button>
 {snapshot?<><p>{snapshot.kind} · {snapshot.size} bytes · executable {String(snapshot.executable)}</p><pre style={{whiteSpace:'pre-wrap'}}>{snapshot.preview??'Binary output; text preview unavailable.'}</pre>{snapshot.preview_truncated&&<p>Preview abbreviated.</p>}</>:<p>Output absent from this construction.</p>}
 <details><summary>Compiler contribution ranges</summary>{output.origins.map((origin,index)=><p key={index}>{origin.phase} · {origin.path} bytes {origin.source_bytes.join('–')} → output bytes {origin.output_bytes.join('–')} · {origin.exact?'equal byte slices':'reported origin; exact bytes unconfirmed'}{origin.fragment&&` · declared ${origin.fragment.kind} #${origin.fragment.name}`}{origin.source_point&&<button aria-label={`Open ${origin.phase} contributor ${origin.path}`} onClick={()=>{const current=model.sourceLink(value);void navigate(origin.source_point!,current).catch(error=>{if(current()){model.notice=String(error);paint();}});}}>Open contributor</button>}</p>)}{output.origins_truncated&&<p>First 100 origins shown.</p>}</details></article>}</>}
 <BehaviorReadingPanel model={behavior} terminal={terminal}/>
 </section>,document.body);
}
export function constructionPointOffset(source:string,point:TestSourcePoint,actualHash:string):number|null {
 if(!/^[0-9a-f]{64}$/.test(point.hash)||point.hash!==actualHash||!Number.isSafeInteger(point.line)||!Number.isSafeInteger(point.character)||point.line<1||point.character<0)return null;
 const rows=source.split('\n');if(point.line>rows.length||point.character>rows[point.line-1].length)return null;
 const offset=rows.slice(0,point.line-1).reduce((sum,row)=>sum+row.length+1,0)+point.character;
 if(offset>0&&offset<source.length&&source.charCodeAt(offset-1)>=0xd800&&source.charCodeAt(offset-1)<=0xdbff&&source.charCodeAt(offset)>=0xdc00&&source.charCodeAt(offset)<=0xdfff)return null;
 return offset;
}
