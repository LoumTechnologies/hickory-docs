import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import type {GitChanges,GitDiff,GitChangeFile} from '../api/git';
export interface GitReviewReader {changes:()=>Promise<GitChanges>;diff:(path:string,staged:boolean)=>Promise<GitDiff>}
export class GitReviewModel {
 files:GitChangeFile[]=[];branch='';selected:string|null=null;staged=false;detail='';notice='';loading=false;page=0;
 private ticket=0;
 constructor(private readonly reader:GitReviewReader,private readonly changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;}
 async refresh(){const id=++this.ticket;this.loading=true;this.files=[];this.branch='';this.selected=null;this.detail='';this.notice='Reading saved Git state…';this.changed();
  try{const value=await this.reader.changes();if(id!==this.ticket)return;this.files=value.files;this.branch=value.branch??'';this.page=0;this.notice=value.repository?'Saved Git state · unsaved editor drafts are not included.':'This workspace is not a Git repository.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async select(path:string,staged:boolean){if(!this.files.some(file=>file.path===path))return;const id=++this.ticket;this.selected=path;this.staged=staged;this.detail='';this.loading=true;this.notice='Reading saved diff…';this.changed();
  try{const value=await this.reader.diff(path,staged);if(id!==this.ticket)return;if(value.path!==path)throw new Error('Diff identity changed. Refresh repository changes.');this.detail=value.diff.slice(0,200000);this.notice=value.binary?'Binary change · text comparison unavailable.':value.diff.length>200000?'First 200,000 characters shown. Use Terminal to read the complete diff.':value.diff?'Captured Git diff · refresh after external changes.':'No changes on this side.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
}
export function GitReviewView({visible,workspace,close}:{visible:boolean;workspace:string;close:()=>void}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitReviewModel({changes:()=>api.gitChanges(),diff:(path,staged)=>api.gitDiff(path,staged)},paint),[workspace]);
 const panel=useRef<HTMLElement>(null);
 useEffect(()=>{if(!visible)return;void model.refresh();panel.current?.focus();return()=>model.close();},[visible,model]);
 if(!visible)return null;
 return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Repository changes" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
  <header><strong>Repository changes {model.branch}</strong><button onClick={()=>void model.refresh()}>Refresh</button><button onClick={close}>Close</button></header>
  <p role="status">{model.notice}</p><p>Saved working tree and index. Unsaved editor drafts are not included.</p>
  <div aria-label="Changed files">{model.files.slice(model.page*20,(model.page+1)*20).map(file=><div key={file.path}><span>{file.from?`${file.from} → `:''}{file.path}</span> <button aria-label={`Working tree ${file.path}`} onClick={()=>void model.select(file.path,false)}>Working tree {file.tree}</button> <button aria-label={`Staged ${file.path}`} onClick={()=>void model.select(file.path,true)}>Staged {file.index}</button></div>)}</div>
  <nav aria-label="Changed file pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>{model.files.length} changed files · page {model.page+1}</span><button disabled={(model.page+1)*20>=model.files.length} onClick={()=>{model.page++;paint();}}>Next</button></nav>
  {model.selected&&<><h3>{model.selected} · {model.staged?'Staged':'Working tree'}</h3><pre aria-label="Git diff" style={{maxHeight:'40vh',overflow:'auto',whiteSpace:'pre'}}>{model.detail}</pre></>}
 </section>,document.body);
}
