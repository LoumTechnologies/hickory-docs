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
export function GitAttributionView({workspace,path,close,readCommit}:{workspace:string;path:string;close:()=>void;readCommit:(sha:string,title:string)=>void}){
 const [,paint]=useReducer((n:number)=>n+1,0),model=useMemo(()=>new GitAttributionModel(path,value=>api.blame(value),paint),[workspace,path]),panel=useRef<HTMLElement>(null);
 useEffect(()=>{void model.refresh();panel.current?.focus();return()=>model.close();},[model]);
 return createPortal(<section ref={panel} tabIndex={-1} role="dialog" aria-label="Saved file attribution" className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Saved attribution · {path}</strong><button onClick={()=>void model.refresh()}>Refresh</button><button onClick={close}>Close</button></header><p role="status">{model.notice}</p><p>Saved-file lines only. Unsaved editor changes are not included.</p>
 <div>{model.lines.slice(model.page*50,(model.page+1)*50).map(line=>{const sha=attributionCommit(line);return <article key={line.line}><strong>Saved line {line.line} · {line.author}</strong><span> · {line.commit||'No committed revision'}</span><p>{line.summary} · {line.provider??(line.uncommitted?'Uncommitted saved line':'Git blame')}</p><pre style={{whiteSpace:'pre-wrap'}}>{line.text===undefined?'Line text unavailable.':line.text.slice(0,2000)}</pre>{(line.text?.length??0)>2000&&<p>Line text abbreviated.</p>}{sha?<button aria-label={`Read commit for saved line ${line.line}`} onClick={()=>{if(model.lines.includes(line))readCommit(sha,`Commit from saved attribution · ${path}:${line.line}`);}}>Read immutable commit</button>:<p>Full immutable commit reference unavailable.</p>}</article>;})}</div>
 <nav aria-label="Attribution pages"><button disabled={model.page===0} onClick={()=>{model.page--;paint();}}>Previous</button><span>{model.lines.length} reported lines · page {model.page+1}</span><button disabled={(model.page+1)*50>=model.lines.length} onClick={()=>{model.page++;paint();}}>Next</button></nav>
 </section>,document.body);
}
