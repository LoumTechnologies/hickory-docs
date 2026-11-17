import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import {prepareResourceRestore,publishResourceRestore,type ResourceRestoreReview,type ResourceEditorRoute} from './ResourceEditors';
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
export function ResourceConsequencesView({review}:{review:ResourceRestoreReview}){
 const [page,setPage]=useState(0),[layer,setLayer]=useState<'saved'|'draft'>('saved');const value=review.facts[layer],rows=[...value.entries.map(path=>'Construction entry: '+path),...value.removed_entries.map(path=>'Removed entry: '+path),...value.output_paths.map(path=>'Generated output: '+path)],last=Math.max(0,Math.ceil(rows.length/100)-1),selected=Math.min(page,last);
 return <section aria-label="Construction consequences"><label>Construction layer <select aria-label="Construction layer" value={layer} onChange={event=>{setLayer(event.target.value as 'saved'|'draft');setPage(0);}}><option value="saved">Saved construction</option><option value="draft">Draft construction</option></select></label><p>{value.output_paths.length} generated output paths · relationships observed by compiler analysis; no checks or coverage queried.</p><pre>{rows.slice(selected*100,(selected+1)*100).join('\n')||'No construction entries or outputs reported.'}</pre>{last>0&&<><button disabled={!selected} onClick={()=>setPage(selected-1)}>Previous consequences</button><span>{selected+1}/{last+1}</span><button disabled={selected===last} onClick={()=>setPage(selected+1)}>Next consequences</button></>}</section>;
}
export function SourceRecoveryView({close,buffers,acquire,adopt,alive,epoch,editorSessions}:{editorSessions?:()=>void;close:()=>void;buffers:Buffers;acquire:(path:string)=>Promise<Buffer>;adopt:(routes:ResourceEditorRoute<Buffer>[],recovery:string)=>void;alive:()=>boolean;epoch:()=>number}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new RecoveryReader(route=>request('GET',route),paint),[]),panel=useRef<HTMLElement>(null);
 const [path,setPath]=useState(''),[snapshot,setSnapshot]=useState<RecoverySnapshot>('draft_before');
 const [review,setReview]=useState<ResourceRestoreReview|null>(null),[busy,setBusy]=useState(false),[message,setMessage]=useState(''),[inverse,setInverse]=useState<string|null>(null);
 const [reviewPage,setReviewPage]=useState(0);
 const lifetime=useRef(true);
 async function prepare(record=model.value){
  if(!record||busy)return;const stamp=epoch();setBusy(true);setReview(null);setMessage('Preparing current saved and draft restoration…');
  try{const next=await prepareResourceRestore(buffers,record,acquire,(route,body)=>request('POST',route,body),()=>lifetime.current&&alive()&&epoch()===stamp&&model.value?.id===record.id&&model.value.fingerprint===record.fingerprint);setReview(next);setReviewPage(0);setPath(next.patches[0].path);setMessage('Review every saved baseline and source draft before restoring.');}
  catch(error){setMessage(String(error));}finally{setBusy(false);}
 }
 async function publish(){if(!review||busy)return;const selected=review;setReview(null);setBusy(true);setMessage('Publishing reviewed source restoration…');
  try{const result=await publishResourceRestore(buffers,selected,(route,body)=>request('POST',route,body));adopt(result.routes,result.recovery);setInverse(result.recovery);setMessage('Restored saved baselines and source drafts. Prepare inverse to undo this restoration. No checks were run.');}
  catch(error){setMessage(String(error)+' Inspect Source recovery and current files before preparing another operation.');}finally{setBusy(false);}
 }
 async function prepareInverse(){if(!inverse||busy)return;setBusy(true);setReview(null);try{const record=await request<RecoveryReading>('GET','/api/changes/resource-recovery?id='+encodeURIComponent(inverse));if(record.id!==inverse||!alive())throw new Error('Inverse recovery identity changed.');model.value=record;setBusy(false);await prepare(record);}catch(error){setMessage(String(error));setBusy(false);}}
 useEffect(()=>{lifetime.current=true;panel.current?.focus();void model.list();return()=>{lifetime.current=false;model.close();};},[model]);
 const value=model.value,selected=value?.paths.find(item=>item.path===path),text=value?.[snapshot.startsWith('saved')?'saved_text':'draft_text'];
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Source recovery" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();if(!busy)close();}}}>
 <header><strong>Source recovery</strong>{editorSessions&&<button disabled={busy} onClick={editorSessions}>Editor sessions…</button>}<button disabled={busy} onClick={close}>Close</button></header>
 <button disabled={model.loading||busy} onClick={()=>{setReview(null);setPath('');void model.list();}}>Refresh records</button><p role="status">{model.notice}</p>
 {model.page&&<><label>Retained record <select aria-label="Retained record" value={value?.id??''} disabled={model.loading||busy} onChange={event=>{if(event.target.value){setReview(null);setPath('');void model.read(event.target.value);}}}><option value="">Choose record</option>{model.page.ids.map(id=><option key={id} value={id}>{id}</option>)}</select></label>{model.page.ignored>0&&<p>{model.page.ignored} unrecognized entries omitted.</p>}{model.page.next&&<button disabled={model.loading||busy} onClick={()=>{setPath('');void model.list(model.page!.next);}}>Next 100 identities</button>}</>}
 {message&&<p role="status">{message}</p>}{inverse&&<button disabled={busy||model.loading} onClick={()=>{void prepareInverse();}}>Prepare inverse restoration</button>}
 {value&&<><h3>{value.title}{value.title_truncated?'…':''}</h3><p>{value.all_paths_recorded_applied?'Every path has an applied receipt.':'The record does not confirm every path was applied.'} These receipts describe the recorded operation, not current disk state.</p>{value.unrecognized_files.length>0&&<p>{value.unrecognized_files.length} unrecognized files in this record.</p>}
 <button disabled={busy||model.loading} onClick={()=>{void prepare();}}>Prepare restoration</button>
 <label>Recorded source <select aria-label="Recorded source" value={path} disabled={model.loading||busy} onChange={event=>{setPath(event.target.value);void model.read(value.id,event.target.value,snapshot);}}><option value="" disabled>Choose source</option>{value.paths.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 <label>Snapshot <select aria-label="Snapshot" value={snapshot} disabled={model.loading||busy} onChange={event=>{const next=event.target.value as RecoverySnapshot;setSnapshot(next);if(path)void model.read(value.id,path,next);}}>{(['saved_before','draft_before','saved_after','draft_after'] as RecoverySnapshot[]).map(side=><option key={side} value={side}>{side.replace('_',' ')}</option>)}</select></label>
 {selected&&<><p>{selected[snapshot].kind==='missing'?'Source absent in this snapshot.':`${selected[snapshot].bytes} UTF-8 bytes · ${selected[snapshot].sha256}`}</p>{text!==null&&text!==undefined&&<><pre style={{whiteSpace:'pre-wrap'}}>{text.slice(0,32000)}</pre>{text.length>32000&&<p>First 32,000 characters shown.</p>}</>}</>}
 </>}
 {review&&<section aria-label="Restoration preview"><h3>Restore saved sources and drafts</h3><p>{review.patches.length} paths · source and construction revisions are checked again on publication.</p>
 <label>Restoration source <select aria-label="Restoration source" value={path} onChange={event=>{setPath(event.target.value);setReviewPage(0);}}>{review.patches.map(item=><option key={item.path} value={item.path}>{item.path}</option>)}</select></label>
 {review.patches.filter(item=>item.path===path).map(item=><div key={item.path}><strong>Current draft</strong><pre>{item.before?.text.slice(reviewPage*16000,(reviewPage+1)*16000)??'Source absent'}</pre><strong>Restored draft</strong><pre>{item.after?.text.slice(reviewPage*16000,(reviewPage+1)*16000)??'Source absent'}</pre><strong>Current saved baseline</strong><pre>{review.savedBefore[path]?.kind==='file'?(review.savedBefore[path] as {text:string}).text.slice(reviewPage*16000,(reviewPage+1)*16000):'Source absent'}</pre><strong>Restored saved baseline</strong><pre>{review.saved[path]?.kind==='file'?(review.saved[path] as {text:string}).text.slice(reviewPage*16000,(reviewPage+1)*16000):'Source absent'}</pre><p>Text page {reviewPage+1} · 16,000 characters per side. Draft text and saved text may differ.</p><button disabled={!reviewPage} onClick={()=>setReviewPage(reviewPage-1)}>Previous restoration text</button><button disabled={Math.max(item.before?.text.length??0,item.after?.text.length??0,review.savedBefore[path]?.kind==='file'?(review.savedBefore[path] as {text:string}).text.length:0,review.saved[path]?.kind==='file'?(review.saved[path] as {text:string}).text.length:0)<=(reviewPage+1)*16000} onClick={()=>setReviewPage(reviewPage+1)}>Next restoration text</button></div>)}
 <ResourceConsequencesView review={review}/><button disabled={busy} onClick={()=>{void publish();}}>Restore reviewed saved sources and drafts</button></section>}
 </section>,document.body);
}
