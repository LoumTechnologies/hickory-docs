<ui:doc xmlns:ui="http://www.hickorydocs.com/1.0">
# Inspect saved attribution without borrowing draft positions

For maintainers changing workspace services independently of the editor renderer.

Saved-file attribution is a captured reading, not a claim that line positions match an unsaved editor draft. Keep its requested path, reported line text, author, provider and immutable commit reference together. Reject foreign paths, repeated line identities and obsolete replies before presentation. Uncommitted lines and older providers without a full reference remain readable; only a valid reported full identity can open an immutable commit reading. This capability performs no source or index mutation.

<ui:copy id="simple-git-attribution-browser">
import {useEffect,useMemo,useReducer,useRef} from 'react';
import {createPortal} from 'react-dom';
import {api} from '../api/client';
import type {BlameLine} from '../api/types';
export function attributionCommit(line:BlameLine):string|null {return line.uncommitted===false&&typeof line.full_commit==='string'&&/^([0-9a-f]{40}|[0-9a-f]{64})$/.test(line.full_commit)?line.full_commit:null;}
export class GitAttributionModel {
 lines:BlameLine[]=[];notice='';page=0;loading=false;private ticket=0;
 constructor(readonly path:string,private readonly load:(path:string)=>Promise<{path:string;lines:BlameLine[]}>,private readonly changed:()=>void=()=>{}){}
 close(){this.ticket++;this.loading=false;this.lines=[];this.notice='';}
 async refresh(){const id=++this.ticket;this.lines=[];this.page=0;this.loading=true;this.notice='Reading saved-file attribution…';this.changed();
  try{const value=await this.load(this.path);if(id!==this.ticket)return;if(value.path!==this.path)throw new Error('Attribution identity changed. Refresh the saved file.');if(value.lines.some(line=>!Number.isSafeInteger(line.line)||line.line<1)||new Set(value.lines.map(line=>line.line)).size!==value.lines.length)throw new Error('Ambiguous attribution lines. Refresh the saved file.');this.lines=value.lines.map(line=>({...line})).sort((a,b)=>a.line-b.line);this.notice=value.lines.length?'Captured saved attribution · draft line positions may differ.':'No saved Git or recorded attribution is available.';}
  catch(error){if(id===this.ticket)this.notice=String(error);}finally{if(id===this.ticket){this.loading=false;this.changed();}}
 }
}
</ui:copy>

## Present saved facts and reuse immutable commit reading

Use the existing floating menu surface, retaining the captured path in its heading. Report saved line numbers and provider labels without anchoring them onto dirty draft positions. Only a current row with a full committed identity can construct a typed commit request for the existing historical reader. Paging and explicit line-text limits keep inspection compact; closing restores code focus and retires the request.

<ui:copy id="simple-git-attribution-browser-view">
export function GitAttributionView({workspace,path,close,readCommit}:{workspace:string;path:string;close:()=>void;readCommit:(sha:string,title:string)=>void}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitAttributionModel(path,value=>api.blame(value),paint),[workspace,path]),panel=useRef<HTMLElement>(null);
 useEffect(()=>{void model.refresh();panel.current?.focus();return()=>model.close();},[model]);
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Saved file attribution" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Saved attribution · {path}</strong><button onClick={()=>void model.refresh()}>Refresh</button><button onClick={close}>Close</button></header><p role="status">{model.notice}</p><p>Saved-file lines only. Unsaved editor changes are not included.</p>
 <div>{model.lines.slice(model.page*50,(model.page+1)*50).map(line=>{const sha=attributionCommit(line);return <article key={line.line}><strong>Saved line {line.line} · {line.author}</strong><span> · {line.commit||'No committed revision'}</span><p>{line.summary} · {line.provider??(line.uncommitted?'Uncommitted saved line':'Git blame')}</p><pre style={{whiteSpace:'pre-wrap'}}>{line.text===undefined?'Line text unavailable.':line.text.slice(0,2000)}</pre>{(line.text?.length??0)>2000&&<p>Line text abbreviated.</p>}{sha?<button aria-label={`Read commit for saved line ${line.line}`} onClick={()=>{if(model.lines.includes(line))readCommit(sha,`Commit from saved attribution · ${path}:${line.line}`);}}>Read immutable commit</button>:<p>Full immutable commit reference unavailable.</p>}</article>;})}</div>
 <nav aria-label="Attribution pages"><button disabled={model.page===0} title={model.page===0 ? "You’re on the first page." : undefined} onClick={()=>{model.page--;paint();}}>Previous</button><span>{model.lines.length} reported lines · page {model.page+1}</span><button disabled={(model.page+1)*50>=model.lines.length} title={(model.page+1)*50>=model.lines.length ? "You’re on the last page." : undefined} onClick={()=>{model.page++;paint();}}>Next</button></nav>
 </section>,document.body);
}
</ui:copy>

<ui:file path="apps/web/src/simple/GitAttribution.tsx"><ui:paste select="#simple-git-attribution-browser"/><ui:paste select="#simple-git-attribution-browser-view"/></ui:file>
## Reject stale attribution and ambiguous references

Hold attribution replies, replace their request, and close the reading before releasing old work. Foreign paths and repeated line identities must refuse without presenting a mixed snapshot. Only a valid full reference on a committed line admits immutable reading; abbreviated, missing and uncommitted references remain display facts.

<ui:copy id="simple-git-attribution-browser-tests">
import {it,expect} from 'vitest';
import {GitAttributionModel,attributionCommit} from './GitAttribution';
import type {BlameLine} from '../api/types';
const row=(line:number,full_commit?:string):BlameLine=>({line,commit:'12345678',full_commit,author:'Fixture',email:'',time:0,summary:'base',uncommitted:false,text:'saved'});
it('keeps captured paths and line identities separate from obsolete replies',async()=>{
 let finish!:(value:{path:string;lines:BlameLine[]})=>void,calls=0;const model=new GitAttributionModel('a.py',path=>++calls===1?new Promise(resolve=>finish=resolve):Promise.resolve({path,lines:[row(2,'a'.repeat(40))]}));
 const old=model.refresh();await model.refresh();finish({path:'a.py',lines:[row(1)]});await old;expect(model.lines.map(line=>line.line)).toEqual([2]);
 const closed=new GitAttributionModel('a.py',()=>new Promise(resolve=>finish=resolve));const held=closed.refresh();closed.close();finish({path:'a.py',lines:[row(1)]});await held;expect(closed.lines).toEqual([]);
 const foreign=new GitAttributionModel('cafe\u0301.py',async()=>({path:'caf\u00e9.py',lines:[row(1)]}));await foreign.refresh();expect(foreign.lines).toEqual([]);expect(foreign.notice).toContain('identity changed');
 const duplicate=new GitAttributionModel('a.py',async path=>({path,lines:[row(1),row(1)]}));await duplicate.refresh();expect(duplicate.lines).toEqual([]);expect(duplicate.notice).toContain('Ambiguous');
});
it('never turns a compact label or dirty line into an immutable reference',()=>{
 expect(attributionCommit(row(1))).toBeNull();expect(attributionCommit(row(1,'12345678'))).toBeNull();expect(attributionCommit({...row(1,'a'.repeat(40)),uncommitted:true})).toBeNull();expect(attributionCommit(row(1,'a'.repeat(40)))).toBe('a'.repeat(40));expect(attributionCommit(row(1,'b'.repeat(64)))).toBe('b'.repeat(64));
});
</ui:copy>

<ui:file path="apps/web/src/simple/GitAttribution.test.ts"><ui:paste select="#simple-git-attribution-browser-tests"/></ui:file>
</ui:doc>
