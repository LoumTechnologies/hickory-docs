import {useEffect,useMemo,useReducer,useState} from 'react';
import {request} from '../api/client';
import type {Buffer} from './buffers';
import {requestWorkspaceProposal} from './WorkspaceChangeEvents';
import {utf16ToPosition} from '../lsp/positions';
export interface ConflictStage {stage:number;mode:string;oid:string;text:string|null;unavailable:string|null}
export interface ConflictReading {version:1;path:string;fingerprint:string;stages:ConflictStage[];index_modified:false;source_files_modified:false}
export function validateConflict(value:ConflictReading,path:string):void {
 if(value.version!==1||value.path!==path||value.index_modified!==false||value.source_files_modified!==false||!/^[a-f0-9]{64}$/i.test(value.fingerprint)||!Array.isArray(value.stages)||value.stages.length<1||value.stages.length>3)throw new Error('Conflict identity changed or unsupported response. Refresh the conflict.');
 const seen=new Set<number>();for(const stage of value.stages){if(![1,2,3].includes(stage.stage)||seen.has(stage.stage)||!/^(?:[a-f0-9]{40}|[a-f0-9]{64})$/i.test(stage.oid)||!/^[0-7]{6}$/.test(stage.mode)||!(typeof stage.text==='string'&&stage.unavailable===null&&['100644','100755'].includes(stage.mode)||stage.text===null&&typeof stage.unavailable==='string'))throw new Error('Malformed conflict stage. Refresh repository changes.');seen.add(stage.stage);}
}
export class GitConflictModel {
 value:ConflictReading|null=null;notice='';loading=false;private ticket=0;
 constructor(private changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.value=null;}
 async read(path:string,load:(path:string)=>Promise<ConflictReading>){const id=++this.ticket;this.value=null;this.loading=true;this.notice='Reading unmerged stages…';this.changed();
 try{const value=await load(path);if(id!==this.ticket)return;validateConflict(value,path);this.value=value;this.notice='Captured immutable Git stages. Working text and unsaved drafts are separate.';}
 catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}}
 async choose(stage:number,load:(path:string)=>Promise<ConflictReading>,offer:(stage:ConflictStage,value:ConflictReading,current:()=>boolean)=>Promise<void>){
  const value=this.value,item=value?.stages.find(item=>item.stage===stage);if(!value||!item||item.text===null){this.notice='Choose an available regular text stage first.';this.changed();return;}
  const id=++this.ticket,current=()=>id===this.ticket&&this.value===value;this.loading=true;this.notice='Checking captured conflict stages…';this.changed();
  try{const fresh=await load(value.path);if(!current())return;validateConflict(fresh,value.path);if(fresh.fingerprint!==value.fingerprint||!fresh.stages.some(other=>other.stage===item.stage&&other.mode===item.mode&&other.oid===item.oid&&other.text===item.text))throw new Error('Conflict stages changed. Inspect them again before choosing a resolution.');await offer(item,value,current);if(current())this.notice='Prepared a source change preview. Nothing saved or staged.';}
  catch(error){if(current())this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }

}
export function GitConflictView({path,acquire,close}:{path:string;acquire:(path:string)=>Promise<Buffer>;close:()=>void}){
 const [,paint]=useReducer(n=>n+1,0),model=useMemo(()=>new GitConflictModel(paint),[path]);const [selected,setSelected]=useState(2),[page,setPage]=useState(0);
 useEffect(()=>()=>model.close(),[model]);
 const load=(path:string)=>request<ConflictReading>('GET','/api/git/conflict?path='+encodeURIComponent(path));
 const choose=(stage:number)=>model.choose(stage,load,async(item,value,current)=>{
  const buffer=await acquire(value.path);if(!current())return;
  if(buffer.readonly||buffer.saving||buffer.transient)throw new Error('Choose an authored, editable source that is not saving.');
  const source=buffer.text,uri='hick:///'+value.path.split('/').map(encodeURIComponent).join('/');
  requestWorkspaceProposal(`Use Git stage ${item.stage} · ${item.oid}`,{changes:{[uri]:[{range:{start:{line:0,character:0},end:utf16ToPosition(source,source.length)},newText:item.text}]}},new Map([[value.path,source]]));close();
 });
 const stage=model.value?.stages.find(item=>item.stage===selected)??model.value?.stages[0],rows=stage?.text?.split('\n')??[],last=Math.max(0,Math.ceil(rows.length/100)-1),index=Math.min(page,last),text=rows.slice(index*100,(index+1)*100).join('\n');
 return <section aria-label="Conflict stages"><button disabled={model.loading} onClick={()=>{setPage(0);void model.read(path,path=>request('GET','/api/git/conflict?path='+encodeURIComponent(path)));}}>Inspect conflict stages</button>{model.notice&&<p role="status">{model.notice}</p>}{model.value&&<>
 <p>Stage 1: base · Stage 2 and Stage 3 retain Git’s meanings, including during rebase.</p>
 {model.value.stages.map(item=><button key={item.stage} aria-pressed={stage===item} onClick={()=>{setSelected(item.stage);setPage(0);}}>Stage {item.stage}{item.stage===1?' · Base':''}</button>)}
 {stage&&<><p>Stage {stage.stage} · mode {stage.mode}</p><code>{stage.oid}</code>{stage.text===null?<p>{stage.unavailable}</p>:<><button disabled={model.loading} onClick={()=>void choose(stage.stage)}>Preview this stage as source</button><p>Lines {index*100+1}–{Math.min(rows.length,(index+1)*100)}{text.length>200000?' · page preview capped at 200,000 characters':''}</p><pre aria-label="Conflict stage text" style={{maxHeight:'30vh',overflow:'auto'}}>{text.slice(0,200000)}</pre><button disabled={!index} onClick={()=>setPage(index-1)}>Previous stage lines</button><button disabled={index===last} onClick={()=>setPage(index+1)}>Next stage lines</button></>}</>}
 </>}</section>;
}
