import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
export interface RecoveryPage {version:number;workspace:string;ids:string[];next:string|null;ignored:number}
export interface RecoverySummary {kind:'file'|'missing';bytes?:number;sha256?:string}
export interface RecoveryReading {version:number;workspace:string;id:string;fingerprint:string;title:string;title_truncated:boolean;paths:{path:string;saved_before:RecoverySummary;saved_after:RecoverySummary;draft_before:RecoverySummary;draft_after:RecoverySummary}[];all_paths_recorded_applied:boolean;unrecognized_files:string[];draft_text:string|null;saved_text:string|null}
export type RecoverySnapshot='saved_before'|'saved_after'|'draft_before'|'draft_after';
export class RecoveryReader {
 page:RecoveryPage|null=null;value:RecoveryReading|null=null;notice='';loading=false;private ticket=0;
 constructor(private load:<T>(route:string)=>Promise<T>,private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.value=null;this.page=null;}
 async list(after:string|null=null){const id=++this.ticket;this.loading=true;this.value=null;this.notice='Reading retained recovery identities…';this.changed();
 try{const page=await this.load<RecoveryPage>('/api/changes/resource-recoveries'+(after?'?after='+encodeURIComponent(after):''));if(id!==this.ticket)return;
 if(page.version!==1||!page.workspace||page.ids.length>100||page.ids.some((value,index)=>!/^([0-9a-f]{32})$/.test(value)||(index>0&&page.ids[index-1]>=value)||(after!==null&&value<=after))||(page.next!==null&&page.next!==page.ids.at(-1))||(after!==null&&page.workspace!==this.page?.workspace))throw new Error('Recovery page identity changed. Refresh from the beginning.');
 this.page=page;this.notice=page.ids.length?'Choose a retained record. Identities are sorted, not chronological.':'No retained resource recovery records.';
 }catch(error){if(id===this.ticket){this.page=null;this.notice=String(error);}}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
 async read(record:string,path?:string,snapshot:RecoverySnapshot='draft_before'){
 const workspace=this.page?.workspace,previous=this.value?.id===record?this.value:null,id=++this.ticket;this.loading=true;this.value=null;this.notice='Reading retained source evidence…';this.changed();
 try{if(!workspace||!this.page?.ids.includes(record))throw new Error('Refresh recovery identities before selecting a record.');const [kind,side]=snapshot.split('_');
 const value=await this.load<RecoveryReading>('/api/changes/resource-recovery?id='+encodeURIComponent(record)+(path!==undefined?'&path='+encodeURIComponent(path)+'&'+kind+'='+side:''));if(id!==this.ticket)return;
 if(value.version!==1||value.workspace!==workspace||value.id!==record||!/^([0-9a-f]{64})$/.test(value.fingerprint)||(previous&&previous.fingerprint!==value.fingerprint)||(path!==undefined&&!value.paths.some(item=>item.path===path)))throw new Error('Recovery evidence changed. Refresh identities and read again.');
 this.value=value;this.notice='Retained evidence only. Current source and editor drafts are unchanged.';
 }catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
}
export function SourceRecoveryView({close}:{close:()=>void}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new RecoveryReader(route=>request('GET',route),paint),[]),panel=useRef<HTMLElement>(null);
 const [path,setPath]=useState(''),[snapshot,setSnapshot]=useState<RecoverySnapshot>('draft_before');
 useEffect(()=>{panel.current?.focus();void model.list();return()=>model.close();},[model]);
 const value=model.value,selected=value?.paths.find(item=>item.path===path),text=value?.[snapshot.startsWith('saved')?'saved_text':'draft_text'];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source recovery" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Source recovery</strong><button onClick={close}>Close</button></header>
 <button disabled={model.loading} onClick={()=>{setPath('');void model.list();}}>Refresh records</button><p role="status">{model.notice}</p>
 {model.page&&<><label>Retained record <select aria-label="Retained record" value={value?.id??''} disabled={model.loading} onChange={event=>{if(event.target.value){setPath('');void model.read(event.target.value);}}}><option value="">Choose record</option>{model.page.ids.map(id=><option key={id} value={id}>{id}</option>)}</select></label>{model.page.ignored>0&&<p>{model.page.ignored} unrecognized entries omitted.</p>}{model.page.next&&<button disabled={model.loading} onClick={()=>{setPath('');void model.list(model.page!.next);}}>Next 100 identities</button>}</>}
 {value&&<><h3>{value.title}{value.title_truncated?'…':''}</h3><p>{value.all_paths_recorded_applied?'Every path has an applied receipt.':'The record does not confirm every path was applied.'} These receipts describe the recorded operation, not current disk state.</p>{value.unrecognized_files.length>0&&<p>{value.unrecognized_files.length} unrecognized files in this record.</p>}
 <label>Recorded source <select aria-label="Recorded source" value={path} disabled={model.loading} onChange={event=>{setPath(event.target.value);void model.read(value.id,event.target.value,snapshot);}}><option value="" disabled>Choose source</option>{value.paths.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 <label>Snapshot <select aria-label="Snapshot" value={snapshot} disabled={model.loading} onChange={event=>{const next=event.target.value as RecoverySnapshot;setSnapshot(next);if(path)void model.read(value.id,path,next);}}>{(['saved_before','draft_before','saved_after','draft_after'] as RecoverySnapshot[]).map(side=><option key={side} value={side}>{side.replace('_',' ')}</option>)}</select></label>
 {selected&&<><p>{selected[snapshot].kind==='missing'?'Source absent in this snapshot.':`${selected[snapshot].bytes} UTF-8 bytes · ${selected[snapshot].sha256}`}</p>{text!==null&&text!==undefined&&<><pre style={{whiteSpace:'pre-wrap'}}>{text.slice(0,32000)}</pre>{text.length>32000&&<p>First 32,000 characters shown.</p>}</>}</>}
 </>}
 </section>,document.body);
}
