import {useEffect,useMemo,useReducer,useRef,useState,useSyncExternalStore} from 'react';
import {createPortal} from 'react-dom';
import {request} from '../api/client';
import type {Buffer,Buffers} from './buffers';
import {editorForBuffer} from './Document';
import {EditorView} from '@codemirror/view';
import {WorkspaceChangeHistory,invertChangePlan,type ChangePlan} from './ChangePlan';
import {captureDraftHistory,applyWorkspacePlan} from './WorkspaceChanges';
import {SourceHistoryModel} from './SourceHistory';
export function SourceHistoryView({buffer,buffers,history,alive,close}:{buffer:Buffer;buffers:Buffers;history:WorkspaceChangeHistory;alive:()=>boolean;close:()=>void}){
 useSyncExternalStore(buffer.subscribe,buffer.snapshot);
 const [,paint]=useReducer((value:number)=>value+1,0),model=useMemo(()=>new SourceHistoryModel((method,path,body)=>request(method,path,body),paint),[]),panel=useRef<HTMLElement>(null),[page,setPage]=useState(0);
 useEffect(()=>{const view=editorForBuffer(buffer)?.view,source=buffer.text,path=buffer.file.path,file=buffer.file,namespace=buffers.workspace,revision=buffer.sourceRevision,selection=view?.state.selection.toJSON(),stamp=history.stamp();let living=true;
 const readCurrent=()=>living&&alive()&&!!view&&view.dom.isConnected&&editorForBuffer(buffer)?.view===view&&view.state.doc.toString()===source&&buffer.text===source&&buffer.file===file&&buffer.file.path===path&&buffers.workspace===namespace&&buffer.namespace===namespace&&!!namespace&&buffers.values.get(path)===buffer&&buffer.sourceRevision===revision&&history.stamp()===stamp;
 const current=()=>readCurrent()&&!!view&&!view.composing&&!view.state.readOnly&&view.state.facet(EditorView.editable)&&view.state.selection.ranges.length===1&&JSON.stringify(view.state.selection.toJSON())===JSON.stringify(selection)&&!buffer.readonly&&!buffer.transient&&!buffer.saving&&!buffer.resourceDeleted;
 void model.open({namespace,path,source,readCurrent,current,apply:(after,missing)=>{if(!current())throw Error('Source editor changed before draft restoration.');const plan:ChangePlan={title:'Restore local history to draft',patches:[{path,before:source,after,edits:[{from:0,to:source.length,text:after}]}]};applyWorkspacePlan(buffers,plan);history.record(captureDraftHistory(buffers,invertChangePlan(plan)));if(missing){buffer.resourceDeleted=true;buffer.saveGuard=undefined;buffer.saveGuardReason='Restored missing-source draft: use guarded Save As to publish to an unused path.';buffer.publish();buffers.changed();}view!.focus();}});panel.current?.focus();return()=>{living=false;model.close();};},[buffer,buffers,history,model]);
 const selected=model.selected,text=model.preview;
 return createPortal(<section role="dialog" aria-label="Local source history" tabIndex={-1} ref={panel} className="simple-workspace-problems simple-source-tests" onKeyDown={event=>{if(event.key==='Escape'){event.preventDefault();close();}}}>
 <header><strong>Local source history</strong><button onClick={close}>Close</button></header><p role="status">{model.notice}</p>{model.reading?.notices.map((notice,index)=><p key={index}>{notice}</p>)}
 <label>Retained save <select aria-label="Retained save" value={selected?.id??''} disabled={model.busy} onChange={event=>{model.choose(event.target.value,model.side);setPage(0);}}><option value="">Choose a save</option>{model.reading?.entries.map(entry=><option key={entry.id} value={entry.id}>{entry.outcome} · {new Date(entry.at).toISOString()} · {entry.id}</option>)}</select></label>
 {selected&&<><label>Snapshot <select aria-label="Snapshot" value={model.side} onChange={event=>{model.choose(selected.id,event.target.value as 'before'|'after');setPage(0);}}><option value="before">Before save</option><option value="after">Intended saved after</option></select></label><p>{selected.outcome} · {selected.fingerprint}</p><strong>Retained text</strong><pre style={{whiteSpace:'pre-wrap'}}>{text?.slice(page*16000,(page+1)*16000)??'Source absent; absence cannot replace an editor draft.'}</pre><strong>Current unsaved draft</strong><pre style={{whiteSpace:'pre-wrap'}}>{buffer.text.slice(page*16000,(page+1)*16000)}</pre><p>Text page {page+1} · 16,000 characters per side</p><button disabled={!page} onClick={()=>setPage(page-1)}>Previous text</button><button disabled={Math.max(text?.length??0,buffer.text.length)<=(page+1)*16000} onClick={()=>setPage(page+1)}>Next text</button><button disabled={model.busy||!selected.available||text===null||!model.current()} onClick={()=>void model.restore()}>Restore selected text to draft</button></>}
 {!model.busy&&model.reading?.entries.length===0&&<p>No ordinary saves are retained for this source.</p>}
 </section>,document.body);
}
