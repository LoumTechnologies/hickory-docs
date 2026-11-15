import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import type {GitCommit} from '../api/gitHistoryTypes';
import type {CommitReadingData} from '../api/git';
export type GitHistoryRequest={kind:'log';path:string|null}|{kind:'commit';sha:string;title:string};
export type HistoryCommit=Pick<GitCommit,'sha'|'short'|'author'|'subject'>;
export interface HistoryLog {repository:boolean;commits:HistoryCommit[]}
export interface GitHistoryReader {log:(limit:number,path:string|null)=>Promise<HistoryLog>;read:(sha:string)=>Promise<CommitReadingData>}
export class GitHistoryModel {
 commits:HistoryCommit[]=[];reading:CommitReadingData|null=null;file=0;side:'before'|'after'='after';page=0;limit=120;notice='';loading=false;private ticket=0;
 constructor(private readonly reader:GitHistoryReader,private readonly changed:()=>void=()=>{},readonly request:GitHistoryRequest={kind:'log',path:null}){}
 close(){this.ticket++;this.loading=false;this.reading=null;this.notice='';}
 async refresh(limit=120){if(this.request.kind==='commit'){await this.readCommit(this.request.sha);return;}const path=this.request.path;const id=++this.ticket;this.limit=Math.max(1,Math.min(1000,limit));this.commits=[];this.reading=null;this.page=0;this.loading=true;this.notice='Reading saved history…';this.changed();
  try{const value=await this.reader.log(this.limit,path);if(id!==this.ticket)return;if(value.commits.some(commit=>!/^([0-9a-f]{40}|[0-9a-f]{64})$/.test(commit.sha)))throw new Error('Malformed commit identity. Refresh repository history.');this.commits=value.commits;this.notice=value.repository?`${value.commits.length} reported commits · saved history only.`:'This workspace is not a Git repository.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async select(sha:string){if(!this.commits.some(commit=>commit.sha===sha))return;await this.readCommit(sha);}
 async readCommit(sha:string){if(!/^([0-9a-f]{40}|[0-9a-f]{64})$/.test(sha)||this.request.kind==='commit'&&this.request.sha!==sha){this.ticket++;this.reading=null;this.loading=false;this.notice='Commit reference is missing or changed. Refresh its source and choose the commit again.';this.changed();return;}const id=++this.ticket;this.reading=null;this.loading=true;this.notice='Reading immutable commit…';this.changed();
  try{const value=await this.reader.read(sha);if(id!==this.ticket)return;if(value.sha!==sha)throw new Error('Commit identity changed. Refresh repository history.');this.reading=value;this.file=0;this.side='after';this.notice='Immutable commit reading · current editor draft is retained.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
}
export function GitHistoryView({visible,workspace,request,close}:{visible:boolean;workspace:string;request:GitHistoryRequest;close:()=>void}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitHistoryModel({log:(limit,path)=>api.gitLog(limit,path??undefined),read:sha=>api.gitReading(sha)},paint,request),[workspace,request]),panel=useRef<HTMLElement>(null);
 const path=request.kind==='log'?request.path:null;
 const refresh=()=>model.refresh(model.limit);
 useEffect(()=>{if(!visible)return;void model.refresh();panel.current?.focus();return()=>model.close();},[visible,model,request]);
 if(!visible)return null;const file=model.reading?.files[model.file],content=file?.[model.side]??'';
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Repository history" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>{request.kind==='commit'?request.title:path?`Saved history · ${path}`:'Repository history'}</strong><button onClick={()=>void refresh()}>Refresh</button><button onClick={close}>Close</button></header><p role="status">{model.notice}</p>
 {request.kind==='log'&&<>
 <div aria-label="Reported commits">{model.commits.slice(model.page*20,(model.page+1)*20).map(commit=><div key={commit.sha}><button aria-label={`Read commit ${commit.sha}`} onClick={()=>void model.select(commit.sha)}>{commit.short} · {commit.subject}</button><span> {commit.author}</span></div>)}</div>
 <nav aria-label="History pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>Page {model.page+1}</span><button disabled={(model.page+1)*20>=model.commits.length} onClick={()=>{model.page++;paint();}}>Next</button><button disabled={model.limit>=1000||model.commits.length<model.limit} onClick={()=>void model.refresh(Math.min(1000,model.limit+120))}>Load more commits</button></nav>
 </>}
 {model.reading&&<><h3>{model.reading.sha}</h3>{model.reading.message.length>20000&&<p>First 20,000 message characters shown.</p>}<pre aria-label="Commit message">{model.reading.message.slice(0,20000)}</pre>{model.reading.files.length?<><select aria-label="Historical file" value={model.file} onChange={event=>{model.file=Number(event.target.value);paint();}}>{model.reading.files.map((file,index)=><option key={index} value={index}>{file.status} · {file.path}</option>)}</select><label>Content side<select aria-label="Historical side" value={model.side} onChange={event=>{model.side=event.target.value as 'before'|'after';paint();}}><option value="before">Before commit</option><option value="after">After commit</option></select></label><p>{file?.binary?'Binary content · text reading unavailable.':content.length>200000?'First 200,000 characters shown.':content?'Immutable file content.':'No text on this side.'}</p><pre aria-label="Historical content" style={{maxHeight:'35vh',overflow:'auto',whiteSpace:'pre'}}>{file?.binary?'':content.slice(0,200000)}</pre></>:<p>No files under this workspace in this commit.</p>}</>}
 </section>,document.body);
}
