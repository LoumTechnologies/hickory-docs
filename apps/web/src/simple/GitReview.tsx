import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import {GitConflictView} from './GitConflict';
import type {GitChanges,GitDiff,GitChangeFile} from '../api/git';
export type ReviewFile=GitChangeFile&{from_outside_workspace?:boolean};
export function reviewIndexPaths(file:ReviewFile,unstage:boolean):string[]{
 const rename=(unstage?file.index:file.tree)==='R';if(rename&&(!file.from||file.from_outside_workspace))throw new Error('Open the repository root to change this complete rename. No index request was sent.');
 return [...new Set(rename?[file.from!,file.path]:[file.path])];
}
export interface GitReviewReader {writeIndex?:(paths:string[],unstage:boolean)=>Promise<{ok:boolean}>;changes:()=>Promise<GitChanges>;diff:(path:string,staged:boolean)=>Promise<GitDiff>}
export class GitReviewModel {
 files:ReviewFile[]=[];indexPending=false;needsRefresh=false;branch='';selected:string|null=null;staged=false;detail='';notice='';loading=false;page=0;
 private ticket=0;
 constructor(private readonly reader:GitReviewReader,private readonly changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;}
 async refresh(){const id=++this.ticket;this.loading=true;this.files=[];this.branch='';this.selected=null;this.detail='';this.notice='Reading saved Git state…';this.changed();
  try{const value=await this.reader.changes();if(id!==this.ticket)return;this.files=value.files;if(!this.indexPending)this.needsRefresh=false;this.branch=value.branch??'';this.page=0;this.notice=value.repository?'Saved Git state · unsaved editor drafts are not included.':'This workspace is not a Git repository.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async select(path:string,staged:boolean){if(!this.files.some(file=>file.path===path))return;const id=++this.ticket;this.selected=path;this.staged=staged;this.detail='';this.loading=true;this.notice='Reading saved diff…';this.changed();
  try{const value=await this.reader.diff(path,staged);if(id!==this.ticket)return;if(value.path!==path)throw new Error('Diff identity changed. Refresh repository changes.');this.detail=value.diff.slice(0,200000);this.notice=value.binary?'Binary change · text comparison unavailable.':value.diff.length>200000?'First 200,000 characters shown. Use Terminal to read the complete diff.':value.diff?'Captured Git diff · refresh after external changes.':'No changes on this side.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async changeIndex(path:string,unstage:boolean){
  if(this.indexPending)return;if(this.needsRefresh){this.notice='Refresh Git state before another index action.';this.changed();return;}
  const file=this.files.find(file=>file.path===path);if(!file||!this.reader.writeIndex)return;
  let paths:string[];try{paths=reviewIndexPaths(file,unstage);}catch(error){this.notice=String(error);this.changed();return;}
  const id=++this.ticket;this.indexPending=true;this.needsRefresh=true;this.loading=false;this.notice=unstage?'Unstaging saved paths…':'Staging saved paths…';this.changed();
  try{const value=await this.reader.writeIndex(paths,unstage);if(!value.ok)throw new Error('Index operation was not confirmed.');if(id!==this.ticket)return;this.indexPending=false;await this.refresh();if(this.ticket===id+1&&!this.needsRefresh)this.notice=unstage?'Saved paths unstaged. Editor drafts retained.':'Saved paths staged. Review the staged side before committing.';}
  catch(error){if(id===this.ticket)this.notice=`Index result not confirmed: ${String(error)} Refresh Git state before retrying.`;}
  finally{this.indexPending=false;this.changed();}
 }

}
export function GitReviewView({visible,workspace,close}:{visible:boolean;workspace:string;close:()=>void}) {
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitReviewModel({changes:()=>api.gitChanges(),diff:(path,staged)=>api.gitDiff(path,staged),writeIndex:(paths,unstage)=>unstage?api.gitUnstage({paths}):api.gitStage({paths})},paint),[workspace]);
 const panel=useRef<HTMLElement>(null);
 useEffect(()=>{if(!visible)return;void model.refresh();panel.current?.focus();return()=>model.close();},[visible,model]);
 if(!visible)return null;
 return createPortal(<section ref={panel} tabIndex={-1} className="simple-workspace-problems simple-source-tests" role="dialog" aria-label="Repository changes" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
  <header><strong>Repository changes {model.branch}</strong><button onClick={()=>void model.refresh()}>Refresh</button><button onClick={close}>Close</button></header>
  <p role="status">{model.notice}</p><p>Saved working tree and index. Unsaved editor drafts are not included.</p>
  <div aria-label="Changed files">{model.files.slice(model.page*20,(model.page+1)*20).map(file=><div key={file.path}><span>{file.from?`${file.from} → `:''}{file.path}</span> <button aria-label={`Working tree ${file.path}`} onClick={()=>void model.select(file.path,false)}>Working tree {file.tree}</button> <button aria-label={`Staged ${file.path}`} onClick={()=>void model.select(file.path,true)}>Staged {file.index}</button></div>)}</div>
  <nav aria-label="Changed file pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>{model.files.length} changed files · page {model.page+1}</span><button disabled={(model.page+1)*20>=model.files.length} onClick={()=>{model.page++;paint();}}>Next</button></nav>
  {model.needsRefresh&&!model.indexPending&&<p>Refresh Git state before another index action.</p>}
  {model.selected&&<><h3>{model.selected} · {model.staged?'Staged':'Working tree'}</h3><div><button disabled={model.indexPending||model.needsRefresh||![...model.files].some(file=>file.path===model.selected&&file.tree!==' ')} onClick={()=>void model.changeIndex(model.selected!,false)}>Stage saved file</button><button disabled={model.indexPending||model.needsRefresh||!model.files.some(file=>file.path===model.selected&&![' ','?','!'].includes(file.index))} onClick={()=>void model.changeIndex(model.selected!,true)}>Unstage file</button></div><GitConflictView key={model.selected} path={model.selected}/><pre aria-label="Git diff" style={{maxHeight:'40vh',overflow:'auto',whiteSpace:'pre'}}>{model.detail}</pre></>}
 </section>,document.body);
}
