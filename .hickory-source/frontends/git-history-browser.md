<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read immutable repository history without replacing the editor

For maintainers changing workspace services independently of the editor renderer.

History is a read-only capability over saved Git facts. Capture the requested workspace/path and request intent independently of the active editor. Retain full commit identities from the log, require an exact identity match when fetching immutable before/after blobs, and retire older or closed replies. Local paging and a bounded log window keep the floating reading compact. No checkout, save, index or source mutation occurs.

<ui:copy id="simple-git-history-browser">
import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import type {GitCommit} from '../api/gitHistoryTypes';
import type {CommitReadingData} from '../api/git';
export type HistoryCommit=Pick<GitCommit,'sha'|'short'|'author'|'subject'>;
export interface HistoryLog {repository:boolean;commits:HistoryCommit[]}
export interface GitHistoryReader {log:(limit:number,path:string|null)=>Promise<HistoryLog>;read:(sha:string)=>Promise<CommitReadingData>}
export class GitHistoryModel {
 commits:HistoryCommit[]=[];reading:CommitReadingData|null=null;file=0;side:'before'|'after'='after';page=0;limit=120;notice='';loading=false;private ticket=0;
 constructor(private readonly reader:GitHistoryReader,private readonly changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.reading=null;this.notice='';}
 async refresh(path:string|null,limit=120){const id=++this.ticket;this.limit=Math.max(1,Math.min(1000,limit));this.commits=[];this.reading=null;this.page=0;this.loading=true;this.notice='Reading saved history…';this.changed();
  try{const value=await this.reader.log(this.limit,path);if(id!==this.ticket)return;if(value.commits.some(commit=>!/^([0-9a-f]{40}|[0-9a-f]{64})$/.test(commit.sha)))throw new Error('Malformed commit identity. Refresh repository history.');this.commits=value.commits;this.notice=value.repository?`${value.commits.length} reported commits · saved history only.`:'This workspace is not a Git repository.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
 async select(sha:string){if(!this.commits.some(commit=>commit.sha===sha))return;const id=++this.ticket;this.reading=null;this.loading=true;this.notice='Reading immutable commit…';this.changed();
  try{const value=await this.reader.read(sha);if(id!==this.ticket)return;if(value.sha!==sha)throw new Error('Commit identity changed. Refresh repository history.');this.reading=value;this.file=0;this.side='after';this.notice='Immutable commit reading · current editor draft is retained.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
}
</ui:copy>

## Present immutable file sides without replacing live source

The existing floating surface pages commit choices and reads exact before/after blobs. Historical file selection is local to the reading; it does not activate, reload or edit the source buffer. Render text literally, retain binary and empty-side distinctions, and report display limits. Closing returns focus and retires outstanding reads. A fresh request context keeps its title and data together: browser models are keyed by workspace/path, and native menu adapters replace the read session before presenting a different request. Closed readings clear their visible content before reuse.

<ui:copy id="simple-git-history-browser-view">
export function GitHistoryView({visible,workspace,path,close}:{visible:boolean;workspace:string;path:string|null;close:()=>void}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitHistoryModel({log:(limit,path)=>api.gitLog(limit,path??undefined),read:sha=>api.gitReading(sha)},paint),[workspace,path]),panel=useRef<HTMLElement>(null);
 useEffect(()=>{if(!visible)return;void model.refresh(path);panel.current?.focus();return()=>model.close();},[visible,model,path]);
 if(!visible)return null;const file=model.reading?.files[model.file],content=file?.[model.side]??'';
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Repository history" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>{path?`Saved history · ${path}`:'Repository history'}</strong><button onClick={()=>void model.refresh(path,model.limit)}>Refresh</button><button onClick={close}>Close</button></header><p role="status">{model.notice}</p>
 <div aria-label="Reported commits">{model.commits.slice(model.page*20,(model.page+1)*20).map(commit=><div key={commit.sha}><button aria-label={`Read commit ${commit.sha}`} onClick={()=>void model.select(commit.sha)}>{commit.short} · {commit.subject}</button><span> {commit.author}</span></div>)}</div>
 <nav aria-label="History pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>Page {model.page+1}</span><button disabled={(model.page+1)*20>=model.commits.length} onClick={()=>{model.page++;paint();}}>Next</button><button disabled={model.limit>=1000||model.commits.length<model.limit} onClick={()=>void model.refresh(path,Math.min(1000,model.limit+120))}>Load more commits</button></nav>
 {model.reading&&<><h3>{model.reading.sha}</h3>{model.reading.message.length>20000&&<p>First 20,000 message characters shown.</p>}<pre aria-label="Commit message">{model.reading.message.slice(0,20000)}</pre>{model.reading.files.length?<><select aria-label="Historical file" value={model.file} onChange={event=>{model.file=Number(event.target.value);paint();}}>{model.reading.files.map((file,index)=><option key={index} value={index}>{file.status} · {file.path}</option>)}</select><label>Content side<select aria-label="Historical side" value={model.side} onChange={event=>{model.side=event.target.value as 'before'|'after';paint();}}><option value="before">Before commit</option><option value="after">After commit</option></select></label><p>{file?.binary?'Binary content · text reading unavailable.':content.length>200000?'First 200,000 characters shown.':content?'Immutable file content.':'No text on this side.'}</p><pre aria-label="Historical content" style={{maxHeight:'35vh',overflow:'auto',whiteSpace:'pre'}}>{file?.binary?'':content.slice(0,200000)}</pre></>:<p>No files under this workspace in this commit.</p>}</>}
 </section>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/GitHistory.tsx"><ui:paste select="#simple-git-history-browser"/><ui:paste select="#simple-git-history-browser-view"/></ui:file>
## Keep immutable readings bound to exact commit identities

Hold an immutable reading while selecting another commit, then release it after the newer reply. Exact full identities, closed-view retirement and malformed log identities must prevent a foreign or obsolete reading from replacing the selected commit. These service witnesses do not assert native pointer or accessibility acceptance.

<ui:copy id="simple-git-history-browser-tests">
import {it,expect} from 'vitest';
import {GitHistoryModel} from './GitHistory';
import type {CommitReadingData} from '../api/git';
it('retains the requested full commit identity across held and closed reads',async()=>{
 const a='a'.repeat(40),b='b'.repeat(40),pending:((value:CommitReadingData)=>void)[]=[];
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[a,b].map(sha=>({sha,short:sha.slice(0,7),author:'fixture',subject:'saved commit'}))}),read:()=>new Promise(resolve=>pending.push(resolve))});
 await model.refresh('folder/é.md');const first=model.select(a),second=model.select(b);pending[1]({sha:b,parent:a,message:'new',files:[]});await second;pending[0]({sha:a,parent:null,message:'obsolete',files:[]});await first;expect(model.reading?.sha).toBe(b);expect(model.reading?.message).toBe('new');
 const closed=model.select(a);model.close();pending[2]({sha:a,parent:null,message:'closed',files:[]});await closed;expect(model.reading).toBeNull();
 const foreign=model.select(a);pending[3]({sha:b,parent:null,message:'foreign',files:[]});await foreign;expect(model.reading).toBeNull();expect(model.notice).toContain('identity changed');
});
it('refuses malformed log identities before admitting immutable reading choices',async()=>{
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[{sha:'HEAD',short:'HEAD',author:'fixture',subject:'ambiguous'}]}),read:async()=>{throw new Error('must not read');}});
 await model.refresh(null,2000);expect(model.limit).toBe(1000);expect(model.commits).toEqual([]);expect(model.notice).toContain('Malformed commit identity');
});
</ui:copy>

<ui:file path="apps/web/src/simple/GitHistory.test.ts"><ui:paste select="#simple-git-history-browser-tests"/></ui:file>
</ui:doc>
