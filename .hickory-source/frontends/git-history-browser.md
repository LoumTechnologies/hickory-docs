<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Read immutable repository history without replacing the editor

For maintainers changing workspace services independently of the editor renderer.

History is a read-only capability over saved Git facts. Capture the requested workspace/path and request intent independently of the active editor. Retain full commit identities from the log, require an exact identity match when fetching immutable before/after blobs, and retire older or closed replies. Local paging and a bounded log window keep the floating reading compact. No checkout, save, index or source mutation occurs.

<ui:copy id="simple-git-history-browser">
import {useEffect,useMemo,useReducer,useRef,useState} from 'react';
import {createPortal} from 'react-dom';
import {api,request as apiRequest} from '../api/client';
import {diffLines} from '../lib/diff';
import type {EditorView} from '@codemirror/view';
import type {Buffer} from './buffers';
import {authoredChanges,authoredSource} from './SourceText';
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
</ui:copy>

## Admit logs and observed commits as distinct reading requests

A typed request owns either a saved-history scope or one full observed commit identity. Attribution can therefore reuse immutable before/after reading without inventing a log result or resolving a display abbreviation. The commit request must match every read and response; a rejected new reference retires older work.

## Present immutable file sides without replacing live source

The existing floating surface pages commit choices and reads exact before/after blobs. Historical file selection is local to the reading; it does not activate, reload or edit the source buffer. Render text literally, retain binary and empty-side distinctions, and report display limits. Closing returns focus and retires outstanding reads. A fresh request context keeps its title and data together: browser models are keyed by workspace/path, and native menu adapters replace the read session before presenting a different request. Closed readings clear their visible content before reuse.

<ui:copy id="simple-git-history-browser-view">
export function GitHistoryView({visible,workspace,request,close,openComparison}:{visible:boolean;workspace:string;request:GitHistoryRequest;close:()=>void;openComparison?:(file:GitComparison,current:()=>boolean)=>Promise<boolean>}){
 const [comparing,setComparing]=useState(false);
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitHistoryModel({log:(limit,path)=>api.gitLog(limit,path??undefined),read:sha=>api.gitReading(sha)},paint,request),[workspace,request]),panel=useRef<HTMLElement>(null);
 const path=request.kind==='log'?request.path:null;
 const refresh=()=>model.refresh(model.limit);
 useEffect(()=>{if(!visible)return;void model.refresh();panel.current?.focus();return()=>model.close();},[visible,model,request]);
 if(!visible)return null;const file=model.reading?.files[model.file],content=file?.[model.side]??'';
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Repository history" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>{request.kind==='commit'?request.title:path?`Saved history · ${path}`:'Repository history'}</strong><button onClick={()=>void refresh()}>Refresh</button><button onClick={close}>Close</button></header><p role="status">{model.notice}</p>
 {request.kind==='log' && openComparison && <button type="button" aria-pressed={comparing} onClick={()=>setComparing(value=>!value)}>Compare two commits</button>}
 {request.kind==='log' && comparing && openComparison && <GitComparisonPicker open={openComparison} close={close}/> }
 {request.kind==='log'&&!comparing&&<>
 <div aria-label="Reported commits">{model.commits.slice(model.page*20,(model.page+1)*20).map(commit=><div key={commit.sha}><button aria-label={`Read commit ${commit.sha}`} onClick={()=>void model.select(commit.sha)}>{commit.short} · {commit.subject}</button><span> {commit.author}</span></div>)}</div>
 <nav aria-label="History pages"><button disabled={model.page===0} title={model.page===0 ? "You’re on the first page." : undefined} onClick={()=>{model.page--;paint();}}>Previous</button><span>Page {model.page+1}</span><button disabled={(model.page+1)*20>=model.commits.length} title={(model.page+1)*20>=model.commits.length ? "You’re on the last page." : undefined} onClick={()=>{model.page++;paint();}}>Next</button><button disabled={model.limit>=1000||model.commits.length<model.limit} title={model.limit>=1000 ? "The history view has reached its 1,000-commit limit." : model.commits.length<model.limit ? "There are no older commits to load." : undefined} onClick={()=>void model.refresh(Math.min(1000,model.limit+120))}>Load more commits</button></nav>
 </>}
 {!comparing&&model.reading&&<><h3>{model.reading.sha}</h3>{model.reading.message.length>20000&&<p>First 20,000 message characters shown.</p>}<pre aria-label="Commit message">{model.reading.message.slice(0,20000)}</pre>{model.reading.files.length?<><select aria-label="Historical file" value={model.file} onChange={event=>{model.file=Number(event.target.value);paint();}}>{model.reading.files.map((file,index)=><option key={index} value={index}>{file.status} · {file.path}</option>)}</select><label>Content side<select aria-label="Historical side" value={model.side} onChange={event=>{model.side=event.target.value as 'before'|'after';paint();}}><option value="before">Before commit</option><option value="after">After commit</option></select></label><p>{file?.binary?'Binary content · text reading unavailable.':content.length>200000?'First 200,000 characters shown.':content?'Immutable file content.':'No text on this side.'}</p><pre aria-label="Historical content" style={{maxHeight:'35vh',overflow:'auto',whiteSpace:'pre'}}>{file?.binary?'':content.slice(0,200000)}</pre></>:<p>No files under this workspace in this commit.</p>}</>}
 </section>,document.body);
}
</ui:copy>

## Attach a pinned Git baseline to the browser editor

The history picker reads changed paths first. It pins the returned commits
before reading a file patch. The working buffer keeps its editor and editing
services while the comparison recomputes against each new draft revision.

<ui:copy id="browser-editable-git-comparison">
type ComparisonFile = {path:string;status:string;before:string;after:string;patch:string;patch_truncated:boolean;binary:boolean};
type ComparisonList = {sha:string;parent:string;files:Pick<ComparisonFile,"path"|"status">[]};
export type GitComparison = ComparisonFile & {base:string;target:string};
const commitID = /^(?:[a-f0-9]{40}|[a-f0-9]{64})$/;
export function gitComparisonRows(before:string,source:string) {
  if (before.length > 200_000 || source.length > 200_000) return null;
  const result = new Map<number,{removed:string[];added?:string}>();
  if (before===source) return result;
  const pieces = (text:string) => text.match(/[^\n]*\n|[^\n]+$/g) ?? [];
  const old = pieces(before), next = pieces(source);
  let first=0, oldEnd=old.length, nextEnd=next.length;
  while(first<oldEnd&&first<nextEnd&&old[first]===next[first]) first++;
  while(oldEnd>first&&nextEnd>first&&old[oldEnd-1]===next[nextEnd-1]) {oldEnd--;nextEnd--;}
  if ((oldEnd-first+1)*(nextEnd-first+1)>4_000_000) return null;
  let line = first;
  const eof = source.split("\n").length - 1;
  for (const row of diffLines(old.slice(first,oldEnd).join(""),next.slice(first,nextEnd).join(""),true)) {
    if (row.kind === "same") { line++; continue; }
    const at = Math.min(line,eof), entry = result.get(at) ?? {removed:[]};
    const text = row.text.replace(/\r?\n$/, "");
    if (row.kind === "del") entry.removed.push(text);
    else { entry.added = text; line++; }
    result.set(at,entry);
  }
  return result;
}

function GitComparisonPicker({open,close}:{open:(file:GitComparison,current:()=>boolean)=>Promise<boolean>;close:()=>void}) {
  const [base,setBase]=useState(""),[target,setTarget]=useState("");
  const [list,setList]=useState<ComparisonList>(),[reading,setReading]=useState<GitComparison>();
  const [choice,setChoice]=useState(""),[notice,setNotice]=useState(""),[busy,setBusy]=useState(false);
  const ticket=useRef(0);
  useEffect(()=>()=>{ticket.current++;},[]);
  function retire() { ticket.current++;setList(undefined);setReading(undefined);setChoice("");setBusy(false);setNotice(""); }
  async function readList() {
    retire(); const id=ticket.current; setBusy(true);
    try {
      const query = new URLSearchParams({base:base.trim(),sha:target.trim()});
      const value=await apiRequest<ComparisonList>("GET",`/api/git/reading?${query}`);
      if(id!==ticket.current)return;
      if(!commitID.test(value.parent)||!commitID.test(value.sha))throw Error("The comparison did not return two commit identities.");
      setList(value);
    } catch(error) {if(id===ticket.current)setNotice(String(error));}
    finally {if(id===ticket.current)setBusy(false);}
  }
  async function readFile(path:string) {
    setChoice(path); setReading(undefined); setNotice("");
    const id=++ticket.current;
    if(!path||!list){setBusy(false);return;}
    setBusy(true);
    try {
      const query=new URLSearchParams({base:list.parent,sha:list.sha,path});
      const value=await apiRequest<{sha:string;parent:string;files:ComparisonFile[]}>("GET",`/api/git/reading?${query}`);
      if(id!==ticket.current)return;
      const file=value.files[0];
      if(value.sha!==list.sha||value.parent!==list.parent||value.files.length!==1||file?.path!==path)
        throw Error("The patch does not match the selected commits and file.");
      setReading({...file,base:value.parent,target:value.sha});
    } catch(error) {if(id===ticket.current)setNotice(String(error));}
    finally {if(id===ticket.current)setBusy(false);}
  }
  return <section aria-label="Compare two Git commits">
    <label>Base commit <input value={base} onChange={event=>{setBase(event.target.value);retire();}}/></label>
    <label>Target commit <input value={target} onChange={event=>{setTarget(event.target.value);retire();}}/></label>
    <button type="button" disabled={busy||!base.trim()||!target.trim()} title={busy ? "An operation is already in progress." : !base.trim() ? "Enter the base Git revision." : !target.trim() ? "Enter the target Git revision." : undefined} onClick={()=>void readList()}>Read comparison</button>
    {list && <><p>{list.parent.slice(0,12)} → {list.sha.slice(0,12)}</p>{list.files.length
      ? <select aria-label="Changed file" value={choice} onChange={event=>void readFile(event.target.value)}>
        <option value="">Choose a file</option>{list.files.map(file=><option key={file.path} value={file.path}>{file.status} · {file.path}</option>)}
      </select> : <p>The selected commits have no changes in this workspace.</p>}</>}
    {reading && (reading.binary ? <p>Binary change</p> : <>
      {reading.patch_truncated && <p>First 200,000 patch characters shown.</p>}
      <pre aria-label="Commit patch" style={{maxHeight:"30vh",overflow:"auto"}}>{reading.patch}</pre>
      <button type="button" disabled={busy} title={busy ? "An operation is already in progress." : undefined} onClick={async()=>{
        const id=ticket.current;setBusy(true);
        try {if(await open(reading,()=>id===ticket.current)){close();return;}
          if(id===ticket.current)setNotice("The working file could not be opened. Check Files and try again.");
        } finally {if(id===ticket.current)setBusy(false);}
      }}>Open working file with comparison</button>
    </>)}
    {notice && <p role="status">{notice}</p>}
  </section>;
}

export function useGitEditorComparison(buffer:Buffer,view:EditorView|null) {
  const comparison=buffer.gitComparison?.path===buffer.file.path ? buffer.gitComparison : undefined;
  const source=buffer.text,revision=buffer.sourceRevision;
  const [reading,setReading]=useState<{comparison:GitComparison;revision:number;rows:ReturnType<typeof gitComparisonRows>}>();
  const [selected,setSelected]=useState<{comparison:GitComparison;line:number;anchor:DOMRect}>();
  const [notice,setNotice]=useState("");
  useEffect(()=>{
    if(!comparison)return;
    const timer=window.setTimeout(()=>setReading({comparison,revision,rows:gitComparisonRows(comparison.before,source)}),100);
    return ()=>window.clearTimeout(timer);
  },[comparison,revision,source]);
  const rows=reading && reading.comparison===comparison && reading.revision===revision ? reading.rows : undefined;
  const active=selected && selected.comparison===comparison && view ? selected : undefined;
  const line=active ? rows?.get(active.line) : undefined;
  const actions=(!comparison||!view)?[]:[...new Set([0,...(rows?.keys()??[])])].filter(line=>line<view.state.doc.lines).map(line=>({
    at:view.state.doc.line(line+1).from,
    label:`Inspect Git comparison on line ${line+1}`,glyph:"±",
    tone: rows?.has(line) ? "var(--accent)" : undefined,
    run:(anchor?:HTMLElement)=>{if(anchor){setNotice("");setSelected({comparison,line,anchor:anchor.getBoundingClientRect()});}},
  }));
  const panel=active && comparison && view ? createPortal(<section className="simple-semantic-results" role="dialog" aria-label="Git comparison on source line"
    style={{position:"fixed",top:Math.max(8,Math.min(active.anchor.top,innerHeight-255)),right:Math.max(8,innerWidth-active.anchor.left+4),left:"auto",width:"min(24rem, 85vw)",zIndex:100}}
    onKeyDown={event=>{if(event.key==="Escape"){setSelected(undefined);view.focus();}}}>
    <strong>Base {comparison.base.slice(0,12)} → working draft</strong>
    {source===comparison.after ? <p>Draft matches target {comparison.target.slice(0,12)}.</p> :
      !buffer.dirty&&!buffer.readonly&&!buffer.saving&&!buffer.resourceDeleted&&!comparison.status.startsWith("D") &&
      <button type="button" onClick={()=>{
        if(buffer.gitComparison!==comparison||buffer.file.path!==comparison.path||buffer.dirty||buffer.readonly||buffer.saving||view.composing||authoredSource(view.state)!==source)return;
        view.dispatch({...authoredChanges(view.state,[{from:0,to:source.length,insert:comparison.after}]),userEvent:"input.git-comparison"});
        if(authoredSource(view.state)!==comparison.after)setNotice("The editor could not start the target draft.");
      }}>Use target {comparison.target.slice(0,12)} as draft</button>}
    {line && <pre>{line.removed.map(text=>`− ${text}\n`)}{line.added!==undefined&&`+ ${line.added}`}</pre>}
    {rows===null&&<p>Line comparison exceeds the editor limit.</p>}
    <details><summary>Commit patch</summary><pre style={{maxHeight:"30vh",overflow:"auto"}}>{comparison.patch}</pre></details>
    {notice && <p role="status">{notice}</p>}
    <button type="button" onClick={()=>{setSelected(undefined);view.focus();}}>Dismiss</button>
    <button type="button" onClick={()=>{buffer.setGitComparison(undefined);view.focus();}}>Close comparison</button>
  </section>,document.body):null;
  return {actions,panel};
}
</ui:copy>

<ui:file path="apps/web/src/simple/GitHistory.tsx"><ui:paste select="#simple-git-history-browser"/><ui:paste select="#simple-git-history-browser-view"/><ui:paste select="#browser-editable-git-comparison"/></ui:file>
## Keep immutable readings bound to exact commit identities

Hold an immutable reading while selecting another commit, then release it after the newer reply. Exact full identities, closed-view retirement and malformed log identities must prevent a foreign or obsolete reading from replacing the selected commit. These service witnesses do not assert native pointer or accessibility acceptance.

<ui:copy id="simple-git-history-browser-tests">
import {it,expect} from 'vitest';
import {GitHistoryModel,gitComparisonRows} from './GitHistory';
import type {CommitReadingData} from '../api/git';
it("projects replacements and end-of-file deletions onto the editable side",()=>{
 expect(gitComparisonRows("first\nold\nlast\n","first\nnew\n")?.get(1)).toEqual({removed:["old","last"],added:"new"});
 expect(gitComparisonRows("first\nlast\n","first\n")?.get(1)).toEqual({removed:["last"]});
 expect(gitComparisonRows("first\n","first")?.size).toBe(1);
});
it('retains the requested full commit identity across held and closed reads',async()=>{
 const a='a'.repeat(40),b='b'.repeat(40),pending:((value:CommitReadingData)=>void)[]=[];
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[a,b].map(sha=>({sha,short:sha.slice(0,7),author:'fixture',subject:'saved commit'}))}),read:()=>new Promise(resolve=>pending.push(resolve))});
 await model.refresh();const first=model.select(a),second=model.select(b);pending[1]({sha:b,parent:a,message:'new',files:[]});await second;pending[0]({sha:a,parent:null,message:'obsolete',files:[]});await first;expect(model.reading?.sha).toBe(b);expect(model.reading?.message).toBe('new');
 const closed=model.select(a);model.close();pending[2]({sha:a,parent:null,message:'closed',files:[]});await closed;expect(model.reading).toBeNull();
 const foreign=model.select(a);pending[3]({sha:b,parent:null,message:'foreign',files:[]});await foreign;expect(model.reading).toBeNull();expect(model.notice).toContain('identity changed');
});
it('refuses malformed log identities before admitting immutable reading choices',async()=>{
 const model=new GitHistoryModel({log:async()=>({repository:true,commits:[{sha:'HEAD',short:'HEAD',author:'fixture',subject:'ambiguous'}]}),read:async()=>{throw new Error('must not read');}});
 await model.refresh(2000);expect(model.limit).toBe(1000);expect(model.commits).toEqual([]);expect(model.notice).toContain('Malformed commit identity');
});

it('an observed full commit request reads directly without manufacturing a log choice',async()=>{
 const sha='c'.repeat(40),calls:string[]=[];
 const model=new GitHistoryModel({log:async()=>{throw new Error('must not discover');},read:async value=>{calls.push(value);return {sha:value,parent:null,message:'observed',files:[]};}},()=>{},{kind:'commit',sha,title:'Saved attribution'});
 await model.refresh();expect(calls).toEqual([sha]);expect(model.reading?.sha).toBe(sha);await model.readCommit('12345678');expect(calls).toEqual([sha]);expect(model.reading).toBeNull();
 await model.readCommit('d'.repeat(40));expect(calls).toEqual([sha]);expect(model.reading).toBeNull();
});
</ui:copy>

<ui:file path="apps/web/src/simple/GitHistory.test.ts"><ui:paste select="#simple-git-history-browser-tests"/></ui:file>
</ui:doc>
